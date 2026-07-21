use crate::post::Post;
use std::fs;
use std::path::Path;
use steel::rvals::{IntoSteelVal, SteelVal};
use steel::steel_vm::engine::Engine;

pub type BuildError = Box<dyn std::error::Error>;

pub fn setup_build_environment() -> Result<Engine, BuildError> {
    fs::create_dir_all("build")?;

    let mut engine = Engine::new();

    let site_scm_path = "site.scm";
    if !Path::new(site_scm_path).exists() {
        eprintln!("Error: site.scm not found at {}", site_scm_path);
        return Err("site.scm not found".into());
    }

    let site_scm = fs::read_to_string(site_scm_path)?;

    // Note: Box::leak is used here because Steel's engine requires 'static lifetime.
    // This is only done once at startup, so it doesn't impact performance.
    let site_scm_static: &'static str = Box::leak(site_scm.into_boxed_str());
    engine.run(site_scm_static)?;

    Ok(engine)
}

/// Parses every `.md` file in `posts/`, sorted by date descending (newest first) -
/// matching the ordering convention of the blog this generator was built for.
pub fn parse_all_posts() -> Result<Vec<(String, Post)>, BuildError> {
    let posts_dir = "posts";
    let post_files = fs::read_dir(posts_dir)?;

    let mut posts = Vec::new();

    for entry in post_files {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() && path.extension().map_or(false, |ext| ext == "md") {
            println!("Processing: {}", path.display());

            if let Some(entry) = parse_one_post(&path)? {
                posts.push(entry);
            }
        }
    }

    sort_posts(&mut posts);

    Ok(posts)
}

/// Parses a single post file, returning `(filename, Post)`. Used both by the
/// full build and by the dev server's incremental rebuild path.
pub fn parse_one_post(path: &Path) -> Result<Option<(String, Post)>, BuildError> {
    if !(path.is_file() && path.extension().map_or(false, |ext| ext == "md")) {
        return Ok(None);
    }

    let content = fs::read_to_string(path)?;
    let post = crate::post::parse_post_file(path.to_str().unwrap(), &content)?;

    let filename = path.file_stem().unwrap().to_str().unwrap().to_string();
    Ok(Some((filename, post)))
}

pub fn sort_posts(posts: &mut Vec<(String, Post)>) {
    posts.sort_by(|(_, a), (_, b)| b.date.cmp(&a.date));
}

/// Renders a single post page to `build/posts/{filename}/index.html`.
pub fn render_one_post(
    engine: &mut Engine,
    filename: &str,
    post: &Post,
) -> Result<(), BuildError> {
    let post_hash = crate::post::post_to_steel_hash(filename, post);

    let args = vec![
        post.title.clone().into_steelval()?,
        crate::post::iso8601(&post.date).into_steelval()?,
        post.content_html.clone().into_steelval()?,
        post_hash,
    ];

    match engine.call_function_by_name_with_args("post", args) {
        Ok(html_sexp) => {
            let full_html = format!(
                "<!doctype html>{}",
                crate::sexp_html::sexp_to_html(&html_sexp)
            );

            let post_output_dir = format!("build/posts/{}", filename);
            fs::create_dir_all(&post_output_dir)?;

            let post_filepath = format!("{}/index.html", post_output_dir);
            fs::write(&post_filepath, &full_html)?;
            println!("  -> Generated: {}", post_filepath);
        }
        Err(e) => eprintln!("Error rendering post {}: {:?}", filename, e),
    }

    Ok(())
}

/// Removes the generated output directory for a post that no longer exists.
pub fn remove_post_output(filename: &str) -> Result<(), BuildError> {
    let post_output_dir = format!("build/posts/{}", filename);
    if Path::new(&post_output_dir).exists() {
        fs::remove_dir_all(&post_output_dir)?;
    }
    Ok(())
}

/// Renders all individual post pages using the Steel engine
pub fn render_all_posts(
    engine: &mut Engine,
    posts: &[(String, Post)],
) -> Result<(), BuildError> {
    for (filename, post) in posts {
        render_one_post(engine, filename, post)?;
    }

    Ok(())
}

/// Groups posts by (UTC) year, preserving the incoming order within each group.
/// `posts` is expected to already be sorted newest-first, so groups come out
/// newest-year-first too.
fn group_by_year(posts: &[(String, Post)]) -> Vec<(i32, Vec<&(String, Post)>)> {
    let mut groups: Vec<(i32, Vec<&(String, Post)>)> = Vec::new();
    for entry @ (_, post) in posts {
        let year = post.date.format("%Y").to_string().parse().unwrap();
        match groups.last_mut() {
            Some((last_year, group)) if *last_year == year => group.push(entry),
            _ => groups.push((year, vec![entry])),
        }
    }
    groups
}

