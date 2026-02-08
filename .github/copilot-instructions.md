# GitHub Copilot Instructions for Bare

## Prosjektoversikt

**Bare** er en eksperimentell markdown-nettleser med fokus på personvern, hastighet og rent innhold. Nettleseren ignorerer tradisjonelle nettsider og rendrer kun `.md`-filer direkte fra HTTP-responser.

### Lisens
- Prosjektet er lisensiert under GNU General Public License v3.0 (GPL-3.0).
- Bidrag til repoet skal kompatible med GPL-3.0.

### Kjernefilosofi
- **Enkelhet:** Minimal funksjonalitet, ingen unødvendig kompleksitet
- **Personvern:** Null sporing, ingen cookies, ingen JavaScript-kjøring
- **Hastighet:** Lav båndbredde, raske lastetider, effektiv rendering
- **Fokus:** Innholdet er i sentrum, ikke designet

## Teknisk Stack

| Komponent | Teknologi | Versjon |
|-----------|-----------|---------|
| App-rammeverk | Tauri | 2.0+ |
| Backend | Rust | Latest stable |
| Markdown parser | pulldown-cmark | Latest |
| HTTP-klient | reqwest | Latest |
| HTML→MD | html2md | Latest |
| Frontend | Vanilla HTML/CSS/JS | - |

## Kodestandard og Prinsipper

### Rust-kode

**Stil:**
```rust
// Bruk idiomatisk Rust
// Foretrekk Result<T, E> over panics
// Bruk ? for error propagation
// Dokumenter alle public APIs med ///

/// Henter en markdown-fil fra en URL
/// 
/// # Arguments
/// * `url` - URL til markdown-filen
/// 
/// # Returns
/// * `Ok(String)` - Innholdet i markdown-filen
/// * `Err(FetchError)` - Hvis nedlasting feiler
pub async fn fetch_markdown(url: &str) -> Result<String, FetchError> {
    let response = reqwest::get(url).await?;
    let content = response.text().await?;
    Ok(content)
}
```

**Prinsipper:**
- Bruk `async/await` for I/O-operasjoner
- Unngå `unwrap()` i produksjonskode, bruk `?` eller `.ok()` + håndtering
- Foretrekk `&str` over `String` for parametere når mulig
- Bruk `Result` og `Option` eksplisitt, ikke implisitt
- Skriv tester for all kjernelogikk

### Sikkerhet og Personvern

**Kritiske regler:**
- ❌ ALDRI implementer cookie-støtte
- ❌ ALDRI implementer JavaScript-kjøring i WebView
- ❌ ALDRI send brukerdata til eksterne tjenester uten eksplisitt samtykke
- ✅ Alltid sanitize HTML før konvertering til markdown
- ✅ Alltid valider URLer før HTTP-forespørsler
- ✅ Bruk HTTPS som standard, advare ved HTTP

**Eksempel på URL-validering:**
```rust
fn validate_url(url: &str) -> Result<Url, ValidationError> {
    let parsed = Url::parse(url)?;
    
    // Kun HTTP/HTTPS
    match parsed.scheme() {
        "http" | "https" => Ok(parsed),
        _ => Err(ValidationError::UnsupportedScheme),
    }
}
```

### Markdown-rendering

**Prinsipper:**
- Bruk pulldown-cmark for all markdown-parsing
- Støtt CommonMark + GitHub Flavored Markdown (tabeller, task lists)
- Konverter markdown til ren HTML for visning i WebView
- Ingen inline JavaScript i generert HTML
- Minimal CSS, fokus på lesbarhet

**Eksempel:**
```rust
use pulldown_cmark::{Parser, Options, html};

fn render_markdown(content: &str) -> String {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TASKLISTS);
    
    let parser = Parser::new_ext(content, options);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);
    
    html_output
}
```

### HTML-til-Markdown Konvertering

**Retningslinjer:**
- Bruk html2md for konvertering
- Fjern scripts, style tags, og tracking-elementer før konvertering
- Bevar semantisk struktur (headings, lists, links)
- Håndter bilder basert på brukerprefereranser

