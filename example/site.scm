;; Site configuration for Bower example
(define title "Ben Godfrey")
(define description "Hi, I'm Ben Godfrey. I'm an Engineering Manager at Meta. I like to make things.")

;; Render a complete HTML page
(define (render-page config content)
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
(define (render-post config post)
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
(define (render-index config posts)
  `(div
    (h2 "All Posts")
    (ul ,@(map render-post-item posts))))

;; Render a complete post (post wrapped in page template)
(define (render-full-post config post)
  (render-page config (render-post config post)))

;; Render a complete index page (index wrapped in page template)
(define (render-full-index config posts)
  (render-page config (render-index config posts)))

;; Batch render all posts - returns a list of (filepath html-sexp) pairs
(define (render-all-posts config posts)
  (map (lambda (post)
         (let ((filepath (hash-ref post 'filepath)))
           (list filepath (render-full-post config post))))
       posts))
