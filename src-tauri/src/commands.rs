//! Tauri commands for Bare
//!
//! IPC-kommandoer som kan kalles fra frontend.

use crate::bookmarks::{self, Bookmark, BookmarkStore};
use crate::converter;
use crate::fetcher::{self, Fetcher};
use crate::gemini::{self, GeminiClient, GeminiError};
use crate::gemtext;
use crate::gopher;
use crate::gophermap;
use crate::markdown;
use crate::settings::{self, ConversionMode, FontFamily, Settings, Theme};
use log::debug;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::{LazyLock, Mutex};
use tauri::Emitter;

// Emoji-konstanter for protokollidentifikasjon
const EMOJI_HTTPS: &str = "🔒";
const EMOJI_HTTP: &str = "🌐";
const EMOJI_GEMINI: &str = "📡";
const EMOJI_GOPHER: &str = "🐿️";
const EMOJI_FILE: &str = "📁";

/// Global HTTP-klient (gjenbrukes for alle forespørsler)
static FETCHER: LazyLock<Fetcher> = LazyLock::new(Fetcher::new);

/// Global Gemini-klient (gjenbrukes for alle Gemini-forespørsler)
static GEMINI_CLIENT: LazyLock<GeminiClient> = LazyLock::new(GeminiClient::new);

/// Global bokmerke-lagring
static BOOKMARKS: LazyLock<Mutex<BookmarkStore>> = LazyLock::new(|| {
    let path = bookmarks::get_bookmarks_path();
    Mutex::new(BookmarkStore::load(&path).unwrap_or_default())
});

/// Global innstillingslagring
static SETTINGS: LazyLock<Mutex<Settings>> = LazyLock::new(|| {
    let path = settings::get_settings_path();
    Mutex::new(Settings::load(&path).unwrap_or_default())
});

/// Global cache for rendrede sider (LRU - Least Recently Used)
/// Lagrer opptil 50 sider for å øke hastigheten på navigasjon.
static RENDER_CACHE: LazyLock<Mutex<lru::LruCache<String, RenderedPage>>> =
    LazyLock::new(|| Mutex::new(lru::LruCache::new(std::num::NonZeroUsize::new(50).unwrap())));

/// Ekstraher vertsnavn fra en URL for visning i statusbar
fn extract_host(url: &str) -> String {
    url::Url::parse(url)
        .ok()
        .and_then(|u| u.host_str().map(|h| h.to_string()))
        .unwrap_or_else(|| url.to_string())
}

/// Henter app-versjon fra Cargo.toml
#[tauri::command]
pub fn get_app_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// Resultat fra markdown-rendering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderedPage {
    /// HTML-innhold
    pub html: String,
    /// Tittel ekstrahert fra markdown (hvis funnet)
    pub title: Option<String>,
    /// URL-en som ble brukt (etter eventuelle redirects)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// Om innholdet ble hentet fra nettverket
    #[serde(default)]
    pub is_remote: bool,
    /// Om innholdet ble konvertert fra HTML
    #[serde(default)]
    pub was_converted: bool,
}

/// Rendrer markdown-tekst til HTML
///
/// # Arguments
/// * `content` - Markdown-innhold som skal rendres
///
/// # Returns
/// RenderedPage med HTML og eventuell tittel
#[tauri::command]
pub fn render_markdown(content: String) -> RenderedPage {
    let html = markdown::render(&content);
    let title = markdown::extract_title(&content);

    RenderedPage {
        html,
        title,
        url: None,
        is_remote: false,
        was_converted: false,
    }
}

/// Åpner og leser en lokal markdown-fil
///
/// # Arguments
/// * `path` - Sti til filen som skal åpnes
///
/// # Returns
/// RenderedPage med HTML og tittel, eller feilmelding
#[tauri::command]
pub fn open_file(path: String, window: tauri::Window) -> Result<RenderedPage, String> {
    let path = PathBuf::from(&path);

    // Sjekk at filen eksisterer
    if !path.exists() {
        return Err(format!("Filen finnes ikke: {}", path.display()));
    }

    // Sjekk at det er en markdown-fil
    match path.extension() {
        Some(ext) if ext == "md" || ext == "markdown" => {}
        _ => {
            return Err("Bare støtter kun markdown-filer (.md, .markdown)".to_string());
        }
    }

    // Steg 1: Åpner fil
    let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("fil");
    let _ = window.emit(
        "loading-status",
        format!("{} Lokal fil: Åpner {}...", EMOJI_FILE, filename),
    );

    // Les innholdet
    let content = fs::read_to_string(&path).map_err(|e| format!("Kunne ikke lese fil: {}", e))?;

    // Steg 2: Rendrer markdown
    let _ = window.emit(
        "loading-status",
        format!("{} Lokal fil: Rendrer markdown...", EMOJI_FILE),
    );
    let html = markdown::render(&content);
    let title = markdown::extract_title(&content);

    let _ = window.emit("loading-status", "Dokument: Ferdig");

    Ok(RenderedPage {
        html,
        title,
        url: Some(format!("file://{}", path.display())),
        is_remote: false,
        was_converted: false,
    })
}

