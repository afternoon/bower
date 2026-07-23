mod build;
mod dev;
mod markdown;
mod post;
mod sexp_html;

use std::env;

fn print_usage() {
    eprintln!(
        "Usage:\n  bower           Build the site once\n  bower build     Build the site once\n  bower dev       Start the dev server (http://localhost:1159) with hot reload"
    );
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let command = env::args().nth(1);

    match command.as_deref() {
        None | Some("build") => {
            println!("Bower - A Static Site Generator in Scheme\n");

            let (_, posts) = build::full_build()?;

            println!("\n✓ Site built successfully!");
            println!("  Output directory: build/");
            println!("  Posts generated: {}", posts.len());

            Ok(())
        }
        Some("dev") => dev::run(),
        Some("-h") | Some("--help") => {
            print_usage();
            Ok(())
        }
        Some(other) => {
            eprintln!("Unknown command: {}\n", other);
            print_usage();
            std::process::exit(1);
        }
    }
}
