//! Tauri commands for Bare
//!
//! IPC-kommandoer som kan kalles fra frontend.

use crate::bookmarks::{self, Bookmark, BookmarkStore};
use crate::converter;
use crate::fetcher::{self, Fetcher};
use crate::gemini::{self, GeminiClient, GeminiError};
use crate::gemtext;
use crate::markdown;
use crate::settings::{self, ConversionMode, FontFamily, Settings, Theme};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::{LazyLock, Mutex};
use tauri::Emitter;

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
pub fn open_file(path: String) -> Result<RenderedPage, String> {
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

    // Les innholdet
    let content = fs::read_to_string(&path).map_err(|e| format!("Kunne ikke lese fil: {}", e))?;

    // Render markdown
    let html = markdown::render(&content);
    let title = markdown::extract_title(&content);

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
    // Steg 1: Slår opp vert
    let _ = window.emit(
        "loading-status",
        format!("Slår opp {}...", extract_host(&url)),
    );

    // Steg 2: Kobler til
    let _ = window.emit(
        "loading-status",
        format!("Kobler til {}...", extract_host(&url)),
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

        return Ok(RenderedPage {
            html,
            title,
            url: Some(result.final_url),
            is_remote: true,
            was_converted: false,
        });
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
            let conversion_result = converter::html_to_markdown(&result.content);

            // Steg 5: Rendrer markdown
            let _ = window.emit("loading-status", "Rendrer markdown...");
            let html = markdown::render(&conversion_result.markdown);

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
    let _ = window.emit(
        "loading-status",
        format!("Slår opp {}...", extract_host(&url)),
    );
    let _ = window.emit(
        "loading-status",
        format!("Kobler til {}...", extract_host(&url)),
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

    // Konverter HTML til markdown
    let _ = window.emit("loading-status", "Konverterer HTML til markdown...");
    let conversion_result = converter::html_to_markdown(&result.content);

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
    pub onboarding_completed: bool,
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
            onboarding_completed: s.onboarding_completed,
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
    pub onboarding_completed: Option<bool>,
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

    if let Some(oc) = params.onboarding_completed {
        settings.onboarding_completed = oc;
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
    let host = extract_host(&url);

    // Steg 1: Kobler til
    let _ = window.emit("loading-status", format!("Kobler til {}:1965...", host));

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

                Ok(RenderedPage {
                    html,
                    title,
                    url: Some(response.final_url),
                    is_remote: true,
                    was_converted: true,
                })
            } else if response.meta.starts_with("text/") {
                // Ren tekst — vis som markdown-kodeblokk
                let _ = window.emit("loading-status", "Rendrer tekst...");
                let markdown_content = format!("```\n{}\n```", body);
                let html = markdown::render(&markdown_content);

                let _ = window.emit("loading-status", "Dokument: Ferdig");

                Ok(RenderedPage {
                    html,
                    title: None,
                    url: Some(response.final_url),
                    is_remote: true,
                    was_converted: true,
                })
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

/// Returnerer velkomst-innhold for når appen starter
#[tauri::command]
pub fn get_welcome_content() -> RenderedPage {
    let welcome_md = r#"# Velkommen til Bare

> "The internet doesn't have to be heavy."

**Bare** er en eksperimentell markdown-nettleser med fokus på:

- **Personvern** — Ingen cookies, ingen JavaScript, ingen sporing
- **Hastighet** — Lynrask lasting av rent innhold
- **Fokus** — Innholdet er i sentrum, ikke designet

## Kom i gang

### Åpne en lokal fil

Klikk på **Åpne fil** i verktøylinjen for å velge en `.md`-fil fra datamaskinen din.

### Skriv inn en URL

Skriv inn en URL til en markdown-fil i adressefeltet og trykk Enter.

### Gemini-protokollen

Bare støtter **Gemini-protokollen** — et enkelt og personvernvennlig alternativ til HTTP.

Prøv en av disse adressene:

- [gemini://geminiprotocol.net/](gemini://geminiprotocol.net/)
- [gemini://gemini.circumlunar.space/](gemini://gemini.circumlunar.space/)
- [gemini://geminiquickst.art/](gemini://geminiquickst.art/)

Gemini-sider bruker et enkelt format kalt gemtext, som automatisk konverteres til markdown.

## Eksempel på markdown

Her er noen eksempler på hva Bare kan vise:

### Tekst-formatering

- **Fet tekst** for viktige ting
- *Kursiv tekst* for vektlegging
- ~~Gjennomstreket~~ for ting som ikke gjelder lenger
- `Kode` for tekniske termer

### Lister

1. Nummererte lister
2. Fungerer også
3. Automatisk nummerering

### Oppgavelister

- [x] Sett opp Tauri-prosjekt
- [x] Implementer markdown-rendering
- [ ] Legg til nettverksstøtte
- [ ] Lag HTML-til-markdown konvertering

### Tabeller

| Funksjon | Status |
|----------|--------|
| Markdown-rendering | ✅ Ferdig |
| Lokale filer | ✅ Ferdig |
| Nettverksforespørsler | 🚧 Kommer |

### Kodeblokker

```rust
fn main() {
    println!("Hello, Bare!");
}
```

---

*Bare v{} — Laget med ❤️ for et enklere internett*
"#;

    let welcome_md = welcome_md.replace("{}", env!("CARGO_PKG_VERSION"));

    let html = markdown::render(&welcome_md);
    let title = markdown::extract_title(&welcome_md);

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
        assert!(result.html.contains("<h1>"));
        assert_eq!(result.title, Some("Test".to_string()));
        assert!(!result.is_remote);
    }

    #[test]
    fn test_get_welcome_content() {
        let result = get_welcome_content();
        assert!(result.html.contains("Velkommen til Bare"));
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