/// Henter og rendrer markdown fra en URL
///
/// # Arguments
/// * `url` - URL til markdown-filen som skal hentes
///
/// # Returns
/// RenderedPage med HTML og tittel, eller feilmelding
#[tauri::command]
pub async fn fetch_url(url: String, window: tauri::Window) -> Result<RenderedPage, String> {
    // Sjekk cache først
    {
        let mut cache = RENDER_CACHE.lock().unwrap();
        if let Some(cached) = cache.get(&url) {
            debug!("Cache hit for URL: {}", url);
            let _ = window.emit("loading-status", "Dokument: Hentet fra cache");
            return Ok(cached.clone());
        }
    }

    // Detekter protokoll
    let parsed_url = url::Url::parse(&url).map_err(|e| e.to_string())?;
    let scheme = parsed_url.scheme();
    let host = extract_host(&url);
    let protocol_emoji = if scheme == "https" {
        EMOJI_HTTPS
    } else {
        EMOJI_HTTP
    };
    let protocol_name = if scheme == "https" { "HTTPS" } else { "HTTP" };

    // Steg 1: Slår opp vert
    let _ = window.emit(
        "loading-status",
        format!("{} {}: Slår opp {}...", protocol_emoji, protocol_name, host),
    );

    // Steg 2: Kobler til
    let tls_info = if scheme == "https" { "/TLS" } else { "" };
    let _ = window.emit(
        "loading-status",
        format!(
            "{} {}{}: Kobler til {}...",
            protocol_emoji, protocol_name, tls_info, host
        ),
    );

    let result = FETCHER.fetch(&url).await.map_err(|e| {
        let _ = window.emit("loading-status", "Feil under henting");
        e.to_string()
    })?;

    // Steg 3: Overfører data
    let bytes = result.content.len();
    let _ = window.emit(
        "loading-status",
        format!("Overfører data... ({} bytes)", bytes),
    );

    // Hent konverteringsinnstillinger
    let settings = SETTINGS.lock().unwrap();
    let conversion_mode = settings.conversion_mode.clone();
    let _readability_enabled = settings.readability_enabled;
    drop(settings);

    if result.is_markdown {
        // Steg 4: Rendrer markdown
        let _ = window.emit("loading-status", "Rendrer markdown...");
        let html = markdown::render(&result.content);
        let title = markdown::extract_title(&result.content);

        let _ = window.emit("loading-status", "Dokument: Ferdig");

        let page = RenderedPage {
            html,
            title,
            url: Some(result.final_url.clone()),
            is_remote: true,
            was_converted: false,
        };

        // Lagre i cache
        {
            let mut cache = RENDER_CACHE.lock().unwrap();
            if result.final_url != url {
                cache.put(result.final_url.clone(), page.clone());
            }
            cache.put(url, page.clone());
        }

        return Ok(page);
    }

    // Ikke-markdown innhold - sjekk konverteringsmodus
    match conversion_mode {
        ConversionMode::MarkdownOnly => {
            let _ = window.emit("loading-status", "Stoppet: Kun markdown");
            Err(format!(
                "Innholdet er ikke markdown (Content-Type: {:?}). Konvertering er deaktivert i innstillingene.",
                result.content_type
            ))
        }
        ConversionMode::AskEverytime => {
            let _ = window.emit("loading-status", "Venter på brukervalg...");
            // Returner en spesiell respons som ber frontend spørre brukeren
            Err(format!(
                "CONVERSION_PROMPT:Innholdet er HTML. Vil du konvertere det til markdown?:{}",
                result.final_url
            ))
        }
        ConversionMode::ConvertAll => {
            // Steg 4: Konverterer HTML
            let _ = window.emit("loading-status", "Konverterer HTML til markdown...");
            let conversion_result = converter::html_to_markdown(
                &result.content,
                Some(&result.final_url),
                _readability_enabled,
            );

            // Steg 5: Rendrer markdown
            let _ = window.emit("loading-status", "Rendrer markdown...");
            let html = markdown::render(&conversion_result.markdown);

            let title = conversion_result
                .title
                .or_else(|| markdown::extract_title(&conversion_result.markdown));

            let _ = window.emit("loading-status", "Dokument: Ferdig");

            let page = RenderedPage {
                html,
                title,
                url: Some(result.final_url.clone()),
                is_remote: true,
                was_converted: true,
            };

            // Lagre i cache
            {
                let mut cache = RENDER_CACHE.lock().unwrap();
                if result.final_url != url {
                    cache.put(result.final_url.clone(), page.clone());
                }
                cache.put(url, page.clone());
            }

            Ok(page)
        }
    }
}

