use chrono::{DateTime, Utc};
use chrono_tz::Europe::London;
use serde::Deserialize;
use steel::rvals::{SteelVal, IntoSteelVal, SteelHashMap};
use steel::gc::Gc;
use steel::HashMap;

#[derive(Debug, Clone)]
pub struct Post {
    pub title: String,
    pub description: Option<String>,
    pub date: DateTime<Utc>,
    pub content_html: String,
    #[allow(dead_code)]
    pub file_path: String,
}

#[derive(Debug, Deserialize)]
struct FrontMatter {
    title: String,
    description: Option<String>,
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

    let date = parse_date(&metadata.date)
        .ok_or_else(|| format!("Invalid date in {}: {}", file_path, metadata.date))?;

    // Convert markdown to HTML
    let content_html = crate::markdown::markdown_to_html(markdown_content);

    Ok(Post {
        title: metadata.title,
        description: metadata.description,
        date,
        content_html,
        file_path: file_path.to_string(),
    })
}

/// Parses either a full ISO 8601 timestamp (with explicit offset) or a bare
/// date like "Mar 24 2025", which is treated as midnight UTC (matching the
/// behavior of JavaScript's `Date` constructor on date-only strings, which
/// the Astro golden master relied on).
fn parse_date(raw: &str) -> Option<DateTime<Utc>> {
    if let Ok(dt) = DateTime::parse_from_rfc3339(raw) {
        return Some(dt.with_timezone(&Utc));
    }
    let naive = chrono::NaiveDate::parse_from_str(raw, "%b %d %Y").ok()?;
    Some(naive.and_hms_opt(0, 0, 0)?.and_utc())
}

/// Creates: (hash 'id "..." 'title "..." 'description "..." 'date "..." 'date-display "..." 'content "...")
pub fn post_to_steel_hash(post_id: &str, post: &Post) -> SteelVal {
    let mut map: HashMap<SteelVal, SteelVal> = HashMap::new();

    map.insert(SteelVal::SymbolV("id".into()), post_id.to_string().into_steelval().unwrap());
    map.insert(SteelVal::SymbolV("title".into()), post.title.clone().into_steelval().unwrap());
    map.insert(
        SteelVal::SymbolV("description".into()),
        post.description.clone().unwrap_or_default().into_steelval().unwrap(),
    );
    map.insert(SteelVal::SymbolV("date".into()), iso8601(&post.date).into_steelval().unwrap());
    map.insert(SteelVal::SymbolV("date-year".into()), post.date.format("%Y").to_string().into_steelval().unwrap());
    map.insert(
        SteelVal::SymbolV("date-display".into()),
        format_date_display(&post.date).into_steelval().unwrap(),
    );
    map.insert(SteelVal::SymbolV("content".into()), post.content_html.clone().into_steelval().unwrap());

    let steel_map = SteelHashMap::from(Gc::new(map));
    SteelVal::HashMapV(steel_map)
}

/// Formats a date the way Astro's `toLocaleDateString('en-us', { year, month: 'short', day })`
/// did on the original build machine, which had its local timezone set to Europe/London.
fn format_date_display(date: &DateTime<Utc>) -> String {
    let london = date.with_timezone(&London);
    format!("{}", london.format("%b %-d, %Y"))
}

/// Formats a date the way JavaScript's `Date.prototype.toISOString()` does,
/// e.g. `2004-01-15T05:23:14.000Z`.
pub fn iso8601(date: &DateTime<Utc>) -> String {
    format!("{}", date.format("%Y-%m-%dT%H:%M:%S%.3fZ"))
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
        assert_eq!(format!("{}", post.date.format("%Y-%m-%dT%H:%M:%S%.3fZ")), "2025-01-01T12:00:00.000Z");
        assert!(post.content_html.contains("Welcome"));
    }

    #[test]
    fn test_bst_display_date() {
        // 2003-05-15T23:55:03Z is 2003-05-16 00:55:03 in BST (Europe/London, UTC+1 in summer)
        let date = parse_date("2003-05-16T00:55:03+01:00").unwrap();
        assert_eq!(format_date_display(&date), "May 16, 2003");
    }

    #[test]
    fn test_bare_date_is_utc_midnight() {
        let date = parse_date("Mar 24 2025").unwrap();
        assert_eq!(format!("{}", date.format("%Y-%m-%dT%H:%M:%S%.3fZ")), "2025-03-24T00:00:00.000Z");
    }
}
