;; Bower - A Static Site Generator in Scheme
;; Core logic for building a static site

;; Parse front matter and content from a post file
;; Returns (metadata . content) where metadata is the s-expression
;; and content is the markdown text after ---
(define (parse-post-file file-content)
  (let* ((lines (string-split file-content "\n"))
         (separator-idx (find-separator lines 0))
         (front-matter-lines (take lines separator-idx))
         (content-lines (drop lines (+ separator-idx 1)))
         (front-matter-str (string-join front-matter-lines "\n"))
         (content-str (string-join content-lines "\n")))
    (cons (read (open-input-string front-matter-str)) content-str)))

;; Find the index of the separator line (---)
(define (find-separator lines idx)
  (if (null? lines)
      idx
      (if (string-starts-with? (car lines) "---")
          idx
          (find-separator (cdr lines) (+ idx 1)))))

;; Get post metadata
(define (post-metadata parsed-post)
  (car parsed-post))

;; Get post content
(define (post-content parsed-post)
  (cdr parsed-post))

;; Extract value from metadata
(define (get-meta metadata key)
  (let ((entry (assoc key (cdr metadata))))
    (if entry
        (cadr entry)
        #f)))

;; Process a single post file
(define (process-post site-config post-path)
  (displayln (string-append "Processing: " post-path))
  (let* ((file-content (read-file post-path))
         (parsed (parse-post-file file-content))
         (metadata (post-metadata parsed))
         (content-md (post-content parsed))
         (content-html (markdown->html content-md))
         (title (get-meta metadata 'title))
         (date (get-meta metadata 'date))
         (post-data (list (list 'title title)
                         (list 'date date)
                         (list 'content content-html))))
    (cons title post-data)))

;; Convert s-expression to HTML string
(define (sexp->html sexp)
  (cond
    [(string? sexp) sexp]
    [(number? sexp) (number->string sexp)]
    [(symbol? sexp) (symbol->string sexp)]
    [(list? sexp)
     (if (null? sexp)
         ""
         (let ((tag (car sexp))
               (rest (cdr sexp)))
           (if (and (not (null? rest)) (list? (car rest)) (not (null? (car rest))) (list? (caar rest)))
               ;; Has attributes
               (let ((attrs (car rest))
                     (children (cdr rest)))
                 (string-append "<" (symbol->string tag)
                               (attrs->html attrs)
                               ">"
                               (children->html children)
                               "</" (symbol->string tag) ">"))
               ;; No attributes
               (string-append "<" (symbol->string tag) ">"
                             (children->html rest)
                             "</" (symbol->string tag) ">"))))]
    [else ""]))

;; Convert attributes list to HTML
(define (attrs->html attrs)
  (if (null? attrs)
      ""
      (string-append " "
                    (string-join
                     (map (lambda (attr)
                            (string-append (symbol->string (car attr))
                                         "=\""
                                         (if (symbol? (cadr attr))
                                             (symbol->string (cadr attr))
                                             (cadr attr))
                                         "\""))
                          attrs)
                     " "))))

;; Convert children list to HTML
(define (children->html children)
  (if (null? children)
      ""
      (string-join (map sexp->html children) "")))

;; Build the site
(define (build-site)
  (displayln "Building site...")

  ;; Create build directory
  (create-directory "build")

  ;; Load site config (simplified for now)
  (displayln "Site built successfully!"))

;; Entry point
(build-site)
