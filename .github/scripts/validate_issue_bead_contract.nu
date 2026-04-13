#!/usr/bin/env nu

use ../../nushell/scripts/maintainer/issue_bead_contract.nu [
    canonical_issue_bead_comment_body
    contract_start
    find_issue_bead_comment
    load_contract_beads
    load_contract_github_issues
    load_issue_comments
]

export def main [] {
    let github_issues = load_contract_github_issues
    let beads = load_contract_beads
    mut errors = []

    for issue in $github_issues {
        let created_at = ($issue.createdAt | into datetime)
        if $created_at < (contract_start) {
            continue
        }

        let matches = (
            $beads
            | where { |bead| (($bead.external_ref? | default "") == $issue.url) }
        )

        if ($matches | is-empty) {
            $errors = ($errors | append $"Missing bead for GitHub issue #($issue.number) (($issue.title))")
            continue
        }

        if (($matches | length) > 1) {
            let ids = ($matches | each { |match| $match.id } | str join ", ")
            $errors = ($errors | append $"Duplicate beads for GitHub issue #($issue.number): ($ids)")
            continue
        }

        let bead = ($matches | first)
        let is_github_open = ($issue.state == "OPEN")
        let is_bead_closed = ($bead.status == "closed")

        if $is_github_open and $is_bead_closed {
            $errors = ($errors | append $"State mismatch for GitHub issue #($issue.number): GitHub is open but bead ($bead.id) is closed")
        }

        if (not $is_github_open) and (not $is_bead_closed) {
            $errors = ($errors | append $"State mismatch for GitHub issue #($issue.number): GitHub is closed but bead ($bead.id) is ($bead.status)")
        }

        let comments = (load_issue_comments $issue.number)
        let comment = (find_issue_bead_comment $comments)
        let expected_comment_body = (canonical_issue_bead_comment_body $bead.id)

        if ($comment | is-empty) {
            $errors = ($errors | append $"Missing Beads comment for GitHub issue #($issue.number): expected `($bead.id)`")
        } else if ((($comment.body? | default "") | str trim) != $expected_comment_body) {
            $errors = ($errors | append $"Incorrect Beads comment for GitHub issue #($issue.number): expected `($bead.id)`")
        }
    }

    if not ($errors | is-empty) {
        print "Bead/GitHub issue contract violations detected:"
        $errors | each { |error| print $"- ($error)" }
        error make { msg: "Bead/GitHub issue contract is invalid" }
    }

    print "Bead/GitHub issue contract is valid."
}
