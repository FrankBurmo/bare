//! Tauri commands for Bare
//!
//! IPC-kommandoer som kan kalles fra frontend.

use crate::fetcher::{self, Fetcher};
use crate::markdown;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::LazyLock;

/// Global HTTP-klient (gjenbrukes for alle forespørsler)
static FETCHER: LazyLock<Fetcher> = LazyLock::new(Fetcher::new);

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
pub async fn fetch_url(url: String) -> Result<RenderedPage, String> {
    let result = FETCHER.fetch(&url).await.map_err(|e| e.to_string())?;

    if !result.is_markdown {
        // For nå, returner en feilmelding for ikke-markdown innhold
        // I Fase 3 vil vi konvertere HTML til markdown
        return Err(format!(
            "Innholdet er ikke markdown (Content-Type: {:?}). HTML-konvertering kommer i en fremtidig versjon.",
            result.content_type
        ));
    }

    let html = markdown::render(&result.content);
    let title = markdown::extract_title(&result.content);

    Ok(RenderedPage {
        html,
        title,
        url: Some(result.final_url),
        is_remote: true,
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

*Bare v0.1.0 — Laget med ❤️ for et enklere internett*
"#;

    let html = markdown::render(welcome_md);
    let title = markdown::extract_title(welcome_md);

    RenderedPage {
        html,
        title,
        url: None,
        is_remote: false,
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