pub fn render_index(engine: &mut Engine, posts: &[(String, Post)]) -> Result<(), BuildError> {
    println!("\nGenerating index.html...");

    let year_groups: SteelVal = group_by_year(posts)
        .into_iter()
        .map(|(year, group)| {
            let posts_in_year: SteelVal = group
                .iter()
                .map(|(filename, post)| crate::post::post_to_steel_hash(filename, post))
                .collect::<Vec<_>>()
                .into_steelval()
                .unwrap();
            vec![year.to_string().into_steelval().unwrap(), posts_in_year]
                .into_steelval()
                .unwrap()
        })
        .collect::<Vec<_>>()
        .into_steelval()?;

    match engine.call_function_by_name_with_args("index", vec![year_groups]) {
        Ok(index_sexp) => {
            let full_html = format!(
                "<!doctype html>{}",
                crate::sexp_html::sexp_to_html(&index_sexp)
            );
            fs::write("build/index.html", &full_html)?;
            println!("  -> Generated: build/index.html");
        }
        Err(e) => eprintln!("Error generating index: {:?}", e),
    }

    Ok(())
}

/// Recursively copies everything under `public/` into `build/` (favicon, images, CSS, ...).
pub fn copy_static_assets() -> Result<(), BuildError> {
    let public_dir = Path::new("public");
    if !public_dir.exists() {
        return Ok(());
    }

    fn copy_dir(src: &Path, dst: &Path) -> std::io::Result<()> {
        fs::create_dir_all(dst)?;
        for entry in fs::read_dir(src)? {
            let entry = entry?;
            let src_path = entry.path();
            let dst_path = dst.join(entry.file_name());
            if src_path.is_dir() {
                copy_dir(&src_path, &dst_path)?;
            } else {
                fs::copy(&src_path, &dst_path)?;
            }
        }
        Ok(())
    }

    copy_dir(public_dir, Path::new("build"))?;
    println!("\nCopied static assets from public/ to build/");
    Ok(())
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('\'', "&apos;")
        .replace('"', "&quot;")
}

/// Generates an RSS 2.0 feed at `build/rss.xml`.
pub fn render_rss(engine: &Engine, posts: &[(String, Post)]) -> Result<(), BuildError> {
    let site_title: String = engine.extract("title")?;
    let site_description: String = engine.extract("description")?;
    let site_url: String = engine.extract("site-url")?;

    let mut items = String::new();
    for (filename, post) in posts {
        let link = format!("{}/posts/{}/", site_url.trim_end_matches('/'), filename);
        items.push_str(&format!(
            "<item><title>{}</title><link>{}</link><guid isPermaLink=\"true\">{}</guid></item>",
            xml_escape(&post.title),
            xml_escape(&link),
            xml_escape(&link),
        ));
    }

    let rss = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?><rss version=\"2.0\"><channel><title>{}</title><description>{}</description><link>{}</link>{}</channel></rss>",
        xml_escape(&site_title),
        xml_escape(&site_description),
        xml_escape(&format!("{}/", site_url.trim_end_matches('/'))),
        items,
    );

    fs::write("build/rss.xml", rss)?;
    println!("  -> Generated: build/rss.xml");
    Ok(())
}

/// Generates `build/sitemap-index.xml` and `build/sitemap-0.xml`.
pub fn render_sitemap(engine: &Engine, posts: &[(String, Post)]) -> Result<(), BuildError> {
    let site_url: String = engine.extract("site-url")?;
    let site_url = site_url.trim_end_matches('/');

    let mut urls = format!("<url><loc>{}/</loc></url>", xml_escape(site_url));
    for (filename, _) in posts {
        urls.push_str(&format!(
            "<url><loc>{}/posts/{}/</loc></url>",
            xml_escape(site_url),
            xml_escape(filename)
        ));
    }

    let sitemap = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?><urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\" xmlns:news=\"http://www.google.com/schemas/sitemap-news/0.9\" xmlns:xhtml=\"http://www.w3.org/1999/xhtml\" xmlns:image=\"http://www.google.com/schemas/sitemap-image/1.1\" xmlns:video=\"http://www.google.com/schemas/sitemap-video/1.1\">{}</urlset>",
        urls
    );
    fs::write("build/sitemap-0.xml", sitemap)?;

    let sitemap_index = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?><sitemapindex xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\"><sitemap><loc>{}/sitemap-0.xml</loc></sitemap></sitemapindex>",
        xml_escape(site_url)
    );
    fs::write("build/sitemap-index.xml", sitemap_index)?;

    println!("  -> Generated: build/sitemap-index.xml, build/sitemap-0.xml");
    Ok(())
}

/// Runs a complete build from scratch: loads `site.scm`, parses every post,
/// and generates all output files. Returns the engine and parsed posts so
/// callers (like the dev server) can reuse them for incremental rebuilds.
pub fn full_build() -> Result<(Engine, Vec<(String, Post)>), BuildError> {
    let mut engine = setup_build_environment()?;
    let posts = parse_all_posts()?;
    render_all_posts(&mut engine, &posts)?;
    render_index(&mut engine, &posts)?;
    render_rss(&engine, &posts)?;
    render_sitemap(&engine, &posts)?;
    copy_static_assets()?;

    Ok((engine, posts))
}
