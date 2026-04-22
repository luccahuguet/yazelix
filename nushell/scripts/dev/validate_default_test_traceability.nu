#!/usr/bin/env nu

use ./contract_traceability_helpers.nu [
    find_contract_item
    is_policy_only_spec_path
    load_bead_ids
    load_contract_items
    load_contract_traceability_quarantine_entries
    parse_contract_marker_ids
    spec_has_contract_items
]

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

def to_dev_relative_path [file_name: string] {
    $"nushell/scripts/dev/($file_name)"
}

def load_all_test_file_paths [] {
    glob (($REPO_ROOT | path join "nushell" "scripts" "dev" "test_*.nu"))
}

def get_test_lane_line [relative_path: string] {
    let full_path = ($REPO_ROOT | path join $relative_path)

    open --raw $full_path
    | lines
    | where { |line| ($line | str trim | str starts-with "# Test lane:") }
    | get -o 0
}

def parse_test_lane [relative_path: string] {
    let lane_line = (get_test_lane_line $relative_path)

    if $lane_line == null {
        null
    } else {
        $lane_line
        | str trim
        | str replace "# Test lane:" ""
        | str trim
    }
}

def has_grandfathered_extended_filename_marker [relative_path: string] {
    let full_path = ($REPO_ROOT | path join $relative_path)

    open --raw $full_path
    | lines
    | any { |line| ($line | str trim | str starts-with "# Grandfathered filename:") }
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
    use ../maintainer/test_runner.nu [get_default_test_file_names]
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

def parse_defends_spec_path [line: string] {
    let candidate = (
        $line
        | str trim
        | str replace "# Defends:" ""
        | str trim
    )

    if not ($candidate | str starts-with "docs/") {
        return null
    }

    if not (($REPO_ROOT | path join $candidate) | path exists) {
        return null
    }

    $candidate
}

def has_existing_spec_reference [relative_path: string] {
    let defends_lines = (load_file_defends_lines $relative_path)

    $defends_lines
    | any { |line|
        (parse_defends_spec_path $line) != null
    }
}

def has_non_policy_spec_reference [relative_path: string] {
    load_file_defends_lines $relative_path
    | each { |line| parse_defends_spec_path $line }
    | where { |spec_path| $spec_path != null }
    | any { |spec_path| not (is_policy_only_spec_path $spec_path) }
}

def find_traceability_quarantine_entry [entries: list<record>, kind: string, path: string] {
    $entries
    | where kind == $kind
    | where path == $path
    | get -o 0
    | default null
}

def load_canonical_test_names [relative_path: string] {
    let lines = (open --raw ($REPO_ROOT | path join $relative_path) | lines)
    let start_index = (
        $lines
        | enumerate
        | where { |entry|
            ($entry.item | str trim) =~ '^export def run_[A-Za-z0-9_]+canonical_tests \[\] \{$'
        }
        | get -o 0.index
    )

    if $start_index == null {
        error make { msg: $"Could not find canonical test runner in: ($relative_path)" }
    }

    mut brace_depth = 1
    mut body_lines = []
    for entry in ($lines | enumerate | skip ($start_index + 1)) {
        let line = $entry.item
        let opens = ($line | split chars | where { |char| $char == "{" } | length)
        let closes = ($line | split chars | where { |char| $char == "}" } | length)
        let next_depth = ($brace_depth + $opens - $closes)

        if $next_depth < 0 {
            error make { msg: $"Canonical test runner braces became unbalanced in: ($relative_path)" }
        }

        if $next_depth == 0 {
            break
        }

        $body_lines = ($body_lines | append $line)
        $brace_depth = $next_depth
    }

    if (($body_lines | is-empty) or ($brace_depth != 1 and $brace_depth != 0)) {
        error make { msg: $"Could not extract canonical test runner body from: ($relative_path)" }
    }

    ($body_lines | str join "\n")
    | parse --regex '\((test_[A-Za-z0-9_]+)\)'
    | get capture0
}

def load_defined_test_names [relative_path: string] {
    let content = (open --raw ($REPO_ROOT | path join $relative_path))

    $content
    | parse --regex '(?m)^def (test_[A-Za-z0-9_]+) \[\] \{'
    | get capture0
}

def get_test_definition_line_index [relative_path: string, test_name: string] {
    let lines = (open --raw ($REPO_ROOT | path join $relative_path) | lines)
    let definition_line = ($"def ($test_name) [] {" | str trim)

    let line_index = (
        $lines
        | enumerate
        | where { |entry| (($entry.item | str trim) == $definition_line) }
        | get -o 0.index
    )

    if $line_index == null {
        error make { msg: $"Could not find definition for ($test_name) in: ($relative_path)" }
    }

    $line_index
}

def get_prior_nonempty_lines_before_index [relative_path: string, line_index: int] {
    let lines = (open --raw ($REPO_ROOT | path join $relative_path) | lines)

    $lines
    | first $line_index
    | reverse
    | where { |line| not (($line | str trim) | is-empty) }
    | first 4
    | each { |line| $line | str trim }
}

def load_definition_traceability_lines [relative_path: string, test_name: string] {
    let line_index = (get_test_definition_line_index $relative_path $test_name)
    let prior_nonempty_lines = (get_prior_nonempty_lines_before_index $relative_path $line_index)

    $prior_nonempty_lines
    | where { |line|
        ["# Defends:", "# Regression:", "# Invariant:", "# Contract:"]
        | any { |prefix| $line | str starts-with $prefix }
    }
}

def load_definition_contract_ids [relative_path: string, test_name: string] {
    load_definition_traceability_lines $relative_path $test_name
    | where { |line| $line | str starts-with "# Contract:" }
    | each { |line| parse_contract_marker_ids $line }
    | flatten
    | uniq
}

def load_definition_defends_spec_paths [relative_path: string, test_name: string] {
    load_definition_traceability_lines $relative_path $test_name
    | where { |line| $line | str starts-with "# Defends:" }
    | each { |line| parse_defends_spec_path $line }
    | where { |spec_path| $spec_path != null }
    | uniq
}

def has_definition_regression_or_invariant [relative_path: string, test_name: string] {
    load_definition_traceability_lines $relative_path $test_name
    | any { |line| ($line | str starts-with "# Regression:") or ($line | str starts-with "# Invariant:") }
}

def definition_has_policy_only_traceability [relative_path: string, test_name: string] {
    let spec_paths = (load_definition_defends_spec_paths $relative_path $test_name)

    if ($spec_paths | is-empty) {
        return false
    }

    if (has_definition_regression_or_invariant $relative_path $test_name) {
        return false
    }

    $spec_paths | all { |spec_path| is_policy_only_spec_path $spec_path }
}

def collect_definition_contract_traceability_errors [
    relative_path: string
    test_name: string
    lane: string
    contract_items: list<record>
] {
    mut errors = []
    let contract_ids = (load_definition_contract_ids $relative_path $test_name)
    let defends_spec_paths = (load_definition_defends_spec_paths $relative_path $test_name)
    let has_regression_or_invariant = (has_definition_regression_or_invariant $relative_path $test_name)

    if (($contract_ids | is-empty) and (definition_has_policy_only_traceability $relative_path $test_name)) {
        $errors = ($errors | append $"Governed test cannot rely only on `docs/specs/test_suite_governance.md` as nearby traceability: ($relative_path) :: ($test_name)")
    }

    if (
        ($lane == "default")
        and ($contract_ids | is-empty)
        and (not $has_regression_or_invariant)
        and ($defends_spec_paths | any { |spec_path| spec_has_contract_items $contract_items $spec_path })
    ) {
        $errors = ($errors | append $"Default-lane governed test defends a spec with indexed contract items but is missing a nearby '# Contract:' marker: ($relative_path) :: ($test_name)")
    }

    for contract_id in $contract_ids {
        let item = (find_contract_item $contract_items $contract_id)

        if $item == null {
            $errors = ($errors | append $"Governed test references unknown contract id `($contract_id)`: ($relative_path) :: ($test_name)")
            continue
        }

        if not (["live" "deprecated" "quarantine"] | any { |allowed| $allowed == $item.status }) {
            $errors = ($errors | append $"Governed test references contract id `($contract_id)` with unsupported status `($item.status)`: ($relative_path) :: ($test_name)")
        }
    }

    $errors
}

def has_valid_definition_test_justification [relative_path: string, test_name: string] {
    let line_index = (get_test_definition_line_index $relative_path $test_name)
    let prior_nonempty_lines = (get_prior_nonempty_lines_before_index $relative_path $line_index)

    ["# Defends:", "# Regression:", "# Invariant:"]
    | any { |prefix| $prior_nonempty_lines | any { |line| $line | str starts-with $prefix } }
}

def get_definition_test_strength [relative_path: string, test_name: string] {
    let line_index = (get_test_definition_line_index $relative_path $test_name)
    let prior_nonempty_lines = (get_prior_nonempty_lines_before_index $relative_path $line_index)
    let strength_line = (
        $prior_nonempty_lines
        | where { |line| $line | str starts-with "# Strength:" }
        | get -o 0
    )

    if $strength_line == null {
        error make { msg: $"Governed test is missing a nearby structured '# Strength:' marker: ($relative_path) :: ($test_name)" }
    }

    parse_structured_strength_line $relative_path $test_name $strength_line
}

def parse_structured_strength_line [relative_path: string, test_name: string, strength_line: string] {
    let parsed = (
        [$strength_line]
        | parse --regex '# Strength:\s+defect=([0-2])\s+behavior=([0-2])\s+resilience=([0-2])\s+cost=([0-2])\s+uniqueness=([0-2])\s+total=([0-9]+)/10'
        | get -o 0
    )

    if $parsed == null {
        error make { msg: $"Could not parse structured '# Strength:' marker near: ($relative_path) :: ($test_name) :: ($strength_line)" }
    }

    let defect = ($parsed.capture0 | into int)
    let behavior = ($parsed.capture1 | into int)
    let resilience = ($parsed.capture2 | into int)
    let cost = ($parsed.capture3 | into int)
    let uniqueness = ($parsed.capture4 | into int)
    let total = ($parsed.capture5 | into int)
    let computed = ($defect + $behavior + $resilience + $cost + $uniqueness)

    if $computed != $total {
        error make { msg: $"Structured '# Strength:' marker total does not match component sum near: ($relative_path) :: ($test_name) :: expected=($computed)/10 declared=($total)/10" }
    }

    $total
}

export def main [] {
    let entrypoints = (load_default_suite_entrypoints)
    let component_files = (load_default_suite_component_files)
    let defended_by_lines = (load_spec_defended_by_lines)
    let bead_ids = (load_bead_ids)
    let contract_items = (load_contract_items)
    let quarantine_entries = (load_contract_traceability_quarantine_entries)
    mut errors = []
    mut warnings = []

    for test_path in (load_all_test_file_paths) {
        let relative_path = ($test_path | path relative-to $REPO_ROOT)
        let lane = (parse_test_lane $relative_path)

        if $lane == null {
            $errors = ($errors | append $"Missing '# Test lane:' declaration in: ($relative_path)")
        } else if not ($ALLOWED_TEST_LANES | any { |allowed| $allowed == $lane }) {
            $errors = ($errors | append $"Test file declares unsupported lane '($lane)': ($relative_path)")
        }

        if (($relative_path | str ends-with "_extended.nu") and not (has_grandfathered_extended_filename_marker $relative_path)) {
            $errors = ($errors | append $"Generic '_extended' test filenames are no longer allowed without an explicit grandfather marker: ($relative_path)")
        }
    }

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

        if not (has_non_policy_spec_reference $dev_relative_path) {
            let quarantine_entry = (find_traceability_quarantine_entry $quarantine_entries "default_suite_component_file" $dev_relative_path)

            if $quarantine_entry == null {
                $errors = ($errors | append $"Default-suite component file cannot rely only on governance-level file traceability without a quarantine entry: ($dev_relative_path)")
            } else if not (($quarantine_entry.bead? | default "") in $bead_ids) {
                $errors = ($errors | append $"Traceability quarantine entry points at a missing bead `($quarantine_entry.bead? | default "")`: ($dev_relative_path)")
            } else {
                $warnings = ($warnings | append $"Quarantined file-level traceability debt: ($dev_relative_path) -> ($quarantine_entry.bead)")
            }
        }

        let lane = (parse_test_lane $dev_relative_path)
        if $lane != "default" {
            $errors = ($errors | append $"Default-suite component file must declare '# Test lane: default': ($dev_relative_path)")
        }

        let canonical_tests = (load_canonical_test_names $dev_relative_path)
        let defined_tests = (load_defined_test_names $dev_relative_path)
        let dead_tests = (
            $defined_tests
            | where { |name| not ($canonical_tests | any { |canonical| $canonical == $name }) }
        )

        for dead_test in $dead_tests {
            $errors = ($errors | append $"Default-suite component file contains a noncanonical dead test: ($dev_relative_path) :: ($dead_test)")
        }

        for canonical_test in $canonical_tests {
            if not (has_valid_definition_test_justification $dev_relative_path $canonical_test) {
                $errors = ($errors | append $"Default-suite canonical test is missing a nearby '# Defends:', '# Regression:', or '# Invariant:' marker at the test definition: ($dev_relative_path) :: ($canonical_test)")
            }

            $errors = ($errors | append (collect_definition_contract_traceability_errors $dev_relative_path $canonical_test "default" $contract_items))

            let strength = (get_definition_test_strength $dev_relative_path $canonical_test)
            let minimum_strength = ($MIN_STRENGTH_BY_LANE.default)
            if $strength < $minimum_strength {
                $errors = ($errors | append $"Default-suite canonical test is below the minimum strength bar of ($minimum_strength)/10: ($dev_relative_path) :: ($canonical_test) :: ($strength)/10")
            }
        }
    }

    let default_component_paths = ($component_files | each { |file| to_dev_relative_path $file })

    for test_path in (load_all_test_file_paths) {
        let relative_path = ($test_path | path relative-to $REPO_ROOT)
        let lane = (parse_test_lane $relative_path)

        if ($lane == null) or ($relative_path in $default_component_paths) {
            continue
        }

        let minimum_strength = ($MIN_STRENGTH_BY_LANE | get -o $lane)
        if $minimum_strength == null {
            continue
        }

        for test_name in (load_defined_test_names $relative_path) {
            if not (has_valid_definition_test_justification $relative_path $test_name) {
                $errors = ($errors | append $"Governed test is missing a nearby '# Defends:', '# Regression:', or '# Invariant:' marker: ($relative_path) :: ($test_name)")
            }

            $errors = ($errors | append (collect_definition_contract_traceability_errors $relative_path $test_name $lane $contract_items))

            let strength = (get_definition_test_strength $relative_path $test_name)
            if $strength < $minimum_strength {
                $errors = ($errors | append $"Governed test is below the minimum strength bar of ($minimum_strength)/10 for lane '($lane)': ($relative_path) :: ($test_name) :: ($strength)/10")
            }
        }
    }

    if not ($warnings | is-empty) {
        $warnings | each { |line| print $"⚠️ ($line)" }
    }

    if not ($errors | is-empty) {
        $errors | each { |line| print $"❌ ($line)" }
        error make { msg: "Governed test traceability validation failed" }
    }
}
