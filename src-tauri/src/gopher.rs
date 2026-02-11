//! Gopher-protokoll klient
//!
//! Implementerer Gopher-protokollen (RFC 1436) med TCP-tilkobling.
//! Støtter menyer, tekstfiler, søk og HTML-lenker.

use log::{debug, info, warn};
use std::time::Duration;
use thiserror::Error;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use url::Url;

/// Standard Gopher-port
const DEFAULT_PORT: u16 = 70;

/// Maksimal respons-størrelse (5 MB)
const MAX_RESPONSE_SIZE: usize = 5 * 1024 * 1024;

/// Maksimal URL-lengde
const MAX_URL_LENGTH: usize = 1024;

/// Timeout i sekunder
const TIMEOUT_SECONDS: u64 = 10;

/// Feil som kan oppstå under Gopher-forespørsler
#[derive(Debug, Error)]
#[allow(dead_code)]
pub enum GopherError {
    #[error("Ugyldig URL: {0}")]
    InvalidUrl(String),

    #[error("Tilkoblingsfeil: {0}")]
    ConnectionError(String),

    #[error("Timeout: Serveren svarte ikke innen {0} sekunder")]
    Timeout(u64),

    #[error("Respons for stor (over {0} bytes)")]
    TooLarge(usize),

    #[error("I/O-feil: {0}")]
    Io(#[from] std::io::Error),

    #[error("Ugyldig respons fra serveren: {0}")]
    InvalidResponse(String),

    #[error("Serveren ber om søkeinput")]
    SearchInputRequired,
}

/// Gopher-elementtype hentet fra meny-respons
#[derive(Debug, Clone, PartialEq)]
pub enum GopherItemType {
    /// Tekstfil (type 0)
    TextFile,
    /// Mappe/meny (type 1)
    Directory,
    /// CSO-telefonbok (type 2)
    CsoPhonebook,
    /// Feilmelding (type 3)
    Error,
    /// BinHex-fil (type 4)
    BinHex,
    /// DOS-binærfil (type 5)
    DosBinary,
    /// UUencodet fil (type 6)
    UuEncoded,
    /// Søk (type 7)
    Search,
    /// Telnet-sesjon (type 8)
    Telnet,
    /// Binærfil (type 9)
    Binary,
    /// GIF-bilde (type g)
    Gif,
    /// Bilde generelt (type I)
    Image,
    /// HTML-fil (type h)
    Html,
    /// Informasjonstekst (type i)
    Info,
    /// Telnet 3270 (type T)
    Telnet3270,
    /// Ukjent type
    Unknown(char),
}

impl GopherItemType {
    /// Konverterer et tegn til GopherItemType
    pub fn from_char(c: char) -> Self {
        match c {
            '0' => GopherItemType::TextFile,
            '1' => GopherItemType::Directory,
            '2' => GopherItemType::CsoPhonebook,
            '3' => GopherItemType::Error,
            '4' => GopherItemType::BinHex,
            '5' => GopherItemType::DosBinary,
            '6' => GopherItemType::UuEncoded,
            '7' => GopherItemType::Search,
            '8' => GopherItemType::Telnet,
            '9' => GopherItemType::Binary,
            'g' => GopherItemType::Gif,
            'I' => GopherItemType::Image,
            'h' => GopherItemType::Html,
            'i' => GopherItemType::Info,
            'T' => GopherItemType::Telnet3270,
            c => GopherItemType::Unknown(c),
        }
    }

    /// Konverterer til type-tegn for URL-bygging
    pub fn to_char(&self) -> char {
        match self {
            GopherItemType::TextFile => '0',
            GopherItemType::Directory => '1',
            GopherItemType::CsoPhonebook => '2',
            GopherItemType::Error => '3',
            GopherItemType::BinHex => '4',
            GopherItemType::DosBinary => '5',
            GopherItemType::UuEncoded => '6',
            GopherItemType::Search => '7',
            GopherItemType::Telnet => '8',
            GopherItemType::Binary => '9',
            GopherItemType::Gif => 'g',
            GopherItemType::Image => 'I',
            GopherItemType::Html => 'h',
            GopherItemType::Info => 'i',
            GopherItemType::Telnet3270 => 'T',
            GopherItemType::Unknown(c) => *c,
        }
    }