**Eksempel:**
```rust
fn convert_html_to_markdown(html: &str) -> String {
    // Sanitize først
    let clean_html = sanitize_html(html);
    
    // Konverter
    html2md::parse_html(&clean_html)
}

fn sanitize_html(html: &str) -> String {
    // Fjern <script>, <style>, tracking pixels, etc.
    // Implementer med ammonia eller lignende
    todo!()
}
```

### Bildehåndtering

**Nåværende implementasjon (Fase 1):**
- Bilder vises inline som standard
- Ingen spesiell håndtering - bruker standard HTML `<img>` tags

**Fremtidig arkitektur (planlagt):**
```rust
/// Brukerpreferanse for bildehåndtering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImageMode {
    /// Vis bilder inline (standard)
    Show,
    /// Skjul alle bilder
    Hide,
    /// Vis placeholder med alt-tekst, klikk for å laste
    Placeholder,
}

/// Konfigurasjon for bildevisning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageSettings {
    /// Global innstilling
    pub global_mode: ImageMode,
    /// Per-side overrides (URL -> modus)
    pub site_overrides: HashMap<String, ImageMode>,
}
```

**UI-plan for brukervalg:**
- Toolbar-knapp for å toggle bildmodus på gjeldende side
- Innstillinger-panel for global preferanse
- Lagres i `~/.config/bare/settings.json` (eller tilsvarende per OS)

## CI/CD

### GitHub Actions

Prosjektet bruker GitHub Actions for automatisert bygg og testing.

**Workflows:**

| Fil | Trigger | Beskrivelse |
|-----|---------|-------------|
| `.github/workflows/ci.yml` | Push/PR til main | Tester og linting |
| `.github/workflows/build.yml` | Release tags | Bygg for alle plattformer |

**Dette MÅ du sjekke hver gang du endrer koden, fiks eventuelle feil:**
```bash
# Formatering
cargo fmt --check

# Linting
cargo clippy -- -D warnings

# Tester
cargo test

# Bygg-verifisering
cargo tauri build
```

**Når du legger til ny funksjonalitet:**
- Skriv tester som kjører i CI
- Alle tester kjører uten feil
- Sørg for at `cargo clippy` passerer uten warnings
- Formater kode med `cargo fmt`
- cargo build --release bygger uten feilmeldinger


## Filstruktur

```
bare/
├── src-tauri/
│   ├── src/
│   │   ├── main.rs           # Tauri entrypoint
│   │   ├── lib.rs            # Library root
│   │   ├── commands.rs       # Tauri IPC commands
│   │   ├── markdown.rs       # Markdown rendering
│   │   ├── fetcher.rs        # HTTP client logic
│   │   ├── converter.rs      # HTML→MD conversion
│   │   ├── settings.rs       # Settings management
│   │   └── bookmarks.rs      # Bookmarks management
│   ├── Cargo.toml
│   └── tauri.conf.json
├── src/
│   ├── index.html            # Main UI
│   ├── styles.css            # Minimal styling
│   └── js/                   # Modularisert frontend
│       ├── constants.js      # Konstanter og konfigurasjoner
│       ├── state.js          # State management (history, bookmarks, settings)
│       ├── dom.js            # DOM-elementer og utilities
│       ├── ui.js             # UI-funksjoner (status, panels, dialogs)
│       ├── settings.js       # Innstillingshåndtering
│       ├── bookmarks.js      # Bokmerkehåndtering
│       ├── search.js         # Søkefunksjonalitet
│       ├── navigation.js     # Navigasjon og URL-håndtering
│       ├── events.js         # Event listeners og shortcuts
│       └── main.js           # Entry point og initialisering
├── .github/
│   └── copilot-instructions.md
├── README.md
├── PLAN.md
└── LICENSE
```

## Tauri-spesifikke Retningslinjer

### Commands (IPC)

**Definisjon:**
```rust
#[tauri::command]
async fn fetch_and_render(url: String) -> Result<RenderedPage, String> {
    let content = fetch_markdown(&url)
        .await
        .map_err(|e| e.to_string())?;
    
    let html = render_markdown(&content);
    
    Ok(RenderedPage { html, title: extract_title(&content) })
}
```

