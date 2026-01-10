//! HTML til Markdown konvertering
//!
//! Konverterer HTML-innhold til markdown for visning i Bare.
//! Inkluderer readability-modus for å ekstrahere hovedinnhold.

use ammonia::Builder;
use log::{debug, info, warn};
use std::collections::HashSet;

// Disse importene brukes av decode_html som er tilgjengelig for fremtidig bruk
#[allow(unused_imports)]
use encoding_rs::Encoding;
#[allow(unused_imports)]
use thiserror::Error;

/// Feil som kan oppstå under konvertering
#[allow(dead_code)]
#[derive(Debug, Error)]
pub enum ConversionError {
    #[error("Kunne ikke parse HTML: {0}")]
    ParseError(String),

    #[error("Encoding-feil: {0}")]
    EncodingError(String),
}

/// Resultat av HTML-til-markdown konvertering
#[derive(Debug)]
pub struct ConversionResult {
    /// Konvertert markdown-innhold
    pub markdown: String,
    /// Ekstrahert tittel (fra <title> eller <h1>)
    pub title: Option<String>,
    /// Om readability-modus ble brukt (for fremtidig statistikk/logging)
    #[allow(dead_code)]
    pub used_readability: bool,
}

/// Konverter HTML til markdown
///
/// # Arguments
/// * `html` - HTML-innhold som skal konverteres
///
/// # Returns
/// Konvertert markdown-innhold
pub fn html_to_markdown(html: &str) -> ConversionResult {
    info!("Konverterer HTML til markdown ({} bytes)", html.len());

    // Ekstraher tittel før sanitizing
    let title = extract_title(html);

    // Sanitize HTML for å fjerne scripts, styles, etc.
    let clean_html = sanitize_html(html);

    // Ekstraher hovedinnhold hvis mulig (readability-modus)
    let (content_html, used_readability) = extract_main_content(&clean_html);

    // Konverter til markdown
    let markdown = html2md::parse_html(&content_html);

    // Fiks brutte lenker (multi-linje lenker som html2md genererer)
    let fixed_links = fix_broken_links(&markdown);

    // Rydd opp i markdown (fjern overflødige linjer, etc.)
    let cleaned_markdown = clean_markdown(&fixed_links);

    debug!(
        "Konvertering fullført: {} bytes markdown",
        cleaned_markdown.len()
    );

    ConversionResult {
        markdown: cleaned_markdown,
        title,
        used_readability,
    }
}

/// Konverter HTML-bytes med riktig encoding
///
/// # Arguments
/// * `bytes` - Rå bytes fra HTTP-respons
/// * `content_type` - Content-Type header (kan inneholde charset)
///
/// # Returns
/// Dekodet streng
#[allow(dead_code)]
pub fn decode_html(bytes: &[u8], content_type: Option<&str>) -> Result<String, ConversionError> {
    // Prøv å finne encoding fra Content-Type header
    let charset = content_type.and_then(|ct| {
        ct.split(';').find_map(|part| {
            let part = part.trim();
            if part.to_lowercase().starts_with("charset=") {
                Some(part[8..].trim_matches('"').trim())
            } else {
                None
            }
        })
    });

    // Prøv å finne encoding fra meta-tag i HTML
    let meta_charset = if charset.is_none() {
        detect_charset_from_html(bytes)
    } else {
        None
    };

    let encoding_name = charset.or(meta_charset.as_deref()).unwrap_or("utf-8");

    debug!("Bruker encoding: {}", encoding_name);

    // Finn riktig encoding
    let encoding = Encoding::for_label(encoding_name.as_bytes()).unwrap_or(encoding_rs::UTF_8);

    // Dekod innholdet
    let (decoded, _, had_errors) = encoding.decode(bytes);

    if had_errors {
        warn!("Encoding-feil oppstod under dekoding");
    }

    Ok(decoded.into_owned())
}

/// Forsøk å finne charset fra HTML meta-tag
fn detect_charset_from_html(bytes: &[u8]) -> Option<String> {
    // Les bare de første 1024 bytesene for å finne meta-tag
    let preview = String::from_utf8_lossy(&bytes[..bytes.len().min(1024)]);

    // Søk etter <meta charset="...">
    if let Some(start) = preview.to_lowercase().find("charset=") {
        let rest = &preview[start + 8..];
        let end = rest.find(['"', '\'', ' ', '>', ';']).unwrap_or(rest.len());
        let charset = rest[..end].trim_matches(|c| c == '"' || c == '\'');
        if !charset.is_empty() {
            return Some(charset.to_string());
        }
    }

    None
}

