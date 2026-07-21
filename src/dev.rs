use crate::build::{self, BuildError};
use crate::post::Post;
use notify::{RecursiveMode, Watcher};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Read;
use std::path::{Component, Path, PathBuf};
use std::sync::mpsc;
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::time::Duration;
use steel::steel_vm::engine::Engine;
use tiny_http::{Header, Request, Response, ResponseBox, Server, StatusCode};

const HOST: &str = "127.0.0.1:1159";
const LIVE_RELOAD_PATH: &str = "/__bower_live_reload";
const RELOAD_SCRIPT: &str =
    "<script>new EventSource(\"/__bower_live_reload\").onmessage=()=>location.reload();</script>";

/// Tracks the current build "version": bumped after every rebuild so that
/// long-polling SSE connections know when to tell the browser to reload.
struct SharedVersion {
    version: Mutex<u64>,
    cvar: Condvar,
}

/// A `Read` impl that blocks until the build version changes, then emits a
/// single SSE `reload` event and closes. The browser's `EventSource` will
/// automatically reconnect (we set a short `retry` hint), so this is called
/// again on the next request.
struct SseReader {
    shared: Arc<SharedVersion>,
    seen_version: u64,
    sent: bool,
}

impl Read for SseReader {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if self.sent {
            return Ok(0);
        }

        let mut version = self.shared.version.lock().unwrap();
        while *version == self.seen_version {
            version = self.shared.cvar.wait(version).unwrap();
        }
        drop(version);

        let payload = b"retry: 200\ndata: reload\n\n";
        let n = payload.len().min(buf.len());
        buf[..n].copy_from_slice(&payload[..n]);
        self.sent = true;
        Ok(n)
    }
}

/// Starts the dev server: an initial full build, then a filesystem watcher
/// that triggers incremental rebuilds, and an HTTP server on `localhost:1159`
/// that serves `build/` and hot-reloads connected browsers after each rebuild.
pub fn run() -> Result<(), BuildError> {
    println!("Bower - A Static Site Generator in Scheme\n");
    println!("Running initial build...");

    let (mut engine, mut posts) = build::full_build()?;
    println!("\n✓ Initial build complete. {} posts.", posts.len());

    let shared = Arc::new(SharedVersion {
        version: Mutex::new(0),
        cvar: Condvar::new(),
    });

    {
        let shared = shared.clone();
        thread::spawn(move || {
            if let Err(e) = run_http_server(shared) {
                eprintln!("Dev server error: {}", e);
                std::process::exit(1);
            }
        });
    }

    println!("\n  Dev server running at http://localhost:1159");
    println!("  Watching for changes... (Ctrl+C to stop)\n");

    let cwd = std::env::current_dir()?;
    let (tx, rx) = mpsc::channel();
    let mut watcher = notify::recommended_watcher(move |res| {
        let _ = tx.send(res);
    })?;
    watcher.watch(Path::new("."), RecursiveMode::Recursive)?;

    loop {
        let first_event = match rx.recv() {
            Ok(Ok(event)) => event,
            Ok(Err(e)) => {
                eprintln!("watch error: {:?}", e);
                continue;
            }
            Err(_) => break,
        };

        // Debounce: a single save can produce several filesystem events
        // (write + rename + metadata change), so batch everything that
        // arrives within a short window into one rebuild.
        let mut changed_paths = first_event.paths;
        loop {
            match rx.recv_timeout(Duration::from_millis(150)) {
                Ok(Ok(event)) => changed_paths.extend(event.paths),
                Ok(Err(e)) => eprintln!("watch error: {:?}", e),
                Err(_) => break,
            }
        }

        match handle_changes(&cwd, &mut engine, &mut posts, changed_paths) {
            Ok(true) => {
                let mut version = shared.version.lock().unwrap();
                *version += 1;
                shared.cvar.notify_all();
                println!("Rebuilt.\n");
            }
            Ok(false) => {}
            Err(e) => eprintln!("Rebuild error: {}", e),
        }
    }

    Ok(())
}