**Bruk fra frontend:**
```javascript
import { invoke } from '@tauri-apps/api/core';

async function loadPage(url) {
    try {
        const result = await invoke('fetch_and_render', { url });
        displayContent(result.html);
    } catch (error) {
        showError(error);
    }
}
```

### Sikkerhetskonfigurasjon

I `tauri.conf.json`:
```json
{
  "security": {
    "csp": "default-src 'none'; style-src 'self' 'unsafe-inline'; img-src 'self' data:",
    "dangerousDisableAssetCspModification": false
  },
  "allowlist": {
    "all": false,
    "http": {
      "all": false,
      "request": true,
      "scope": ["http://**", "https://**"]
    }
  }
}
```

## Frontend (HTML/CSS/JS)

### Prinsipper
- Vanilla JavaScript, ingen frameworks
- Modularisert arkitektur med klare avhengigheter
- Minimal CSS, fokus på typografi og lesbarhet
- Keyboard-first navigation
- Responsive design

### JavaScript-arkitektur

**Moduler lastes i avhengighetsrekkefølge:**
1. `constants.js` - Konstanter (ingen avhengigheter)
2. `state.js` - State management (bruker konstanter)
3. `dom.js` - DOM-elementer og utilities
4. `ui.js` - UI-funksjoner (bruker state, dom, constants)
5. `settings.js` - Innstillinger (bruker state, ui)
6. `bookmarks.js` - Bokmerker (bruker state, ui)
7. `search.js` - Søk (bruker state, dom)
8. `navigation.js` - Navigasjon (bruker alt over)
9. `events.js` - Event handlers (bruker alt over)
10. `main.js` - Entry point (orkestrerer alt)

**Designprinsipper:**
- Ingen globale klasser, kun funksjoner og globale variabler
- Klare navnekonvensjoner (f.eks. `elements.btnMenu`, `toggleDropdownMenu()`)
- Separasjon av concerns (UI, state, navigation, events)

### UI-komponenter

**Toolbar:**
- URL-bar med Enter-to-submit
- Navigasjonsknapper (back, forward, home)
- Bokmerkeknapper (add, view list)
- Åpne fil-knapp
- Innstillinger-knapp
- **3-prikks meny** med mindre brukte funksjoner:
  - Zoom-kontroller (−/+)
  - Bytt tema
  - Om-dialog

**Side-paneler:**
- Bokmerker-panel (høyre side)
- Innstillinger-panel (høyre side)
- Kan kun ha ett panel åpent om gangen

**Modaler:**
- Om-dialog med versjonsinformasjon og app-beskrivelse
- Overlay med klikk-utenfor-for-å-lukke

**Søkebar:**
- Toggle med Ctrl+F
- Inline highlight av søkeresultater
- Navigasjon mellom treff

**Markdown Viewport:**
- Ingen custom scrollbars (bruk native)
- Systemfonter for optimal lesbarhet
- Lys/mørk modus basert på system-preferanser

### Keyboard Shortcuts

| Shortcut | Handling |
|----------|----------|
| `Ctrl+O` | Åpne fil |
| `Ctrl+F` | Søk i side |
| `Ctrl+D` | Toggle bokmerke |
| `Ctrl+B` | Vis bokmerker |
| `Ctrl++` | Zoom inn |
| `Ctrl+-` | Zoom ut |
| `Ctrl+0` | Tilbakestill zoom |
| `Ctrl+L` | Fokuser URL-bar |
| `Alt+←` | Tilbake |
| `Alt+→` | Fremover |
| `Escape` | Lukk paneler/dialogs/søk |
| `g` | Gå hjem (ikke i input) |
| `j/k` | Scroll ned/opp (Vim-stil) |
| `G` | Scroll til bunnen |

## Testing