    /// Beskrivelse av elementtypen
    pub fn description(&self) -> &str {
        match self {
            GopherItemType::TextFile => "Tekstfil",
            GopherItemType::Directory => "Mappe",
            GopherItemType::CsoPhonebook => "CSO-telefonbok",
            GopherItemType::Error => "Feil",
            GopherItemType::BinHex => "BinHex-fil",
            GopherItemType::DosBinary => "DOS-binærfil",
            GopherItemType::UuEncoded => "UUencodet fil",
            GopherItemType::Search => "Søk",
            GopherItemType::Telnet => "Telnet",
            GopherItemType::Binary => "Binærfil",
            GopherItemType::Gif => "GIF-bilde",
            GopherItemType::Image => "Bilde",
            GopherItemType::Html => "HTML",
            GopherItemType::Info => "Info",
            GopherItemType::Telnet3270 => "Telnet 3270",
            GopherItemType::Unknown(_) => "Ukjent",
        }
    }
}

/// Et enkelt element i en Gopher-meny
#[derive(Debug, Clone)]
pub struct GopherItem {
    /// Elementtype
    pub item_type: GopherItemType,
    /// Visningstekst
    pub display: String,
    /// Selektor (sti på serveren)
    pub selector: String,
    /// Vertsnavn
    pub host: String,
    /// Portnummer
    pub port: u16,
}

/// Parsed Gopher-URL
#[derive(Debug, Clone)]
pub struct GopherUrl {
    /// Vertsnavn
    pub host: String,
    /// Portnummer
    pub port: u16,
    /// Elementtype
    pub item_type: GopherItemType,
    /// Selektor
    pub selector: String,
}

/// Type innhold i responsen
#[derive(Debug)]
#[allow(dead_code)]
pub enum GopherContentType {
    /// Gopher-meny
    Menu,
    /// Ren tekst
    Text,
    /// HTML-innhold
    Html,
    /// Søk (input påkrevd)
    Search,
    /// Feilmelding
    Error,
}

/// Respons fra en Gopher-server
#[derive(Debug)]
pub struct GopherResponse {
    /// Type innhold
    pub content_type: GopherContentType,
    /// Rå respons-body
    pub body: String,
    /// Parsed meny-elementer (kun for menyer)
    pub items: Vec<GopherItem>,
    /// Den endelige URL-en
    pub final_url: String,
}

/// Parser en Gopher-URL til komponentene
///
/// Format: gopher://host[:port]/[type][selector]
///
/// Eksempler:
///   gopher://gopher.floodgap.com/          → host, port=70, type=1, selector=""
///   gopher://example.com/0/about.txt       → host, port=70, type=0, selector="/about.txt"
///   gopher://example.com:7070/1/docs       → host, port=7070, type=1, selector="/docs"
pub fn parse_gopher_url(url: &str) -> Result<GopherUrl, GopherError> {
    // Valider URL-lengde
    if url.len() > MAX_URL_LENGTH {
        return Err(GopherError::InvalidUrl("URL for lang".into()));
    }

    let parsed =
        Url::parse(url).map_err(|e| GopherError::InvalidUrl(format!("Ugyldig URL: {}", e)))?;

    // Kun gopher://-skjema
    if parsed.scheme() != "gopher" {
        return Err(GopherError::InvalidUrl("Ikke en gopher-URL".into()));
    }

    // Host er påkrevd
    let host = parsed
        .host_str()
        .ok_or_else(|| GopherError::InvalidUrl("Mangler host".into()))?
        .to_string();

    if host.is_empty() {
        return Err(GopherError::InvalidUrl("Mangler host".into()));
    }

    // Port (standard: 70)
    let port = parsed.port().unwrap_or(DEFAULT_PORT);

    // Parse sti for type og selektor
    let path = parsed.path();

    let (item_type, selector) = if path.is_empty() || path == "/" {
        // Rot-meny
        (GopherItemType::Directory, String::new())
    } else {
        // Fjern ledende '/'
        let path_without_slash = &path[1..];

        if path_without_slash.is_empty() {
            (GopherItemType::Directory, String::new())
        } else {
            // Første tegn er typen
            let type_char = path_without_slash.chars().next().unwrap();
            let item_type = GopherItemType::from_char(type_char);
            let selector = if path_without_slash.len() > 1 {
                path_without_slash[1..].to_string()
            } else {
                String::new()
            };
            (item_type, selector)
        }
    };

    Ok(GopherUrl {
        host,
        port,
        item_type,
        selector,
    })
}

/// Parser en enkelt meny-linje fra en Gopher-respons
///
/// Format: <type><display>\t<selector>\t<host>\t<port>\r\n
pub fn parse_menu_line(line: &str) -> Option<GopherItem> {
    let line = line.trim_end_matches('\r');

    if line.is_empty() || line == "." {
        return None;
    }

    // Første tegn er typen
    let type_char = line.chars().next()?;
    let item_type = GopherItemType::from_char(type_char);
    let rest = &line[type_char.len_utf8()..];

    // Splitt på tabs
    let parts: Vec<&str> = rest.split('\t').collect();

    let display = parts.first().unwrap_or(&"").to_string();
    let selector = parts.get(1).unwrap_or(&"").to_string();
    let host = parts.get(2).unwrap_or(&"").to_string();
    let port = parts
        .get(3)
        .and_then(|p| p.trim().parse::<u16>().ok())
        .unwrap_or(DEFAULT_PORT);

    Some(GopherItem {
        item_type,
        display,
        selector,
        host,
        port,
    })
}

/// Parser en komplett Gopher-meny respons til en liste av elementer
pub fn parse_menu(response: &str) -> Vec<GopherItem> {
    let mut items = Vec::new();

    for line in response.lines() {
        let line = line.trim_end_matches('\r');

        // Slutt-markør
        if line == "." {
            break;
        }

        // Hopp over tomme linjer
        if line.is_empty() {
            continue;
        }

        if let Some(item) = parse_menu_line(line) {
            items.push(item);
        }
    }

    items
}

/// Henter en Gopher-ressurs via TCP
///
/// # Arguments
/// * `url` - Gopher-URL (gopher://host[:port]/[type][selector])
///
/// # Returns
/// * `Ok(GopherResponse)` - Parsed respons
/// * `Err(GopherError)` - Ved feil
pub async fn fetch(url: &str) -> Result<GopherResponse, GopherError> {
    let parsed = parse_gopher_url(url)?;

    info!("Gopher: Kobler til {}:{}", parsed.host, parsed.port);

    // Sjekk om dette er et søk som krever input
    if parsed.item_type == GopherItemType::Search && parsed.selector.find('\t').is_none() {
        // Søk uten query - be om input
        return Err(GopherError::SearchInputRequired);
    }

    // TCP-tilkobling med timeout
    let addr = format!("{}:{}", parsed.host, parsed.port);
    let stream = tokio::time::timeout(
        Duration::from_secs(TIMEOUT_SECONDS),
        TcpStream::connect(&addr),
    )
    .await
    .map_err(|_| GopherError::Timeout(TIMEOUT_SECONDS))?
    .map_err(|e| GopherError::ConnectionError(format!("Kunne ikke koble til {}: {}", addr, e)))?;

    info!("Gopher: Tilkoblet til {}", addr);

    // Send selektor + CRLF
    let selector_str = format!("{}\r\n", parsed.selector);
    debug!("Gopher: Sender selektor: {:?}", selector_str.trim());

    let (mut reader, mut writer) = stream.into_split();

    writer
        .write_all(selector_str.as_bytes())
        .await
        .map_err(|e| GopherError::ConnectionError(format!("Kunne ikke sende selektor: {}", e)))?;

    // Les respons med timeout og størrelsesbegrensning
    let mut buffer = Vec::new();
    let mut total_read = 0;
    let mut temp_buf = [0u8; 8192];

    loop {
        let read_result = tokio::time::timeout(
            Duration::from_secs(TIMEOUT_SECONDS),
            reader.read(&mut temp_buf),
        )
        .await;

        match read_result {
            Ok(Ok(0)) => break, // Tilkobling lukket
            Ok(Ok(n)) => {
                total_read += n;
                if total_read > MAX_RESPONSE_SIZE {
                    return Err(GopherError::TooLarge(MAX_RESPONSE_SIZE));
                }
                buffer.extend_from_slice(&temp_buf[..n]);
            }
            Ok(Err(e)) => {
                warn!("Gopher: Lesefeil: {}", e);
                if buffer.is_empty() {
                    return Err(GopherError::Io(e));
                }
                // Bruk det vi har lest så langt
                break;
            }
            Err(_) => {
                if buffer.is_empty() {
                    return Err(GopherError::Timeout(TIMEOUT_SECONDS));
                }
                // Bruk det vi har lest så langt
                break;
            }
        }
    }

    debug!("Gopher: Mottok {} bytes", buffer.len());

    // Konverter til streng, forsøk UTF-8 først, deretter Latin-1
    let body = match String::from_utf8(buffer.clone()) {
        Ok(s) => s,
        Err(_) => {
            // Fallback til Latin-1 (ISO-8859-1)
            info!("Gopher: UTF-8-dekoding feilet, bruker Latin-1 fallback");
            buffer.iter().map(|&b| b as char).collect()
        }
    };

    // Bygg respons basert på URL-type
    match parsed.item_type {
        GopherItemType::Directory | GopherItemType::Search => {
            let items = parse_menu(&body);
            Ok(GopherResponse {
                content_type: GopherContentType::Menu,
                body,
                items,
                final_url: url.to_string(),
            })
        }
        GopherItemType::TextFile => {
            // Fjern terminering (. på egen linje)
            let text = strip_termination(&body);
            Ok(GopherResponse {
                content_type: GopherContentType::Text,
                body: text,
                items: Vec::new(),
                final_url: url.to_string(),
            })
        }
        GopherItemType::Html => {
            let text = strip_termination(&body);
            Ok(GopherResponse {
                content_type: GopherContentType::Html,
                body: text,
                items: Vec::new(),
                final_url: url.to_string(),
            })
        }
        GopherItemType::Error => Ok(GopherResponse {
            content_type: GopherContentType::Error,
            body: body.clone(),
            items: parse_menu(&body),
            final_url: url.to_string(),
        }),
        GopherItemType::Info => {
            // Info-type i URL → behandle som tekstfil
            let text = strip_termination(&body);
            Ok(GopherResponse {
                content_type: GopherContentType::Text,
                body: text,
                items: Vec::new(),
                final_url: url.to_string(),
            })
        }
        _ => {
            // Andre typer (binær, bilder, etc.) — prøv å tolke som meny eller tekst
            warn!(
                "Gopher: Ukjent/usupported type '{}', forsøker tekstvisning",
                parsed.item_type.to_char()
            );
            let text = strip_termination(&body);
            Ok(GopherResponse {
                content_type: GopherContentType::Text,
                body: text,
                items: Vec::new(),
                final_url: url.to_string(),
            })
        }
    }
}

/// Utfører et Gopher-søk (type 7)
///
/// # Arguments
/// * `url` - Gopher-søke-URL (gopher://host/7/selector)
/// * `query` - Søkestreng fra brukeren
///
/// # Returns
/// * `Ok(GopherResponse)` - Søkeresultater som meny
/// * `Err(GopherError)` - Ved feil
pub async fn search(url: &str, query: &str) -> Result<GopherResponse, GopherError> {
    let parsed = parse_gopher_url(url)?;

    info!(
        "Gopher: Søker på {}:{} med query: {}",
        parsed.host, parsed.port, query
    );

    // TCP-tilkobling med timeout
    let addr = format!("{}:{}", parsed.host, parsed.port);
    let stream = tokio::time::timeout(
        Duration::from_secs(TIMEOUT_SECONDS),
        TcpStream::connect(&addr),
    )
    .await
    .map_err(|_| GopherError::Timeout(TIMEOUT_SECONDS))?
    .map_err(|e| GopherError::ConnectionError(format!("Kunne ikke koble til {}: {}", addr, e)))?;

    // Send selektor\tsøkestreng\r\n
    let search_str = format!("{}\t{}\r\n", parsed.selector, query);
    debug!("Gopher: Sender søk: {:?}", search_str.trim());

    let (mut reader, mut writer) = stream.into_split();

    writer.write_all(search_str.as_bytes()).await.map_err(|e| {
        GopherError::ConnectionError(format!("Kunne ikke sende søkeforespørsel: {}", e))
    })?;

    // Les respons
    let mut buffer = Vec::new();
    let mut total_read = 0;
    let mut temp_buf = [0u8; 8192];

    loop {
        let read_result = tokio::time::timeout(
            Duration::from_secs(TIMEOUT_SECONDS),
            reader.read(&mut temp_buf),
        )
        .await;

        match read_result {
            Ok(Ok(0)) => break,
            Ok(Ok(n)) => {
                total_read += n;
                if total_read > MAX_RESPONSE_SIZE {
                    return Err(GopherError::TooLarge(MAX_RESPONSE_SIZE));
                }
                buffer.extend_from_slice(&temp_buf[..n]);
            }
            Ok(Err(e)) => {
                if buffer.is_empty() {
                    return Err(GopherError::Io(e));
                }
                break;
            }
            Err(_) => {
                if buffer.is_empty() {
                    return Err(GopherError::Timeout(TIMEOUT_SECONDS));
                }
                break;
            }
        }
    }

    // Konverter til streng
    let body = match String::from_utf8(buffer.clone()) {
        Ok(s) => s,
        Err(_) => buffer.iter().map(|&b| b as char).collect(),
    };

    let items = parse_menu(&body);
    Ok(GopherResponse {
        content_type: GopherContentType::Menu,
        body,
        items,
        final_url: url.to_string(),
    })
}

/// Fjerner Gopher-terminering (. på egen linje i slutten)
fn strip_termination(text: &str) -> String {
    let trimmed = text.trim_end();
    if trimmed.ends_with("\r\n.") {
        let end = trimmed.len() - 3; // fjern \r\n.
        trimmed[..end].to_string()
    } else if trimmed.ends_with("\n.") {
        let end = trimmed.len() - 2; // fjern \n.
        trimmed[..end].to_string()
    } else if trimmed == "." {
        String::new()
    } else {
        text.to_string()
    }
}

/// Bygger en Gopher-URL fra komponentene til et GopherItem
pub fn build_gopher_url(item: &GopherItem) -> String {
    let type_char = item.item_type.to_char();
    let selector = if item.selector.starts_with('/') {
        item.selector.clone()
    } else if item.selector.is_empty() {
        String::new()
    } else {
        format!("/{}", item.selector)
    };

    if item.port == DEFAULT_PORT {
        format!("gopher://{}/{}{}", item.host, type_char, selector)
    } else {
        format!(
            "gopher://{}:{}/{}{}",
            item.host, item.port, type_char, selector
        )
    }
}

/// Løser en relativ Gopher-URL mot en base-URL
pub fn resolve_gopher_url(base_url: &str, relative_url: &str) -> Result<String, GopherError> {
    // Hvis det allerede er en absolutt URL, returner den
    if relative_url.starts_with("gopher://")
        || relative_url.starts_with("http://")
        || relative_url.starts_with("https://")
        || relative_url.starts_with("gemini://")
    {
        return Ok(relative_url.to_string());
    }

    let base = Url::parse(base_url)
        .map_err(|e| GopherError::InvalidUrl(format!("Ugyldig base-URL: {}", e)))?;

    let resolved = base
        .join(relative_url)
        .map_err(|e| GopherError::InvalidUrl(format!("Kan ikke løse relativ URL: {}", e)))?;

    Ok(resolved.to_string())
}

// ===== Tester =====

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_gopher_url_basic() {
        let url = parse_gopher_url("gopher://example.com/").unwrap();
        assert_eq!(url.host, "example.com");
        assert_eq!(url.port, 70);
        assert_eq!(url.item_type, GopherItemType::Directory);
        assert_eq!(url.selector, "");
    }

