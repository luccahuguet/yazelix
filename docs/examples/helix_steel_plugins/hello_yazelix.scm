(require (only-in "helix/misc.scm" set-status!))

(provide hello-yazelix)

(define (hello-yazelix)
  (set-status! "Hello from a custom Yazelix Steel plugin"))
