use pulldown_cmark::{html, Options, Parser};

/// Parse markdown content and return HTML string
pub fn markdown_to_html(markdown: &str) -> String {
    let options = Options::all();
    let parser = Parser::new_ext(markdown, options);

    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);

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
}
