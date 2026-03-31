#!/usr/bin/env nu

const REPO_ROOT = (path self | path dirname | path dirname | path dirname | path dirname)
const DEFAULT_SUITE_MAX_CANONICAL_TESTS = 56

def to_dev_relative_path [file_name: string] {
    $"nushell/scripts/dev/($file_name)"
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

def load_default_suite_entrypoints [] {
    use ../utils/test_runner.nu [get_default_test_file_names]
    get_default_test_file_names
}

def load_default_suite_component_files [] {
    let suite_runner = ($REPO_ROOT | path join "nushell" "scripts" "dev" "test_yzx_commands.nu")
    let content = (open --raw $suite_runner)

    $content
    | lines
    | where { |line| ($line | str trim | str starts-with "use ./test_") and ($line | str contains "[run_") and ($line | str contains "canonical_tests]") }
    | parse --regex 'use \./([^ ]+) \['
    | get capture0
    | uniq
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

def load_file_defends_lines [relative_path: string] {
    let full_path = ($REPO_ROOT | path join $relative_path)

    open --raw $full_path
    | lines
    | where { |line| $line | str starts-with "# Defends:" }
}

def has_existing_spec_reference [relative_path: string] {
    let defends_lines = (load_file_defends_lines $relative_path)

    $defends_lines
    | any { |line|
        let spec_path = ($line | str replace "# Defends:" "" | str trim)
        ($spec_path | str starts-with "docs/") and (($REPO_ROOT | path join $spec_path) | path exists)
    }
}

def load_canonical_test_names [relative_path: string] {
    let content = (open --raw ($REPO_ROOT | path join $relative_path))
    let matches = (
        $content
        | parse --regex '(?s)export def run_[A-Za-z0-9_]+canonical_tests \[\] \{\s*\[(.*?)\]\s*\}'
    )

    if ($matches | is-empty) {
        error make { msg: $"Could not find canonical test list in: ($relative_path)" }
    }

    let capture = ($matches | get -o 0.capture0)
    if $capture == null {
        error make { msg: $"Could not extract canonical test list capture from: ($relative_path)" }
    }

    $capture
    | parse --regex '\((test_[A-Za-z0-9_]+)\)'
    | get capture0
}

def load_defined_test_names [relative_path: string] {
    let content = (open --raw ($REPO_ROOT | path join $relative_path))

    $content
    | parse --regex '(?m)^def (test_[A-Za-z0-9_]+) \[\] \{'
    | get capture0
}

export def main [] {
    let entrypoints = (load_default_suite_entrypoints)
    let component_files = (load_default_suite_component_files)
    let defended_by_lines = (load_spec_defended_by_lines)
    mut errors = []
    mut canonical_test_count = 0

    for entrypoint in $entrypoints {
        if not (is_spec_backed $entrypoint $defended_by_lines) {
            $errors = ($errors | append $"Default-suite entrypoint is not tied to any spec traceability line: ($entrypoint)")
        }
    }

    for relative_path in $component_files {
        let dev_relative_path = (to_dev_relative_path $relative_path)

        if not (has_existing_spec_reference $dev_relative_path) {
            $errors = ($errors | append $"Default-suite component file is missing a valid '# Defends:' spec reference: ($dev_relative_path)")
        }

        let canonical_tests = (load_canonical_test_names $dev_relative_path)
        let defined_tests = (load_defined_test_names $dev_relative_path)
        let dead_tests = (
            $defined_tests
            | where { |name| not ($canonical_tests | any { |canonical| $canonical == $name }) }
        )

        $canonical_test_count = ($canonical_test_count + ($canonical_tests | length))

        for dead_test in $dead_tests {
            $errors = ($errors | append $"Default-suite component file contains a noncanonical dead test: ($dev_relative_path) :: ($dead_test)")
        }
    }

    if $canonical_test_count > $DEFAULT_SUITE_MAX_CANONICAL_TESTS {
        $errors = ($errors | append $"Default-suite canonical test count exceeded budget: ($canonical_test_count) > ($DEFAULT_SUITE_MAX_CANONICAL_TESTS)")
    }

    if not ($errors | is-empty) {
        $errors | each { |line| print $"❌ ($line)" }
        error make { msg: "Default test suite traceability validation failed" }
    }
}
