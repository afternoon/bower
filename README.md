# Bower

A static site generator using Steel Scheme and Rust. Markdown parsing in Rust, templating with Scheme s-expressions.

## Features

- S-expression HTML templates using Steel Scheme
- Markdown parsing with `pulldown-cmark`
- YAML frontmatter for post metadata
- Customizable rendering functions in Scheme

## Building

```bash
cargo build
```

## Usage

1. Create a `site.scm` file with your site configuration and rendering functions:

```scheme
; Site metadata as simple variables
(define title "My Site")
(define description "Welcome to my site")

; Render a complete HTML page
(define (page content)
  `(html ((lang "en"))
    (head
      (meta ((charset "utf-8")))
      (meta ((name "viewport") (content "width=device-width, initial-scale=1")))
      (title ,title))
    (body
      (header
        (h1 ,title))
      (main
        ,content))))

; Render a blog post
; `post-metadata` is a hash table with keys: 'title, 'date, 'content, 'id
(define (post post-title post-date post-content post-metadata)
  (page
    `(article
      (h2 ,post-title)
      (time ((datetime ,post-date)) ,post-date)
      (div ((class "content"))
        ,post-content))))

; Render an index page
(define (index posts)
  (page
    `(div
      (h1 ,title)
      (section
        ,@(map (lambda (post)
                 (let ([post-title (hash-ref post 'title)]
                       [post-id (hash-ref post 'id)])
                   `(li ((class "mb-2"))
                     (a ((href ,(string-append "posts/" post-id "/")))
                       ,post-title))))
               posts)))))
```

2. Create posts in a `posts/` directory with YAML frontmatter:

```markdown
---
title: My First Post
date: 2025-01-01T12:00:00+00:00
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

1. Load `site.scm` into the Steel engine
2. Parse all `.md` files in `posts/` directory
3. For each post:
   - Parse YAML frontmatter
   - Convert markdown to HTML
   - Create hash table with post metadata (`title`, `date`, `content`, `id`, ...)
   - Call `post` and `page` functions
   - Convert s-expression result to HTML
   - Write to `build/posts/{filename}/index.html`
4. Generate `index.html` listing all posts

## Template Syntax

Templates use Scheme's quasiquote syntax (backtick `` ` `` and comma `,`):

- `` `(tag-name child1 child2) `` → `<tag-name>child1child2</tag-name>`
- `` `(tag-name ((attr1 val1)) child) `` → `<tag-name attr1="val1">child</tag-name>`
- `,variable` splices in the value

Example:
```scheme
(let ((page-title "Hello World")
      (message "This is a paragraph"))
  `(div ((class "container"))
    (h1 ,page-title)
    (p ,message)))
```

Produces:
```html
<div class="container"><h1>Hello World</h1><p>This is a paragraph</p></div>
```

## Data Structures

Post data is passed as Steel hash tables with the following keys:
- `'filepath` - filename without extension
- `'title` - post title from frontmatter
- `'date` - ISO 8601 date string
- `'content` - rendered HTML content

Access values with `hash-ref`:
```scheme
(hash-ref post 'title)
(hash-ref post 'date)
```

## License

Educational project.

## Acknowledgments

- [Steel](https://github.com/mattwparas/steel) - Matt Paras
- [pulldown-cmark](https://github.com/raphlinus/pulldown-cmark) - Raph Levien