/// Konverter HTML til markdown manuelt (for "Spør hver gang"-modus)
///
/// # Arguments
/// * `url` - URL til siden som skal konverteres
///
/// # Returns
/// RenderedPage med konvertert innhold
#[tauri::command]
pub async fn convert_url(url: String, window: tauri::Window) -> Result<RenderedPage, String> {
    // Detekter protokoll
    let parsed_url = url::Url::parse(&url).map_err(|e| e.to_string())?;
    let scheme = parsed_url.scheme();
    let host = extract_host(&url);
    let protocol_emoji = if scheme == "https" {
        EMOJI_HTTPS
    } else {
        EMOJI_HTTP
    };
    let protocol_name = if scheme == "https" { "HTTPS" } else { "HTTP" };

    let _ = window.emit(
        "loading-status",
        format!("{} {}: Slår opp {}...", protocol_emoji, protocol_name, host),
    );
    let tls_info = if scheme == "https" { "/TLS" } else { "" };
    let _ = window.emit(
        "loading-status",
        format!(
            "{} {}{}: Kobler til {}...",
            protocol_emoji, protocol_name, tls_info, host
        ),
    );

    let result = FETCHER.fetch(&url).await.map_err(|e| {
        let _ = window.emit("loading-status", "Feil under henting");
        e.to_string()
    })?;

    let bytes = result.content.len();
    let _ = window.emit(
        "loading-status",
        format!("Overfører data... ({} bytes)", bytes),
    );

    // Hent readability-innstilling
    let settings = SETTINGS.lock().unwrap();
    let readability_enabled = settings.readability_enabled;
    drop(settings);

    // Konverter HTML til markdown
    let _ = window.emit("loading-status", "Konverterer HTML til markdown...");
    let conversion_result = converter::html_to_markdown(
        &result.content,
        Some(&result.final_url),
        readability_enabled,
    );

    // Render markdown til HTML for visning
    let _ = window.emit("loading-status", "Rendrer markdown...");
    let html = markdown::render(&conversion_result.markdown);

    // Bruk tittel fra konvertering eller markdown
    let title = conversion_result
        .title
        .or_else(|| markdown::extract_title(&conversion_result.markdown));

    let _ = window.emit("loading-status", "Dokument: Ferdig");

    Ok(RenderedPage {
        html,
        title,
        url: Some(result.final_url),
        is_remote: true,
        was_converted: true,
    })
}

/// Løser en relativ URL mot en base-URL
///
/// # Arguments
/// * `base_url` - Nåværende side sin URL
/// * `relative_url` - Relativ URL som skal løses
///
/// # Returns
/// Absolutt URL
#[tauri::command]
pub fn resolve_url(base_url: String, relative_url: String) -> Result<String, String> {
    fetcher::resolve_url(&base_url, &relative_url).map_err(|e| e.to_string())
}

// ===== Bokmerke-commands =====

/// Bokmerke-info for frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookmarkInfo {
    pub id: String,
    pub title: String,
    pub url: String,
    pub created_at: u64,
}

impl From<&Bookmark> for BookmarkInfo {
    fn from(b: &Bookmark) -> Self {
        Self {
            id: b.id.clone(),
            title: b.title.clone(),
            url: b.url.clone(),
            created_at: b.created_at,
        }
    }
}

/// Hent alle bokmerker
#[tauri::command]
pub fn get_bookmarks() -> Vec<BookmarkInfo> {
    let store = BOOKMARKS.lock().unwrap();
    store.list().iter().map(BookmarkInfo::from).collect()
}

/// Legg til et nytt bokmerke
#[tauri::command]
pub fn add_bookmark(title: String, url: String) -> Result<BookmarkInfo, String> {
    let mut store = BOOKMARKS.lock().unwrap();

    let bookmark = Bookmark {
        id: bookmarks::generate_id(),
        title,
        url,
        created_at: bookmarks::current_timestamp(),
    };

    store.add(bookmark.clone()).map_err(|e| e.to_string())?;

    // Lagre til fil
    let path = bookmarks::get_bookmarks_path();
    store.save(&path).map_err(|e| e.to_string())?;

    Ok(BookmarkInfo::from(&bookmark))
}

/// Fjern et bokmerke
#[tauri::command]
pub fn remove_bookmark(id: String) -> Result<(), String> {
    let mut store = BOOKMARKS.lock().unwrap();
    store.remove(&id).map_err(|e| e.to_string())?;

    // Lagre til fil
    let path = bookmarks::get_bookmarks_path();
    store.save(&path).map_err(|e| e.to_string())
}

/// Sjekk om en URL er bokmerket
#[tauri::command]
pub fn is_bookmarked(url: String) -> bool {
    let store = BOOKMARKS.lock().unwrap();
    store.is_bookmarked(&url)
}

// ===== Innstillinger-commands =====

/// Innstillinger for frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsInfo {
    pub theme: String,
    pub font_size: u32,
    pub zoom: u32,
    pub font_family: String,
    pub content_width: u32,
    pub show_line_numbers: bool,
    pub conversion_mode: String,
    pub readability_enabled: bool,
    pub image_mode: String,
    pub onboarding_completed: bool,
    pub language: String,
    pub search_engine: String,
}

