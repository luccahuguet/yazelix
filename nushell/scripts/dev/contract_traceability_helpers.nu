#!/usr/bin/env nu

const REPO_ROOT = (path self | path dirname | path dirname | path dirname | path dirname)
const POLICY_ONLY_SPEC_PATHS = [
    "docs/specs/test_suite_governance.md"
]
const QUARANTINE_MANIFEST_PATH = ($REPO_ROOT | path join "docs" "contract_traceability_quarantine.toml")

export def load_bead_ids [] {
    let issues_path = ($REPO_ROOT | path join ".beads" "issues.jsonl")

    if not ($issues_path | path exists) {
        return []
    }

    open --raw $issues_path
    | lines
    | where { |line| ($line | str trim) != "" }
    | each { |line| $line | from json | get id }
}

def normalize_contract_field_name [field_name: string] {
    $field_name
    | str downcase
    | str replace -a " " "_"
}

def finalize_contract_item [item: record] {
    $item
    | upsert type ($item.type? | default null)
    | upsert status ($item.status? | default null)
    | upsert owner ($item.owner? | default null)
    | upsert statement ($item.statement? | default null)
    | upsert verification ($item.verification? | default null)
    | upsert notes ($item.notes? | default null)
    | upsert source ($item.source? | default null)
    | upsert related_bead ($item.related_bead? | default null)
    | upsert deletion_note ($item.deletion_note? | default null)
}

def parse_contract_heading [line: string] {
    (
        [$line]
        | parse --regex '^####\s+([A-Z0-9]{2,8}-[0-9]{3,})$'
        | get -o 0.capture0
        | default null
    )
}

export def load_contract_items [] {
    let spec_files = (
        glob (($REPO_ROOT | path join "docs" "specs" "*.md"))
        | where { |path| ($path | path basename) != "template.md" }
    )

    mut items = []

    for spec_path in $spec_files {
        let relative_path = ($spec_path | path relative-to $REPO_ROOT)
        let lines = (open --raw $spec_path | lines)
        mut current = {}

        for line in $lines {
            let trimmed = ($line | str trim)
            let heading_id = (parse_contract_heading $trimmed)

            if $heading_id != null {
                if not (($current | columns) | is-empty) {
                    $items = ($items | append (finalize_contract_item $current))
                }

                $current = {
                    id: $heading_id
                    spec: $relative_path
                }
                continue
            }

            if (($current | columns) | is-empty) {
                continue
            }

            if (
                (($trimmed | str starts-with "# ") or ($trimmed | str starts-with "## ") or ($trimmed | str starts-with "### "))
                and not ($trimmed | str starts-with "#### ")
            ) {
                $items = ($items | append (finalize_contract_item $current))
                $current = {}
                continue
            }

            let parsed_field = (
                [$trimmed]
                | parse --regex '^- ([A-Za-z][A-Za-z ]+):\s*(.*)$'
                | get -o 0
                | default null
            )

            if $parsed_field != null {
                let key = (normalize_contract_field_name $parsed_field.capture0)
                $current = ($current | upsert $key $parsed_field.capture1)
            }
        }

        if not (($current | columns) | is-empty) {
            $items = ($items | append (finalize_contract_item $current))
        }
    }

    $items
}

export def find_contract_item [contract_items: list<record>, contract_id: string] {
    $contract_items
    | where id == $contract_id
    | get -o 0
    | default null
}

export def spec_has_contract_items [contract_items: list<record>, spec_path: string] {
    $contract_items | any { |item| $item.spec == $spec_path }
}

export def is_policy_only_spec_path [spec_path: string] {
    $POLICY_ONLY_SPEC_PATHS | any { |path| $path == $spec_path }
}

export def parse_contract_marker_ids [line: string] {
    let trimmed = ($line | str trim)
    let payload = if ($trimmed | str starts-with "# Contract:") {
        $trimmed | str replace "# Contract:" ""
    } else if ($trimmed | str starts-with "// Contract:") {
        $trimmed | str replace "// Contract:" ""
    } else {
        ""
    }

    if (($payload | str trim) | is-empty) {
        return []
    }

    $payload
    | split row ","
    | each { |entry| $entry | str trim }
    | where { |entry| not ($entry | is-empty) }
    | uniq
}

export def load_contract_traceability_quarantine_entries [] {
    if not ($QUARANTINE_MANIFEST_PATH | path exists) {
        return []
    }

    let manifest = (open $QUARANTINE_MANIFEST_PATH)
    $manifest.entries? | default []
}