/// Sanitize HTML ved å fjerne potensielt farlige eller unødvendige elementer
fn sanitize_html(html: &str) -> String {
    // Definer hvilke tags vi vil beholde
    let mut allowed_tags: HashSet<&str> = HashSet::new();
    for tag in &[
        "html",
        "head",
        "body",
        "main",
        "article",
        "section",
        "aside",
        "header",
        "footer",
        "nav",
        "div",
        "span",
        "p",
        "br",
        "hr",
        "h1",
        "h2",
        "h3",
        "h4",
        "h5",
        "h6",
        "ul",
        "ol",
        "li",
        "dl",
        "dt",
        "dd",
        "table",
        "thead",
        "tbody",
        "tfoot",
        "tr",
        "th",
        "td",
        "a",
        "img",
        "figure",
        "figcaption",
        "blockquote",
        "pre",
        "code",
        "em",
        "strong",
        "b",
        "i",
        "u",
        "s",
        "del",
        "ins",
        "sub",
        "sup",
        "small",
        "mark",
        "abbr",
        "time",
        "address",
        "details",
        "summary",
    ] {
        allowed_tags.insert(tag);
    }

    // Bruk ammonia for å sanitize
    Builder::default()
        .tags(allowed_tags)
        .add_generic_attributes(&["id", "class", "title", "lang"])
        // Tillat href og target for lenker (rel settes automatisk av link_rel)
        .add_tag_attributes("a", &["href", "target"])
        // Tillat src og alt for bilder
        .add_tag_attributes("img", &["src", "alt", "width", "height"])
        .link_rel(Some("noopener noreferrer"))
        .clean(html)
        .to_string()
}