impl From<&Settings> for SettingsInfo {
    fn from(s: &Settings) -> Self {
        Self {
            theme: match s.theme {
                Theme::Light => "light".to_string(),
                Theme::Dark => "dark".to_string(),
                Theme::System => "system".to_string(),
            },
            font_size: s.font_size,
            zoom: s.zoom,
            font_family: match s.font_family {
                FontFamily::System => "system".to_string(),
                FontFamily::Serif => "serif".to_string(),
                FontFamily::SansSerif => "sans-serif".to_string(),
                FontFamily::Mono => "mono".to_string(),
            },
            content_width: s.content_width,
            show_line_numbers: s.show_line_numbers,
            conversion_mode: match s.conversion_mode {
                ConversionMode::MarkdownOnly => "markdown-only".to_string(),
                ConversionMode::ConvertAll => "convert-all".to_string(),
                ConversionMode::AskEverytime => "ask-everytime".to_string(),
            },
            readability_enabled: s.readability_enabled,
            image_mode: match s.image_mode {
                settings::ImageMode::Block => "block".to_string(),
                settings::ImageMode::Placeholder => "placeholder".to_string(),
                settings::ImageMode::Show => "show".to_string(),
            },
            onboarding_completed: s.onboarding_completed,
            language: s.language.clone(),
            search_engine: s.search_engine.clone(),
        }
    }
}

/// Hent gjeldende innstillinger
#[tauri::command]
pub fn get_settings() -> SettingsInfo {
    let settings = SETTINGS.lock().unwrap();
    SettingsInfo::from(&*settings)
}

/// Parametere for oppdatering av innstillinger
#[derive(serde::Deserialize)]
pub struct UpdateSettingsParams {
    pub theme: Option<String>,
    pub font_size: Option<u32>,
    pub zoom: Option<u32>,
    pub font_family: Option<String>,
    pub content_width: Option<u32>,
    pub show_line_numbers: Option<bool>,
    pub conversion_mode: Option<String>,
    pub readability_enabled: Option<bool>,
    pub image_mode: Option<String>,
    pub onboarding_completed: Option<bool>,
    pub language: Option<String>,
    pub search_engine: Option<String>,
}

/// Oppdater innstillinger
#[tauri::command]
pub fn update_settings(params: UpdateSettingsParams) -> Result<SettingsInfo, String> {
    let mut settings = SETTINGS.lock().unwrap();

    if let Some(t) = params.theme {
        settings.theme = match t.as_str() {
            "dark" => Theme::Dark,
            "system" => Theme::System,
            _ => Theme::Light,
        };
    }

    if let Some(size) = params.font_size {
        settings.font_size = size.clamp(70, 150);
    }

    if let Some(z) = params.zoom {
        settings.zoom = z.clamp(50, 200);
    }

    if let Some(ff) = params.font_family {
        settings.font_family = match ff.as_str() {
            "serif" => FontFamily::Serif,
            "sans-serif" => FontFamily::SansSerif,
            "mono" => FontFamily::Mono,
            _ => FontFamily::System,
        };
    }

    if let Some(width) = params.content_width {
        settings.content_width = width.clamp(400, 1200);
    }

    if let Some(ln) = params.show_line_numbers {
        settings.show_line_numbers = ln;
    }

    if let Some(cm) = params.conversion_mode {
        settings.conversion_mode = match cm.as_str() {
            "markdown-only" => ConversionMode::MarkdownOnly,
            "ask-everytime" => ConversionMode::AskEverytime,
            _ => ConversionMode::ConvertAll,
        };
    }

    if let Some(re) = params.readability_enabled {
        settings.readability_enabled = re;
    }

    if let Some(im) = params.image_mode {
        settings.image_mode = match im.as_str() {
            "placeholder" => settings::ImageMode::Placeholder,
            "show" => settings::ImageMode::Show,
            _ => settings::ImageMode::Block,
        };
    }

    if let Some(oc) = params.onboarding_completed {
        settings.onboarding_completed = oc;
    }

    if let Some(lang) = params.language {
        settings.language = lang;
    }

    if let Some(se) = params.search_engine {
        settings.search_engine = se;
    }

    // Lagre til fil
    let path = settings::get_settings_path();
    settings.save(&path).map_err(|e| e.to_string())?;

    Ok(SettingsInfo::from(&*settings))
}

/// Zoom inn
#[tauri::command]
pub fn zoom_in() -> Result<SettingsInfo, String> {
    let mut settings = SETTINGS.lock().unwrap();
    settings.zoom_in();

    let path = settings::get_settings_path();
    settings.save(&path).map_err(|e| e.to_string())?;

    Ok(SettingsInfo::from(&*settings))
}

/// Zoom ut
#[tauri::command]
pub fn zoom_out() -> Result<SettingsInfo, String> {
    let mut settings = SETTINGS.lock().unwrap();
    settings.zoom_out();

    let path = settings::get_settings_path();
    settings.save(&path).map_err(|e| e.to_string())?;

    Ok(SettingsInfo::from(&*settings))
}

/// Tilbakestill zoom
#[tauri::command]
pub fn zoom_reset() -> Result<SettingsInfo, String> {
    let mut settings = SETTINGS.lock().unwrap();
    settings.zoom_reset();

    let path = settings::get_settings_path();
    settings.save(&path).map_err(|e| e.to_string())?;

    Ok(SettingsInfo::from(&*settings))
}

// ===== Gemini-commands =====

