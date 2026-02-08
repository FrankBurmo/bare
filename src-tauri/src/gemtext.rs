//! Gemtext til Markdown konverterer
//!
//! Konverterer text/gemini (gemtext) format til standard Markdown.
//! Gemtext er et enkelt, linjebasert format brukt av Gemini-protokollen.

/// Resultat fra gemtext-konvertering
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct GemtextResult {
    /// Konvertert markdown-innhold
    pub markdown: String,
    /// Tittel ekstrahert fra første heading
    pub title: Option<String>,
    /// Lenker funnet i dokumentet
    pub links: Vec<GeminiLink>,
}

/// En lenke funnet i gemtext-innhold
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct GeminiLink {
    /// URL lenken peker til
    pub url: String,
    /// Visningstekst (kan være tom)
    pub text: Option<String>,
}

/// Konverterer gemtext-innhold til markdown
///
/// # Arguments
/// * `input` - Gemtext-innhold som skal konverteres
///
/// # Returns
/// GemtextResult med markdown, tittel og lenker
pub fn gemtext_to_markdown(input: &str) -> GemtextResult {
    let mut markdown_lines: Vec<String> = Vec::new();
    let mut title: Option<String> = None;
    let mut links: Vec<GeminiLink> = Vec::new();
    let mut in_preformatted = false;

    for line in input.lines() {
        // Håndter preformatert blokk (toggle)
        if let Some(rest) = line.strip_prefix("```") {
            in_preformatted = !in_preformatted;
            if in_preformatted {
                // Bruk alt-tekst som språk-hint hvis tilgjengelig
                let alt_text = rest.trim();
                if alt_text.is_empty() {
                    markdown_lines.push("```".to_string());
                } else {
                    markdown_lines.push(format!("```{}", alt_text));
                }
            } else {
                markdown_lines.push("```".to_string());
            }
            continue;
        }

        // I preformatert blokk: bevar innholdet uendret
        if in_preformatted {
            markdown_lines.push(line.to_string());
            continue;
        }

        // Lenke-linjer: => URL [tekst]
        if let Some(rest) = line.strip_prefix("=>") {
            let rest = rest.trim();
            if let Some((url, text)) = parse_link_line(rest) {
                let display_text = text.clone().unwrap_or_else(|| url.clone());
                links.push(GeminiLink {
                    url: url.clone(),
                    text: text.clone(),
                });
                markdown_lines.push(format!("[{}]({})", display_text, url));
            } else if !rest.is_empty() {
                // Bare en URL uten mellomrom
                links.push(GeminiLink {
                    url: rest.to_string(),
                    text: None,
                });
                markdown_lines.push(format!("[{}]({})", rest, rest));
            }
            continue;
        }

        // Headings (identisk syntax som markdown)
        if let Some(heading_text) = line.strip_prefix("### ") {
            let heading_text = heading_text.trim();
            if title.is_none() {
                title = Some(heading_text.to_string());
            }
            markdown_lines.push(line.to_string());
            continue;
        }
        if let Some(heading_text) = line.strip_prefix("## ") {
            let heading_text = heading_text.trim();
            if title.is_none() {
                title = Some(heading_text.to_string());
            }
            markdown_lines.push(line.to_string());
            continue;
        }
        if let Some(heading_text) = line.strip_prefix("# ") {
            let heading_text = heading_text.trim();
            if title.is_none() {
                title = Some(heading_text.to_string());
            }
            markdown_lines.push(line.to_string());
            continue;
        }

        // Listeelementer: * element → - element
        if let Some(rest) = line.strip_prefix("* ") {
            markdown_lines.push(format!("- {}", rest));
            continue;
        }

        // Sitatblokker (identisk syntax)
        if line.starts_with('>') {
            markdown_lines.push(line.to_string());
            continue;
        }

        // Vanlig tekst — bevar som den er
        markdown_lines.push(line.to_string());
    }

    // Lukk eventuell åpen preformatert blokk
    if in_preformatted {
        markdown_lines.push("```".to_string());
    }

    GemtextResult {
        markdown: markdown_lines.join("\n"),
        title,
        links,
    }
}

