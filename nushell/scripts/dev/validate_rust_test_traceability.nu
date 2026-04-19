#!/usr/bin/env nu

const REPO_ROOT = (path self | path dirname | path dirname | path dirname | path dirname)
const MIN_STRENGTH_BY_LANE = {
    default: 7
    maintainer: 6
    sweep: 6
    manual: 6
}
const ALLOWED_TEST_LANES = [
    "default"
    "maintainer"
    "sweep"
    "manual"
]

def load_rust_test_file_paths [] {
    (
        (glob (($REPO_ROOT | path join "rust_core" "**" "*.rs")))
        | append (glob (($REPO_ROOT | path join "rust_plugins" "**" "*.rs")))
        | where { |path| not (($path | into string) | str contains "/target/") }
        | uniq
    )
}

def get_rust_lines [relative_path: string] {
    open --raw ($REPO_ROOT | path join $relative_path) | lines
}

def is_rust_test_attribute_line [line: string] {
    let trimmed = ($line | str trim)
    $trimmed =~ '^#\[(?:[A-Za-z0-9_]+::)*test(?:\([^]]*\))?\]$'
}

def file_contains_rust_tests [relative_path: string] {
    get_rust_lines $relative_path
    | any { |line| is_rust_test_attribute_line $line }
}

def parse_test_lane [relative_path: string] {
    (
        get_rust_lines $relative_path
        | where { |line| (($line | str trim) | str starts-with "// Test lane:") }
        | get -o 0
        | default null
    )
    | if $in == null {
        null
    } else {
        $in
        | str trim
        | str replace "// Test lane:" ""
        | str trim
    }
}

def get_prior_nonempty_lines_before_index [relative_path: string, line_index: int] {
    get_rust_lines $relative_path
    | first $line_index
    | reverse
    | where { |line| not (($line | str trim) | is-empty) }
    | first 4
    | each { |line| $line | str trim }
}

def parse_rust_test_name_after_index [relative_path: string, attribute_index: int] {
    let candidate_line = (
        get_rust_lines $relative_path
        | skip ($attribute_index + 1)
        | where { |line| not (($line | str trim) | is-empty) }
        | get -o 0
        | default null
    )

    if $candidate_line == null {
        error make {msg: $"Could not find Rust test function after attribute in: ($relative_path) :: line ($attribute_index + 1)"}
    }

    let parsed = (
        [$candidate_line]
        | parse --regex '^\s*(?:pub\s+)?(?:async\s+)?fn\s+([A-Za-z0-9_]+)\s*\('
        | get -o 0
    )

    if $parsed == null {
        error make {msg: $"Could not parse Rust test function after attribute in: ($relative_path) :: ($candidate_line | str trim)"}
    }

    $parsed.capture0
}

def load_defined_rust_tests [relative_path: string] {
    let lines = (get_rust_lines $relative_path)

    $lines
    | enumerate
    | where { |entry| is_rust_test_attribute_line $entry.item }
    | each { |entry|
        {
            attribute_index: $entry.index
            test_name: (parse_rust_test_name_after_index $relative_path $entry.index)
        }
    }
}

def has_valid_definition_test_justification [relative_path: string, attribute_index: int] {
    let prior_nonempty_lines = (get_prior_nonempty_lines_before_index $relative_path $attribute_index)

    ["// Defends:", "// Regression:", "// Invariant:"]
    | any { |prefix| $prior_nonempty_lines | any { |line| $line | str starts-with $prefix } }
}

def parse_structured_strength_line [relative_path: string, test_name: string, strength_line: string] {
    let parsed = (
        [$strength_line]
        | parse --regex '// Strength:\s+defect=([0-2])\s+behavior=([0-2])\s+resilience=([0-2])\s+cost=([0-2])\s+uniqueness=([0-2])\s+total=([0-9]+)/10'
        | get -o 0
    )

    if $parsed == null {
        error make {msg: $"Could not parse structured '// Strength:' marker near: ($relative_path) :: ($test_name) :: ($strength_line)"}
    }

    let defect = ($parsed.capture0 | into int)
    let behavior = ($parsed.capture1 | into int)
    let resilience = ($parsed.capture2 | into int)
    let cost = ($parsed.capture3 | into int)
    let uniqueness = ($parsed.capture4 | into int)
    let total = ($parsed.capture5 | into int)
    let computed = ($defect + $behavior + $resilience + $cost + $uniqueness)

    if $computed != $total {
        error make {msg: $"Structured '// Strength:' marker total does not match component sum near: ($relative_path) :: ($test_name) :: expected=($computed)/10 declared=($total)/10"}
    }

    $total
}

def get_definition_test_strength [relative_path: string, attribute_index: int, test_name: string] {
    let strength_line = (
        get_prior_nonempty_lines_before_index $relative_path $attribute_index
        | where { |line| $line | str starts-with "// Strength:" }
        | get -o 0
    )

    if $strength_line == null {
        error make {msg: $"Governed Rust test is missing a nearby structured '// Strength:' marker: ($relative_path) :: ($test_name)"}
    }

    parse_structured_strength_line $relative_path $test_name $strength_line
}

export def main [] {
    mut errors = []

    for rust_path in (load_rust_test_file_paths) {
        let relative_path = ($rust_path | path relative-to $REPO_ROOT)
        if not (file_contains_rust_tests $relative_path) {
            continue
        }

        let lane = (parse_test_lane $relative_path)
        if $lane == null {
            $errors = ($errors | append $"Missing '// Test lane:' declaration in Rust test file: ($relative_path)")
            continue
        }

        if not ($ALLOWED_TEST_LANES | any { |allowed| $allowed == $lane }) {
            $errors = ($errors | append $"Rust test file declares unsupported lane '($lane)': ($relative_path)")
            continue
        }

        let minimum_strength = ($MIN_STRENGTH_BY_LANE | get -o $lane)
        for test_record in (load_defined_rust_tests $relative_path) {
            if not (has_valid_definition_test_justification $relative_path $test_record.attribute_index) {
                $errors = ($errors | append $"Governed Rust test is missing a nearby '// Defends:', '// Regression:', or '// Invariant:' marker: ($relative_path) :: ($test_record.test_name)")
            }

            let strength = (get_definition_test_strength $relative_path $test_record.attribute_index $test_record.test_name)
            if $strength < $minimum_strength {
                $errors = ($errors | append $"Governed Rust test is below the minimum strength bar of ($minimum_strength)/10 for lane '($lane)': ($relative_path) :: ($test_record.test_name) :: ($strength)/10")
            }
        }
    }

    if not ($errors | is-empty) {
        $errors | each { |line| print $"❌ ($line)" }
        error make {msg: "Rust test traceability validation failed"}
    }
}
