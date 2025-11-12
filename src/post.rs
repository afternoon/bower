use serde::Deserialize;
use steel::rvals::{SteelVal, IntoSteelVal, SteelHashMap};
use steel::gc::Gc;
use steel::HashMap;

#[derive(Debug, Clone)]
pub struct Post {
    pub title: String,
    pub date: String,
    pub content_html: String,
    #[allow(dead_code)]
    pub file_path: String,
}

#[derive(Debug, Deserialize)]
struct FrontMatter {
    title: String,
    date: String,
}

/// Parse a post file with YAML front matter
pub fn parse_post_file(file_path: &str, content: &str) -> Result<Post, String> {
    // YAML frontmatter format: starts with --- and ends with ---
    let trimmed = content.trim_start();

    if !trimmed.starts_with("---") {
        return Err(format!("Invalid post format in {}: must start with ---", file_path));
    }

    // Find the closing ---
    let after_first = &trimmed[3..]; // Skip first "---"
    let end_pos = after_first.find("\n---\n")
        .or_else(|| after_first.find("\n---"))
        .ok_or_else(|| format!("Invalid post format in {}: missing closing ---", file_path))?;

    let front_matter = &after_first[..end_pos];
    let markdown_content = &after_first[end_pos + 4..].trim_start(); // Skip "\n---" or "\n---\n"

    // Parse the YAML front matter
    let metadata: FrontMatter = serde_yaml::from_str(front_matter)
        .map_err(|e| format!("Failed to parse YAML frontmatter in {}: {}", file_path, e))?;

    // Convert markdown to HTML
    let content_html = crate::markdown::markdown_to_html(markdown_content);

    Ok(Post {
        title: metadata.title,
        date: metadata.date,
        content_html,
        file_path: file_path.to_string(),
    })
}

/// Creates: (hash 'id "..." 'title "..." 'date "..." 'content "...")
pub fn post_to_steel_hash(post_id: &str, post: &Post) -> SteelVal {
    // Create symbol for keys
    let id_key: SteelVal = SteelVal::SymbolV("id".into());
    let title_key: SteelVal = SteelVal::SymbolV("title".into());
    let date_key: SteelVal = SteelVal::SymbolV("date".into());
    let content_key: SteelVal = SteelVal::SymbolV("content".into());

    // Create string values
    let id_val: SteelVal = post_id.to_string().into_steelval().unwrap();
    let title_val: SteelVal = post.title.clone().into_steelval().unwrap();
    let date_val: SteelVal = post.date.clone().into_steelval().unwrap();
    let content_val: SteelVal = post.content_html.clone().into_steelval().unwrap();

    // Create a Rust HashMap and populate it
    let mut map: HashMap<SteelVal, SteelVal> = HashMap::new();
    map.insert(id_key, id_val);
    map.insert(title_key, title_val);
    map.insert(date_key, date_val);
    map.insert(content_key, content_val);

    // Convert to Steel hash map using Gc and SteelHashMap
    let steel_map = SteelHashMap::from(Gc::new(map));

    // Return as a SteelVal
    SteelVal::HashMapV(steel_map)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_yaml_frontmatter() {
        let post_content = r#"---
title: Hello World
date: 2025-01-01T12:00:00+00:00
---

# Welcome

This is the post content."#;

        let post = parse_post_file("test.md", post_content).unwrap();
        assert_eq!(post.title, "Hello World");
        assert_eq!(post.date, "2025-01-01T12:00:00+00:00");
        assert!(post.content_html.contains("Welcome"));
    }
}
