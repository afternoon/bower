;; Site configuration for Bower example
(define title "Ben Godfrey")
(define description "Hi, I'm Ben Godfrey. I'm an Engineering Manager at Meta. I like to make things.")

;; Render a complete HTML page
(define (render-page content)
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

;; Render a blog post
(define (render-post post)
  (let ((post-title (hash-ref post 'title))
        (post-date (hash-ref post 'date))
        (post-content (hash-ref post 'content)))
    `(article
      (h2 ,post-title)
      (time ((datetime ,post-date)) ,post-date)
      (div ((class "content"))
        ,post-content))))

;; Render a single post item for the index
(define (render-post-item post)
  (let ((filepath (hash-ref post 'filepath))
        (title (hash-ref post 'title))
        (date (hash-ref post 'date)))
    `(li
      (a ((href ,(string-append filepath ".html"))) ,title)
      " - "
      ,date)))

;; Render the index page with a list of posts
(define (render-index posts)
  `(div
    (h2 "All Posts")
    (ul ,@(map render-post-item posts))))
