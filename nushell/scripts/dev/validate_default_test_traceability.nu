#!/usr/bin/env nu

const REPO_ROOT = (path self | path dirname | path dirname | path dirname | path dirname)
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

def load_default_suite_entrypoints [] {
    use ../utils/test_runner.nu [get_default_test_file_names]
    get_default_test_file_names
}

def load_spec_defended_by_lines [] {
    let spec_files = (
        glob (($REPO_ROOT | path join "docs" "specs" "*.md"))
        | where { |path| ($path | path basename) != "template.md" }
    )

    $spec_files
    | each { |spec_path|
        let relative_path = ($spec_path | path relative-to $REPO_ROOT)
        let traceability = (get_traceability_section (open --raw $spec_path))
        $traceability
        | lines
        | where { |line| $line | str starts-with "- Defended by:" }
        | each { |line|
            {
                spec: $relative_path
                line: $line
            }
        }
    }
    | flatten
}

def is_spec_backed [entrypoint: string, defended_by_lines: list<record>] {
    let full_path = $"nushell/scripts/dev/($entrypoint)"

    $defended_by_lines
    | any { |record|
        ($record.line | str contains $entrypoint) or ($record.line | str contains $full_path)
    }
}

export def main [] {
    let entrypoints = (load_default_suite_entrypoints)
    let defended_by_lines = (load_spec_defended_by_lines)
    mut errors = []

    for entrypoint in $entrypoints {
        if not (is_spec_backed $entrypoint $defended_by_lines) {
            $errors = ($errors | append $"Default-suite entrypoint is not tied to any spec traceability line: ($entrypoint)")
        }
    }

    if not ($errors | is-empty) {
        $errors | each { |line| print $"❌ ($line)" }
        error make { msg: "Default test suite traceability validation failed" }
    }
}
