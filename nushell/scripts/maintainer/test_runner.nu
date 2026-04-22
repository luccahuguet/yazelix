#!/usr/bin/env nu
# Yazelix maintainer test runner

use ../utils/common.nu [get_yazelix_runtime_dir]

const TEST_RUNNER_MODULE_PATH = (path self)
const TEST_RUNNER_REPO_ROOT = (
    $TEST_RUNNER_MODULE_PATH
    | path dirname
    | path join ".." ".." ".."
    | path expand
)
const TEST_SUITE_INVENTORY_PATH = (
    $TEST_RUNNER_MODULE_PATH
    | path dirname
    | path join "test_suite_inventory.toml"
    | path expand
)

def get_test_runner_repo_root [] {
    let cwd = (pwd | path expand)
    let cwd_inventory = ($cwd | path join "nushell" "scripts" "maintainer" "test_suite_inventory.toml")
    if ($cwd_inventory | path exists) {
        $cwd
    } else {
        $TEST_RUNNER_REPO_ROOT
    }
}

def load_test_suite_inventory []: nothing -> record {
    open $TEST_SUITE_INVENTORY_PATH
}

export def get_default_test_file_names [] {
    let inventory = (load_test_suite_inventory)
    $inventory.default.transitional_nu_entrypoints? | default []
}

def get_default_nextest_suites [] {
    let inventory = (load_test_suite_inventory)
    $inventory.default.nextest_suites? | default []
}

def get_default_cargo_test_exceptions [] {
    let inventory = (load_test_suite_inventory)
    $inventory.default.default_cargo_test_exceptions? | default []
}

def build_nix_develop_cargo_args [cargo_args: list<string>] {
    ["develop", "-c", "cargo"] | append $cargo_args
}

def test_profiling_enabled [] {
    let raw_value = ($env.YAZELIX_TEST_PROFILE? | default "false" | into string | str downcase | str trim)
    $raw_value in ["1", "true", "yes", "on"]
}

def render_profile_summary [records: list<record>, title: string] {
    let sorted = ($records | sort-by elapsed_ms --reverse)
    let lines = (
        $sorted
        | each {|record|
            let seconds = (($record.elapsed_ms | into float) / 1000.0 | into string | str substring 0..4)
            $"  - ($record.suite): ($seconds)s"
        }
    )

    [
        $title
        ...$lines
    ] | str join "\n"
}

def summarize_failure_output [stdout: string, stderr: string] {
    let stdout_tail = (
        $stdout
        | lines
        | last 40
        | str join "\n"
        | str trim
    )
    let stderr_tail = (
        $stderr
        | lines
        | last 40
        | str join "\n"
        | str trim
    )

    mut sections = []
    if not ($stdout_tail | is-empty) {
        $sections = ($sections | append $"Stdout tail:\n($stdout_tail)")
    }
    if not ($stderr_tail | is-empty) {
        $sections = ($sections | append $"Stderr tail:\n($stderr_tail)")
    }

    $sections | str join "\n"
}

def run_external_in_repo_root [repo_root: string, program: string, args: list<string>, extra_env: record = {}] {
    if ($extra_env | is-empty) {
        do {
            cd $repo_root
            ^$program ...$args
        } | complete
    } else {
        with-env $extra_env {
            do {
                cd $repo_root
                ^$program ...$args
            } | complete
        }
    }
}

def append_command_output_to_log [log_file: string, suite_name: string, output: record] {
    $"Suite: ($suite_name)\nExit code: ($output.exit_code)\nStdout:\n($output.stdout)\n" | save --append $log_file
    if not ($output.stderr | is-empty) {
        $"Stderr:\n($output.stderr)\n" | save --append $log_file
    }
    $"---\n" | save --append $log_file
}

def run_logged_suite [
    suite_name: string
    display_name: string
    repo_root: string
    program: string
    args: list<string>
    log_file: string
    verbose: bool
    extra_env: record = {}
] {
    let started = (date now)

    if $verbose {
        print $"📋 Running: ($display_name)"
        print "─────────────────────────────────────"
        print $"Running: ($program) (($args | str join ' '))"
    } else {
        print $"  Running ($display_name)..."
    }

    let output = (run_external_in_repo_root $repo_root $program $args $extra_env)

    if $verbose {
        if ($output.stdout | is-not-empty) {
            print --raw $output.stdout
        }
        if ($output.stderr | is-not-empty) {
            print --stderr --raw $output.stderr
        }
        print ""
    }

    append_command_output_to_log $log_file $suite_name $output

    let result = if $output.exit_code == 0 {
        {status: "✅ PASS", suite: $suite_name, error: null}
    } else {
        let failure_details = (summarize_failure_output $output.stdout $output.stderr)
        {status: "❌ FAIL", suite: $suite_name, error: $"Exit code: ($output.exit_code)\n($failure_details)"}
    }

    let elapsed_ms = (((date now) - $started) / 1ms)
    $result | upsert elapsed_ms $elapsed_ms
}

