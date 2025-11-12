mod markdown;
mod post;
mod sexp_html;

use steel::steel_vm::engine::Engine;
use steel::rvals::{SteelVal, IntoSteelVal, SteelHashMap};
use steel::gc::Gc;
use steel::HashMap;
use std::fs;
use std::path::Path;

// Default helper functions injected into site.scm if not already defined
const HELPER_RENDER_FULL_POST: &str = r#"
(define (render-full-post post)
  (render-page (render-post post)))
"#;

const HELPER_RENDER_FULL_INDEX: &str = r#"
(define (render-full-index posts)
  (render-page (render-index posts)))
"#;

const HELPER_RENDER_ALL_POSTS: &str = r#"
(define (render-all-posts posts)
  (map (lambda (post)
         (list (hash-ref post 'filepath)
               (render-full-post post)))
       posts))
"#;

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

/// Sets up the build environment and initializes the Steel engine with site configuration
fn setup_build_environment() -> Result<Engine, Box<dyn std::error::Error>> {
    // Create build directory
    fs::create_dir_all("build")?;

    // Create Steel engine
    let mut engine = Engine::new();

    // Load site.scm
    let site_scm_path = "site.scm";
    if !Path::new(site_scm_path).exists() {
        eprintln!("Error: site.scm not found at {}", site_scm_path);
        return Err("site.scm not found".into());
    }

    let site_scm = fs::read_to_string(site_scm_path)?;

    // Check which helper functions are missing from site.scm and inject them
    let has_render_full_post = site_scm.contains("(define (render-full-post");
    let has_render_full_index = site_scm.contains("(define (render-full-index");
    let has_render_all_posts = site_scm.contains("(define (render-all-posts");

    let mut helpers = String::new();

    if !has_render_full_post {
        helpers.push_str(HELPER_RENDER_FULL_POST);
    }

    if !has_render_full_index {
        helpers.push_str(HELPER_RENDER_FULL_INDEX);
    }

    if !has_render_all_posts {
        helpers.push_str(HELPER_RENDER_ALL_POSTS);
    }

    // Concatenate site.scm with needed Bower helpers
    let combined = format!("{}{}", site_scm, helpers);

    // Note: Box::leak is used here because Steel's engine requires 'static lifetime.
    // This is only done once at startup, so it doesn't impact performance.
    let combined_static: &'static str = Box::leak(combined.into_boxed_str());
    engine.run(combined_static)?;

    Ok(engine)
}

/// Parses all markdown posts from the posts directory
fn parse_all_posts() -> Result<Vec<(String, SteelVal)>, Box<dyn std::error::Error>> {
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

    Ok(posts_data)
}

/// Renders all individual post pages using the Steel engine
fn render_all_posts(engine: &mut Engine, posts_data: &[(String, SteelVal)]) -> Result<(), Box<dyn std::error::Error>> {
    if posts_data.is_empty() {
        return Ok(());
    }

    let posts_list: SteelVal = posts_data.iter()
        .map(|(_, alist)| alist.clone())
        .collect::<Vec<_>>()
        .into_steelval()
        .unwrap();

    match engine.call_function_by_name_with_args("render-all-posts", vec![posts_list]) {
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

    Ok(())
}

/// Renders the index page listing all posts
fn render_index(engine: &mut Engine, posts_data: &[(String, SteelVal)]) -> Result<(), Box<dyn std::error::Error>> {
    println!("\nGenerating index.html...");

    // Build the posts list for the index (only metadata, no content)
    let index_posts_list: SteelVal = posts_data.iter()
        .map(|(_, alist)| alist.clone())
        .collect::<Vec<_>>()
        .into_steelval()
        .unwrap();

    // Call render-full-index with strongly-typed arguments
    match engine.call_function_by_name_with_args("render-full-index", vec![index_posts_list]) {
        Ok(index_sexp) => {
            let full_html = format!("<!DOCTYPE html>\n{}", sexp_html::sexp_to_html(&index_sexp));
            fs::write("build/index.html", &full_html)?;
            println!("  → Generated: build/index.html");
        }
        Err(e) => eprintln!("Error generating index: {:?}", e),
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Bower - A Static Site Generator in Scheme\n");

    let mut engine = setup_build_environment()?;
    let posts_data = parse_all_posts()?;
    render_all_posts(&mut engine, &posts_data)?;
    render_index(&mut engine, &posts_data)?;

    println!("\n✓ Site built successfully!");
    println!("  Output directory: build/");
    println!("  Posts generated: {}", posts_data.len());

    Ok(())
}