    #[test]
    fn test_parse_gopher_url_with_type() {
        let url = parse_gopher_url("gopher://example.com/0/readme.txt").unwrap();
        assert_eq!(url.item_type, GopherItemType::TextFile);
        assert_eq!(url.selector, "/readme.txt");
    }

    #[test]
    fn test_parse_gopher_url_custom_port() {
        let url = parse_gopher_url("gopher://example.com:7070/1/docs").unwrap();
        assert_eq!(url.port, 7070);
        assert_eq!(url.item_type, GopherItemType::Directory);
        assert_eq!(url.selector, "/docs");
    }

    #[test]
    fn test_parse_gopher_url_invalid_scheme() {
        assert!(parse_gopher_url("http://example.com/").is_err());
    }

    #[test]
    fn test_parse_gopher_url_no_path() {
        let url = parse_gopher_url("gopher://example.com").unwrap();
        assert_eq!(url.host, "example.com");
        assert_eq!(url.port, 70);
        assert_eq!(url.item_type, GopherItemType::Directory);
    }

    #[test]
    fn test_parse_gopher_url_search() {
        let url = parse_gopher_url("gopher://example.com/7/search").unwrap();
        assert_eq!(url.item_type, GopherItemType::Search);
        assert_eq!(url.selector, "/search");
    }

