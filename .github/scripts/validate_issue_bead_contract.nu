#!/usr/bin/env nu

def contract_start [] {
    "2026-03-22T00:00:00Z" | into datetime
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

def load_github_issues [] {
    let listed = (^gh issue list --state all --limit 1000 --json number,state,title,url,createdAt | complete)
    if $listed.exit_code != 0 {
        error make {
            msg: $"Failed to load GitHub issues: ($listed.stderr | str trim)"
        }
    }

    $listed.stdout | parse_json_output
}

def load_beads [] {
    let listed = (^br list --all --limit 0 --json | complete)
    if $listed.exit_code != 0 {
        error make {
            msg: $"Failed to load Beads issues: ($listed.stderr | str trim)"
        }
    }

    $listed.stdout | parse_json_output
}

export def main [] {
    let github_issues = load_github_issues
    let beads = load_beads
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
    }

    if not ($errors | is-empty) {
        print "Bead/GitHub issue contract violations detected:"
        $errors | each { |error| print $"- ($error)" }
        error make { msg: "Bead/GitHub issue contract is invalid" }
    }

    print "Bead/GitHub issue contract is valid."
}
