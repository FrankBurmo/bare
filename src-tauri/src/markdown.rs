//! Markdown rendering modul
//!
//! Bruker pulldown-cmark for å konvertere markdown til HTML.

use pulldown_cmark::{html, Options, Parser};
use ammonia::Builder;
use std::collections::HashSet;

/// Rendrer markdown-innhold til HTML
///
/// # Arguments
/// * `content` - Markdown-tekst som skal konverteres
///
/// # Returns
/// HTML-representasjon av markdown-innholdet
pub fn render(content: &str) -> String {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TASKLISTS);
    options.insert(Options::ENABLE_HEADING_ATTRIBUTES);

    let parser = Parser::new_ext(content, options);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);

    // Sanitize output for å fjerne eventuell rå HTML som pulldown-cmark slapp gjennom
    sanitize_rendered_html(&html_output)
}

/// Sanitize HTML-output fra markdown-rendereren
fn sanitize_rendered_html(html: &str) -> String {
    let mut allowed_tags: HashSet<&str> = HashSet::new();
    for tag in &[
        "main", "article", "section", "aside", "header", "footer", "nav",
        "div", "span", "p", "br", "hr", "h1", "h2", "h3", "h4", "h5", "h6",
        "ul", "ol", "li", "dl", "dt", "dd", "table", "thead", "tbody", "tfoot",
        "tr", "th", "td", "a", "img", "figure", "figcaption", "blockquote",
        "pre", "code", "em", "strong", "b", "i", "u", "s", "del", "ins",
        "sub", "sup", "small", "mark", "abbr", "time", "address", "details", "summary",
        "input", // For task lists
    ] {
        allowed_tags.insert(tag);
    }

    Builder::default()
        .tags(allowed_tags)
        .add_generic_attributes(&["id", "class", "title", "lang"])
        .add_tag_attributes("a", &["href", "target"])
        .add_tag_attributes("img", &["src", "alt", "width", "height"])
        .add_tag_attributes("input", &["type", "checked", "disabled"])
        .link_rel(Some("noopener noreferrer"))
        .clean(html)
        .to_string()
}

/// Ekstraherer tittelen fra markdown-innhold (første H1)
///
/// # Arguments
/// * `content` - Markdown-tekst
///
/// # Returns
/// Tittel hvis funnet, ellers None
pub fn extract_title(content: &str) -> Option<String> {
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(title) = trimmed.strip_prefix("# ") {
            return Some(title.trim().to_string());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_heading() {
        let input = "# Hello World";
        let output = render(input);
        assert!(output.contains("<h1>"));
        assert!(output.contains("Hello World"));
    }

    #[test]
    fn test_render_bold() {
        let input = "This is **bold** text";
        let output = render(input);
        assert!(output.contains("<strong>bold</strong>"));
    }

    #[test]
    fn test_render_italic() {
        let input = "This is *italic* text";
        let output = render(input);
        assert!(output.contains("<em>italic</em>"));
    }

    #[test]
    fn test_render_link() {
        let input = "[Link](https://example.com)";
        let output = render(input);
        assert!(output.contains("<a href=\"https://example.com\""));
        assert!(output.contains("Link</a>"));
    }

    #[test]
    fn test_render_list() {
        let input = "- Item 1\n- Item 2\n- Item 3";
        let output = render(input);
        assert!(output.contains("<ul>"));
        assert!(output.contains("<li>Item 1</li>"));
    }

    #[test]
    fn test_render_code_block() {
        let input = "```rust\nfn main() {}\n```";
        let output = render(input);
        assert!(output.contains("<pre>"));
        assert!(output.contains("<code"));
    }

    #[test]
    fn test_render_table() {
        let input = "| A | B |\n|---|---|\n| 1 | 2 |";
        let output = render(input);
        assert!(output.contains("<table>"));
        assert!(output.contains("<th>"));
    }

    #[test]
    fn test_render_task_list() {
        let input = "- [ ] Todo\n- [x] Done";
        let output = render(input);
        assert!(output.contains("type=\"checkbox\""));
    }

    #[test]
    fn test_extract_title_found() {
        let input = "# My Title\n\nSome content";
        let title = extract_title(input);
        assert_eq!(title, Some("My Title".to_string()));
    }

    #[test]
    fn test_extract_title_not_found() {
        let input = "Some content without heading";
        let title = extract_title(input);
        assert_eq!(title, None);
    }

    #[test]
    fn test_extract_title_with_whitespace() {
        let input = "  # Spaced Title  \n\nContent";
        let title = extract_title(input);
        assert_eq!(title, Some("Spaced Title".to_string()));
    }

    #[test]
    fn test_render_sanitization() {
        let input = "Raw <script>alert('xss')</script> HTML";
        let output = render(input);
        assert!(!output.contains("<script>"));
        // Ammonia fjerner innholdet i script-tagger
        assert!(output.contains("Raw  HTML"));
    }
}
