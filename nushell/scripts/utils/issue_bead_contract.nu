#!/usr/bin/env nu

def contract_start [] {
    "2026-03-22T00:00:00Z" | into datetime
}

def canonical_issue_bead_comment_body [bead_id: string] {
    $"Automated: Tracked in Beads as `($bead_id)`."
}

def parse_json_output [] {
    let value = $in
    if ($value | is-empty) {
        return []
    }

    let parsed = if (($value | describe) == "string") {
        $value | from json
    } else {
        $value
    }

    if ($parsed == null) {
        return []
    }

    if (($parsed | describe | str starts-with "record<") and (($parsed | columns) | any { |column| $column == "issues" })) {
        return $parsed.issues
    }

    $parsed
}

def issue_is_in_contract [issue: record] {
    let created_at = ($issue.createdAt | into datetime)
    $created_at >= (contract_start)
}

export def load_contract_github_issues [] {
    let listed = (^gh issue list --state all --limit 1000 --json number,state,title,url,createdAt,body | complete)
    if $listed.exit_code != 0 {
        error make {
            msg: $"Failed to load GitHub issues: ($listed.stderr | str trim)"
        }
    }

    $listed.stdout | parse_json_output
}

export def load_issue_comments [issue_number: int] {
    let viewed = (^gh issue view $issue_number --json comments | complete)
    if $viewed.exit_code != 0 {
        error make {
            msg: $"Failed to load comments for GitHub issue #($issue_number): ($viewed.stderr | str trim)"
        }
    }

    let parsed = ($viewed.stdout | from json)
    ($parsed.comments? | default [])
}

export def load_contract_beads [] {
    let listed = (^br list --all --limit 0 --json | complete)
    if $listed.exit_code != 0 {
        error make {
            msg: $"Failed to load Beads issues: ($listed.stderr | str trim)"
        }
    }

    $listed.stdout | parse_json_output
}

export def infer_issue_type_from_body [body?: string] {
    let trimmed = (($body | default "") | str trim)
    if ($trimmed | is-empty) {
        return "task"
    }

    let matches = (
        $trimmed
        | parse --regex '(?ms)^### Issue Type\s+(?<issue_type>[a-z_]+)(?:\s+### |\s*$)'
    )
    let extracted = if ($matches | is-empty) {
        ""
    } else {
        (
            $matches
            | get -o 0.issue_type
            | into string
            | str trim
            | str downcase
        )
    }

    if ($extracted | is-empty) {
        return "task"
    }

    let allowed = ["task" "bug" "feature" "docs" "question" "epic" "chore"]
    if $extracted in $allowed {
        $extracted
    } else {
        "task"
    }
}

export def build_imported_issue_description [issue: record] {
    let issue_number = ($issue.number | into string)
    let body = (($issue.body? | default "") | str trim)

    if ($body | is-empty) {
        return $"Imported GitHub issue #($issue_number)."
    }

    $"Imported GitHub issue #($issue_number).\n\n($body)"
}

export def plan_issue_bead_reconciliation [github_issues: list, beads: list] {
    mut actions = []
    mut errors = []

    for issue in $github_issues {
        if not (issue_is_in_contract $issue) {
            continue
        }

        let matches = (
            $beads
            | where { |bead| (($bead.external_ref? | default "") == $issue.url) }
        )

        if ($matches | is-empty) {
            $actions = ($actions | append {
                kind: "create"
                issue: $issue
            })
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
            $actions = ($actions | append {
                kind: "reopen"
                issue: $issue
                bead: $bead
            })
        } else if (not $is_github_open) and (not $is_bead_closed) {
            $actions = ($actions | append {
                kind: "close"
                issue: $issue
                bead: $bead
            })
        } else {
            $actions = ($actions | append {
                kind: "noop"
                issue: $issue
                bead: $bead
            })
        }
    }

    {
        actions: $actions
        errors: $errors
    }
}

export def find_issue_bead_comment [comments: list] {
    let matching = (
        $comments
        | where { |comment|
            let body = (($comment.body? | default "") | into string | str trim)
            ($body == "$action.body") or ($body | str starts-with "Tracked in Beads as `") or ($body | str starts-with "Automated: Tracked in Beads as `")
        }
    )

    $matching | get -o 0
}

export def plan_issue_bead_comment_sync [issue: record, bead: record, comments: list] {
    let expected_body = (canonical_issue_bead_comment_body $bead.id)
    let existing_comment = (find_issue_bead_comment $comments)

    if ($existing_comment | is-empty) {
        return {
            kind: "create"
            issue: $issue
            bead: $bead
            body: $expected_body
        }
    }

    if ((($existing_comment.body? | default "") | str trim) == $expected_body) {
        return {
            kind: "noop"
            issue: $issue
            bead: $bead
            body: $expected_body
            comment: $existing_comment
        }
    }

    {
        kind: "update"
        issue: $issue
        bead: $bead
        body: $expected_body
        comment: $existing_comment
    }
}
