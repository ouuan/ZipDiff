#lang racket
(require file/unzip)

(let ([args (current-command-line-arguments)])
  (define src (vector-ref args 0))
  (define dest (vector-ref args 1))
  (unzip src (make-filesystem-entry-reader #:dest dest #:exists 'replace)))
