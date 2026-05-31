(require "helix/editor.scm")

;; Book keeping for keymaps
(require (only-in "keymaps.scm" *reverse-buffer-map-insert*))

(require (prefix-in helix. "helix/commands.scm"))
(require (prefix-in helix.static. "helix/static.scm"))

(provide make-new-labelled-buffer!
         temporarily-switch-focus
         open-or-switch-focus
         currently-in-labelled-buffer?
         open-labelled-buffer
         open-or-create-labelled-buffer!
         maybe-fetch-doc-id
         fetch-doc-id)

;; Temporary buffer map, key -> doc id
(define *temporary-buffer-map* (hash))
;; Last focused - will allow us to swap between the last view we were at
(define *last-focus* 'uninitialized)

;; Mark the last focused document, so that we can return to it
(define (mark-last-focused!)
  (let* ([focus (editor-focus)])
    (set! *last-focus* focus)
    focus))

;; TODO: These appear to be the same function
(define (currently-focused)
  (editor-focus))

;; Grab whatever we're currently focused on
(define (get-current-focus)
  (editor-focus))

;; Get the current document id
(define (get-current-doc-id)
  (let* ([focus (editor-focus)]) (editor->doc-id focus)))

;;@doc
;; Attempts to find the doc id associated with the given key, returns #false if
;; the key doesn't exist
(define (maybe-fetch-doc-id key)
  (hash-try-get *temporary-buffer-map* key))

;;@doc
;; Attempts to find the doc id associated with the given key, errors if
;; the key does not exist
(define (fetch-doc-id key)
  (hash-get *temporary-buffer-map* key))

;;@doc
;; Creates a new labelled buffer that can be access by the key `label`.
;; Optionally sets the language type if provided
(define (make-new-labelled-buffer! #:label label
                                   #:language-type (language-type #f)
                                   #:side (side 'none))

  ;; Save our last state to return to it afterwards
  (define last-focused (currently-focused))
  (define last-mode (editor-mode))

  ;; Open up the new labelled buffer in a vertical split, set the language accordingly
  ;; if it has been passed in
  (helix.vsplit-new)

  ;; Label this buffer - it will now show up instead of `[scratch]`
  (set-scratch-buffer-name! (string-append "[" label "]"))

  (when (eq? side 'left)
    (helix.static.move-window-far-left))

  (when (eq? side 'right)
    (helix.static.move-window-far-right))

  (when language-type
    (helix.set-language language-type))

  ;; Add the document id to our internal mapping.
  (set! *temporary-buffer-map* (hash-insert *temporary-buffer-map* label (get-current-doc-id)))

  (*reverse-buffer-map-insert* (doc-id->usize (get-current-doc-id)) label)

  ;; Go back to where we were before
  (editor-set-focus! last-focused)
  (editor-set-mode! last-mode))

;; Switch the focus for the duration of the thunk, and return to where we were previously
(define (temporarily-switch-focus thunk)
  (define last-focused (mark-last-focused!))
  (define last-mode (editor-mode))
  (thunk)
  (editor-set-focus! last-focused)
  (editor-set-mode! last-mode))

(define (open-or-switch-focus document-id)
  (define maybe-view-id? (editor-doc-in-view? document-id))
  (if maybe-view-id? (editor-set-focus! maybe-view-id?) (editor-switch! document-id)))

(define (open-labelled-buffer label)
  (open-or-switch-focus (hash-ref *temporary-buffer-map* label)))

(define (open-or-create-labelled-buffer!
         #:label label
         #:language-type (language-type #f)
         #:side (side 'none))
  (define maybe-doc-id (hash-try-get *temporary-buffer-map* label))
  (if maybe-doc-id
      (open-or-switch-focus maybe-doc-id)
      (begin
        (make-new-labelled-buffer! #:label label
                                   #:language-type language-type
                                   #:side side)
        (open-labelled-buffer label))))

(define (currently-in-labelled-buffer? label)
  (define requested-label (hash-try-get *temporary-buffer-map* label))
  (equal? (doc-id->usize requested-label) (doc-id->usize (get-current-doc-id))))
