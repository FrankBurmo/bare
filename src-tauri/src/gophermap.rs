//! Gopher-meny til Markdown konvertering
//!
//! Konverterer Gopher-menyer (gophermap) til Markdown-format.
//! Hver elementtype konverteres til passende markdown med emoji-ikoner.

use crate::gopher::{build_gopher_url, GopherItem, GopherItemType};

/// Resultat fra gophermap-konvertering
#[derive(Debug, Clone)]
pub struct GophermapResult {
    /// Konvertert markdown-innhold
    pub markdown: String,
    /// Tittel ekstrahert fra menyen
    pub title: Option<String>,
}

/// Konverterer en liste av Gopher-elementer til markdown
///
/// # Arguments
/// * `items` - Gopher-meny-elementer
/// * `_base_url` - Base-URL for å løse relative lenker
///
/// # Returns
/// GophermapResult med markdown og eventuell tittel
pub fn to_markdown(items: &[GopherItem], _base_url: &str) -> GophermapResult {
    let mut output = String::new();
    let mut title: Option<String> = None;
    let mut prev_was_info = false;

    for item in items {
        let line = match &item.item_type {
            GopherItemType::Info => {
                // Ekstraher tittel fra første info-linje som ikke er tom
                if title.is_none() && !item.display.trim().is_empty() {
                    title = Some(item.display.trim().to_string());
                }
                prev_was_info = true;
                convert_info_line(item)
            }
            GopherItemType::Directory => {
                if prev_was_info {
                    output.push('\n');
                }
                prev_was_info = false;
                convert_directory(item)
            }
            GopherItemType::TextFile => {
                if prev_was_info {
                    output.push('\n');
                }
                prev_was_info = false;
                convert_text_file(item)
            }
            GopherItemType::Search => {
                if prev_was_info {
                    output.push('\n');
                }
                prev_was_info = false;
                convert_search(item)
            }
            GopherItemType::Html => {
                if prev_was_info {
                    output.push('\n');
                }
                prev_was_info = false;
                convert_html_link(item)
            }
            GopherItemType::Error => {
                prev_was_info = false;
                format!("⚠️ {}", item.display)
            }
            GopherItemType::Gif | GopherItemType::Image => {
                if prev_was_info {
                    output.push('\n');
                }
                prev_was_info = false;
                convert_image(item)
            }
            GopherItemType::Telnet | GopherItemType::Telnet3270 => {
                prev_was_info = false;
                format!("  {} *(Telnet — ikke støttet)*", item.display)
            }
            GopherItemType::Binary
            | GopherItemType::BinHex
            | GopherItemType::DosBinary
            | GopherItemType::UuEncoded => {
                if prev_was_info {
                    output.push('\n');
                }
                prev_was_info = false;
                format!("  {} *({})*", item.display, item.item_type.description())
            }
            GopherItemType::CsoPhonebook => {
                prev_was_info = false;
                format!("📖 {} *(CSO-telefonbok)*", item.display)
            }
            GopherItemType::Unknown(_) => {
                prev_was_info = false;
                format!("  {} *({})*", item.display, item.item_type.description())
            }
        };

        output.push_str(&line);
        output.push('\n');
    }

    GophermapResult {
        markdown: output,
        title,
    }
}

/// Konverterer en informasjonslinje
fn convert_info_line(item: &GopherItem) -> String {
    item.display.clone()
}

/// Konverterer en mappe/meny-lenke
fn convert_directory(item: &GopherItem) -> String {
    let url = build_gopher_url(item);
    format!("📁 [{}]({})", item.display, url)
}

/// Konverterer en tekstfil-lenke
fn convert_text_file(item: &GopherItem) -> String {
    let url = build_gopher_url(item);
    format!("📄 [{}]({})", item.display, url)
}

/// Konverterer en søke-lenke
fn convert_search(item: &GopherItem) -> String {
    let url = build_gopher_url(item);
    format!("🔍 [{}]({})", item.display, url)
}

/// Konverterer en HTML-lenke (type h)
fn convert_html_link(item: &GopherItem) -> String {
    let url = if let Some(stripped) = item.selector.strip_prefix("URL:") {
        stripped.to_string()
    } else {
        build_gopher_url(item)
    };
    format!("🌐 [{}]({})", item.display, url)
}

/// Konverterer en bilde-lenke
fn convert_image(item: &GopherItem) -> String {
    let url = build_gopher_url(item);
    format!("🖼️ [{}]({})", item.display, url)
}

