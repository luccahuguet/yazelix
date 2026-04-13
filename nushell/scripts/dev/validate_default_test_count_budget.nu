#!/usr/bin/env nu

const REPO_ROOT = (path self | path dirname | path dirname | path dirname | path dirname)
const DEFAULT_SUITE_MAX_TESTS = 90

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

def load_canonical_test_names [file_name: string] {
    let full_path = ($REPO_ROOT | path join "nushell" "scripts" "dev" $file_name)
    let content = (open --raw $full_path)
    let matches = (
        $content
        | parse --regex '(?s)export def run_[A-Za-z0-9_]+canonical_tests \[\] \{\s*\[(.*?)\]\s*\}'
    )

    if ($matches | is-empty) {
        error make { msg: $"Could not find canonical test list in: ($file_name)" }
    }

    let capture = ($matches | get -o 0.capture0)
    if $capture == null {
        error make { msg: $"Could not extract canonical test list capture from: ($file_name)" }
    }

    $capture
    | parse --regex '\((test_[A-Za-z0-9_]+)\)'
    | get capture0
}

export def main [] {
    let counts = (
        load_default_suite_component_files
        | each { |file_name|
            let test_count = (load_canonical_test_names $file_name | length)
            {
                file: $file_name
                count: $test_count
            }
        }
    )
    let total = ($counts | get count | math sum)

    if $total > $DEFAULT_SUITE_MAX_TESTS {
        let breakdown = (
            $counts
            | each { |entry| $"($entry.file): ($entry.count)" }
            | str join ", "
        )
        error make { msg: $"Default-suite test-count budget exceeded: ($total) > ($DEFAULT_SUITE_MAX_TESTS). Breakdown: ($breakdown)" }
    }

    print $"✅ Default-suite test-count budget ok: ($total) <= ($DEFAULT_SUITE_MAX_TESTS)"
}
