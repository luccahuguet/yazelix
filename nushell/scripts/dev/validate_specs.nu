#!/usr/bin/env nu

use ../utils/common.nu [get_yazelix_dir]

def load_bead_ids [] {
    let issues_path = ((get_yazelix_dir) | path join ".beads" "issues.jsonl")

    open --raw $issues_path
    | lines
    | where { |line| ($line | str trim) != "" }
    | each { |line| $line | from json | get id }
}

def get_traceability_section [content: string] {
    let sections = (
        $content
        | split row "## "
        | skip 1
        | each { |section|
            let lines = ($section | lines)
            {
                header: ($lines | first | str trim)
                body: ($lines | skip 1 | str join "\n")
            }
        }
    )

    $sections
    | where header == "Traceability"
    | get -o 0.body
    | default ""
}

def validate_spec_file [spec_path: string, bead_ids: list<string>] {
    let relative_path = ($spec_path | path relative-to (get_yazelix_dir))
    let content = (open --raw $spec_path)
    let traceability = (get_traceability_section $content)

    if ($traceability | is-empty) {
        return [$"($relative_path): missing `## Traceability` section"]
    }

    let bead_match = (
        $traceability
        | lines
        | where { |line| $line | str starts-with "- Bead:" }
        | get -o 0
        | default ""
        | parse --regex '^- Bead:\s+`(?<bead_id>[^`]+)`$'
        | get -o 0.bead_id
        | default null
    )

    let defended_by = (
        $traceability
        | lines
        | where { |line| $line | str starts-with "- Defended by:" }
    )

    mut errors = []

    if $bead_match == null {
        $errors = ($errors | append $"($relative_path): missing valid `- Bead: ` traceability entry")
    } else if not ($bead_match in $bead_ids) {
        $errors = ($errors | append $"($relative_path): traceability bead `($bead_match)` does not exist in .beads/issues.jsonl")
    }

    if ($defended_by | is-empty) {
        $errors = ($errors | append $"($relative_path): expected at least one `- Defended by:` traceability entry")
    }

    $errors
}

export def main [] {
    let yazelix_dir = get_yazelix_dir
    let spec_files = (
        glob (($yazelix_dir | path join "docs" "specs" "*.md"))
        | where { |path| ($path | path basename) != "template.md" }
    )
    let bead_ids = (load_bead_ids)

    if ($spec_files | is-empty) {
        return
    }

    let errors = (
        $spec_files
        | each { |spec_path| validate_spec_file $spec_path $bead_ids }
        | flatten
    )

    if not ($errors | is-empty) {
        $errors | each { |line| print $"❌ ($line)" }
        error make { msg: "Spec traceability validation failed" }
    }
}
