# Bower Template Guide

## Why S-Expressions?

Bower uses Scheme s-expressions for templates because they offer:

- **Elegance**: Templates look almost like HTML, but with the full power of a programming language
- **Composability**: Build complex layouts from simple, reusable functions
- **Type Safety**: Templates are data structures, not strings
- **No String Concatenation**: No risk of XSS or malformed HTML

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
  (let ((title (alist-get post 'title))
        (date (alist-get post 'date))
        (author (alist-get post 'author)))
    `(article
      (header
        (h1 ,title)
        (div ((class "meta"))
          (time ,date)
          (span ,author)))
      (div ((class "content"))
        ,(alist-get post 'content)))))
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

## Comparison to Other Template Languages

### JSX (React)
```jsx
<div className="card">
  <h1>{title}</h1>
  <p>{content}</p>
</div>
```

### Bower (Scheme)
```scheme
`(div ((class "card"))
  (h1 ,title)
  (p ,content))
```

### Hiccup (Clojure)
```clojure
[:div {:class "card"}
  [:h1 title]
  [:p content]]
```

### Bower (Scheme)
```scheme
`(div ((class "card"))
  (h1 ,title)
  (p ,content))
```

## Next Steps

- Check out `example/site.scm` for a working example
- See `example/advanced-template.scm` for component patterns
- Read the main README for full documentation