enum Change {
    Site,
    Public,
    Post(String),
    Ignore,
}

/// Classifies a changed path so `handle_changes` can decide the cheapest
/// correct way to bring `build/` back up to date.
fn classify(path: &Path, cwd: &Path) -> Change {
    let rel = path.strip_prefix(cwd).unwrap_or(path);
    let mut comps = rel.components();
    let first = match comps.next() {
        Some(Component::Normal(s)) => s.to_string_lossy().into_owned(),
        _ => return Change::Ignore,
    };

    match first.as_str() {
        "build" | "target" | ".git" => Change::Ignore,
        "posts" => {
            if rel.extension().map_or(false, |e| e == "md") {
                if let Some(stem) = rel.file_stem().and_then(|s| s.to_str()) {
                    return Change::Post(stem.to_string());
                }
            }
            Change::Ignore
        }
        "public" => Change::Public,
        _ => {
            // Only top-level *.scm files (site.scm and any siblings it
            // requires) count - not nested/unrelated files.
            if comps.next().is_none() && rel.extension().map_or(false, |e| e == "scm") {
                Change::Site
            } else {
                Change::Ignore
            }
        }
    }
}

/// Applies a batch of changed paths with the minimal amount of rework:
/// - `site.scm` (or a sibling `.scm`) changing reloads the Steel engine and
///   re-renders every page (the templates may have changed).
/// - A single post's markdown changing only reparses and re-renders that one
///   post, then regenerates the index/RSS/sitemap (which list all posts).
/// - `public/` changing just re-copies static assets.
///
/// Returns whether anything was actually rebuilt.
fn handle_changes(
    cwd: &Path,
    engine: &mut Engine,
    posts: &mut Vec<(String, Post)>,
    changed_paths: Vec<PathBuf>,
) -> Result<bool, BuildError> {
    let mut site_changed = false;
    let mut public_changed = false;
    let mut post_changes: HashMap<String, PathBuf> = HashMap::new();

    for path in changed_paths {
        match classify(&path, cwd) {
            Change::Site => site_changed = true,
            Change::Public => public_changed = true,
            Change::Post(stem) => {
                post_changes.insert(stem, path);
            }
            Change::Ignore => {}
        }
    }

    if !site_changed && !public_changed && post_changes.is_empty() {
        return Ok(false);
    }

    if site_changed {
        println!("\nsite.scm changed, reloading engine...");
        *engine = build::setup_build_environment()?;
    }

    let mut touched_filenames: Vec<String> = Vec::new();

    for (stem, path) in &post_changes {
        if path.exists() {
            if let Some((filename, post)) = build::parse_one_post(path)? {
                println!("Post changed: {}", filename);
                if let Some(existing) = posts.iter_mut().find(|(f, _)| f == &filename) {
                    existing.1 = post;
                } else {
                    posts.push((filename.clone(), post));
                }
                touched_filenames.push(filename);
            }
        } else {
            println!("Post removed: {}", stem);
            posts.retain(|(f, _)| f != stem);
            build::remove_post_output(stem)?;
        }
    }

    if site_changed || !touched_filenames.is_empty() {
        build::sort_posts(posts);
    }

    if public_changed {
        println!("public/ changed, re-copying static assets...");
        build::copy_static_assets()?;
    }

    if site_changed {
        build::render_all_posts(engine, posts)?;
    } else {
        for filename in &touched_filenames {
            if let Some((f, post)) = posts.iter().find(|(f, _)| f == filename) {
                build::render_one_post(engine, f, post)?;
            }
        }
    }

    if site_changed || !post_changes.is_empty() {
        build::render_index(engine, posts)?;
        build::render_rss(engine, posts)?;
        build::render_sitemap(engine, posts)?;
    }

    Ok(true)
}

fn run_http_server(shared: Arc<SharedVersion>) -> Result<(), BuildError> {
    let server = Server::http(HOST).map_err(|e| -> BuildError {
        format!("failed to bind {}: {}", HOST, e).into()
    })?;

    for request in server.incoming_requests() {
        let shared = shared.clone();
        thread::spawn(move || handle_request(request, shared));
    }

    Ok(())
}

