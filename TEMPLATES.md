# Bower Template Guide

## S-Expression Templates

Bower uses Scheme s-expressions for HTML templates. Templates are data structures that compose into complex layouts without string concatenation.

## Quasiquote Syntax

Bower templates use Scheme's **quasiquote** syntax:

```scheme
`(element content)     ; Backtick for template
,variable              ; Comma to insert variable value
```

### Basic Example

```scheme
(let ((name "World"))
  `(h1 "Hello, " ,name "!"))
```

Produces: `<h1>Hello, World!</h1>`

## HTML Structure

### Simple Elements

```scheme
`(p "This is a paragraph")
```
→ `<p>This is a paragraph</p>`

### Elements with Attributes

Attributes are a list of `(key value)` pairs as the second element:

```scheme
`(a ((href "/about") (class "link")) "About")
```
→ `<a href="/about" class="link">About</a>`

### Nested Elements

```scheme
`(div
  (h1 "Title")
  (p "Paragraph 1")
  (p "Paragraph 2"))
```
→ `<div><h1>Title</h1><p>Paragraph 1</p><p>Paragraph 2</p></div>`

### Attributes + Children

```scheme
`(div ((class "container"))
  (h1 "Hello")
  (p "Content"))
```
→ `<div class="container"><h1>Hello</h1><p>Content</p></div>`

## Variable Interpolation

Use comma `,` to insert variable values:

```scheme
(define title "My Page")
(define content "Welcome!")

`(div
  (h1 ,title)
  (p ,content))
```

## Component Functions

Build reusable components as functions:

```scheme
;; Define a card component
(define (card title body)
  `(div ((class "card"))
    (div ((class "card-header"))
      (h3 ,title))
    (div ((class "card-body"))
      ,body)))

;; Use it
`(div
  ,(card "First Card" `(p "Content 1"))
  ,(card "Second Card" `(p "Content 2")))
```

## Complete Page Example

```scheme
(define (page-layout title content)
  `(html ((lang "en"))
    (head
      (meta ((charset "utf-8")))
      (meta ((name "viewport")
             (content "width=device-width, initial-scale=1")))
      (title ,title)
      (link ((rel "stylesheet") (href "/style.css"))))
    (body
      (header
        (h1 ,title)
        (nav
          (a ((href "/")) "Home")
          (a ((href "/about")) "About")))
      (main
        ,content)
      (footer
        (p "© 2025")))))
```

## Tips & Best Practices

### 1. Extract Reusable Components

**Bad:**
```scheme
(define (page)
  `(div
    (a ((href "/") (class "nav-link")) "Home")
    (a ((href "/about") (class "nav-link")) "About")
    ;; ... repeated nav links everywhere
    ))
```

**Good:**
```scheme
(define (nav-link href text)
  `(a ((href ,href) (class "nav-link")) ,text))

(define (page)
  `(div
    ,(nav-link "/" "Home")
    ,(nav-link "/about" "About")))
```

### 2. Use Let for Complex Data

```scheme
(define (render-post post)
  (let ((title (hash-ref post 'title))
        (date (hash-ref post 'date))
        (content (hash-ref post 'content)))
    `(article
      (header
        (h1 ,title)
        (div ((class "meta"))
          (time ,date)))
      (div ((class "content"))
        ,content))))
```

### 3. Conditional Rendering

```scheme
(define (sidebar show-ads?)
  `(aside
    (h2 "Categories")
    (ul (li "Tech") (li "Life"))
    ;; Only show ads if flag is true
    ,(if show-ads?
         `(div ((class "ad")) "Advertisement")
         "")))
```

### 4. Lists and Mapping

```scheme
(define (tag-list tags)
  `(ul ((class "tags"))
    ,@(map (lambda (tag)
             `(li ((class "tag")) ,tag))
           tags)))

;; Usage
(tag-list '("scheme" "rust" "web"))
```

## Common Patterns

### Navigation Menu

```scheme
(define (nav items)
  `(nav
    ,@(map (lambda (item)
             `(a ((href ,(car item))) ,(cadr item)))
           items)))

;; Usage
(nav '(("/" "Home") ("/about" "About") ("/blog" "Blog")))
```

### Card Grid

```scheme
(define (card-grid cards)
  `(div ((class "grid"))
    ,@(map (lambda (c)
             (card (car c) (cadr c)))
           cards)))
```

### Post List

```scheme
(define (post-list posts)
  `(ul ((class "posts"))
    ,@(map (lambda (post)
             `(li
               (a ((href ,(string-append "/" (post-slug post) ".html")))
                 ,(post-title post))
               (time ,(post-date post))))
           posts)))
```

## Accessing Post Data

Posts are passed as hash tables. Access fields with `hash-ref`:

```scheme
(define (render-post config post)
  (let ((title (hash-ref post 'title))
        (date (hash-ref post 'date))
        (filepath (hash-ref post 'filepath))
        (content (hash-ref post 'content)))
    `(article
      (h1 ,title)
      (time ,date)
      (div ((class "content")) ,content))))
```

Available post fields:
- `'title` - Post title from frontmatter
- `'date` - ISO 8601 date string
- `'filepath` - Filename without extension
- `'content` - Rendered HTML content

## Comparison to Other Languages

JSX (React):
```jsx
<div className="card">
  <h1>{title}</h1>
  <p>{content}</p>
</div>
```

Bower (Scheme):
```scheme
`(div ((class "card"))
  (h1 ,title)
  (p ,content))
```

## Examples

See `example/site.scm` for a complete working example.
