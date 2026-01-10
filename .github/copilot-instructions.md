# GitHub Copilot Instructions for Bare

## Prosjektoversikt

**Bare** er en eksperimentell markdown-nettleser med fokus på personvern, hastighet og rent innhold. Nettleseren ignorerer tradisjonelle nettsider og rendrer kun `.md`-filer direkte fra HTTP-responser.

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

**CI sjekker:**
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
│   │   ├── commands.rs       # Tauri IPC commands
│   │   ├── markdown.rs       # Markdown rendering
│   │   ├── fetcher.rs        # HTTP client logic
│   │   ├── converter.rs      # HTML→MD conversion
│   │   ├── history.rs        # Navigation history
│   │   └── bookmarks.rs      # Bookmarks management
│   ├── Cargo.toml
│   └── tauri.conf.json
├── src/
│   ├── index.html            # Main UI
│   ├── styles.css            # Minimal styling
│   └── app.js                # Frontend logic
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
- Minimal CSS, fokus på typografi og lesbarhet
- Keyboard-first navigation
- Responsive design

### UI-komponenter

**URL Bar:**
```javascript
class URLBar {
    constructor() {
        this.input = document.getElementById('url-input');
        this.input.addEventListener('keypress', this.handleKeyPress.bind(this));
    }
    
    handleKeyPress(event) {
        if (event.key === 'Enter') {
            const url = this.input.value;
            loadPage(url);
        }
    }
}
```

**Markdown Viewport:**
- Ingen custom scrollbars (bruk native)
- Systemfonter for optimal lesbarhet
- Lys/mørk modus basert på system-preferanser

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

- ✅ Gemini-protokoll støtte (gemini://)
- ✅ Gopher-protokoll støtte (gopher://)
- ✅ Lokale markdown-filer (file://)
- ✅ Eksport til PDF
- ✅ Custom themes/CSS
- ⚠️ Tab-støtte (vurder kompleksitet vs. nytte)
- ⚠️ Synkronisering (kun hvis lokalt, ikke cloud)

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
