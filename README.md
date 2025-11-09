# Bower

A proof-of-concept static site generator inspired by Jekyll and Astro, implemented using Steel Scheme and Rust.

## Overview

Bower combines the power of Rust for performance-critical operations (like markdown parsing) with the elegance of Scheme for templating and HTML generation using s-expressions.

## Features

- **Steel Scheme Integration**: Uses [Steel](https://github.com/mattwparas/steel), an embeddable Scheme interpreter written in Rust
- **Fast Markdown Parsing**: Uses `pulldown-cmark` for high-performance markdown to HTML conversion
- **S-expression Templates**: HTML templates are written as Scheme s-expressions, similar to Hiccup in Clojure
- **S-expression Front Matter**: Post metadata is defined using s-expressions instead of YAML
- **Customizable Rendering**: Site owners define their own `render-page` and `render-post` functions in Scheme

## Building

```bash
cargo build --release
```

## Usage

1. Create a `site.scm` file with your site configuration and rendering functions:

```scheme
(define site
  '(site
    (title "My Site")
    (description "Welcome to my site")))

;; Helper to get values from association list
(define (alist-get lst key)
  (if (null? lst)
      #f
      (if (equal? (car (car lst)) key)
          (car (cdr (car lst)))
          (alist-get (cdr lst) key))))

;; Render a complete HTML page
(define (render-page config content)
  (let ((title (alist-get (cdr config) 'title)))
    `(html ((lang "en"))
      (head
        (meta ((charset "utf-8")))
        (meta ((name "viewport") (content "width=device-width, initial-scale=1")))
        (title ,title))
      (body
        (header
          (h1 ,title))
        (main
          ,content)))))

;; Render a blog post
(define (render-post config post)
  (let ((post-title (alist-get post 'title))
        (post-date (alist-get post 'date))
        (post-content (alist-get post 'content)))
    `(article
      (h2 ,post-title)
      (time ((datetime ,post-date)) ,post-date)
      (div ((class "content"))
        ,post-content))))
```

2. Create posts in a `posts/` directory with s-expression front matter:

```markdown
(post
 (title "My First Post")
 (date "2025-01-01T12:00:00+00:00")
 (tags ()))
---
# Hello World

This is my first post!
```

3. Run bower:

```bash
cargo run
```

4. Your generated site will be in the `build/` directory.

## Example

An example site is included in the `example/` directory. To build it:

```bash
cargo run
```

This will process the posts in `example/posts/` and generate HTML files in `build/`.

## Architecture

### Rust Components

- **main.rs**: Entry point, orchestrates the build process
- **markdown.rs**: Wraps pulldown-cmark for markdown to HTML conversion
- **post.rs**: Parses post files with s-expression front matter
- **sexp_html.rs**: Converts Steel s-expressions to HTML strings

### Steel Scheme Components

- **site.scm**: Contains site configuration and rendering functions defined by the user
- s-expression templates that define the HTML structure

### Build Process

1. Rust reads and parses the `site.scm` file, loading it into the Steel engine
2. Rust enumerates all `.md` files in the `posts/` directory
3. For each post:
   - Rust parses the s-expression front matter
   - Rust converts the markdown content to HTML
   - Rust calls the Scheme `render-post` function with the post data
   - Rust calls the Scheme `render-page` function to wrap the post
   - Rust converts the resulting s-expression to HTML
   - Rust writes the final HTML to `build/{filename}.html`
4. Rust generates an `index.html` file listing all posts

## Template Syntax

Templates use Scheme's elegant **quasiquote** syntax (backtick `` ` `` and comma `,`):

- `` `(tag-name child1 child2 ...) `` → `<tag-name>child1child2...</tag-name>`
- `` `(tag-name ((attr1 val1) (attr2 val2)) child1 ...) `` → `<tag-name attr1="val1" attr2="val2">child1...</tag-name>`
- `,variable` inside a quasiquote splices in the value of the variable
- Strings and numbers are inserted directly

**Example:**
```scheme
(let ((page-title "Hello World")
      (message "This is a paragraph"))
  `(div ((class "container"))
    (h1 ,page-title)
    (p ,message)))
```

**Becomes:**
```html
<div class="container"><h1>Hello World</h1><p>This is a paragraph</p></div>
```

This syntax is:
- **Clean**: Looks almost like HTML, but with the power of Scheme
- **Composable**: Build complex templates from simple functions
- **Type-safe**: Templates are just data structures, checked at compile time

## Technology Stack

- **Rust**: System programming language for the core
- **Steel Scheme**: Embedded Scheme interpreter for templating
- **pulldown-cmark**: Fast, compliant CommonMark parser
- **steel-core**: Steel VM and compiler

## License

This is a proof-of-concept project for educational purposes.

## Acknowledgments

- [Steel](https://github.com/mattwparas/steel) by Matt Paras
- [pulldown-cmark](https://github.com/raphlinus/pulldown-cmark) by Raph Levien
- Inspired by Jekyll, Astro, and Hiccup
