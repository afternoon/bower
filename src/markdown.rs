use pulldown_cmark::{html, CodeBlockKind, CowStr, Event, Options, Parser, Tag, TagEnd};
use std::sync::OnceLock;
use syntect::easy::HighlightLines;
use syntect::html::{styled_line_to_highlighted_html, IncludeBackground};
use syntect::parsing::SyntaxSet;
use syntect::highlighting::{Theme, ThemeSet};

fn syntax_set() -> &'static SyntaxSet {
    static SET: OnceLock<SyntaxSet> = OnceLock::new();
    SET.get_or_init(SyntaxSet::load_defaults_newlines)
}

fn theme() -> &'static Theme {
    static THEME: OnceLock<Theme> = OnceLock::new();
    THEME.get_or_init(|| {
        let mut theme_set = ThemeSet::load_defaults();
        theme_set.themes.remove("base16-ocean.dark").expect("bundled theme present")
    })
}

/// Renders a fenced code block's contents as HTML with syntax highlighting,
/// producing `<pre><code>...</code></pre>` with inline per-token color spans.
fn highlight_code_block(code: &str, lang: &str) -> String {
    let ss = syntax_set();
    let syntax = ss
        .find_syntax_by_token(lang)
        .unwrap_or_else(|| ss.find_syntax_plain_text());

    let mut highlighter = HighlightLines::new(syntax, theme());
    let mut body = String::new();

    for line in split_keep_newlines(code) {
        let regions = match highlighter.highlight_line(line, ss) {
            Ok(regions) => regions,
            Err(_) => {
                body.push_str(&html_escape(line));
                continue;
            }
        };
        match styled_line_to_highlighted_html(&regions[..], IncludeBackground::No) {
            Ok(html) => body.push_str(&html),
            Err(_) => body.push_str(&html_escape(line)),
        }
    }

    format!("<pre><code>{}</code></pre>\n", body)
}

fn split_keep_newlines(s: &str) -> Vec<&str> {
    let mut lines = Vec::new();
    let mut start = 0;
    for (i, c) in s.char_indices() {
        if c == '\n' {
            lines.push(&s[start..=i]);
            start = i + 1;
        }
    }
    if start < s.len() {
        lines.push(&s[start..]);
    }
    lines
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}

/// Parse markdown content and return HTML string. Fenced code blocks with a
/// recognized language hint are syntax-highlighted; everything else is
/// handled by pulldown-cmark's standard HTML rendering.
pub fn markdown_to_html(markdown: &str) -> String {
    let options = Options::all();
    let parser = Parser::new_ext(markdown, options);

    let mut events = Vec::new();
    let mut in_fenced_code: Option<String> = None;
    let mut code_buffer = String::new();

    for event in parser {
        match event {
            Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(lang))) => {
                in_fenced_code = Some(lang.to_string());
                code_buffer.clear();
            }
            Event::Text(text) if in_fenced_code.is_some() => {
                code_buffer.push_str(&text);
            }
            Event::End(TagEnd::CodeBlock) if in_fenced_code.is_some() => {
                let lang = in_fenced_code.take().unwrap();
                let html = if lang.is_empty() {
                    format!("<pre><code>{}</code></pre>\n", html_escape(&code_buffer))
                } else {
                    highlight_code_block(&code_buffer, &lang)
                };
                events.push(Event::Html(CowStr::from(html)));
            }
            other => events.push(other),
        }
    }

    let mut html_output = String::new();
    html::push_html(&mut html_output, events.into_iter());
    html_output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_markdown() {
        let markdown = "# Hello\n\nThis is a **test**.";
        let html = markdown_to_html(markdown);
        assert!(html.contains("<h1>"));
        assert!(html.contains("<strong>test</strong>"));
    }

    #[test]
    fn test_links() {
        let markdown = "[Link](https://example.com)";
        let html = markdown_to_html(markdown);
        assert!(html.contains("<a href=\"https://example.com\">Link</a>"));
    }

    #[test]
    fn test_fenced_code_highlighting() {
        let markdown = "```js\nvar x = 1;\n```";
        let html = markdown_to_html(markdown);
        assert!(html.contains("<pre><code>"));
        assert!(html.contains("span style"));
    }

    #[test]
    fn test_fenced_code_no_lang() {
        let markdown = "```\nplain text\n```";
        let html = markdown_to_html(markdown);
        assert!(html.contains("<pre><code>plain text\n</code></pre>"));
    }
}