/// Henter og rendrer innhold fra en Gemini-URL
///
/// # Arguments
/// * `url` - Gemini-URL å hente (gemini://...)
///
/// # Returns
/// RenderedPage med konvertert gemtext→markdown→HTML, eller feilmelding
#[tauri::command]
pub async fn fetch_gemini(url: String, window: tauri::Window) -> Result<RenderedPage, String> {
    // Sjekk cache først
    {
        let mut cache = RENDER_CACHE.lock().unwrap();
        if let Some(cached) = cache.get(&url) {
            debug!("Cache hit for Gemini URL: {}", url);
            let _ = window.emit("loading-status", "Dokument: Hentet fra cache");
            return Ok(cached.clone());
        }
    }

    let host = extract_host(&url);

    // Steg 1: Gemini TLS-handshake
    let _ = window.emit(
        "loading-status",
        format!(
            "{} Gemini TLS-handshake (port 1965) med {}...",
            EMOJI_GEMINI, host
        ),
    );

    let result = GEMINI_CLIENT.fetch(&url).await;

    match result {
        Ok(response) => {
            let body = response.body.unwrap_or_default();
            let bytes = body.len();

            // Steg 2: Overfører data
            let _ = window.emit(
                "loading-status",
                format!("Overfører data... ({} bytes)", bytes),
            );

            // Sjekk om innholdet er gemtext
            let is_gemtext = response.meta.is_empty()
                || response.meta.starts_with("text/gemini")
                || response.meta == "text/gemini";

            if is_gemtext {
                // Steg 3: Konverterer gemtext
                let _ = window.emit("loading-status", "Konverterer gemtext...");
                let gemtext_result = gemtext::gemtext_to_markdown(&body);

                // Steg 4: Rendrer markdown
                let _ = window.emit("loading-status", "Rendrer markdown...");
                let html = markdown::render(&gemtext_result.markdown);

                let title = gemtext_result
                    .title
                    .or_else(|| markdown::extract_title(&gemtext_result.markdown));

                let _ = window.emit("loading-status", "Dokument: Ferdig");

                let page = RenderedPage {
                    html,
                    title,
                    url: Some(response.final_url.clone()),
                    is_remote: true,
                    was_converted: true,
                };

                // Lagre i cache
                {
                    let mut cache = RENDER_CACHE.lock().unwrap();
                    if response.final_url != url {
                        cache.put(response.final_url.clone(), page.clone());
                    }
                    cache.put(url, page.clone());
                }

                Ok(page)
            } else if response.meta.starts_with("text/") {
                // Ren tekst — vis som markdown-kodeblokk
                let _ = window.emit("loading-status", "Rendrer tekst...");
                let markdown_content = format!("```\n{}\n```", body);
                let html = markdown::render(&markdown_content);

                let _ = window.emit("loading-status", "Dokument: Ferdig");

                let page = RenderedPage {
                    html,
                    title: None,
                    url: Some(response.final_url.clone()),
                    is_remote: true,
                    was_converted: true,
                };

                // Lagre i cache
                {
                    let mut cache = RENDER_CACHE.lock().unwrap();
                    if response.final_url != url {
                        cache.put(response.final_url.clone(), page.clone());
                    }
                    cache.put(url, page.clone());
                }

                Ok(page)
            } else {
                // Ikke-tekstinnhold
                Err(format!(
                    "Innholdstypen '{}' støttes ikke. Bare kan kun vise tekst-basert innhold.",
                    response.meta
                ))
            }
        }
        Err(GeminiError::InputRequired(prompt)) => {
            let _ = window.emit("loading-status", "Venter på brukerinput...");
            Err(format!("GEMINI_INPUT_PROMPT:{}", prompt))
        }
        Err(GeminiError::SensitiveInputRequired(prompt)) => {
            let _ = window.emit("loading-status", "Venter på brukerinput...");
            Err(format!("GEMINI_SENSITIVE_INPUT_PROMPT:{}", prompt))
        }
        Err(GeminiError::CertificateChanged {
            host,
            old_fp,
            new_fp,
        }) => {
            let _ = window.emit("loading-status", "Sertifikat-feil");
            Err(format!(
                "⚠️ Sertifikatadvarsel for {}!\n\n\
                 Sertifikatet har endret seg siden forrige besøk.\n\
                 Dette kan indikere et sikkerhetsbrudd.\n\n\
                 Gammelt fingerprint: {}\nNytt fingerprint: {}",
                host, old_fp, new_fp
            ))
        }
        Err(GeminiError::ClientCertRequired) => {
            let _ = window.emit("loading-status", "Klientsertifikat påkrevd");
            Err("Denne Gemini-kapselen krever klientsertifikat.\n\
                 Denne funksjonaliteten er ikke støttet ennå."
                .to_string())
        }
        Err(e) => {
            let _ = window.emit("loading-status", "Feil under henting");
            Err(e.to_string())
        }
    }
}