    #[test]
    fn test_parse_menu_line_directory() {
        let line = "1Documents\t/docs\texample.com\t70";
        let item = parse_menu_line(line).unwrap();
        assert_eq!(item.item_type, GopherItemType::Directory);
        assert_eq!(item.display, "Documents");
        assert_eq!(item.selector, "/docs");
        assert_eq!(item.host, "example.com");
        assert_eq!(item.port, 70);
    }

    #[test]
    fn test_parse_menu_line_info() {
        let line = "iWelcome text\tfake\t(NULL)\t0";
        let item = parse_menu_line(line).unwrap();
        assert_eq!(item.item_type, GopherItemType::Info);
        assert_eq!(item.display, "Welcome text");
    }

    #[test]
    fn test_parse_menu_line_html_link() {
        let line = "hGoogle\tURL:https://google.com\texample.com\t70";
        let item = parse_menu_line(line).unwrap();
        assert_eq!(item.item_type, GopherItemType::Html);
        assert_eq!(item.selector, "URL:https://google.com");
    }

    #[test]
    fn test_parse_menu_termination() {
        let response = "iHello\tfake\t(NULL)\t0\r\n.\r\n";
        let items = parse_menu(response);
        assert_eq!(items.len(), 1);
    }

    #[test]
    fn test_parse_menu_ignores_empty_lines() {
        let response = "iHello\tfake\t(NULL)\t0\r\n\r\niWorld\tfake\t(NULL)\t0\r\n.\r\n";
        let items = parse_menu(response);
        assert_eq!(items.len(), 2);
    }

