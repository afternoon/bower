;; Site configuration for Bower example
(define site
  '(site
    (title "Ben Godfrey")
    (description "Hi, I'm Ben Godfrey. I'm an Engineering Manager at Meta. I like to make things.")))

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

;; Render a single post item for the index
(define (render-post-item post)
  (let ((filepath (alist-get post 'filepath))
        (title (alist-get post 'title))
        (date (alist-get post 'date)))
    `(li
      (a ((href ,(string-append filepath ".html"))) ,title)
      " - "
      ,date)))

;; Render the index page with a list of posts
(define (render-index config posts)
  `(div
    (h2 "All Posts")
    (ul ,@(map render-post-item posts))))