/// Sender brukerinput til en Gemini-server (statuskode 10)
///
/// Konstruerer en ny URL med ?input lagt til, og henter innholdet.
///
/// # Arguments
/// * `url` - Original Gemini-URL som ba om input
/// * `input` - Brukerens input-tekst
///
/// # Returns
/// RenderedPage med resultatet, eller feilmelding
#[tauri::command]
pub async fn submit_gemini_input(
    url: String,
    input: String,
    window: tauri::Window,
) -> Result<RenderedPage, String> {
    // Konstruer URL med input som query-parameter
    let mut parsed = url::Url::parse(&url).map_err(|e| format!("Ugyldig URL: {}", e))?;
    parsed.set_query(Some(&input));

    let input_url = parsed.to_string();
    fetch_gemini(input_url, window).await
}

/// Løser en relativ URL mot en Gemini base-URL
///
/// # Arguments
/// * `base_url` - Nåværende Gemini-side sin URL
/// * `relative_url` - Relativ URL som skal løses
///
/// # Returns
/// Absolutt URL
#[tauri::command]
pub fn resolve_gemini_url(base_url: String, relative_url: String) -> Result<String, String> {
    gemini::resolve_gemini_url(&base_url, &relative_url).map_err(|e| e.to_string())
}

// ===== Gopher-commands =====

/// Henter og rendrer innhold fra en Gopher-URL
///
/// # Arguments
/// * `url` - Gopher-URL å hente (gopher://...)
///
/// # Returns
/// RenderedPage med konvertert gophermap→markdown→HTML, eller feilmelding
#[tauri::command]
pub async fn fetch_gopher(url: String, window: tauri::Window) -> Result<RenderedPage, String> {
    // Sjekk cache først
    {
        let mut cache = RENDER_CACHE.lock().unwrap();
        if let Some(cached) = cache.get(&url) {
            debug!("Cache hit for Gopher URL: {}", url);
            let _ = window.emit("loading-status", "Dokument: Hentet fra cache");
            return Ok(cached.clone());
        }
    }

    let host = extract_host(&url);

    // Steg 1: Kobler til
    let _ = window.emit(
        "loading-status",
        format!("{} Gopher: Kobler til {} (port 70)...", EMOJI_GOPHER, host),
    );

    let result = gopher::fetch(&url).await;

    match result {
        Ok(response) => {
            let bytes = response.body.len();

            // Steg 2: Overfører data
            let _ = window.emit(
                "loading-status",
                format!("Overfører data... ({} bytes)", bytes),
            );

            match response.content_type {
                gopher::GopherContentType::Menu => {
                    // Steg 3: Konverterer gophermap
                    let _ = window.emit("loading-status", "Konverterer gophermap...");
                    let gophermap_result =
                        gophermap::to_markdown(&response.items, &response.final_url);

                    // Steg 4: Rendrer markdown
                    let _ = window.emit("loading-status", "Rendrer markdown...");
                    let html = markdown::render(&gophermap_result.markdown);

                    let title = gophermap_result
                        .title
                        .or_else(|| markdown::extract_title(&gophermap_result.markdown));

                    let _ = window.emit("loading-status", "Dokument: Ferdig");

                    let page = RenderedPage {
                        html,
                        title,
                        url: Some(response.final_url.clone()),
                        is_remote: true,
                        was_converted: true,
                    };

                    // Lagre i cache
                    {
                        let mut cache = RENDER_CACHE.lock().unwrap();
                        if response.final_url != url {
                            cache.put(response.final_url.clone(), page.clone());
                        }
                        cache.put(url, page.clone());
                    }

                    Ok(page)
                }
                gopher::GopherContentType::Text => {
                    // Steg 3: Rendrer tekst som markdown
                    let _ = window.emit("loading-status", "Rendrer markdown...");
                    let html = markdown::render(&response.body);
                    let title = markdown::extract_title(&response.body);

                    let _ = window.emit("loading-status", "Dokument: Ferdig");

                    let page = RenderedPage {
                        html,
                        title,
                        url: Some(response.final_url.clone()),
                        is_remote: true,
                        was_converted: false,
                    };

                    // Lagre i cache
                    {
                        let mut cache = RENDER_CACHE.lock().unwrap();
                        if response.final_url != url {
                            cache.put(response.final_url.clone(), page.clone());
                        }
                        cache.put(url, page.clone());
                    }

                    Ok(page)
                }
                gopher::GopherContentType::Html => {
                    // Konverter HTML til markdown
                    let _ = window.emit("loading-status", "Konverterer HTML til markdown...");
                    // Hent readability-innstilling
                    let settings = SETTINGS.lock().unwrap();
                    let readability_enabled = settings.readability_enabled;
                    drop(settings);

                    let conversion_result = converter::html_to_markdown(
                        &response.body,
                        Some(&response.final_url),
                        readability_enabled,
                    );

                    let _ = window.emit("loading-status", "Rendrer markdown...");
                    let html = markdown::render(&conversion_result.markdown);
                    let title = conversion_result
                        .title
                        .or_else(|| markdown::extract_title(&conversion_result.markdown));

                    let _ = window.emit("loading-status", "Dokument: Ferdig");

                    let page = RenderedPage {
                        html,
                        title,
                        url: Some(response.final_url.clone()),
                        is_remote: true,
                        was_converted: true,
                    };

                    // Lagre i cache
                    {
                        let mut cache = RENDER_CACHE.lock().unwrap();
                        if response.final_url != url {
                            cache.put(response.final_url.clone(), page.clone());
                        }
                        cache.put(url, page.clone());
                    }

                    Ok(page)
                }
                gopher::GopherContentType::Error => {
                    // Vis feilmeny som markdown
                    let _ = window.emit("loading-status", "Konverterer feilmelding...");
                    let gophermap_result =
                        gophermap::to_markdown(&response.items, &response.final_url);
                    let html = markdown::render(&gophermap_result.markdown);

                    let _ = window.emit("loading-status", "Dokument: Ferdig");

                    let page = RenderedPage {
                        html,
                        title: Some("Gopher-feil".to_string()),
                        url: Some(response.final_url.clone()),
                        is_remote: true,
                        was_converted: true,
                    };

                    // Lagre i cache
                    {
                        let mut cache = RENDER_CACHE.lock().unwrap();
                        if response.final_url != url {
                            cache.put(response.final_url.clone(), page.clone());
                        }
                        cache.put(url, page.clone());
                    }

                    Ok(page)
                }
                gopher::GopherContentType::Search => {
                    // Bør ikke skje — search håndteres via SearchInputRequired error
                    Err("GOPHER_SEARCH_PROMPT:".to_string() + &url)
                }
            }
        }
        Err(gopher::GopherError::SearchInputRequired) => {
            let _ = window.emit("loading-status", "Venter på søkeinput...");
            Err(format!("GOPHER_SEARCH_PROMPT:{}", url))
        }
        Err(e) => {
            let _ = window.emit("loading-status", "Feil under henting");
            Err(e.to_string())
        }
    }
}

