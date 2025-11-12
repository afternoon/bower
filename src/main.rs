mod markdown;
mod post;
mod sexp_html;

use steel::steel_vm::engine::Engine;
use steel::rvals::{SteelString, SteelVal, IntoSteelVal, SteelHashMap};
use steel::steel_vm::register_fn::RegisterFn;
use steel::gc::Gc;
use steel::HashMap;
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

// Symbol to string (in Steel, we'll just pass strings)
fn displayln_steel(s: SteelString) {
    println!("{}", s.as_str());
}

// Convert a Post to a SteelVal hash table
// Creates: (hash 'filepath "..." 'title "..." 'date "..." 'content "...")
fn post_to_steel_hash(filename: &str, post: &post::Post) -> SteelVal {
    // Create symbol for keys
    let filepath_key: SteelVal = SteelVal::SymbolV("filepath".into());
    let title_key: SteelVal = SteelVal::SymbolV("title".into());
    let date_key: SteelVal = SteelVal::SymbolV("date".into());
    let content_key: SteelVal = SteelVal::SymbolV("content".into());

    // Create string values
    let filepath_val: SteelVal = filename.to_string().into_steelval().unwrap();
    let title_val: SteelVal = post.title.clone().into_steelval().unwrap();
    let date_val: SteelVal = post.date.clone().into_steelval().unwrap();
    let content_val: SteelVal = post.content_html.clone().into_steelval().unwrap();

    // Create a Rust HashMap and populate it
    let mut map: HashMap<SteelVal, SteelVal> = HashMap::new();
    map.insert(filepath_key, filepath_val);
    map.insert(title_key, title_val);
    map.insert(date_key, date_val);
    map.insert(content_key, content_val);

    // Convert to Steel hash map using Gc and SteelHashMap
    let steel_map = SteelHashMap::from(Gc::new(map));

    // Return as a SteelVal
    SteelVal::HashMapV(steel_map)
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
    let site_scm_path = "site.scm";
    if !Path::new(site_scm_path).exists() {
        eprintln!("Error: site.scm not found at {}", site_scm_path);
        return Ok(());
    }

    let site_scm = fs::read_to_string(site_scm_path)?;
    // Note: Box::leak is used here because Steel's engine requires 'static lifetime.
    // This is only done once at startup, so it doesn't impact performance.
    let site_scm_static: &'static str = Box::leak(site_scm.into_boxed_str());
    engine.run(site_scm_static)?;

    // Read site configuration
    let config_result = engine.run("site")?;
    println!("Site configuration loaded: {:?}\n", config_result);

    // Process posts - collect all post data first
    let posts_dir = "posts";
    let post_files = fs::read_dir(posts_dir)?;

    let mut posts_data = Vec::new();

    for entry in post_files {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() && path.extension().map_or(false, |ext| ext == "md") {
            println!("Processing: {}", path.display());

            let content = fs::read_to_string(&path)?;
            let post = post::parse_post_file(path.to_str().unwrap(), &content)?;

            let filename = path.file_stem().unwrap().to_str().unwrap();
            let post_hash = post_to_steel_hash(filename, &post);

            posts_data.push((filename.to_string(), post_hash));
        }
    }

    // Get site config as a SteelVal
    let site_config = engine.run("site")?.into_iter().next()
        .ok_or("Failed to get site config")?;

    // Batch render all posts with a single Steel call
    if !posts_data.is_empty() {
        let posts_list: SteelVal = posts_data.iter()
            .map(|(_, alist)| alist.clone())
            .collect::<Vec<_>>()
            .into_steelval()
            .unwrap();

        match engine.call_function_by_name_with_args("render-all-posts", vec![site_config.clone(), posts_list]) {
            Ok(result) => {
                // result is a list of (filepath html-sexp) 2-element lists
                if let SteelVal::ListV(items) = &result {
                    for item in items.iter() {
                        // Each item is a list with [filename, html-sexp]
                        if let SteelVal::ListV(pair_list) = item {
                            if pair_list.len() == 2 {
                                if let SteelVal::StringV(fname) = &pair_list[0] {
                                    let html_sexp = &pair_list[1];

                                    let full_html = format!(
                                        "<!DOCTYPE html>\n{}",
                                        sexp_html::sexp_to_html(html_sexp)
                                    );

                                    let output_path = format!("build/{}.html", fname.as_str());
                                    fs::write(&output_path, &full_html)?;
                                    println!("  → Generated: {}", output_path);
                                }
                            }
                        }
                    }
                }
            }
            Err(e) => eprintln!("Error batch rendering posts: {:?}", e),
        }
    }

    // Generate index.html
    println!("\nGenerating index.html...");

    // Build the posts list for the index (only metadata, no content)
    let index_posts_list: SteelVal = posts_data.iter()
        .map(|(_, alist)| alist.clone())
        .collect::<Vec<_>>()
        .into_steelval()
        .unwrap();

    // Call render-full-index with strongly-typed arguments
    match engine.call_function_by_name_with_args("render-full-index", vec![site_config.clone(), index_posts_list]) {
        Ok(index_sexp) => {
            let full_html = format!("<!DOCTYPE html>\n{}", sexp_html::sexp_to_html(&index_sexp));
            fs::write("build/index.html", &full_html)?;
            println!("  → Generated: build/index.html");
        }
        Err(e) => eprintln!("Error generating index: {:?}", e),
    }

    println!("\n✓ Site built successfully!");
    println!("  Output directory: build/");
    println!("  Posts generated: {}", posts_data.len());

    Ok(())
}
