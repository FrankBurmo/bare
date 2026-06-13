//! Markdown rendering modul
//!
//! Bruker pulldown-cmark for å konvertere markdown til HTML,
//! og syntect for server-side syntaksutheving.
//! Genererer også automatiske ID-er for overskrifter og innholdsfortegnelse (TOC).

use ammonia::Builder;
use pulldown_cmark::{html, CodeBlockKind, Event, Options, Parser, Tag, TagEnd};
use slug::slugify;
use std::collections::HashSet;
use syntect::easy::HighlightLines;
use syntect::highlighting::ThemeSet;
use syntect::html::{styled_line_to_highlighted_html, IncludeBackground};
use syntect::parsing::SyntaxSet;

#[derive(Debug)]
struct Heading {
    level: u32,
    text: String,
    id: String,
}

/// Rendrer markdown-innhold til HTML
///
/// # Arguments
/// * `content` - Markdown-tekst som skal konverteres
///
/// # Returns
/// HTML-representasjon av markdown-innholdet inkludert TOC
pub fn render(content: &str) -> String {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TASKLISTS);
    options.insert(Options::ENABLE_HEADING_ATTRIBUTES);
    options.insert(Options::ENABLE_FOOTNOTES);

    let ss = SyntaxSet::load_defaults_newlines();
    let ts = ThemeSet::load_defaults();
    let theme = &ts.themes["base16-ocean.dark"];

    let parser = Parser::new_ext(content, options);

    let mut events = Vec::new();
    let mut in_code_block = false;
    let mut code_lang = String::new();
    let mut code_content = String::new();

    let mut headings = Vec::new();
    let mut in_heading = false;
    let mut current_heading_level = 1;
    let mut current_heading_text = String::new();

    for event in parser {
        let mut skip_push = false;

        match &event {
            // Syntaksutheving
            Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(lang))) => {
                in_code_block = true;
                code_lang = lang.to_string();
                code_content.clear();
                skip_push = true;
            }
            Event::End(TagEnd::CodeBlock) if in_code_block => {
                in_code_block = false;
                let syntax = ss
                    .find_syntax_by_token(&code_lang)
                    .unwrap_or_else(|| ss.find_syntax_plain_text());
                let mut h = HighlightLines::new(syntax, theme);

                let mut html = String::new();
                html.push_str("<pre class=\"syntax-highlight\"><code>");

                for line in code_content.lines() {
                    let regions = h.highlight_line(line, &ss).unwrap_or_default();
                    let line_html =
                        styled_line_to_highlighted_html(&regions, IncludeBackground::No)
                            .unwrap_or_else(|_| line.to_string());
                    html.push_str(&line_html);
                    html.push('\n');
                }

                html.push_str("</code></pre>");
                events.push(Event::Html(html.into()));
                skip_push = true;
            }
            Event::Text(t) if in_code_block => {
                code_content.push_str(t);
                skip_push = true;
            }

            // Overskrifter og TOC
            Event::Start(Tag::Heading { level, .. }) => {
                in_heading = true;
                current_heading_level = *level as u32;
                current_heading_text.clear();
            }
            Event::End(TagEnd::Heading(_)) if in_heading => {
                in_heading = false;
                let id = slugify(&current_heading_text);
                headings.push(Heading {
                    level: current_heading_level,
                    text: current_heading_text.clone(),
                    id,
                });
            }
            Event::Text(t) if in_heading => {
                current_heading_text.push_str(t);
            }
            _ => {}
        }

        if !skip_push && !in_code_block {
            events.push(event);
        }
    }

    // Pass 2: Oppdater heading-events med ID-er
    let mut heading_idx = 0;
    let mut final_events = Vec::new();
    for event in events {
        match event {
            Event::Start(Tag::Heading {
                level,
                classes,
                attrs,
                ..
            }) => {
                if heading_idx < headings.len() {
                    let id = Some(headings[heading_idx].id.clone().into());
                    final_events.push(Event::Start(Tag::Heading {
                        level,
                        id,
                        classes,
                        attrs,
                    }));
                } else {
                    final_events.push(Event::Start(Tag::Heading {
                        level,
                        id: None,
                        classes,
                        attrs,
                    }));
                }
            }
            Event::End(TagEnd::Heading(_)) => {
                final_events.push(event);
                heading_idx += 1;
            }
            _ => final_events.push(event),
        }
    }

    let mut html_output = String::new();

    // Generer TOC hvis vi har nok overskrifter (f.eks. > 2)
    if headings.len() > 2 {
        html_output.push_str("<details class=\"toc-container\">\n");
        html_output.push_str("<summary>Innholdsfortegnelse</summary>\n");
        html_output.push_str("<ul class=\"toc-list\">\n");
        for h in &headings {
            if h.level > 3 {
                continue;
            } // Bare vis opp til H3 i TOC
            let indent = (h.level - 1) * 20;
            html_output.push_str(&format!(
                "<li style=\"padding-left: {}px\"><a href=\"#{}\">{}</a></li>\n",
                indent, h.id, h.text
            ));
        }
        html_output.push_str("</ul>\n");
        html_output.push_str("</details>\n");
    }

    html::push_html(&mut html_output, final_events.into_iter());

    sanitize_rendered_html(&html_output)
}

/// Sanitize HTML-output fra markdown-rendereren
fn sanitize_rendered_html(html: &str) -> String {
    let mut allowed_tags: HashSet<&str> = HashSet::new();
    for tag in &[
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
        "input",
    ] {
        allowed_tags.insert(tag);
    }

    Builder::default()
        .tags(allowed_tags)
        .add_generic_attributes(&["id", "class", "title", "lang", "style"])
        .add_tag_attributes("a", &["href", "target"])
        .add_tag_attributes("img", &["src", "alt", "width", "height"])
        .add_tag_attributes("input", &["type", "checked", "disabled"])
        .link_rel(Some("noopener noreferrer"))
        .clean(html)
        .to_string()
}

/// Ekstraherer tittelen fra markdown-innhold (første H1)
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
    fn test_render_heading_with_id() {
        let input = "# Hello World";
        let output = render(input);
        assert!(output.contains("<h1 id=\"hello-world\">"));
    }

    #[test]
    fn test_render_toc() {
        let input = "# H1\n## H2\n### H3\n#### H4";
        let output = render(input);
        assert!(output.contains("toc-container"));
        assert!(output.contains("href=\"#h1\""));
        assert!(output.contains("href=\"#h2\""));
        assert!(output.contains("href=\"#h3\""));
        assert!(!output.contains("href=\"#h4\"")); // H4 skal ikke være i TOC
    }

    #[test]
    fn test_render_code_block() {
        let input = "```rust\nfn main() {}\n```";
        let output = render(input);
        assert!(output.contains("syntax-highlight"));
    }
}