/// Utfører et Gopher-søk (type 7)
///
/// # Arguments
/// * `url` - Gopher-søke-URL
/// * `query` - Brukerens søkestreng
///
/// # Returns
/// RenderedPage med søkeresultater
#[tauri::command]
pub async fn gopher_search(
    url: String,
    query: String,
    window: tauri::Window,
) -> Result<RenderedPage, String> {
    let host = extract_host(&url);

    let _ = window.emit(
        "loading-status",
        format!("{} Gopher: Søker på {}...", EMOJI_GOPHER, host),
    );

    let result = gopher::search(&url, &query)
        .await
        .map_err(|e| e.to_string())?;

    let bytes = result.body.len();
    let _ = window.emit(
        "loading-status",
        format!("Overfører data... ({} bytes)", bytes),
    );

    let _ = window.emit("loading-status", "Konverterer søkeresultater...");
    let gophermap_result = gophermap::to_markdown(&result.items, &result.final_url);

    let _ = window.emit("loading-status", "Rendrer markdown...");
    let html = markdown::render(&gophermap_result.markdown);

    let title = gophermap_result
        .title
        .or_else(|| Some(format!("Søkeresultater: {}", query)));

    let _ = window.emit("loading-status", "Dokument: Ferdig");

    let page = RenderedPage {
        html,
        title,
        url: Some(result.final_url.clone()),
        is_remote: true,
        was_converted: true,
    };

    // Lagre i cache
    {
        let mut cache = RENDER_CACHE.lock().unwrap();
        if result.final_url != url {
            cache.put(result.final_url.clone(), page.clone());
        }
        cache.put(url, page.clone());
    }

    Ok(page)
}

/// Løser en relativ URL mot en Gopher base-URL
///
/// # Arguments
/// * `base_url` - Nåværende Gopher-side sin URL
/// * `relative_url` - Relativ URL som skal løses
///
/// # Returns
/// Absolutt URL
#[tauri::command]
pub fn resolve_gopher_url(base_url: String, relative_url: String) -> Result<String, String> {
    gopher::resolve_gopher_url(&base_url, &relative_url).map_err(|e| e.to_string())
}