def run_syntax_validation [
    verbose: bool
    log_file: string
    repo_root: string
] {
    print "🔍 Phase 1: Syntax Validation"
    print "─────────────────────────────────────"
    "=== Syntax Validation ===\n" | save --append $log_file

    let validate_script = ($repo_root | path join "nushell" "scripts" "dev" "validate_syntax.nu")
    let validate_args = if $verbose {
        [$validate_script, "--verbose"]
    } else {
        [$validate_script, "--quiet"]
    }
    let result = (run_external_in_repo_root $repo_root "nu" $validate_args)

    if $result.exit_code == 0 {
        print "✅ All scripts passed syntax validation"
        "✅ Syntax validation passed\n\n" | save --append $log_file
        true
    } else {
        print "❌ Syntax validation failed"
        if not ($result.stderr | is-empty) {
            print --stderr --raw $result.stderr
        }
        if not ($result.stdout | is-empty) {
            print --raw $result.stdout
        }
        $"❌ Syntax validation failed\n($result.stdout)\n($result.stderr)\n\n" | save --append $log_file
        false
    }
}

def run_default_functional_suites [
    repo_root: string
    log_file: string
    verbose: bool
    profiling: bool
] {
    print ""
    print "🧪 Phase 2: Functional Tests"
    print "─────────────────────────────────────"
    "=== Functional Tests ===\n" | save --append $log_file

    let nextest_results = (
        get_default_nextest_suites
        | each {|suite|
            run_logged_suite $suite.name $"Rust nextest: ($suite.name)" $repo_root "nix" (build_nix_develop_cargo_args (
                ["nextest", "run", "--profile", "ci", "--manifest-path", ($repo_root | path join $suite.manifest_path)]
                | append ($suite.args? | default [])
            )) $log_file $verbose
        }
    )

    let cargo_test_results = (
        get_default_cargo_test_exceptions
        | each {|suite|
            run_logged_suite $suite.name $"Rust cargo test exception: ($suite.name)" $repo_root "nix" (build_nix_develop_cargo_args (
                ["test", "--manifest-path", ($repo_root | path join $suite.manifest_path)]
                | append ($suite.args? | default [])
            )) $log_file $verbose
        }
    )

    let nu_env = if $profiling { {YAZELIX_TEST_PROFILE: "1"} } else { {} }
    let nu_results = (
        get_default_test_file_names
        | each {|file_name|
            let test_file = ($repo_root | path join "nushell" "scripts" "dev" $file_name)
            if not ($test_file | path exists) {
                error make { msg: $"Missing transitional Nu suite entrypoint: ($test_file)" }
            }

            run_logged_suite ($file_name | str replace ".nu" "") $"Transitional Nu suite: ($file_name)" $repo_root "nu" [$test_file] $log_file $verbose $nu_env
        }
    )

    $nextest_results | append $cargo_test_results | append $nu_results | flatten
}

def render_suite_summary [results: list<record>, log_file: string, profiling: bool] {
    print ""
    print "=== Test Results Summary ==="

    let passed = ($results | where status == "✅ PASS" | length)
    let failed = ($results | where status == "❌ FAIL" | length)
    let total = ($results | length)

    $results | each { |result|
        print $"($result.status) ($result.suite)"
        if ($result.status == "❌ FAIL") and (not ($result.error | is-empty)) {
            print $"   Error: ($result.error)"
        }
    }

    print ""
    let summary = $"Total: ($total) | Passed: ($passed) | Failed: ($failed)"
    print $summary

    $"\n=== Test Results Summary ===\n" | save --append $log_file
    $results | each { |result|
        $"($result.status) ($result.suite)\n" | save --append $log_file
        if ($result.status == "❌ FAIL") and (not ($result.error | is-empty)) {
            $"   Error: ($result.error)\n" | save --append $log_file
        }
    }
    $"\n($summary)\n" | save --append $log_file

    if $profiling {
        print ""
        let profile_report = (render_profile_summary $results "=== Default Suite Profile ===")
        print $profile_report
        $"($profile_report)\n" | save --append $log_file
    }

    if $failed > 0 {
        print ""
        print "❌ Some tests failed"
        $"\n❌ Some tests failed\n" | save --append $log_file
        print $"📝 Full log: ($log_file)"
        print ""
        error make { msg: "Test suite failed" }
    }

    print ""
    print "✅ All tests passed!"
    $"\n✅ All tests passed!\n" | save --append $log_file
    print $"📝 Full log: ($log_file)"
    print ""
}

def run_nonvisual_sweep_tests [verbose: bool] {
    print ""
    print "=== Running Non-Visual Configuration Sweep Tests ==="
    print ""

    let runtime_root = (get_yazelix_runtime_dir)
    let repo_root = (get_test_runner_repo_root)
    let sweep_script = ($runtime_root | path join "nushell" "scripts" "dev" "config_sweep_runner.nu")
    let args = if $verbose {
        [$sweep_script, "--verbose"]
    } else {
        [$sweep_script]
    }

    let output = (run_external_in_repo_root $repo_root "nu" $args)
    if ($output.stdout | is-not-empty) {
        print --raw $output.stdout
    }
    if ($output.stderr | is-not-empty) {
        print --stderr --raw $output.stderr
    }
    if $output.exit_code != 0 {
        error make { msg: "Non-visual sweep tests failed" }
    }
}