// ===== Tester =====

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_info_line_to_markdown() {
        let item = GopherItem {
            item_type: GopherItemType::Info,
            display: "Welcome to Gopher".to_string(),
            selector: "fake".to_string(),
            host: "(NULL)".to_string(),
            port: 0,
        };
        let result = to_markdown(&[item], "gopher://example.com/");
        assert!(result.markdown.contains("Welcome to Gopher"));
        assert!(!result.markdown.contains("[")); // Ingen lenke
    }

    #[test]
    fn test_directory_to_markdown() {
        let item = GopherItem {
            item_type: GopherItemType::Directory,
            display: "Documents".to_string(),
            selector: "/docs".to_string(),
            host: "example.com".to_string(),
            port: 70,
        };
        let result = to_markdown(&[item], "gopher://example.com/");
        assert!(result.markdown.contains("📁"));
        assert!(result.markdown.contains("[Documents]"));
        assert!(result.markdown.contains("gopher://example.com/1/docs"));
    }

    #[test]
    fn test_text_file_to_markdown() {
        let item = GopherItem {
            item_type: GopherItemType::TextFile,
            display: "README".to_string(),
            selector: "/readme.txt".to_string(),
            host: "example.com".to_string(),
            port: 70,
        };
        let result = to_markdown(&[item], "gopher://example.com/");
        assert!(result.markdown.contains("📄"));
        assert!(result.markdown.contains("[README]"));
    }

    #[test]
    fn test_search_to_markdown() {
        let item = GopherItem {
            item_type: GopherItemType::Search,
            display: "Search".to_string(),
            selector: "/search".to_string(),
            host: "example.com".to_string(),
            port: 70,
        };
        let result = to_markdown(&[item], "gopher://example.com/");
        assert!(result.markdown.contains("🔍"));
        assert!(result.markdown.contains("[Search]"));
    }

    #[test]
    fn test_html_link_to_markdown() {
        let item = GopherItem {
            item_type: GopherItemType::Html,
            display: "Google".to_string(),
            selector: "URL:https://google.com".to_string(),
            host: "example.com".to_string(),
            port: 70,
        };
        let result = to_markdown(&[item], "gopher://example.com/");
        assert!(result.markdown.contains("🌐"));
        assert!(result.markdown.contains("[Google]"));
        assert!(result.markdown.contains("https://google.com"));
    }

    #[test]
    fn test_error_to_markdown() {
        let item = GopherItem {
            item_type: GopherItemType::Error,
            display: "Not found".to_string(),
            selector: "".to_string(),
            host: "".to_string(),
            port: 0,
        };
        let result = to_markdown(&[item], "gopher://example.com/");
        assert!(result.markdown.contains("⚠️"));
        assert!(result.markdown.contains("Not found"));
    }

    #[test]
    fn test_complete_menu_conversion() {
        let items = vec![
            GopherItem {
                item_type: GopherItemType::Info,
                display: "Welcome".to_string(),
                selector: "fake".to_string(),
                host: "(NULL)".to_string(),
                port: 0,
            },
            GopherItem {
                item_type: GopherItemType::Directory,
                display: "Docs".to_string(),
                selector: "/docs".to_string(),
                host: "example.com".to_string(),
                port: 70,
            },
        ];
        let result = to_markdown(&items, "gopher://example.com/");
        assert!(result.markdown.contains("Welcome"));
        assert!(result.markdown.contains("📁 [Docs]"));
    }

    #[test]
    fn test_title_extraction() {
        let items = vec![
            GopherItem {
                item_type: GopherItemType::Info,
                display: "  ".to_string(),
                selector: "fake".to_string(),
                host: "(NULL)".to_string(),
                port: 0,
            },
            GopherItem {
                item_type: GopherItemType::Info,
                display: "Welcome to My Server".to_string(),
                selector: "fake".to_string(),
                host: "(NULL)".to_string(),
                port: 0,
            },
        ];
        let result = to_markdown(&items, "gopher://example.com/");
        assert_eq!(result.title, Some("Welcome to My Server".to_string()));
    }

    #[test]
    fn test_image_to_markdown() {
        let item = GopherItem {
            item_type: GopherItemType::Image,
            display: "Photo".to_string(),
            selector: "/photo.jpg".to_string(),
            host: "example.com".to_string(),
            port: 70,
        };
        let result = to_markdown(&[item], "gopher://example.com/");
        assert!(result.markdown.contains("🖼️"));
        assert!(result.markdown.contains("[Photo]"));
    }

    #[test]
    fn test_telnet_blocked() {
        let item = GopherItem {
            item_type: GopherItemType::Telnet,
            display: "Remote Login".to_string(),
            selector: "".to_string(),
            host: "example.com".to_string(),
            port: 23,
        };
        let result = to_markdown(&[item], "gopher://example.com/");
        assert!(result.markdown.contains("ikke støttet"));
        assert!(!result.markdown.contains("[")); // Ingen klikkbar lenke
    }

    #[test]
    fn test_empty_menu() {
        let result = to_markdown(&[], "gopher://example.com/");
        assert!(result.markdown.is_empty());
        assert!(result.title.is_none());
    }
}
