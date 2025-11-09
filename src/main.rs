mod markdown;
mod post;
mod sexp_html;

use steel::steel_vm::engine::Engine;
use steel::rvals::SteelString;
use steel::steel_vm::register_fn::RegisterFn;
use std::fs;
use std::path::Path;

// Wrapper function that works with Steel types
fn markdown_to_html_steel(input: SteelString) -> String {
    markdown::markdown_to_html(input.as_str())
}

// Read a file's contents as a string
fn read_file_steel(path: SteelString) -> Result<String, String> {
    fs::read_to_string(path.as_str())
        .map_err(|e| format!("Failed to read file {}: {}", path.as_str(), e))
}

// Write content to a file
fn write_file_steel(path: SteelString, content: SteelString) -> Result<String, String> {
    fs::write(path.as_str(), content.as_str())
        .map(|_| format!("Written to {}", path.as_str()))
        .map_err(|e| format!("Failed to write file {}: {}", path.as_str(), e))
}

// List files in a directory
fn list_files_steel(dir: SteelString) -> Result<Vec<String>, String> {
    let entries = fs::read_dir(dir.as_str())
        .map_err(|e| format!("Failed to read directory {}: {}", dir.as_str(), e))?;

    let mut files = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|e| format!("Error reading entry: {}", e))?;
        let path = entry.path();
        if path.is_file() {
            if let Some(path_str) = path.to_str() {
                files.push(path_str.to_string());
            }
        }
    }

    Ok(files)
}

// Create a directory (including parents)
fn create_directory_steel(path: SteelString) -> Result<String, String> {
    fs::create_dir_all(path.as_str())
        .map(|_| format!("Created directory {}", path.as_str()))
        .map_err(|e| format!("Failed to create directory {}: {}", path.as_str(), e))
}

// Check if a file exists
fn file_exists_steel(path: SteelString) -> bool {
    Path::new(path.as_str()).exists()
}

// Split a string by a delimiter
fn string_split_steel(s: SteelString, delimiter: SteelString) -> Vec<String> {
    s.as_str()
        .split(delimiter.as_str())
        .map(|s| s.to_string())
        .collect()
}

// Join a list of strings with a delimiter
fn string_join_steel(strings: Vec<SteelString>, delimiter: SteelString) -> String {
    strings
        .iter()
        .map(|s| s.as_str())
        .collect::<Vec<_>>()
        .join(delimiter.as_str())
}

// Convert s-expression to HTML (simplified version)
fn sexp_to_html_steel(sexp_str: SteelString) -> Result<String, String> {
    // For now, this is a placeholder - we'll implement proper s-exp to HTML conversion
    Ok(sexp_str.to_string())
}

// Check if a string starts with a prefix
fn string_starts_with_steel(s: SteelString, prefix: SteelString) -> bool {
    s.as_str().starts_with(prefix.as_str())
}

// Number to string
fn number_to_string_steel(n: i64) -> String {
    n.to_string()
}

// Symbol to string (in Steel, we'll just pass strings)
fn displayln_steel(s: SteelString) {
    println!("{}", s.as_str());
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Bower - A Static Site Generator in Scheme\n");

    // Create build directory
    fs::create_dir_all("build")?;

    // Create Steel engine
    let mut engine = Engine::new();

    // Register helper functions (keeping them for potential use in site.scm)
    engine.register_fn("markdown->html", markdown_to_html_steel);
    engine.register_fn("read-file", read_file_steel);
    engine.register_fn("write-file", write_file_steel);
    engine.register_fn("displayln", displayln_steel);

    // Load site.scm
    let site_scm_path = "example/site.scm";
    if !Path::new(site_scm_path).exists() {
        eprintln!("Error: site.scm not found at {}", site_scm_path);
        return Ok(());
    }

    let site_scm = fs::read_to_string(site_scm_path)?;
    let site_scm_static: &'static str = Box::leak(site_scm.into_boxed_str());
    engine.run(site_scm_static)?;

    // Read site configuration
    let config_result = engine.run("site")?;
    println!("Site configuration loaded: {:?}\n", config_result);

    // Process posts
    let posts_dir = "example/posts";
    let post_files = fs::read_dir(posts_dir)?;

    let mut posts = Vec::new();

    for entry in post_files {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() && path.extension().map_or(false, |ext| ext == "md") {
            println!("Processing: {}", path.display());

            let content = fs::read_to_string(&path)?;
            let post = post::parse_post_file(path.to_str().unwrap(), &content)?;

            // Generate HTML for this post using render-post
            let post_sexp = format!(
                "(render-post site '((title \"{}\") (date \"{}\") (content \"{}\")))",
                post.title,
                post.date,
                post.content_html.replace("\"", "\\\"").replace("\n", " ")
            );
            let post_sexp_static: &'static str = Box::leak(post_sexp.into_boxed_str());

            match engine.run(post_sexp_static) {
                Ok(result) => {
                    if let Some(sexp) = result.first() {
                        let post_html = sexp_html::sexp_to_html(sexp);

                        // Wrap in render-page
                        let page_call = format!(
                            "(render-page site \"{}\")",
                            post_html.replace("\"", "\\\"").replace("\n", " ")
                        );
                        let page_call_static: &'static str = Box::leak(page_call.into_boxed_str());

                        match engine.run(page_call_static) {
                            Ok(page_result) => {
                                if let Some(page_sexp) = page_result.first() {
                                    let full_html = format!(
                                        "<!DOCTYPE html>\n{}",
                                        sexp_html::sexp_to_html(page_sexp)
                                    );

                                    // Write to build directory
                                    let output_filename = path
                                        .file_stem()
                                        .unwrap()
                                        .to_str()
                                        .unwrap();
                                    let output_path = format!("build/{}.html", output_filename);
                                    fs::write(&output_path, &full_html)?;
                                    println!("  → Generated: {}", output_path);

                                    posts.push(post);
                                }
                            }
                            Err(e) => eprintln!("  Error rendering page: {:?}", e),
                        }
                    }
                }
                Err(e) => eprintln!("  Error rendering post: {:?}", e),
            }
        }
    }

    // Generate index.html
    println!("\nGenerating index.html...");
    let post_list: Vec<String> = posts
        .iter()
        .map(|p| {
            format!(
                "<li><a href=\"{}.html\">{}</a> - {}</li>",
                Path::new(&p.file_path)
                    .file_stem()
                    .unwrap()
                    .to_str()
                    .unwrap(),
                p.title,
                p.date
            )
        })
        .collect();

    let index_content = format!("<h2>All Posts</h2><ul>{}</ul>", post_list.join(""));

    let index_call = format!(
        "(render-page site \"{}\")",
        index_content.replace("\"", "\\\"")
    );
    let index_call_static: &'static str = Box::leak(index_call.into_boxed_str());

    match engine.run(index_call_static) {
        Ok(result) => {
            if let Some(sexp) = result.first() {
                let full_html = format!("<!DOCTYPE html>\n{}", sexp_html::sexp_to_html(sexp));
                fs::write("build/index.html", &full_html)?;
                println!("  → Generated: build/index.html");
            }
        }
        Err(e) => eprintln!("Error generating index: {:?}", e),
    }

    println!("\n✓ Site built successfully!");
    println!("  Output directory: build/");
    println!("  Posts generated: {}", posts.len());

    Ok(())
}
