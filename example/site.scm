;; Site configuration for Bower example
(define title "Ben Godfrey")
(define description "Hi, I'm Ben Godfrey. I'm an Engineering Manager at Meta. I like to make things.")
(define site-url "https://example.com")

;; Render a complete HTML page
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

;; Render a blog post. Bower calls this as (post title date content post-metadata).
(define (post post-title post-date post-content post-metadata)
  (page
    `(article
      (h2 ,post-title)
      (time ((datetime ,post-date)) ,post-date)
      (div ((class "content"))
        (raw-html ,post-content)))))

;; Render a single post item for the index
(define (render-post-item post)
  (let ([post-id (hash-ref post 'id)]
        [title (hash-ref post 'title)]
        [date-display (hash-ref post 'date-display)])
    `(li
      (a ((href ,(string-append "posts/" post-id "/"))) ,title)
      " - "
      ,date-display)))

;; Render the index page. Bower calls this as (index year-groups), where
;; year-groups is a list of (year posts) pairs, newest year first.
(define (index year-groups)
  (page
    `(div
      (h1 ,title)
      ,@(map (lambda (year-group)
               (let ([year (car year-group)]
                     [posts (cadr year-group)])
                 `(section
                   (h2 ,year)
                   (ul ,@(map render-post-item posts)))))
             year-groups))))
