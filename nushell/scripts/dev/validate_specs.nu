#!/usr/bin/env nu

use ./contract_traceability_helpers.nu [load_bead_ids load_contract_items]

const REPO_ROOT = (path self | path dirname | path dirname | path dirname | path dirname)
const ALLOWED_CONTRACT_TYPES = [
    "behavior"
    "invariant"
    "boundary"
    "ownership"
    "failure_mode"
    "non_goal"
]
const ALLOWED_CONTRACT_STATUSES = [
    "live"
    "planning"
    "deprecated"
    "historical"
    "quarantine"
]
const ALLOWED_VERIFICATION_MODES = [
    "automated"
    "validator"
    "manual"
    "unverified"
]

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
    let relative_path = ($spec_path | path relative-to $REPO_ROOT)
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

def has_allowed_verification_mode [verification: string] {
    $ALLOWED_VERIFICATION_MODES
    | any { |mode| $verification | str contains $mode }
}

def validate_contract_item [item: record] {
    mut errors = []

    if $item.type == null {
        $errors = ($errors | append $"($item.spec): contract item `($item.id)` is missing `- Type:`")
    } else if not ($ALLOWED_CONTRACT_TYPES | any { |allowed| $allowed == $item.type }) {
        $errors = ($errors | append $"($item.spec): contract item `($item.id)` declares unsupported type `($item.type)`")
    }

    if $item.status == null {
        $errors = ($errors | append $"($item.spec): contract item `($item.id)` is missing `- Status:`")
        return $errors
    }

    if not ($ALLOWED_CONTRACT_STATUSES | any { |allowed| $allowed == $item.status }) {
        $errors = ($errors | append $"($item.spec): contract item `($item.id)` declares unsupported status `($item.status)`")
        return $errors
    }

    if $item.status != "historical" {
        for field_name in ["owner" "statement" "verification"] {
            if (($item | get -o $field_name) == null) {
                let field_label = if $field_name == "owner" {
                    "Owner"
                } else if $field_name == "statement" {
                    "Statement"
                } else {
                    "Verification"
                }
                $errors = ($errors | append $"($item.spec): contract item `($item.id)` is missing `- ($field_label):`")
            }
        }
    }

    if ($item.verification != null) and not (has_allowed_verification_mode $item.verification) {
        $errors = ($errors | append $"($item.spec): contract item `($item.id)` has `- Verification:` but no allowed verification mode keyword")
    }

    if ($item.status == "live") and ($item.verification == null) {
        $errors = ($errors | append $"($item.spec): live contract item `($item.id)` must name a verification path or explicit manual/unverified reason")
    }

    $errors
}

export def main [] {
    let spec_files = (
        glob (($REPO_ROOT | path join "docs" "specs" "*.md"))
        | where { |path| ($path | path basename) != "template.md" }
    )
    let bead_ids = (load_bead_ids)
    let contract_items = (load_contract_items)

    if ($spec_files | is-empty) {
        return
    }

    mut errors = (
        $spec_files
        | each { |spec_path| validate_spec_file $spec_path $bead_ids }
        | flatten
    )

    mut seen_ids = {}
    for item in $contract_items {
        if ($seen_ids | columns | any { |column| $column == $item.id }) {
            let existing_spec = ($seen_ids | get $item.id)
            $errors = ($errors | append $"Duplicate contract item id `($item.id)` appears in both ($existing_spec) and ($item.spec)")
        } else {
            $seen_ids = ($seen_ids | upsert $item.id $item.spec)
        }

        $errors = ($errors | append (validate_contract_item $item))
    }

    if not ($errors | is-empty) {
        $errors | each { |line| print $"❌ ($line)" }
        error make { msg: "Spec traceability validation failed" }
    }
}