fn handle_request(request: Request, shared: Arc<SharedVersion>) {
    let path_only = request.url().split('?').next().unwrap_or("/").to_string();

    let response = if path_only == LIVE_RELOAD_PATH {
        live_reload_response(&shared)
    } else {
        serve_static(&path_only)
    };

    let _ = request.respond(response);
}

fn live_reload_response(shared: &Arc<SharedVersion>) -> ResponseBox {
    let seen_version = *shared.version.lock().unwrap();
    let stream = SseReader {
        shared: shared.clone(),
        seen_version,
        sent: false,
    };

    Response::new(
        StatusCode(200),
        vec![
            Header::from_bytes(&b"Content-Type"[..], &b"text/event-stream"[..]).unwrap(),
            Header::from_bytes(&b"Cache-Control"[..], &b"no-cache"[..]).unwrap(),
        ],
        stream,
        None,
        None,
    )
    .boxed()
}

fn serve_static(url_path: &str) -> ResponseBox {
    match resolve_file(url_path) {
        Some(path) => {
            let content_type = mime_type(&path);
            if content_type.starts_with("text/html") {
                match fs::read_to_string(&path) {
                    Ok(html) => Response::from_string(inject_reload_script(&html))
                        .with_header(content_type_header(content_type))
                        .boxed(),
                    Err(_) => not_found(),
                }
            } else {
                match File::open(&path) {
                    Ok(file) => Response::from_file(file)
                        .with_header(content_type_header(content_type))
                        .boxed(),
                    Err(_) => not_found(),
                }
            }
        }
        None => not_found(),
    }
}

fn content_type_header(content_type: &str) -> Header {
    Header::from_bytes(&b"Content-Type"[..], content_type.as_bytes()).unwrap()
}

fn not_found() -> ResponseBox {
    Response::from_string("404 Not Found")
        .with_status_code(StatusCode(404))
        .boxed()
}

/// Maps a request path to a file under `build/`, resolving directory
/// requests to their `index.html` and rejecting any attempt to escape the
/// build directory via `..`.
fn resolve_file(url_path: &str) -> Option<PathBuf> {
    let decoded = percent_decode(url_path);
    let trimmed = decoded.trim_start_matches('/');
    let rel = Path::new(trimmed);

    if rel.components().any(|c| matches!(c, Component::ParentDir)) {
        return None;
    }

    let build_dir = Path::new("build");
    let candidate = if trimmed.is_empty() {
        build_dir.join("index.html")
    } else {
        build_dir.join(rel)
    };

    if candidate.is_dir() {
        let index = candidate.join("index.html");
        return index.is_file().then_some(index);
    }

    candidate.is_file().then_some(candidate)
}

fn percent_decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let Ok(byte) = u8::from_str_radix(&s[i + 1..i + 3], 16) {
                out.push(byte);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn mime_type(path: &Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()).unwrap_or("") {
        "html" => "text/html; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "js" => "text/javascript; charset=utf-8",
        "json" => "application/json; charset=utf-8",
        "xml" => "application/xml; charset=utf-8",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "ico" => "image/x-icon",
        "webp" => "image/webp",
        "txt" => "text/plain; charset=utf-8",
        "woff" => "font/woff",
        "woff2" => "font/woff2",
        "pdf" => "application/pdf",
        _ => "application/octet-stream",
    }
}

/// Injects the live-reload `<script>` before `</body>`, or appends it if the
/// page has no `</body>` tag.
fn inject_reload_script(html: &str) -> String {
    match html.to_ascii_lowercase().rfind("</body>") {
        Some(pos) => {
            let mut out = String::with_capacity(html.len() + RELOAD_SCRIPT.len());
            out.push_str(&html[..pos]);
            out.push_str(RELOAD_SCRIPT);
            out.push_str(&html[pos..]);
            out
        }
        None => format!("{}{}", html, RELOAD_SCRIPT),
    }
}
