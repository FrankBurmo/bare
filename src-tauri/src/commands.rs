//! Tauri commands for Bare
//!
//! IPC-kommandoer som kan kalles fra frontend.

use crate::markdown;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Resultat fra markdown-rendering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderedPage {
    /// HTML-innhold
    pub html: String,
    /// Tittel ekstrahert fra markdown (hvis funnet)
    pub title: Option<String>,
}

/// Feil som kan oppstå
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BareError {
    pub message: String,
}

impl From<std::io::Error> for BareError {
    fn from(err: std::io::Error) -> Self {
        BareError {
            message: err.to_string(),
        }
    }
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

    RenderedPage { html, title }
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

    Ok(RenderedPage { html, title })
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

    RenderedPage { html, title }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_markdown_command() {
        let result = render_markdown("# Test".to_string());
        assert!(result.html.contains("<h1>"));
        assert_eq!(result.title, Some("Test".to_string()));
    }

    #[test]
    fn test_get_welcome_content() {
        let result = get_welcome_content();
        assert!(result.html.contains("Velkommen til Bare"));
        assert!(result.title.is_some());
    }
}
