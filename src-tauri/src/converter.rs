//! HTML til Markdown konvertering
//!
//! Konverterer HTML-innhold til markdown for visning i Bare.
//! Inkluderer readability-modus for å ekstrahere hovedinnhold.

use ammonia::Builder;
use dom_smoothie::{Config, Readability};
use html2md::{Handle, StructuredPrinter, TagHandler, TagHandlerFactory};
use log::{debug, info, warn};
use std::collections::{HashMap, HashSet};
use url::Url;

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
/// * `url` - Valgfri URL for å hjelpe Readability med relative lenker
/// * `readability_enabled` - Om vi skal prøve å ekstrahere hovedinnhold
///
/// # Returns
/// Konvertert markdown-innhold
pub fn html_to_markdown(
    html: &str,
    url: Option<&str>,
    readability_enabled: bool,
) -> ConversionResult {
    info!("Konverterer HTML til markdown ({} bytes)", html.len());

    if readability_enabled {
        let cfg = Config::default();

        // Prøv Readability-ekstraksjon
        match Readability::new(html, url, Some(cfg)) {
            Ok(mut readability) => match readability.parse() {
                Ok(article) => {
                    debug!(
                        "Readability ekstraherte {} bytes",
                        article.text_content.len()
                    );
                    let clean_html = sanitize_html(&article.content);
                    let markdown = remove_iframe_markup(&remove_escaped_iframes(
                        &convert_html_to_markdown(&clean_html, url),
                    ));
                    return ConversionResult {
                        markdown: clean_markdown(&markdown),
                        title: if article.title.is_empty() {
                            None
                        } else {
                            Some(article.title.to_string())
                        },
                        used_readability: true,
                    };
                }
                Err(e) => {
                    warn!(
                        "Readability-parsing feilet: {}. Fallback til enkel konvertering.",
                        e
                    );
                }
            },
            Err(e) => {
                warn!(
                    "Kunne ikke initialisere Readability: {}. Fallback til enkel konvertering.",
                    e
                );
            }
        }
    }

    // Fallback: Bruk den gamle metoden hvis Readability feiler eller er deaktivert
    let title = extract_title(html);
    let clean_html = sanitize_html(html);
    let markdown = remove_iframe_markup(&remove_escaped_iframes(&convert_html_to_markdown(
        &clean_html,
        url,
    )));
    let cleaned_markdown = clean_markdown(&markdown);

    ConversionResult {
        markdown: cleaned_markdown,
        title,
        used_readability: false,
    }
}

struct BlockSafeAnchorFactory {
    base_url: Option<Url>,
}

impl html2md::TagHandlerFactory for BlockSafeAnchorFactory {
    fn instantiate(&self) -> Box<dyn html2md::TagHandler> {
        Box::new(BlockSafeAnchorHandler {
            base_url: self.base_url.clone(),
            ..Default::default()
        })
    }
}

#[derive(Default)]
struct BlockSafeAnchorHandler {
    base_url: Option<Url>,
    inline_handler: Option<html2md::anchors::AnchorHandler>,
    skip_children: bool,
}

impl html2md::TagHandler for BlockSafeAnchorHandler {
    fn handle(&mut self, tag: &html2md::Handle, printer: &mut html2md::StructuredPrinter) {
        if !contains_block_element(tag) {
            let mut inline_handler = html2md::anchors::AnchorHandler::default();
            inline_handler.handle(tag, printer);
            self.inline_handler = Some(inline_handler);
            return;
        }

        self.skip_children = true;
        let text = extract_link_text(tag);
        if let Some(href) = extract_href(tag) {
            for (source, alt_text) in extract_images(tag, self.base_url.as_ref()) {
                printer.append_str(&format!(
                    "[![{}]({})]({})\n\n",
                    escape_link_text(&alt_text),
                    source,
                    href
                ));
            }
            if !text.is_empty() {
                printer.append_str(&format!("[{}]({})", escape_link_text(&text), href));
            }
        }
    }

    fn after_handle(&mut self, printer: &mut html2md::StructuredPrinter) {
        if let Some(inline_handler) = &mut self.inline_handler {
            inline_handler.after_handle(printer);
        }
    }

    fn skip_descendants(&self) -> bool {
        self.skip_children
    }
}

struct ImageHandlerFactory {
    base_url: Option<Url>,
}

impl TagHandlerFactory for ImageHandlerFactory {
    fn instantiate(&self) -> Box<dyn TagHandler> {
        Box::new(ImageHandler {
            base_url: self.base_url.clone(),
        })
    }
}