def run_visual_sweep_tests [verbose: bool, delay: int] {
    print ""
    print "=== Running Visual Terminal Sweep Tests ==="
    print ""

    let runtime_root = (get_yazelix_runtime_dir)
    let repo_root = (get_test_runner_repo_root)
    let sweep_script = ($runtime_root | path join "nushell" "scripts" "dev" "config_sweep_runner.nu")
    let args = if $verbose {
        [$sweep_script, "--visual", "--visual-delay", ($delay | into string), "--verbose"]
    } else {
        [$sweep_script, "--visual", "--visual-delay", ($delay | into string)]
    }

    let output = (run_external_in_repo_root $repo_root "nu" $args)
    if ($output.stdout | is-not-empty) {
        print --raw $output.stdout
    }
    if ($output.stderr | is-not-empty) {
        print --stderr --raw $output.stderr
    }
    if $output.exit_code != 0 {
        error make { msg: "Visual sweep tests failed" }
    }
}

export def run_all_tests [
    --verbose(-v)
    --new-window(-n)
    --lint-only
    --profile
    --sweep
    --visual
    --all(-a)
    --delay: int = 3
] {
    let profiling = ($profile or (test_profiling_enabled))
    let visual_delay = ($delay | default 3)
    let run_only_sweep = ($sweep and not $visual and not $all)
    let run_only_visual = ($visual and not $sweep and not $all)
    let run_only_both_sweeps = ($sweep and $visual and not $all)
    let repo_root = (get_test_runner_repo_root)

    if $new_window {
        print "🚀 Launching new Yazelix window for testing..."
        print ""

        mut test_args = ["yzx", "dev", "test"]
        if $verbose { $test_args = ($test_args | append "--verbose") }
        if $lint_only { $test_args = ($test_args | append "--lint-only") }
        if $profile { $test_args = ($test_args | append "--profile") }
        if $sweep { $test_args = ($test_args | append "--sweep") }
        if $visual { $test_args = ($test_args | append "--visual") }
        if $all { $test_args = ($test_args | append "--all") }
        if $visual or $all {
            $test_args = ($test_args | append ["--delay", ($visual_delay | into string)])
        }
        let test_cmd = ($test_args | str join " ")
        let logs_dir = ($repo_root | path join "logs")

        print $"💡 In the new window, run: ($test_cmd)"
        print $"📝 Test logs will be saved to: ($logs_dir)"
        print ""

        with-env {YAZELIX_SHELLHOOK_SKIP_WELCOME: "true"} {
            nu ($repo_root | path join "nushell" "scripts" "core" "launch_yazelix.nu")
        }

        return
    }

    let log_dir = ($repo_root | path join "logs")
    mkdir $log_dir
    let timestamp = (date now | into int)
    let log_file = $"($log_dir)/test_run_($timestamp).log"
    let header = $"=== Yazelix Test Run ===\nDate: (date now)\nVerbose: ($verbose)\n\n"
    $header | save $log_file

    if $lint_only {
        let syntax_passed = (run_syntax_validation $verbose $log_file $repo_root)
        print $"📝 Full log: ($log_file)"
        if not $syntax_passed {
            error make { msg: "Syntax validation failed" }
        }
        return
    }

    if $run_only_visual {
        run_visual_sweep_tests $verbose $visual_delay
        return
    }

    if $run_only_sweep {
        run_nonvisual_sweep_tests $verbose
        return
    }

    if $run_only_both_sweeps {
        run_nonvisual_sweep_tests $verbose
        run_visual_sweep_tests $verbose $visual_delay
        return
    }

    print "=== Yazelix Default Test Suite ==="
    print "Running fixed Rust nextest suites plus the transitional default Nu suite..."
    print $"📝 Logging to: ($log_file)"
    print ""
    $"=== Yazelix Default Test Suite ===\nRunning fixed Rust nextest suites plus the transitional default Nu suite...\n\n" | save --append $log_file

    let syntax_passed = (run_syntax_validation $verbose $log_file $repo_root)
    if not $syntax_passed {
        print ""
        print "❌ Test suite aborted due to syntax errors"
        print "   Fix syntax errors and try again"
        print $"📝 Full log: ($log_file)"
        error make { msg: "Syntax validation failed" }
    }

    let results = (run_default_functional_suites $repo_root $log_file $verbose $profiling)
    render_suite_summary $results $log_file $profiling

    if $sweep or $all {
        run_nonvisual_sweep_tests $verbose
    }

    if $visual or $all {
        run_visual_sweep_tests $verbose $visual_delay
    }
}