/// Returnerer filosofi-innhold for når appen starter
#[tauri::command]
pub fn get_welcome_content() -> RenderedPage {
    let philosophy_md = r#"# Philosophy

> *"There is an unresolved tension between the sender and recipient of information — who is to be in charge of the final form presentation? HTML clearly champions the recipient."*
> — Solvoll, Ivarsøy, Lie & Dybvik, *Telektronikk* 4/93

Bare is the answer to a question the web has spent thirty years trying to forget: **when you open a document, who is in charge — the person who wrote it, or the person reading it?**

In 1993, four researchers at Norwegian Telecom Research wrote that the young World Wide Web "championed the recipient." The reader decided how a page looked.
 One of those researchers, Håkon Wium Lie, would propose CSS two years later — and the balance began tilting back toward the author. Three decades on, it has tilted so far that the reader has all but disappeared beneath layout, scripts, pop-ups, and surveillance.

Bare takes the 1993 position and refuses to compromise on it: **the reader is sovereign.**

---

## The reader is sovereign

The author supplies meaning. *You* supply the presentation.

Theme, typography, line width, zoom level, whether images load at all — these are your decisions, made once and honored on every page you visit. A document author can tell you what they mean, but they cannot dictate the font you read it in, hijack your scroll, or decide that you must see their advertising to reach their words.

This is not a "reader mode" you toggle on for the rare unbearable page. In Bare it is the *only* mode. Every document, from every source, arrives in one consistent, legible form that you control.

---

## Simplicity is the winning strategy, not a sacrifice

The history of document formats is a history of simplicity defeating power.

In the early 1990s, the rich and ambitious **ODA** (Office Document Architecture) standard competed with the humble **SGML**, and SGML's simplest application — **HTML** — won the web outright. It won precisely *because* it was easy to implement and easy to read. As the 1993 paper concluded: the simplest format that is "good enough" wins, every time.

```
ODA  →  SGML  →  HTML  →  Markdown
(rich, complex)        (simple, human-readable)
```

Bare bets that the same evolution is happening again. HTML has become the new ODA: technically universal, but bloated past the point of usefulness. **Markdown is the next "good enough" format** — readable as plain text, trivial to parse, impossible to weaponize. Bare is built on that bet.

Fewer moving parts is itself the feature:

- **Fewer features** → fewer bugs → more stability
- **Less code** → faster rendering → a calmer experience
- **A smaller surface** → less to attack, less to track, less to break

---

## Privacy is the architecture, not a checkbox

Most browsers treat privacy as a setting — something you can enable, forget, misconfigure, or have silently overridden by a website. Bare treats it as a structural property that cannot be switched off, because the capabilities that enable tracking simply do not exist.

| Capability | Status in Bare | Why it matters |
| JavaScript | Not supported | No scripts means no behavioral tracking and no malware execution |
| Cookies | Not supported | Nothing can persist a unique identifier between visits |
| Remote CSS / fonts | Not loaded | Closes the door on CSS fingerprinting and font enumeration |
| Images | Blocked by default | Tracking pixels never fire unless *you* choose to load them |
| External requests | Zero by default | One click fetches one document — and nothing else |

The Gemini protocol community calls this principle **"break all loops"**: a design where nothing a server sends can ever make its way back to that server to re-identify you. Bare applies the same logic to the whole browsing experience.

You never have to wonder whether your privacy is on. It is the only state the program can be in.

---

## Non-extensibility is a promise

The web became a surveillance platform not through any single decision, but through *extensibility*. HTML and HTTP were designed to be easy to add to, and so — feature by reasonable-sounding feature — they grew until the document-reading tool became a general-purpose computing platform that runs untrusted code on your machine by default.

Bare makes the opposite promise. There is deliberately **no plugin system, no scripting hook, no mechanism for a page to extend what the browser can do.** A document cannot ask Bare to connect somewhere else, run a computation, or store state. This is not a missing feature; it is the central guarantee. A tool that cannot be extended cannot be slowly corrupted into something that works against you.

---

## What Bare refuses — and why

Saying "no" clearly is how Bare stays true to its purpose:

- **No JavaScript.** The single largest source of tracking, fingerprinting, and attack surface on the web. Removing it removes the problem at the root.
- **No author-controlled styling.** Presentation belongs to the reader. A document with bad contrast or a hostile layout is the author's failure to impose, not yours to suffer.
- **No editing or publishing.** Bare is a reading instrument. It does one thing and gets out of the way.
- **No telemetry, ever.** Bare does not phone home. There is no "anonymized usage data," because the most private data is the data that is never collected.

These refusals are not limitations to apologize for. They are the entire point — the things that make Bare *Bare*.

---

## A different kind of internet

The Gemini FAQ puts it well: browsing should feel "more like browsing a library than wandering through a shopping mall or a casino." A library makes a world of material available and then leaves you alone with it. Nobody follows you between the shelves. Nobody redecorates the book while you read. Nobody reports your borrowing habits to a marketing department.

That is the internet Bare is trying to give back to you — one document at a time, on your terms.

---

## Related reading

- [About Bare](https://frankburmo.github.io/bare/sider/about.md) — what the browser is and who it's for
- [Technology](https://frankburmo.github.io/bare/sider/technology.md) — how these principles are enforced in code
- [History](https://frankburmo.github.io/bare/sider/history.md) — the lineage Bare belongs to, from MultiTorg to Gemini

---

[Back to Home](https://frankburmo.github.io/bare/index.md)"#;

    let html = markdown::render(philosophy_md);
    let title = markdown::extract_title(philosophy_md);

    RenderedPage {
        html,
        title,
        url: None,
        is_remote: false,
        was_converted: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_markdown_command() {
        let result = render_markdown("# Test".to_string());
        assert!(result.html.contains("<h1"));
        assert_eq!(result.title, Some("Test".to_string()));
        assert!(!result.is_remote);
    }

    #[test]
    fn test_get_welcome_content() {
        let result = get_welcome_content();
        assert!(result.html.contains("Philosophy"));
        assert!(result.title.is_some());
        assert!(!result.is_remote);
    }

    #[test]
    fn test_resolve_url_command() {
        let result = resolve_url(
            "https://example.com/docs/readme.md".to_string(),
            "other.md".to_string(),
        );
        assert_eq!(result.unwrap(), "https://example.com/docs/other.md");
    }
}