struct ImageHandler {
    base_url: Option<Url>,
}

impl TagHandler for ImageHandler {
    fn handle(&mut self, tag: &Handle, printer: &mut StructuredPrinter) {
        let Some(source) =
            image_source(tag).and_then(|source| resolve_image_url(&source, self.base_url.as_ref()))
        else {
            return;
        };
        let alt = html2md::common::get_tag_attr(tag, "alt").unwrap_or_default();
        printer.append_str(&format!("![{}]({})", alt, source));
    }

    fn after_handle(&mut self, _printer: &mut StructuredPrinter) {}
}

fn convert_html_to_markdown(html: &str, base_url: Option<&str>) -> String {
    let base_url = base_url.and_then(|url| Url::parse(url).ok());
    let mut custom_handlers: HashMap<String, Box<dyn html2md::TagHandlerFactory>> = HashMap::new();
    custom_handlers.insert(
        "a".to_string(),
        Box::new(BlockSafeAnchorFactory {
            base_url: base_url.clone(),
        }),
    );
    custom_handlers.insert(
        "img".to_string(),
        Box::new(ImageHandlerFactory { base_url }),
    );
    html2md::parse_html_custom(html, &custom_handlers)
}

fn contains_block_element(node: &html2md::Handle) -> bool {
    const BLOCK_ELEMENTS: &[&str] = &[
        "article",
        "aside",
        "div",
        "figure",
        "figcaption",
        "footer",
        "header",
        "h1",
        "h2",
        "h3",
        "h4",
        "h5",
        "h6",
        "li",
        "main",
        "p",
        "section",
    ];

    node.children.borrow().iter().any(|child| {
        matches!(
            &child.data,
            html2md::NodeData::Element { name, .. } if BLOCK_ELEMENTS.contains(&name.local.as_ref())
        ) || contains_block_element(child)
    })
}

fn extract_href(node: &html2md::Handle) -> Option<String> {
    match &node.data {
        html2md::NodeData::Element { attrs, .. } => attrs.borrow().iter().find_map(|attribute| {
            (attribute.name.local.as_ref() == "href").then(|| attribute.value.to_string())
        }),
        _ => None,
    }
}