/// Parser en lenke-linje etter "=>" prefikset
///
/// Formater:
/// - "url tekst her" → Some((url, Some(tekst)))
/// - "url" → Some((url, None))
/// - "" → None
fn parse_link_line(rest: &str) -> Option<(String, Option<String>)> {
    if rest.is_empty() {
        return None;
    }

    // Finn første mellomrom/tab som skiller URL fra tekst
    let url_end = rest.find([' ', '\t']).unwrap_or(rest.len());

    let url = rest[..url_end].to_string();
    let text = if url_end < rest.len() {
        let t = rest[url_end..].trim();
        if t.is_empty() {
            None
        } else {
            Some(t.to_string())
        }
    } else {
        None
    };

    Some((url, text))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plain_text() {
        let result = gemtext_to_markdown("Hello, world!");
        assert_eq!(result.markdown, "Hello, world!");
        assert!(result.title.is_none());
    }

    #[test]
    fn test_headings() {
        let input = "# Heading 1\n## Heading 2\n### Heading 3";
        let result = gemtext_to_markdown(input);
        assert_eq!(result.markdown, "# Heading 1\n## Heading 2\n### Heading 3");
        assert_eq!(result.title, Some("Heading 1".to_string()));
    }

    #[test]
    fn test_title_from_h2() {
        let input = "Some text\n## First Heading";
        let result = gemtext_to_markdown(input);
        assert_eq!(result.title, Some("First Heading".to_string()));
    }

    #[test]
    fn test_link_with_text() {
        let input = "=> gemini://example.com Example Site";
        let result = gemtext_to_markdown(input);
        assert_eq!(result.markdown, "[Example Site](gemini://example.com)");
        assert_eq!(result.links.len(), 1);
        assert_eq!(result.links[0].url, "gemini://example.com");
        assert_eq!(result.links[0].text, Some("Example Site".to_string()));
    }

    #[test]
    fn test_link_without_text() {
        let input = "=> gemini://example.com";
        let result = gemtext_to_markdown(input);
        assert_eq!(
            result.markdown,
            "[gemini://example.com](gemini://example.com)"
        );
        assert_eq!(result.links.len(), 1);
        assert!(result.links[0].text.is_none());
    }

    #[test]
    fn test_link_with_tab_separator() {
        let input = "=> gemini://example.com\tA tab-separated link";
        let result = gemtext_to_markdown(input);
        assert_eq!(
            result.markdown,
            "[A tab-separated link](gemini://example.com)"
        );
    }

    #[test]
    fn test_link_with_extra_spaces() {
        let input = "=>   gemini://example.com   Spaced Link  ";
        let result = gemtext_to_markdown(input);
        assert_eq!(result.markdown, "[Spaced Link](gemini://example.com)");
    }

    #[test]
    fn test_list_items() {
        let input = "* First item\n* Second item\n* Third item";
        let result = gemtext_to_markdown(input);
        assert_eq!(result.markdown, "- First item\n- Second item\n- Third item");
    }

    #[test]
    fn test_blockquote() {
        let input = "> This is a quote";
        let result = gemtext_to_markdown(input);
        assert_eq!(result.markdown, "> This is a quote");
    }

    #[test]
    fn test_preformatted_block() {
        let input = "```rust\nfn main() {\n    println!(\"Hello\");\n}\n```";
        let result = gemtext_to_markdown(input);
        assert_eq!(
            result.markdown,
            "```rust\nfn main() {\n    println!(\"Hello\");\n}\n```"
        );
    }

    #[test]
    fn test_preformatted_without_alt() {
        let input = "```\nsome code\n```";
        let result = gemtext_to_markdown(input);
        assert_eq!(result.markdown, "```\nsome code\n```");
    }

    #[test]
    fn test_links_not_parsed_in_preformatted() {
        let input = "```\n=> gemini://example.com Not a link\n```";
        let result = gemtext_to_markdown(input);
        assert!(result.links.is_empty());
        assert!(result
            .markdown
            .contains("=> gemini://example.com Not a link"));
    }

    #[test]
    fn test_empty_input() {
        let result = gemtext_to_markdown("");
        assert_eq!(result.markdown, "");
        assert!(result.title.is_none());
        assert!(result.links.is_empty());
    }

    #[test]
    fn test_complete_document() {
        let input = "\
# Welcome to my capsule

This is a gemini page.

=> gemini://example.com Home
=> gemini://other.com/page Other Page

## Links

* Item one
* Item two

> A wise quote

```python
print('hello')
```

That's all folks.";

        let result = gemtext_to_markdown(input);
        assert_eq!(result.title, Some("Welcome to my capsule".to_string()));
        assert_eq!(result.links.len(), 2);
        assert!(result.markdown.contains("[Home](gemini://example.com)"));
        assert!(result
            .markdown
            .contains("[Other Page](gemini://other.com/page)"));
        assert!(result.markdown.contains("- Item one"));
        assert!(result.markdown.contains("> A wise quote"));
        assert!(result.markdown.contains("```python"));
    }

    #[test]
    fn test_unclosed_preformatted() {
        let input = "```\nunclosed block";
        let result = gemtext_to_markdown(input);
        assert!(result.markdown.ends_with("```"));
    }

    #[test]
    fn test_empty_link_line() {
        let input = "=>";
        let result = gemtext_to_markdown(input);
        assert!(result.links.is_empty());
    }

    #[test]
    fn test_mixed_headings_title_first_wins() {
        let input = "## Second level\n# First level";
        let result = gemtext_to_markdown(input);
        assert_eq!(result.title, Some("Second level".to_string()));
    }
}