### Unit Tests (Rust)
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_markdown_rendering() {
        let input = "# Hello\n\nThis is **bold**.";
        let output = render_markdown(input);
        assert!(output.contains("<h1>Hello</h1>"));
        assert!(output.contains("<strong>bold</strong>"));
    }
    
    #[tokio::test]
    async fn test_fetch_markdown() {
        // Mock HTTP client eller bruk wiremock
    }
}
```

### Integration Tests
- Test komplett flyt: URL input → fetch → render → display
- Mock eksterne HTTP-kall
- Test feilhåndtering (404, timeout, invalid URL)

## Error Handling

**Definér custom error types:**
```rust
#[derive(Debug, thiserror::Error)]
pub enum BareError {
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),
    
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),
    
    #[error("Conversion error: {0}")]
    Conversion(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
```

**Bruk i hele kodebasen:**
```rust
pub type BareResult<T> = Result<T, BareError>;

pub async fn process_url(url: &str) -> BareResult<String> {
    let validated = validate_url(url)?;
    let content = fetch_content(&validated).await?;
    let markdown = convert_if_needed(content)?;
    Ok(markdown)
}
```

## Performance

### Optimalisering
- Cache konverterte sider i minnet (LRU cache)
- Stream store filer i stedet for å laste alt i minnet
- Bruk async/await for ikke-blokkerende I/O
- Lazy-load bilder hvis støtte legges til

### Minne
- Begrens history stack (f.eks. siste 50 sider)
- Rydd opp i cache regelmessig
- Unngå minnelekkasjer i WebView

## Tilgjengelighet

- Semantisk HTML i rendret markdown
- Keyboard navigation (Tab, Shift+Tab, Enter)
- Screen reader-vennlig
- Høy kontrast i mørk/lys modus
- Skalerbar tekst

## Debugging

**Logging:**
```rust
use log::{info, warn, error, debug};

pub async fn fetch_content(url: &Url) -> BareResult<String> {
    info!("Fetching content from: {}", url);
    
    let response = reqwest::get(url.as_str()).await?;
    let status = response.status();
    
    if !status.is_success() {
        warn!("Non-success status: {} for {}", status, url);
    }
    
    let content = response.text().await?;
    debug!("Fetched {} bytes", content.len());
    
    Ok(content)
}
```

**Tauri DevTools:**
- Bruk `tauri dev` for hot-reload
- Åpne DevTools i WebView for frontend-debugging
- Bruk `dbg!()` macro for quick Rust-debugging

## Bidrag

Når du foreslår kode:
- Følg eksisterende kodestruktur
- Skriv tester for ny funksjonalitet
- Dokumenter public APIs
- Vurder personvern og sikkerhet først
- Hold det enkelt - ikke legg til unødvendig kompleksitet

## Hva Bare IKKE skal være

- ❌ En fullverdig nettleser (vi vil aldri støtte JavaScript)
- ❌ En HTML-renderer (kun markdown er førsteklasses)
- ❌ Et sosiale medie-verktøy (ingen commenting, sharing, etc.)
- ❌ En tekstbehandler (kun visning, ikke redigering)

## Fremtidige Utvidelser (Vurder disse)

- ✅ Gemini-protokoll støtte (gemini://) - **IMPLEMENTERT i v0.1.3**
- ⚠️ Gopher-protokoll støtte (gopher://)
- ✅ Lokale markdown-filer (file://)
- ⚠️ Eksport til PDF
- ⚠️ Custom themes/CSS
- ⚠️ Tab-støtte (vurder kompleksitet vs. nytte)
- ⚠️ Synkronisering (kun hvis lokalt, ikke cloud)

## Gemini-protokoll Håndtering

**Implementasjon (v0.1.3):**

Bare støtter nå fullstendig Gemini-protokollen med følgende arkitektur:

### Rust Backend

**gemini.rs:**
```rust
// Hovedstrukturer
pub struct GeminiClient {
    tls_config: Arc<ClientConfig>,
    tofu_store: Arc<Mutex<TofuStore>>,
}

pub struct GeminiResponse {
    pub status: u8,
    pub meta: String,
    pub body: String,
    pub final_url: String,
}
```

**Nøkkelprinsipper:**
- TOFU (Trust On First Use) for TLS-sertifikater
- SHA-256 fingerprint-lagring i `~/.config/bare/known_hosts.json`
- Iterativ redirect-håndtering (maks 5 redirects)
- Alle statuskoder håndteres: 10-11 (input), 20 (success), 30-31 (redirect), 40-69 (errors)
- Timeout: 10 sekunder
- Maksimal respons: 5 MB
- Custom TofuVerifier som implementerer ServerCertVerifier

**gemtext.rs:**
```rust
pub fn gemtext_to_markdown(input: &str) -> GemtextResult {
    // Linje-basert parsing
    // => url text → [text](url)
    // * item → - item
    // ### heading → ### heading
    // ``` preformatted blocks → ``` code blocks
}
```

**Konverteringsregler:**
- Link-linjer: `=> gemini://example.com/page Example` → `[Example](gemini://example.com/page)`
- List-items: `* Item` → `- Item`
- Headings: Direkte pass-through (`# `, `## `, `### `)
- Preformatted: ` ```alt-text ` → ` ```alt-text ` (bevares)
- Blockquotes: `> text` → `> text` (bevares)

### Frontend

**navigation.js:**
```javascript
async function loadGeminiUrl(url, addHistory = true) {
    const result = await invoke('fetch_gemini', { url, window: appWindow });
    
    if (result.startsWith('GEMINI_INPUT_PROMPT:')) {
        // Vis input-dialog
        showGeminiInputDialog(prompt, url, sensitive);
    } else {
        // Render innhold
        displayContent(result);
    }
}
```

**Gemini Input Dialog:**
- Modal overlay med prompt-tekst fra server
- Støtte for sensitive input (status 11 = passord-type)
- Enter-to-submit, Escape-to-cancel
- Automatisk fokus på input-felt

### Sikkerhetsvurderinger

**TOFU-implementasjon:**
- Første besøk: Lagre sertifikat-fingerprint
- Påfølgende besøk: Verifiser mot lagret fingerprint
- Endring: Vis feilmelding til bruker (mulig MITM-angrep)
- Ikke CA-basert PKI (i tråd med Gemini-filosofien)

**Validering:**
```rust
fn validate_gemini_url(url: &str) -> Result<Url, GeminiError> {
    let parsed = Url::parse(url)?;
    
    if parsed.scheme() != "gemini" {
        return Err(GeminiError::InvalidUrl("Not a gemini URL".into()));
    }
    
    if url.len() > MAX_URL_LENGTH {
        return Err(GeminiError::InvalidUrl("URL too long".into()));
    }
    
    Ok(parsed)
}
```

### Testing

**Unit tests:**
- gemini.rs: 19 tests (URL-validering, response-parsing, TOFU-logikk, redirect-håndtering)
- gemtext.rs: 17 tests (alle gemtext-elementer, edge cases)
- fetcher.rs: 4 nye tests (gemini:// URL-håndtering)

**Manuelle test-scenarier:**
1. Naviger til `gemini://geminiprotocol.net/`
2. Klikk på lenker innenfor Gemini-space
3. Test input-forespørsler (status 10)
4. Verifiser TOFU-lagring i `~/.config/bare/known_hosts.json`
5. Test cross-protokoll navigasjon (gemini → http og omvendt)
6. Test back/forward mellom protokoller

### Feilhåndtering

```rust
#[derive(Debug, thiserror::Error)]
pub enum GeminiError {
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),
    
    #[error("TLS error: {0}")]
    TlsError(String),
    
    #[error("Certificate changed for {0} - possible MITM attack")]
    CertificateChanged(String),
    
    #[error("Input required: {0}")]
    InputRequired(String),
    
    #[error("Redirect loop detected")]
    RedirectLoop,
    
    // ... flere error-typer
}
```

**Brukervendte feilmeldinger:**
- Norsk språk i UI
- Tydelige forklaringer på TOFU-brudd
- Valgfrie handlinger (f.eks. "Gå tilbake" vs "Fortsett likevel")

## Nyttige Kommandoer

```bash
# Utvikle
cargo tauri dev

# Bygg for produksjon
cargo tauri build

# Run Rust tests
cargo test

# Format kode
cargo fmt

# Linting
cargo clippy

# Sjekk for sikkerhetssårbarheter
cargo audit
```

---

**Husk:** Når du jobber på Bare, prioriter alltid **personvern** og **enkelhet** over funksjonalitet. Hvis et feature krever kompromiss på kjerneverdiene, så er det ikke riktig for Bare.
