//! HTTP/HTTPS fetching modul
//!
//! Håndterer nettverksforespørsler for å hente markdown-filer fra internett.

use log::{debug, info, warn};
use reqwest::header::{HeaderMap, CONTENT_TYPE, USER_AGENT};
use std::time::Duration;
use thiserror::Error;
use url::Url;

/// Feil som kan oppstå under henting av innhold
#[derive(Debug, Error)]
pub enum FetchError {
    #[error("Ugyldig URL: {0}")]
    InvalidUrl(String),

    #[error("Ustøttet protokoll: {0}. Kun http og https støttes.")]
    UnsupportedScheme(String),

    #[error("Nettverksfeil: {0}")]
    Network(#[from] reqwest::Error),

    #[error("Ressurs ikke funnet (404): {0}")]
    NotFound(String),

    #[error("Server-feil ({0}): {1}")]
    ServerError(u16, String),

    #[error("Timeout: Serveren svarte ikke innen {0} sekunder")]
    Timeout(u64),
}

/// Resultat fra en vellykket fetch-operasjon
#[derive(Debug)]
pub struct FetchResult {
    /// Innholdet som ble hentet
    pub content: String,
    /// Content-Type header fra responsen
    pub content_type: Option<String>,
    /// Den endelige URL-en (etter eventuelle redirects)
    pub final_url: String,
    /// Om innholdet er markdown
    pub is_markdown: bool,
}

/// HTTP-klient for Bare
pub struct Fetcher {
    client: reqwest::Client,
    timeout_seconds: u64,
}

impl Default for Fetcher {
    fn default() -> Self {
        Self::new()
    }
}

impl Fetcher {
    /// Opprett en ny Fetcher med standard innstillinger
    pub fn new() -> Self {
        Self::with_timeout(30)
    }

    /// Opprett en Fetcher med egendefinert timeout
    pub fn with_timeout(timeout_seconds: u64) -> Self {
        // Prøv først å bygge klient med custom user agent
        let user_agent = format!("Bare/{} (Markdown Browser)", env!("CARGO_PKG_VERSION"));

        let client = if let Ok(value) = user_agent.parse() {
            let mut headers = HeaderMap::new();
            headers.insert(USER_AGENT, value);

            reqwest::Client::builder()
                .timeout(Duration::from_secs(timeout_seconds))
                .default_headers(headers)
                .build()
                .ok()
        } else {
            None
        };

        // Fallback: bygg minimal klient uten headers
        let client = client.or_else(|| {
            warn!("Kunne ikke opprette HTTP-klient med headers. Prøver uten.");
            reqwest::Client::builder()
                .timeout(Duration::from_secs(timeout_seconds))
                .build()
                .ok()
        });

        // Siste fallback: helt minimal klient uten timeout
        let client = client.unwrap_or_else(|| {
            warn!("Kunne ikke opprette HTTP-klient med timeout. Bruker default.");
            reqwest::Client::new()
        });

        Self {
            client,
            timeout_seconds,
        }
    }

    /// Valider og parse en URL
    pub fn validate_url(url_str: &str) -> Result<Url, FetchError> {
        let parsed = Url::parse(url_str)
            .map_err(|e| FetchError::InvalidUrl(format!("{}: {}", url_str, e)))?;

        match parsed.scheme() {
            "http" | "https" => Ok(parsed),
            scheme => Err(FetchError::UnsupportedScheme(scheme.to_string())),
        }
    }

    /// Sjekk om en Content-Type indikerer markdown
    fn is_markdown_content_type(content_type: &str) -> bool {
        let ct_lower = content_type.to_lowercase();
        ct_lower.contains("text/markdown")
            || ct_lower.contains("text/x-markdown")
            || ct_lower.contains("text/plain")
    }

    /// Sjekk om URL-en peker til en markdown-fil basert på extension
    fn is_markdown_url(url: &Url) -> bool {
        url.path().to_lowercase().ends_with(".md")
            || url.path().to_lowercase().ends_with(".markdown")
    }

