;; Example showing the elegance of Scheme templates in Bower
;; This demonstrates how clean and readable templates can be

;; A simple navigation component
(define (nav-link href text)
  `(a ((href ,href) (class "nav-link")) ,text))

;; A reusable card component
(define (card title content)
  `(div ((class "card"))
    (div ((class "card-header"))
      (h3 ,title))
    (div ((class "card-body"))
      ,content)))

;; Build a sidebar with multiple cards
(define (sidebar)
  `(aside ((class "sidebar"))
    ,(card "Recent Posts"
           `(ul
             (li ,(nav-link "/post-1.html" "My First Post"))
             (li ,(nav-link "/post-2.html" "Another Post"))))
    ,(card "About"
           `(p "This is a static site built with Bower!"))))

;; Main layout composition
(define (page-layout title content)
  `(html ((lang "en"))
    (head
      (meta ((charset "utf-8")))
      (title ,title)
      (link ((rel "stylesheet") (href "/style.css"))))
    (body
      (div ((class "container"))
        (header
          (h1 ,title)
          (nav
            ,(nav-link "/" "Home")
            ,(nav-link "/about.html" "About")
            ,(nav-link "/blog.html" "Blog")))
        (div ((class "layout"))
          (main
            ,content)
          ,(sidebar))))))

;; Example usage:
;; (page-layout "My Blog"
;;   `(article
;;     (h2 "Welcome!")
;;     (p "This shows how elegant Scheme templates can be.")))
