#!/usr/bin/env nu

def write_outputs [values: record] {
    let output_path = ($env.GITHUB_OUTPUT? | default "")
    if ($output_path | is-empty) {
        return
    }

    $values
    | transpose key value
    | each { |row| $"($row.key)=($row.value)" }
    | append ""
    | str join "\n"
    | save --append $output_path
}

def flush_beads_export [] {
    let flushed = (^br sync --flush-only | complete)

    if $flushed.exit_code != 0 {
        error make {
            msg: $"Failed to flush Beads export: ($flushed.stderr | str trim)"
        }
    }
}

def infer_issue_type [issue: record] {
    let labels = (
        $issue.labels?
        | default []
        | each { |label| $label.name | str downcase }
    )
    let title = ($issue.title | str downcase)

    if ("bug" in $labels) or ($title | str starts-with "[bug]") {
        return "bug"
    }
    if ("docs" in $labels) or ("documentation" in $labels) {
        return "docs"
    }
    if ("question" in $labels) {
        return "question"
    }
    if ("epic" in $labels) {
        return "epic"
    }
    if ("feature" in $labels) or ("enhancement" in $labels) {
        return "feature"
    }

    "task"
}

def build_description [issue: record] {
    let summary = $"Imported GitHub issue #($issue.number)."
    let body = ($issue.body? | default "" | str trim)

    if ($body | is-empty) {
        return $summary
    }

    [$summary, "", "GitHub body:", $body] | str join "\n"
}

def parse_br_json [] {
    let value = $in
    let parsed = if (($value | describe) == "string") {
        $value | from json
    } else {
        $value
    }

    if (($parsed | describe | str starts-with "record<") and (($parsed | columns) | any { |column| $column == "issues" })) {
        return $parsed.issues
    }

    $parsed
}

def list_beads [] {
    ^br list --all --limit 0 --json
    | complete
    | get stdout
    | parse_br_json
}

def find_bead_by_external_ref [issue_url: string] {
    list_beads
    | where { |bead| (($bead.external_ref? | default "") == $issue_url) }
    | get -o 0
}

def maybe_comment_tracking [issue_number: int, bead_id: string] {
    let body = [
        $"Tracked in Beads as `($bead_id)`."
        ""
        "Contract:"
        "- GitHub owns the public issue and open/closed lifecycle."
        "- Beads owns planning metadata, dependencies, priority, and execution notes."
        "- The stable link between them is the GitHub issue URL stored as the bead `external_ref`."
    ] | str join "\n"

    ^gh issue comment ($issue_number | into string) -R $env.GITHUB_REPOSITORY --body $body
    | complete
    | ignore
}

def create_bead [issue: record] {
    let issue_url = $issue.html_url

    let created = (
        ^br create
            --title $issue.title
            -t (infer_issue_type $issue)
            -p 2
            --description (build_description $issue)
            --external-ref $issue_url
            --status open
            --json
        | complete
    )

    if $created.exit_code != 0 {
        error make {
            msg: $"Failed to create bead for issue #($issue.number): ($created.stderr | str trim)"
        }
    }

    let bead = ($created.stdout | parse_br_json)
    flush_beads_export
    maybe_comment_tracking $issue.number $bead.id
    write_outputs {
        bead_id: $bead.id
        created: "true"
    }
}

export def main [] {
    let issue = (open --raw $env.GITHUB_EVENT_PATH | from json | get issue)
    let issue_url = $issue.html_url
    let action = $env.GITHUB_EVENT_ACTION
    let bead = (find_bead_by_external_ref $issue_url)

    if $action == "opened" {
        if $bead == null {
            create_bead $issue
        } else {
            write_outputs {
                bead_id: $bead.id
                created: "false"
            }
        }
        return
    }

    if $action == "reopened" {
        if $bead == null {
            create_bead $issue
            return
        }

        if $bead.status == "closed" {
            ^br reopen $bead.id --reason "Reopened on GitHub" | complete | ignore
            flush_beads_export
        }

        write_outputs {
            bead_id: $bead.id
            created: "false"
        }
        return
    }

    if $action == "closed" {
        if $bead == null {
            write_outputs {
                bead_id: ""
                created: "false"
            }
            return
        }

        if $bead.status != "closed" {
            ^br close $bead.id --reason "Closed on GitHub" | complete | ignore
            flush_beads_export
        }

        write_outputs {
            bead_id: $bead.id
            created: "false"
        }
        return
    }

    error make { msg: $"Unsupported action: ($action)" }
}