    /// Hent innhold fra en URL
    pub async fn fetch(&self, url_str: &str) -> Result<FetchResult, FetchError> {
        let url = Self::validate_url(url_str)?;
        info!("Fetching content from: {}", url);

        let response = self.client.get(url.as_str()).send().await.map_err(|e| {
            if e.is_timeout() {
                FetchError::Timeout(self.timeout_seconds)
            } else {
                FetchError::Network(e)
            }
        })?;

        let status = response.status();
        let final_url = response.url().to_string();

        debug!("Response status: {} for {}", status, final_url);

        if status.as_u16() == 404 {
            return Err(FetchError::NotFound(final_url));
        }

        if !status.is_success() {
            warn!("Non-success status: {} for {}", status, final_url);
            return Err(FetchError::ServerError(status.as_u16(), final_url.clone()));
        }

        // Hent Content-Type
        let content_type = response
            .headers()
            .get(CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        // Sjekk om det er markdown basert på URL eller Content-Type
        let is_markdown = Self::is_markdown_url(&url)
            || content_type
                .as_ref()
                .map(|ct| Self::is_markdown_content_type(ct))
                .unwrap_or(false);

        debug!(
            "Content-Type: {:?}, is_markdown: {}",
            content_type, is_markdown
        );

        let content = response.text().await.map_err(FetchError::Network)?;
        debug!("Fetched {} bytes", content.len());

        Ok(FetchResult {
            content,
            content_type,
            final_url,
            is_markdown,
        })
    }
}

/// Løs en relativ URL mot en base-URL
pub fn resolve_url(base: &str, relative: &str) -> Result<String, FetchError> {
    // Hvis det allerede er en absolutt URL, returner den
    if relative.starts_with("http://") || relative.starts_with("https://") {
        return Ok(relative.to_string());
    }

    // Håndter protokoll-relative URLer
    if relative.starts_with("//") {
        let base_url = Url::parse(base)
            .map_err(|e| FetchError::InvalidUrl(format!("Ugyldig base URL: {}", e)))?;
        return Ok(format!("{}:{}", base_url.scheme(), relative));
    }

    // Parse base URL og resolve relativ path
    let base_url =
        Url::parse(base).map_err(|e| FetchError::InvalidUrl(format!("Ugyldig base URL: {}", e)))?;

    let resolved = base_url
        .join(relative)
        .map_err(|e| FetchError::InvalidUrl(format!("Kunne ikke løse relativ URL: {}", e)))?;

    Ok(resolved.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_url_https() {
        let result = Fetcher::validate_url("https://example.com/test.md");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_url_http() {
        let result = Fetcher::validate_url("http://example.com/test.md");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_url_invalid_scheme() {
        let result = Fetcher::validate_url("ftp://example.com/test.md");
        assert!(matches!(result, Err(FetchError::UnsupportedScheme(_))));
    }

    #[test]
    fn test_validate_url_invalid() {
        let result = Fetcher::validate_url("not a url");
        assert!(matches!(result, Err(FetchError::InvalidUrl(_))));
    }

    #[test]
    fn test_is_markdown_content_type() {
        assert!(Fetcher::is_markdown_content_type("text/markdown"));
        assert!(Fetcher::is_markdown_content_type("text/x-markdown"));
        assert!(Fetcher::is_markdown_content_type(
            "text/plain; charset=utf-8"
        ));
        assert!(!Fetcher::is_markdown_content_type("text/html"));
        assert!(!Fetcher::is_markdown_content_type("application/json"));
    }

    #[test]
    fn test_resolve_url_absolute() {
        let result = resolve_url("https://example.com/docs/", "https://other.com/test.md");
        assert_eq!(result.unwrap(), "https://other.com/test.md");
    }

    #[test]
    fn test_resolve_url_relative_path() {
        let result = resolve_url("https://example.com/docs/readme.md", "other.md");
        assert_eq!(result.unwrap(), "https://example.com/docs/other.md");
    }

    #[test]
    fn test_resolve_url_relative_parent() {
        let result = resolve_url("https://example.com/docs/readme.md", "../other.md");
        assert_eq!(result.unwrap(), "https://example.com/other.md");
    }

    #[test]
    fn test_resolve_url_absolute_path() {
        let result = resolve_url("https://example.com/docs/readme.md", "/root.md");
        assert_eq!(result.unwrap(), "https://example.com/root.md");
    }

    #[test]
    fn test_resolve_url_protocol_relative() {
        let result = resolve_url("https://example.com/docs/", "//other.com/test.md");
        assert_eq!(result.unwrap(), "https://other.com/test.md");
    }
}
