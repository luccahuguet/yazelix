#!/usr/bin/env nu
# Yazelix Test Runner
# Runs all tests in the dev/ directory and reports results

use ./common.nu [get_yazelix_dir]

# Run syntax validation before tests
def run_syntax_validation [
    verbose: bool
    log_file: string
] {
    print "🔍 Phase 1: Syntax Validation"
    print "─────────────────────────────────────"

    let syntax_log = "=== Syntax Validation ===\n"
    $syntax_log | save --append $log_file

    # Run validate_syntax.nu, mirroring the caller's requested verbosity.
    let validate_script = ((get_yazelix_dir) | path join "nushell" "scripts" "dev" "validate_syntax.nu")
    let result = if $verbose {
        do {
            nu $validate_script --verbose
        } | complete
    } else {
        do {
            nu $validate_script --quiet
        } | complete
    }

    if $result.exit_code == 0 {
        print "✅ All scripts passed syntax validation"
        "✅ Syntax validation passed\n\n" | save --append $log_file
        true
    } else {
        print "❌ Syntax validation failed"
        if not ($result.stderr | is-empty) {
            print $result.stderr
        }
        if not ($result.stdout | is-empty) {
            print $result.stdout
        }
        $"❌ Syntax validation failed\n($result.stdout)\n($result.stderr)\n\n" | save --append $log_file
        false
    }
}

