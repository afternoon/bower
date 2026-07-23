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

1. Create a `site.scm` file with your site configuration and rendering functions. Bower
   calls two functions directly: `post` to render each post page, and `index` to render
   the home page.

```scheme
; Site metadata as simple variables. `site-url` is required if you want RSS/sitemap output.
(define title "My Site")
(define description "Welcome to my site")
(define site-url "https://example.com")

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

; Render a blog post. Bower calls this as (post title date content post-metadata).
; `post-metadata` is a hash table - see "Data Structures" below for its keys.
; `post-content` is pre-rendered HTML from the post's markdown body; wrap it in
; `raw-html` so it's emitted unescaped rather than as literal text.
(define (post post-title post-date post-content post-metadata)
  (page
    `(article
      (h2 ,post-title)
      (time ((datetime ,post-date)) ,post-date)
      (div ((class "content"))
        (raw-html ,post-content)))))

; Render the index page. Bower calls this as (index year-groups), where
; year-groups is a list of (year posts) pairs, newest year first, and each
; `posts` is a list of post-metadata hash tables for that year.
(define (index year-groups)
  (page
    `(div
      (h1 ,title)
      ,@(map (lambda (year-group)
               (let ([year (car year-group)]
                     [posts (cadr year-group)])
                 `(section
                   (h2 ,year)
                   (ul ,@(map (lambda (post)
                                (let ([post-title (hash-ref post 'title)]
                                      [post-id (hash-ref post 'id)])
                                  `(li ((class "mb-2"))
                                    (a ((href ,(string-append "posts/" post-id "/")))
                                      ,post-title))))
                              posts)))))
             year-groups))))
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

3. Optionally, put static assets (CSS, images, favicon, ...) in a `public/` directory.
   Everything under `public/` is copied as-is into the build output.

4. Run bower:

```bash
cargo run
```

5. Your generated site will be in the `build/` directory, including `index.html`,
   `posts/<id>/index.html` for each post, `rss.xml`, and a sitemap.

## Dev Server

`bower dev` builds the site once, then starts a local server with hot reload:

```bash
cargo run -- dev
```

This serves the `build/` directory at [http://localhost:1159](http://localhost:1159) and
watches `site.scm`, `posts/`, and `public/` for changes. On a change it rebuilds - reusing
the already-parsed posts and only reparsing/re-rendering what actually changed, where
possible - and tells any open browser tabs to reload:

- A post's markdown file changing only reparses and re-renders that post, then regenerates
  `index.html`, `rss.xml`, and the sitemap (since they list all posts).
- `site.scm` changing reloads the Steel engine and re-renders every page, since templates
  may have changed.
- `public/` changing re-copies static assets.

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
2. Parse all `.md` files in `posts/` directory, sorted by date descending
3. For each post:
   - Parse YAML frontmatter (`title`, optional `description`, `date`)
   - Convert markdown to HTML, syntax-highlighting fenced code blocks
   - Call `(post title date content post-metadata)`
   - Convert the returned s-expression to HTML
   - Write to `build/posts/{filename}/index.html`
4. Group posts by year and call `(index year-groups)` to generate `build/index.html`
5. Generate `build/rss.xml` and `build/sitemap-index.xml`/`build/sitemap-0.xml`
6. Copy everything under `public/` into `build/`

## Template Syntax

Templates use Scheme's quasiquote syntax (backtick `` ` `` and comma `,`):

- `` `(tag-name child1 child2) `` → `<tag-name>child1child2</tag-name>`
- `` `(tag-name ((attr1 val1)) child) `` → `<tag-name attr1="val1">child</tag-name>`
- `,variable` splices in the value
- Text content and attribute values are HTML-escaped automatically
- `` `(raw-html "<b>...</b>") `` emits its string argument unescaped - use this for
  pre-rendered HTML, such as a post's markdown-rendered `content`
- HTML5 void elements (`img`, `br`, `link`, `meta`, ...) are rendered without a closing tag

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

Post metadata is passed as a Steel hash table with the following keys:
- `'id` - filename without extension, used for the post's URL (`/posts/{id}/`)
- `'title` - post title from frontmatter
- `'description` - post description from frontmatter, or `""` if absent
- `'date` - ISO 8601 date string in UTC, e.g. `2004-01-15T05:23:14.000Z`
- `'date-year` - the post's year as a string, e.g. `"2004"`
- `'date-display` - the date formatted for display, e.g. `"Jan 15, 2004"`
- `'content` - rendered HTML content (pass to `raw-html` before splicing into a template)

Access values with `hash-ref`:
```scheme
(hash-ref post 'title)
(hash-ref post 'date-display)
```

## License

Educational project.

## Acknowledgments

- [Steel](https://github.com/mattwparas/steel) - Matt Paras
- [pulldown-cmark](https://github.com/raphlinus/pulldown-cmark) - Raph Levien