    #[test]
    fn test_validate_url_too_long() {
        let long_url = format!("gopher://example.com/{}", "a".repeat(2000));
        assert!(parse_gopher_url(&long_url).is_err());
    }

    #[test]
    fn test_validate_url_missing_host() {
        assert!(parse_gopher_url("gopher:///path").is_err());
    }

    #[test]
    fn test_item_type_from_char() {
        assert_eq!(GopherItemType::from_char('0'), GopherItemType::TextFile);
        assert_eq!(GopherItemType::from_char('1'), GopherItemType::Directory);
        assert_eq!(GopherItemType::from_char('7'), GopherItemType::Search);
        assert_eq!(GopherItemType::from_char('i'), GopherItemType::Info);
        assert_eq!(GopherItemType::from_char('h'), GopherItemType::Html);
    }

    #[test]
    fn test_strip_termination() {
        assert_eq!(strip_termination("Hello\n."), "Hello");
        assert_eq!(strip_termination("Hello\r\n."), "Hello");
        assert_eq!(strip_termination("Hello"), "Hello");
        assert_eq!(strip_termination("."), "");
        assert_eq!(strip_termination("Line 1\nLine 2\n."), "Line 1\nLine 2");
    }

    #[test]
    fn test_build_gopher_url() {
        let item = GopherItem {
            item_type: GopherItemType::Directory,
            display: "Docs".to_string(),
            selector: "/docs".to_string(),
            host: "example.com".to_string(),
            port: 70,
        };
        let url = build_gopher_url(&item);
        assert_eq!(url, "gopher://example.com/1/docs");
    }

    #[test]
    fn test_build_gopher_url_empty_selector() {
        let item = GopherItem {
            item_type: GopherItemType::Directory,
            display: "Root".to_string(),
            selector: "".to_string(),
            host: "example.com".to_string(),
            port: 70,
        };
        let url = build_gopher_url(&item);
        assert_eq!(url, "gopher://example.com/1");
    }

    #[test]
    fn test_build_gopher_url_html_link() {
        let item = GopherItem {
            item_type: GopherItemType::Html,
            display: "Google".to_string(),
            selector: "URL:https://google.com".to_string(),
            host: "example.com".to_string(),
            port: 70,
        };
        // HTML-lenker bruker selektoren direkte, men build_gopher_url bygger en gopher-URL
        let url = build_gopher_url(&item);
        assert!(url.starts_with("gopher://"));
    }

    #[test]
    fn test_resolve_gopher_url_absolute() {
        let result = resolve_gopher_url("gopher://example.com/1/docs", "gopher://other.com/0/file");
        assert_eq!(result.unwrap(), "gopher://other.com/0/file");
    }
}
