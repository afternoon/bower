mod markdown;
mod post;
mod sexp_html;

use steel::steel_vm::engine::Engine;
use steel::rvals::{SteelString, SteelVal, IntoSteelVal};
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

// Symbol to string (in Steel, we'll just pass strings)
fn displayln_steel(s: SteelString) {
    println!("{}", s.as_str());
}

// Convert a Post to a SteelVal association list
// Creates: '((filepath "...") (title "...") (date "..."))
fn post_to_steel_alist(filename: &str, post: &post::Post) -> SteelVal {
    // Create symbol for keys
    let filepath_key: SteelVal = SteelVal::SymbolV("filepath".into());
    let title_key: SteelVal = SteelVal::SymbolV("title".into());
    let date_key: SteelVal = SteelVal::SymbolV("date".into());

    // Create string values
    let filepath_val: SteelVal = filename.to_string().into_steelval().unwrap();
    let title_val: SteelVal = post.title.clone().into_steelval().unwrap();
    let date_val: SteelVal = post.date.clone().into_steelval().unwrap();

    // Create pairs (key value)
    let filepath_pair = vec![filepath_key, filepath_val].into_steelval().unwrap();
    let title_pair = vec![title_key, title_val].into_steelval().unwrap();
    let date_pair = vec![date_key, date_val].into_steelval().unwrap();

    // Create the association list
    vec![filepath_pair, title_pair, date_pair].into_steelval().unwrap()
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
    let site_scm_static: &'static str = Box::leak(site_scm.into_boxed_str());
    engine.run(site_scm_static)?;

    // Read site configuration
    let config_result = engine.run("site")?;
    println!("Site configuration loaded: {:?}\n", config_result);

    // Process posts
    let posts_dir = "posts";
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

    // Build a strongly-typed list of post data using SteelVal
    let posts_data: Vec<SteelVal> = posts
        .iter()
        .map(|p| {
            let filename = Path::new(&p.file_path)
                .file_stem()
                .unwrap()
                .to_str()
                .unwrap();
            post_to_steel_alist(filename, p)
        })
        .collect();

    // Convert Vec<SteelVal> to a SteelVal list
    let posts_list: SteelVal = posts_data.into_steelval().unwrap();

    // Get site config as a SteelVal
    let site_config = engine.run("site")?.into_iter().next()
        .ok_or("Failed to get site config")?;

    // Call render-index with strongly-typed arguments
    match engine.call_function_by_name_with_args("render-index", vec![site_config.clone(), posts_list]) {
        Ok(index_sexp) => {
            // Wrap the index content in render-page
            let index_html = sexp_html::sexp_to_html(&index_sexp);
            let page_content = index_html.replace("\"", "\\\"").replace("\n", " ");

            // Call render-page with the index content
            let page_call = format!("(render-page site \"{}\")", page_content);
            let page_call_static: &'static str = Box::leak(page_call.into_boxed_str());

            match engine.run(page_call_static) {
                Ok(page_result) => {
                    if let Some(page_sexp) = page_result.first() {
                        let full_html = format!("<!DOCTYPE html>\n{}", sexp_html::sexp_to_html(page_sexp));
                        fs::write("build/index.html", &full_html)?;
                        println!("  → Generated: build/index.html");
                    }
                }
                Err(e) => eprintln!("Error rendering index page: {:?}", e),
            }
        }
        Err(e) => eprintln!("Error generating index: {:?}", e),
    }

    println!("\n✓ Site built successfully!");
    println!("  Output directory: build/");
    println!("  Posts generated: {}", posts.len());

    Ok(())
}