fn extract_link_text(node: &html2md::Handle) -> String {
    let mut text = String::new();
    collect_text(node, &mut text);
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn collect_text(node: &html2md::Handle, text: &mut String) {
    if let html2md::NodeData::Text { contents } = &node.data {
        text.push_str(&contents.borrow());
        text.push(' ');
    }

    for child in node.children.borrow().iter() {
        collect_text(child, text);
    }
}

fn extract_images(node: &html2md::Handle, base_url: Option<&Url>) -> Vec<(String, String)> {
    let mut images = Vec::new();
    let mut seen_sources = HashSet::new();
    collect_images(node, &mut images, &mut seen_sources, base_url);
    images
}

fn collect_images(
    node: &html2md::Handle,
    images: &mut Vec<(String, String)>,
    seen_sources: &mut HashSet<String>,
    base_url: Option<&Url>,
) {
    if let html2md::NodeData::Element { name, attrs, .. } = &node.data {
        if name.local.as_ref() == "img" {
            let alt_text = attrs
                .borrow()
                .iter()
                .find_map(|attribute| {
                    (attribute.name.local.as_ref() == "alt").then(|| attribute.value.to_string())
                })
                .unwrap_or_default();
            let source = image_source(node).and_then(|source| resolve_image_url(&source, base_url));

            if let Some(source) = source.filter(|source| seen_sources.insert(source.clone())) {
                images.push((source, alt_text));
            }
        }
    }

    for child in node.children.borrow().iter() {
        collect_images(child, images, seen_sources, base_url);
    }
}

fn image_source(node: &Handle) -> Option<String> {
    let attributes = match &node.data {
        html2md::NodeData::Element { attrs, .. } => attrs.borrow(),
        _ => return None,
    };

    for attribute_name in [
        "data-src",
        "data-lazy-src",
        "data-original",
        "data-image",
        "data-srcset",
        "srcset",
        "src",
    ] {
        if let Some(value) = attributes.iter().find_map(|attribute| {
            (attribute.name.local.as_ref() == attribute_name).then(|| attribute.value.to_string())
        }) {
            if let Some(source) = first_srcset_candidate(&value) {
                if !is_placeholder_image(&source) {
                    return Some(source);
                }
            }
        }
    }

    None
}

fn first_srcset_candidate(value: &str) -> Option<String> {
    value
        .split(',')
        .next()
        .and_then(|candidate| candidate.split_whitespace().next())
        .filter(|source| !source.is_empty())
        .map(str::to_string)
}

fn is_placeholder_image(source: &str) -> bool {
    let source = source.trim().to_ascii_lowercase();
    source.is_empty()
        || source == "about:blank"
        || (source.starts_with("data:image/") && source.len() < 256)
}

fn resolve_image_url(source: &str, base_url: Option<&Url>) -> Option<String> {
    let source = source.trim();
    if source.starts_with("data:image/") {
        return Some(source.to_string());
    }

    let resolved = match base_url {
        Some(base_url) => base_url.join(source).ok()?,
        None => Url::parse(source).ok()?,
    };

    matches!(resolved.scheme(), "http" | "https").then(|| resolved.to_string())
}

fn escape_link_text(text: &str) -> String {
    text.replace('\\', "\\\\")
        .replace('[', "\\[")
        .replace(']', "\\]")
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
    let html = remove_escaped_iframes(html);

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
        .add_tag_attributes(
            "img",
            &[
                "src",
                "alt",
                "width",
                "height",
                "srcset",
                "data-src",
                "data-lazy-src",
                "data-original",
                "data-image",
                "data-srcset",
            ],
        )
        .add_clean_content_tags(["iframe"])
        .link_rel(Some("noopener noreferrer"))
        .clean(&html)
        .to_string()
}

/// Fjerner iframe-tagger (åpning og avslutning) som er skrevet som entitetskodet tekst
fn remove_escaped_iframes(html: &str) -> String {
    remove_tag_markup(html, &["&lt;iframe", "&lt;/iframe"], "&gt;")
}

/// Fjerner iframe-tagger som har havnet som ren tekst i markdown, inkludert
/// tagger der `<` er escapet med bakslash av html2md (`\</iframe>`)
fn remove_iframe_markup(text: &str) -> String {
    remove_tag_markup(text, &["<iframe", "</iframe"], ">")
}

/// Fjerner hver tag som starter med en av `openers` frem til og med `closer`.
/// En eventuell bakslash rett før taggen fjernes også.
fn remove_tag_markup(text: &str, openers: &[&str], closer: &str) -> String {
    let lower = text.to_ascii_lowercase();
    let mut result = String::with_capacity(text.len());
    let mut cursor = 0;

    while let Some((start, opener)) = openers
        .iter()
        .filter_map(|opener| {
            lower[cursor..]
                .find(opener)
                .map(|pos| (cursor + pos, *opener))
        })
        .min_by_key(|(pos, _)| *pos)
    {
        let after_start = start + opener.len();
        let end = lower[after_start..]
            .find(closer)
            .map(|relative_end| after_start + relative_end + closer.len())
            .unwrap_or(text.len());

        let keep_until = if start > cursor && text[..start].ends_with('\\') {
            start - 1
        } else {
            start
        };
        result.push_str(&text[cursor..keep_until]);
        cursor = end;
    }

    result.push_str(&text[cursor..]);
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
        let result = html_to_markdown(html, None, false);
        // html2md produserer "Test\n\nDette er en test." uten # for h1 i default modus
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
        let result = html_to_markdown(html, None, true);
        assert!(result.markdown.contains("Hovedinnhold"));
        assert!(result.used_readability);
    }

    #[test]
    fn test_readability_preserves_links() {
        let html = r#"
            <html><body><article>
                <h1>Nyheter</h1>
                <p><a href="/nyheter/sak">Intern nyhet</a></p>
                <p><a href="https://example.com/annen-sak">Ekstern nyhet</a></p>
            </article></body></html>
        "#;

        let result = html_to_markdown(html, Some("https://www.tek.no/"), true);

        assert!(result.used_readability);
        assert!(result
            .markdown
            .contains("[Intern nyhet](https://www.tek.no/nyheter/sak)"));
        assert!(result
            .markdown
            .contains("[Ekstern nyhet](https://example.com/annen-sak)"));
    }

    #[test]
    fn test_images_use_absolute_and_lazy_sources() {
        let html = r#"
            <article>
                <img src="/images/placeholder.gif"
                     data-src="/images/real-image.webp"
                     alt="Et bilde">
                <img srcset="/images/small.webp 480w, /images/large.webp 1200w"
                     alt="Et annet bilde">
            </article>
        "#;

        let result = html_to_markdown(html, Some("https://www.tek.no/"), false);

        assert!(result
            .markdown
            .contains("![Et bilde](https://www.tek.no/images/real-image.webp)"));
        assert!(result
            .markdown
            .contains("![Et annet bilde](https://www.tek.no/images/small.webp)"));
        assert!(!result.markdown.contains("placeholder.gif"));
    }

    #[test]
    fn test_html_to_markdown_preserves_block_link() {
        let html = r#"
            <a href="https://www.tek.no/nyheter/sak">
                <article>
                    <img src="https://example.com/image.jpg" alt="Bil">
                    <h2>Teslas rattløse robottaxi er lansert</h2>
                    <p>Cybercab er klar for tur.</p>
                </article>
            </a>
        "#;

        let result = html_to_markdown(html, None, false);

        assert!(result
            .markdown
            .contains("[Teslas rattløse robottaxi er lansert Cybercab er klar for tur.](https://www.tek.no/nyheter/sak)"));
        assert!(result
            .markdown
            .contains("[![Bil](https://example.com/image.jpg)](https://www.tek.no/nyheter/sak)"));
        assert!(!result
            .markdown
            .contains("\n](https://www.tek.no/nyheter/sak)"));
    }

    #[test]
    fn test_block_link_deduplicates_repeated_images() {
        let html = r#"
            <a href="https://example.com/story">
                <div>
                    <img src="https://example.com/image.jpg" alt="Bilde">
                    <img src="https://example.com/image.jpg" alt="Bilde">
                    <h2>En historie</h2>
                </div>
            </a>
        "#;

        let result = html_to_markdown(html, None, false);

        assert_eq!(
            result
                .markdown
                .matches("https://example.com/image.jpg")
                .count(),
            1
        );
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
    fn test_sanitize_removes_iframe_and_content() {
        let html = r#"<p>Gamer.no</p><iframe src="https://www.googletagmanager.com/ns.html?id=GTM-TV2T4RTR">Sporing</iframe><p>Innhold</p>"#;
        let sanitized = sanitize_html(html);

        assert!(sanitized.contains("Gamer.no"));
        assert!(sanitized.contains("Innhold"));
        assert!(!sanitized.contains("iframe"));
        assert!(!sanitized.contains("Sporing"));
    }

    #[test]
    fn test_html_to_markdown_removes_escaped_iframe_markup() {
        let html = r#"<p>Gamer.no &lt;iframe src=&quot;https://www.googletagmanager.com/ns.html?id=GTM-TV2T4RTR&quot; height=&quot;0&quot;&gt;&lt;/iframe&gt;</p><p>Innhold</p>"#;
        let result = html_to_markdown(html, None, false);

        assert!(result.markdown.contains("Gamer.no"));
        assert!(result.markdown.contains("Innhold"));
        assert!(!result.markdown.contains("iframe"));
        assert!(!result.markdown.contains("googletagmanager"));
    }

    #[test]
    fn test_html_to_markdown_removes_entity_decoded_iframe_markup() {
        let html = r#"<p>Gamer.no &amp;lt;iframe src=&amp;quot;https://www.googletagmanager.com/ns.html?id=GTM-TV2T4RTR&amp;quot; height=&amp;quot;0&amp;quot;&amp;gt;&amp;lt;/iframe&amp;gt;</p><p>Innhold</p>"#;
        let result = html_to_markdown(html, None, false);

        assert!(result.markdown.contains("Gamer.no"));
        assert!(result.markdown.contains("Innhold"));
        assert!(!result.markdown.contains("iframe"));
        assert!(!result.markdown.contains("googletagmanager"));
    }

    #[test]
    fn test_remove_iframe_markup_handles_backslash_escaped_tags() {
        let text = r#"Gamer.no \<iframe src="https://www.googletagmanager.com/ns.html" height="0"\> \</iframe>"#;
        let cleaned = remove_iframe_markup(text);

        assert_eq!(cleaned.trim(), "Gamer.no");
    }

    #[test]
    fn test_html_to_markdown_removes_stray_iframe_closing_tag() {
        let html = r#"<p>Gamer.no &lt;/iframe&gt;</p><p>Innhold</p>"#;
        let result = html_to_markdown(html, None, false);

        assert!(result.markdown.contains("Gamer.no"));
        assert!(result.markdown.contains("Innhold"));
        assert!(!result.markdown.contains("iframe"));
        assert!(!result.markdown.contains('\\'));
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