def run_standard_test [test_file: string] {
    if (test_profiling_enabled) {
        with-env {YAZELIX_TEST_PROFILE: "1"} {
            do { nu $test_file } | complete
        }
    } else {
        do { nu $test_file } | complete
    }
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
            $"  - ($record.test): ($seconds)s"
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

export def get_default_test_file_names [] {
    [
        "test_yzx_commands.nu"
    ]
}

def resolve_suite_test_files [test_dir: string, file_names: list<string>] {
    $file_names
    | each {|name|
        let path = ($test_dir | path join $name)
        if not ($path | path exists) {
            error make { msg: $"Missing test file declared in suite: ($path)" }
        }
        $path
    }
}

# Run all tests and report results
export def run_all_tests [
    --verbose(-v)  # Show detailed output
    --new-window(-n)  # Run tests in a new Yazelix window
    --lint-only  # Run only syntax validation
    --profile  # Print timing summaries for the default suite
    --sweep  # Run the non-visual configuration sweep only
    --visual  # Run the visual terminal sweep only
    --all(-a)  # Run the default suite plus sweep + visual lanes
    --delay: int = 3  # Delay between visual terminal launches in seconds
] {
    let profiling = ($profile or (test_profiling_enabled))
    let visual_delay = ($delay | default 3)
    let run_only_sweep = ($sweep and not $visual and not $all)
    let run_only_visual = ($visual and not $sweep and not $all)
    let run_only_both_sweeps = ($sweep and $visual and not $all)

    # If --new-window flag is set, launch tests in a new Yazelix instance
    if $new_window {
        print "🚀 Launching new Yazelix window for testing..."
        print ""

        # Build the command to run in the new window
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
        let logs_dir = ((get_yazelix_dir) | path join "logs")

        # Launch Yazelix with skip welcome screen
        print $"💡 In the new window, run: ($test_cmd)"
        print $"📝 Test logs will be saved to: ($logs_dir)"
        print ""

        with-env {YAZELIX_SHELLHOOK_SKIP_WELCOME: "true"} {
            nu ((get_yazelix_dir) | path join "nushell" "scripts" "core" "launch_yazelix.nu")
        }

        return
    }

    if $lint_only {
        let log_dir = ((get_yazelix_dir) | path join "logs")
        mkdir $log_dir
        let timestamp = (date now | into int)
        let log_file = $"($log_dir)/test_run_($timestamp).log"
        let header = $"=== Yazelix Test Run ===\nDate: (date now)\nVerbose: ($verbose)\nMode: lint-only\n\n"
        $header | save $log_file

        let syntax_passed = run_syntax_validation $verbose $log_file
        if $syntax_passed {
            print $"📝 Full log: ($log_file)"
        } else {
            print $"📝 Full log: ($log_file)"
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

    let test_dir = ((get_yazelix_dir) | path join "nushell" "scripts" "dev")
    let log_dir = ((get_yazelix_dir) | path join "logs")

    # Create log directory if it doesn't exist
    mkdir $log_dir

    # Create timestamped log file
    let timestamp = (date now | into int)
    let log_file = $"($log_dir)/test_run_($timestamp).log"

    # Log header
    let header = $"=== Yazelix Test Run ===\nDate: (date now)\nVerbose: ($verbose)\n\n"
    $header | save $log_file

    let test_files = (resolve_suite_test_files $test_dir (get_default_test_file_names))

    if ($test_files | is-empty) {
        print "❌ No test files found for the selected suite"
        return
    }

    let msg_header = "=== Yazelix Default Test Suite ==="
    let msg_count = $"Running ($test_files | length) test file\(s\)..."

    print $msg_header
    print $msg_count
    print $"📝 Logging to: ($log_file)"
    print ""

    # Log to file
    $"($msg_header)\n($msg_count)\n\n" | save --append $log_file

    # Run syntax validation first
    let syntax_passed = run_syntax_validation $verbose $log_file
    if not $syntax_passed {
        print ""
        print "❌ Test suite aborted due to syntax errors"
        print "   Fix syntax errors and try again"
        print $"📝 Full log: ($log_file)"
        error make { msg: "Syntax validation failed" }
    }

    print ""
    print "🧪 Phase 2: Functional Tests"
    print "─────────────────────────────────────"
    "=== Functional Tests ===\n" | save --append $log_file

    let results = $test_files | each { |test_file|
        let test_name = ($test_file | path basename | str replace ".nu" "")
        let started = (date now)

        if $verbose {
            print $"📋 Running: ($test_name)"
            print "─────────────────────────────────────"
        } else {
            print $"  Running ($test_name)..."
        }

        # Run the test and capture result
        let result = try {
            if $verbose {
                print $"Running: nu ($test_file)"
                $"Running: nu ($test_file)\n" | save --append $log_file

                let output = (run_standard_test $test_file)
                print $output.stdout

                # Save to log
                $"($output.stdout)\n" | save --append $log_file
                if $output.exit_code != 0 {
                    $"STDERR: ($output.stderr)\n" | save --append $log_file
                }

                if $output.exit_code == 0 {
                    {status: "✅ PASS", test: $test_name, error: null}
                } else {
                    let failure_details = (summarize_failure_output $output.stdout $output.stderr)
                    {status: "❌ FAIL", test: $test_name, error: $"Exit code: ($output.exit_code)\n($failure_details)"}
                }
            } else {
                let output = (run_standard_test $test_file)

                # Log output
                $"Test: ($test_name)\nExit code: ($output.exit_code)\nStdout:\n($output.stdout)\n" | save --append $log_file
                if not ($output.stderr | is-empty) {
                    $"Stderr:\n($output.stderr)\n" | save --append $log_file
                }
                $"---\n" | save --append $log_file

                if $output.exit_code == 0 {
                    {status: "✅ PASS", test: $test_name, error: null}
                } else {
                    let failure_details = (summarize_failure_output $output.stdout $output.stderr)
                    {status: "❌ FAIL", test: $test_name, error: $"Exit code: ($output.exit_code)\n($failure_details)"}
                }
            }
        } catch { |err|
            let error_msg = $"EXCEPTION: ($err.msg)"
            $"($error_msg)\n" | save --append $log_file
            {status: "❌ FAIL", test: $test_name, error: $"($err.msg)"}
        }

        if $verbose {
            print ""
        }

        let elapsed_ms = (((date now) - $started) / 1ms)
        $result | upsert elapsed_ms $elapsed_ms
    }

    # Summary
    print ""
    print "=== Test Results Summary ==="

    let passed = ($results | where status == "✅ PASS" | length)
    let failed = ($results | where status == "❌ FAIL" | length)
    let total = ($results | length)

    $results | each { |r|
        if $r.status == "❌ FAIL" {
            print $"($r.status) ($r.test)"
            if not ($r.error | is-empty) {
                print $"   Error: ($r.error)"
            }
        } else {
            print $"($r.status) ($r.test)"
        }
    }

    print ""
    let summary = $"Total: ($total) | Passed: ($passed) | Failed: ($failed)"
    print $summary

    # Save summary to log
    $"\n=== Test Results Summary ===\n" | save --append $log_file
    $results | each { |r|
        $"($r.status) ($r.test)\n" | save --append $log_file
        if $r.status == "❌ FAIL" and not ($r.error | is-empty) {
            $"   Error: ($r.error)\n" | save --append $log_file
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
    } else {
        print ""
        print "✅ All tests passed!"
        $"\n✅ All tests passed!\n" | save --append $log_file
        print $"📝 Full log: ($log_file)"
        print ""

        if $sweep or $all {
            run_nonvisual_sweep_tests $verbose
        }

        if $visual or $all {
            run_visual_sweep_tests $verbose $visual_delay
        }
    }
}

def run_nonvisual_sweep_tests [verbose: bool] {
    print ""
    print "=== Running Non-Visual Configuration Sweep Tests ==="
    print ""

    let verbose_arg = if $verbose { " --verbose" } else { "" }
    let sweep_script = ((get_yazelix_dir) | path join "nushell" "scripts" "dev" "test_config_sweep.nu")
    nu -c $"use \"($sweep_script)\" run_all_sweep_tests; run_all_sweep_tests($verbose_arg)"
}

def run_visual_sweep_tests [verbose: bool, delay: int] {
    print ""
    print "=== Running Visual Terminal Sweep Tests ==="
    print ""

    let verbose_arg = if $verbose { " --verbose" } else { "" }
    let sweep_script = ((get_yazelix_dir) | path join "nushell" "scripts" "dev" "test_config_sweep.nu")
    nu -c $"use \"($sweep_script)\" run_all_sweep_tests; run_all_sweep_tests --visual --visual-delay ($delay)($verbose_arg)"
}