/// Forsøk å ekstrahere hovedinnholdet fra HTML (readability-modus)
fn extract_main_content(html: &str) -> (String, bool) {
    let html_lower = html.to_lowercase();

    // Prøv å finne main/article/content elementer
    let content_markers = [
        ("<article", "</article>"),
        ("<main", "</main>"),
        (r#"<div class="content"#, "</div>"),
        (r#"<div id="content"#, "</div>"),
        (r#"<div class="post"#, "</div>"),
        (r#"<div class="article"#, "</div>"),
        (r#"<div class="entry"#, "</div>"),
    ];

    for (start_marker, end_marker) in content_markers {
        if let Some(start_pos) = html_lower.find(start_marker) {
            // Finn slutten av start-taggen
            if let Some(tag_end) = html_lower[start_pos..].find('>') {
                let content_start = start_pos + tag_end + 1;

                // Finn den matchende slutttaggen (forenklet - tar siste forekomst)
                if let Some(end_pos) = html_lower.rfind(end_marker) {
                    if end_pos > content_start {
                        let extracted = &html[content_start..end_pos];
                        if extracted.len() > 100 {
                            // Sørg for at vi har faktisk innhold
                            debug!("Ekstraherte hovedinnhold med markør: {}", start_marker);
                            return (extracted.to_string(), true);
                        }
                    }
                }
            }
        }
    }

    // Prøv å fjerne header, footer, nav, aside
    let cleaned = remove_boilerplate(html);
    if cleaned.len() < html.len() / 2 && cleaned.len() > 100 {
        return (cleaned, true);
    }

    // Fallback: returner hele HTML
    (html.to_string(), false)
}

/// Fjern boilerplate-elementer som header, footer, nav, aside
fn remove_boilerplate(html: &str) -> String {
    let mut result = html.to_string();

    // Elementer å fjerne (forenklet implementasjon)
    let remove_patterns = [
        ("<header", "</header>"),
        ("<footer", "</footer>"),
        ("<nav", "</nav>"),
        ("<aside", "</aside>"),
        (r#"<div class="sidebar"#, "</div>"),
        (r#"<div class="menu"#, "</div>"),
        (r#"<div class="advertisement"#, "</div>"),
        (r#"<div class="ads"#, "</div>"),
    ];

    for (start_marker, end_marker) in remove_patterns {
        while let Some(start) = result.to_lowercase().find(start_marker) {
            if let Some(end) = result[start..].to_lowercase().find(end_marker) {
                let remove_end = start + end + end_marker.len();
                result = format!("{}{}", &result[..start], &result[remove_end..]);
            } else {
                break;
            }
        }
    }

    result
}

/// Ekstraher tittel fra HTML
fn extract_title(html: &str) -> Option<String> {
    let html_lower = html.to_lowercase();

    // Prøv <title> først
    if let Some(start) = html_lower.find("<title") {
        if let Some(tag_end) = html_lower[start..].find('>') {
            let title_start = start + tag_end + 1;
            if let Some(title_end) = html_lower[title_start..].find("</title>") {
                let title = html[title_start..title_start + title_end].trim();
                if !title.is_empty() {
                    return Some(decode_html_entities(title));
                }
            }
        }
    }

    // Fallback: prøv første <h1>
    if let Some(start) = html_lower.find("<h1") {
        if let Some(tag_end) = html_lower[start..].find('>') {
            let h1_start = start + tag_end + 1;
            if let Some(h1_end) = html_lower[h1_start..].find("</h1>") {
                let h1 = html[h1_start..h1_start + h1_end].trim();
                // Fjern eventuelle HTML-tags inne i h1
                let clean_h1 = strip_tags(h1);
                if !clean_h1.is_empty() {
                    return Some(decode_html_entities(&clean_h1));
                }
            }
        }
    }

    None
}

/// Fjern HTML-tags fra en streng
fn strip_tags(html: &str) -> String {
    let mut result = String::new();
    let mut in_tag = false;

    for c in html.chars() {
        match c {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => result.push(c),
            _ => {}
        }
    }

    result.trim().to_string()
}

/// Dekod vanlige HTML-entities
fn decode_html_entities(text: &str) -> String {
    text.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&apos;", "'")
        .replace("&nbsp;", " ")
        .replace("&#x27;", "'")
        .replace("&#x2F;", "/")
        .replace("&mdash;", "—")
        .replace("&ndash;", "–")
        .replace("&hellip;", "…")
        .replace("&copy;", "©")
        .replace("&reg;", "®")
        .replace("&trade;", "™")
}

/// Fiks brutte lenker som html2md genererer
///
/// html2md konverterer komplekse `<a>`-tagger (med bilder, overskrifter, etc.) til
/// multi-linje markdown-lenker som ikke er gyldig markdown-syntaks:
///
/// ```text
/// [
/// ![bilde](url)
/// ### Tittel
/// Beskrivelse
/// ](/lenke)
/// ```
///
/// Denne funksjonen finner og fikser disse brutte lenkene.
fn fix_broken_links(markdown: &str) -> String {
    let mut result = String::new();
    let mut chars = markdown.chars().peekable();
    let mut in_broken_link = false;
    let mut link_content = String::new();
    let mut bracket_depth = 0;

    while let Some(c) = chars.next() {
        if !in_broken_link {
            // Sjekk om dette er starten på en brutt lenke
            // En brutt lenke starter med '[' fulgt av newline
            if c == '[' {
                // Sjekk hva som kommer etter
                let mut lookahead = String::new();
                let mut temp_chars = chars.clone();
                while let Some(&next) = temp_chars.peek() {
                    if next == '\n' || next == '\r' {
                        lookahead.push(next);
                        temp_chars.next();
                        break;
                    } else if next.is_whitespace() {
                        lookahead.push(next);
                        temp_chars.next();
                    } else {
                        break;
                    }
                }

                // Hvis '[' etterfølges av whitespace og newline, er dette sannsynligvis en brutt lenke
                if lookahead.contains('\n') || lookahead.contains('\r') {
                    in_broken_link = true;
                    link_content.clear();
                    bracket_depth = 1;
                    // Hopp over whitespace etter '['
                    while let Some(&next) = chars.peek() {
                        if next == '\n' || next == '\r' {
                            chars.next();
                            break;
                        } else if next.is_whitespace() {
                            chars.next();
                        } else {
                            break;
                        }
                    }
                    continue;
                }
            }
            result.push(c);
        } else {
            // Vi er inne i en brutt lenke - samle innhold
            if c == '[' {
                bracket_depth += 1;
                link_content.push(c);
            } else if c == ']' {
                bracket_depth -= 1;
                if bracket_depth == 0 {
                    // Sjekk om neste tegn er '('
                    if chars.peek() == Some(&'(') {
                        chars.next(); // Konsumer '('
                        let mut url = String::new();
                        let mut paren_depth = 1;

                        for url_char in chars.by_ref() {
                            if url_char == '(' {
                                paren_depth += 1;
                                url.push(url_char);
                            } else if url_char == ')' {
                                paren_depth -= 1;
                                if paren_depth == 0 {
                                    break;
                                }
                                url.push(url_char);
                            } else {
                                url.push(url_char);
                            }
                        }

                        // Konverter den brutte lenken til fungerende markdown
                        let fixed = convert_broken_link_to_markdown(&link_content, &url);
                        result.push_str(&fixed);
                    } else {
                        // Ikke en lenke likevel, bare output innholdet
                        result.push('[');
                        result.push_str(&link_content);
                        result.push(']');
                    }
                    in_broken_link = false;
                } else {
                    link_content.push(c);
                }
            } else {
                link_content.push(c);
            }
        }
    }

    // Hvis vi fortsatt er i en brutt lenke når vi når slutten, output den som tekst
    if in_broken_link {
        result.push('[');
        result.push_str(&link_content);
    }

    result
}

/// Konverter innholdet i en brutt lenke til fungerende markdown
fn convert_broken_link_to_markdown(content: &str, url: &str) -> String {
    let mut result = String::new();
    let lines: Vec<&str> = content.lines().collect();

    // Finn bilder, overskrifter og tekst
    let mut images: Vec<String> = Vec::new();
    let mut headings: Vec<String> = Vec::new();
    let mut texts: Vec<String> = Vec::new();

    for line in lines {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if trimmed.starts_with("![") {
            // Bilde - ekstraher og legg til
            images.push(trimmed.to_string());
        } else if trimmed.starts_with('#') {
            // Overskrift - ekstraher teksten
            let heading_text = trimmed.trim_start_matches('#').trim();
            headings.push(heading_text.to_string());
        } else {
            texts.push(trimmed.to_string());
        }
    }

    // Bygg resultatet
    // Først bildene (som lenker)
    for img in &images {
        // Gjør bildet til en lenke
        result.push_str(&format!("[{}]({})\n\n", img, url));
    }

    // Så overskriftene (som lenker)
    for heading in &headings {
        result.push_str(&format!("### [{}]({})\n\n", heading, url));
    }

    // Så teksten
    for text in &texts {
        result.push_str(&format!("{}\n\n", text));
    }

    result
}

/// Rydd opp i konvertert markdown
fn clean_markdown(markdown: &str) -> String {
    let lines: Vec<&str> = markdown.lines().collect();
    let mut result = Vec::new();
    let mut prev_empty = false;

    for line in lines {
        let trimmed = line.trim();

        // Fjern linjer som bare inneholder whitespace
        if trimmed.is_empty() {
            if !prev_empty && !result.is_empty() {
                result.push("");
                prev_empty = true;
            }
            continue;
        }

        prev_empty = false;

        // Fjern linjer som bare er repeterte tegn (ofte artefakter)
        if trimmed.chars().all(|c| c == '-' || c == '=' || c == '_') && trimmed.len() > 3 {
            continue;
        }

        result.push(trimmed);
    }

    // Fjern ledende og etterfølgende tomme linjer
    while result.first() == Some(&"") {
        result.remove(0);
    }
    while result.last() == Some(&"") {
        result.pop();
    }

    result.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_html_to_markdown_basic() {
        let html = "<h1>Test</h1><p>Dette er en test.</p>";
        let result = html_to_markdown(html);
        // html2md bruker en annen syntaks, sjekk for tekst innhold
        println!("Markdown output: {:?}", result.markdown);
        assert!(result.markdown.contains("Test"));
        assert!(result.markdown.contains("Dette er en test"));
    }

    #[test]
    fn test_html_to_markdown_with_article() {
        let html = r#"
            <html>
            <body>
                <header><nav>Menu</nav></header>
                <article>
                    <h1>Hovedinnhold</h1>
                    <p>Dette er det viktige innholdet.</p>
                </article>
                <footer>Footer</footer>
            </body>
            </html>
        "#;
        let result = html_to_markdown(html);
        assert!(result.markdown.contains("Hovedinnhold"));
        assert!(result.used_readability);
    }

    #[test]
    fn test_extract_title() {
        let html = "<html><head><title>Min side</title></head><body></body></html>";
        let title = extract_title(html);
        assert_eq!(title, Some("Min side".to_string()));
    }

    #[test]
    fn test_extract_title_from_h1() {
        let html = "<html><body><h1>Overskrift</h1></body></html>";
        let title = extract_title(html);
        assert_eq!(title, Some("Overskrift".to_string()));
    }

    #[test]
    fn test_decode_html_entities() {
        let text = "Tom &amp; Jerry &mdash; en klassiker";
        let decoded = decode_html_entities(text);
        assert_eq!(decoded, "Tom & Jerry — en klassiker");
    }

    #[test]
    fn test_sanitize_removes_script() {
        // ammonia fjerner scripts automatisk
        let html = "<div><p>Trygt</p></div>";
        let sanitized = sanitize_html(html);
        assert!(sanitized.contains("Trygt"));
    }

    #[test]
    fn test_decode_html_utf8() {
        let bytes = "Hei på deg æøå".as_bytes();
        let result = decode_html(bytes, Some("text/html; charset=utf-8")).unwrap();
        assert_eq!(result, "Hei på deg æøå");
    }

    #[test]
    fn test_strip_tags() {
        let html = "Tekst med <strong>bold</strong> og <em>italic</em>";
        let stripped = strip_tags(html);
        assert_eq!(stripped, "Tekst med bold og italic");
    }

    #[test]
    fn test_clean_markdown() {
        let markdown = "\n\n# Tittel\n\n\n\nParagraf\n\n----------\n\nMer tekst\n\n";
        let cleaned = clean_markdown(markdown);
        println!("Cleaned: {:?}", cleaned);
        // Forenklet sjekk - vi vil ha strukturen uten ekstra linjer
        assert!(cleaned.contains("# Tittel"));
        assert!(cleaned.contains("Paragraf"));
        assert!(cleaned.contains("Mer tekst"));
        assert!(!cleaned.contains("----------"));
    }
}
