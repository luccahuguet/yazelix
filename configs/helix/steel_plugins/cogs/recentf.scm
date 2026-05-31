; (require-builtin helix/core/typable as helix.)
; (require-builtin helix/core/static as helix.static.)
; (require-builtin helix/core/editor)

(require "helix/editor.scm")
(require "helix/misc.scm")

(provide refresh-files
         flush-recent-files
         recentf-open-files
         recentf-snapshot
         get-recent-files
         set-recent-file-location!)

(define MAX-FILE-COUNT 25)

(define RECENTF-FILE ".helix/recent-files.txt")

(define ENABLED #f)

(define (set-recent-file-location! path)
  (set! RECENTF-FILE path))

;; Only get the doc if it exists - also use real options instead of false here cause it kinda sucks
; (define (editor-get-doc-if-exists doc-id)
;   (if (editor-doc-exists? doc-id) (editor->get-document doc-id) #f))

(define (read-recent-files)
  (unless (path-exists? ".helix")
    (create-directory! ".helix"))

  (cond
    ;; We're just storing these as strings with the quotes still there, so that we
    ;; can call `read` on them accordingly
    [(path-exists? RECENTF-FILE)

     (call-with-input-file RECENTF-FILE (lambda (f) (~> f read-port-to-string read!)))]
    [else '()]))

(define *recent-files* (read-recent-files))

(define (get-recent-files)
  *recent-files*)

(define (remove-duplicates lst)
  ;; Iterate over, grabbing each value, check if its in the hash, otherwise skip it
  (define (remove-duplicates-via-hash lst accum set)
    (cond
      [(null? lst) accum]
      [else
       (let ([elem (car lst)])
         (if (hashset-contains? set elem)
             (remove-duplicates-via-hash (cdr lst) accum set)
             (remove-duplicates-via-hash (cdr lst) (cons elem accum) (hashset-insert set elem))))]))

  (reverse (remove-duplicates-via-hash lst '() (hashset))))

(define (refresh-files)
  (let* ([document-ids (editor-all-documents)]
         [currently-opened-files
          (filter string? (map (lambda (doc-id) (editor-document->path doc-id)) document-ids))])

    ;; Merge the files with the existing list
    (let* ([full-list (append currently-opened-files *recent-files*)]
           [deduped (remove-duplicates full-list)])

      (set! *recent-files* (take deduped MAX-FILE-COUNT)))))

(define (flush-recent-files)
  ;; Open the output file, and then we'll write all the recent files down
  (call-with-port (open-output-file RECENTF-FILE #:exists 'truncate)
                  (lambda (output-file)
                    (map (lambda (line)
                           (when (string? line)
                             (write-line! output-file line)))
                         *recent-files*))))

(define (helix-picker! pick-list)
  (push-component! (picker pick-list)))

(define (recentf-open-files)
  (helix-picker! *recent-files*))

;; Runs every 2 minutes, and snapshots the visited files
(define (recentf-snapshot)

  (refresh-files)
  (flush-recent-files)

  (enqueue-thread-local-callback-with-delay (* 1000 60 2) recentf-snapshot))
