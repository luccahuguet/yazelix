;; Yazelix Nova packaged Steel init.
(require (only-in "@bridgeModule@" yzx-helix-start))
(require (only-in "steel/result" unwrap-ok))
(require-builtin steel/process)
(require-builtin yazelix/transport)

(define (yzx-env-or-false name)
  (with-handler (lambda (_) #f) (env-var name)))

(define (yzx-register-bridge server)
  (define status
    (unwrap-ok
     (wait
      (unwrap-ok
       (spawn-process
        (command "@bridgeRegister@" (list (transport-local-addr server))))))))
  (if (equal? status 0)
      server
      (begin
        (transport-stop! server)
        (error "Yazelix could not publish the Helix bridge registry"))))

(define yzx-helix-server
  (if (equal? (yzx-env-or-false "YAZELIX_HELIX_BRIDGE") "1")
      (yzx-register-bridge
       (yzx-helix-start (env-var "YAZELIX_HELIX_BRIDGE_AUTH_TOKEN")))
      #f))

(define yzx-user-init (yzx-env-or-false "YAZELIX_HELIX_USER_STEEL_INIT"))
(if (string? yzx-user-init) (load yzx-user-init) #f)
