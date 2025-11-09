use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Post {
    pub title: String,
    pub date: String,
    pub tags: Vec<String>,
    pub content_html: String,
    pub file_path: String,
}

/// Parse a post file with s-expression front matter
pub fn parse_post_file(file_path: &str, content: &str) -> Result<Post, String> {
    // Find the separator (---)
    let parts: Vec<&str> = content.splitn(2, "\n---\n").collect();

    if parts.len() != 2 {
        return Err(format!("Invalid post format in {}: missing --- separator", file_path));
    }

    let front_matter = parts[0];
    let markdown_content = parts[1];

    // Parse the front matter (s-expression)
    let metadata = parse_front_matter(front_matter)?;

    // Convert markdown to HTML
    let content_html = crate::markdown::markdown_to_html(markdown_content);

    Ok(Post {
        title: metadata.get("title").cloned().unwrap_or_else(|| "Untitled".to_string()),
        date: metadata.get("date").cloned().unwrap_or_else(|| "".to_string()),
        tags: vec![], // Simplified for now
        content_html,
        file_path: file_path.to_string(),
    })
}

/// Simple s-expression parser for front matter
fn parse_front_matter(front_matter: &str) -> Result<HashMap<String, String>, String> {
    let mut metadata = HashMap::new();

    // Remove outer (post ...) wrapper
    let trimmed = front_matter.trim();
    if !trimmed.starts_with("(post") {
        return Err("Front matter must start with (post".to_string());
    }

    // Simple parser: extract (key value) pairs
    let mut in_paren = false;
    let mut current_key = String::new();
    let mut current_value = String::new();
    let mut in_key = false;
    let mut in_value = false;
    let mut in_string = false;
    let mut paren_depth = 0;

    for ch in trimmed.chars() {
        match ch {
            '(' => {
                paren_depth += 1;
                if paren_depth == 2 {
                    in_paren = true;
                    in_key = true;
                    current_key.clear();
                    current_value.clear();
                }
            }
            ')' => {
                paren_depth -= 1;
                if in_paren && paren_depth == 1 {
                    in_paren = false;
                    in_key = false;
                    in_value = false;
                    in_string = false;
                    if !current_key.is_empty() {
                        metadata.insert(current_key.clone(), current_value.trim().to_string());
                    }
                }
            }
            '"' if in_paren => {
                in_string = !in_string;
            }
            ' ' | '\n' | '\t' if in_paren && !in_string => {
                if in_key && !current_key.is_empty() {
                    in_key = false;
                    in_value = true;
                }
            }
            _ if in_paren => {
                if in_key {
                    current_key.push(ch);
                } else if in_value {
                    current_value.push(ch);
                }
            }
            _ => {}
        }
    }

    Ok(metadata)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_front_matter() {
        let front_matter = r#"(post
 (title "Hello World")
 (date "2025-01-01T12:00:00+00:00")
 (tags ()))"#;

        let metadata = parse_front_matter(front_matter).unwrap();
        assert_eq!(metadata.get("title"), Some(&"Hello World".to_string()));
        assert_eq!(metadata.get("date"), Some(&"2025-01-01T12:00:00+00:00".to_string()));
    }
}
