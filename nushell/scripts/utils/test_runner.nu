#!/usr/bin/env nu
# Yazelix Test Runner
# Runs all tests in the dev/ directory and reports results

# Run all tests and report results
export def run_all_tests [
    --verbose(-v)  # Show detailed output
    --new-window(-n)  # Run tests in a new Yazelix window
] {

    # If --new-window flag is set, launch tests in a new Yazelix instance
    if $new_window {
        print "🚀 Launching new Yazelix window for testing..."
        print ""

        # Build the command to run in the new window
        let test_cmd = if $verbose {
            "yzx test --verbose"
        } else {
            "yzx test"
        }

        # Launch Yazelix with skip welcome screen
        print $"💡 In the new window, run: ($test_cmd)"
        print "📝 Test logs will be saved to: ~/.config/yazelix/logs/"
        print ""

        with-env {YAZELIX_SKIP_WELCOME: "true"} {
            nu ~/.config/yazelix/nushell/scripts/core/launch_yazelix.nu
        }

        return
    }
    let test_dir = $"($env.HOME)/.config/yazelix/nushell/scripts/dev"
    let log_dir = $"($env.HOME)/.config/yazelix/logs"

    # Create log directory if it doesn't exist
    mkdir $log_dir

    # Create timestamped log file
    let timestamp = (date now | format date "%Y-%m-%d_%H-%M-%S")
    let log_file = $"($log_dir)/test_run_($timestamp).log"

    # Log header
    let header = $"=== Yazelix Test Run ===\nDate: (date now)\nFilter: ($filter)\nVerbose: ($verbose)\n\n"
    $header | save $log_file

    # Find all test_*.nu files (excluding test_fonts.nu which is for manual testing)
    let test_files = try {
        glob $"($test_dir)/test_*.nu" | where $it !~ "test_fonts"
    } catch {
        []
    }

    if ($test_files | is-empty) {
        print "❌ No test files found"
        return
    }

    let msg_header = "=== Yazelix Test Suite ==="
    let msg_count = $"Running ($filtered_tests | length) test file\(s\)..."

    print $msg_header
    print $msg_count
    print $"📝 Logging to: ($log_file)"
    print ""

    # Log to file
    $"($msg_header)\n($msg_count)\n\n" | save --append $log_file

    let results = $filtered_tests | each { |test_file|
        let test_name = ($test_file | path basename | str replace ".nu" "")

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

                let output = (do { nu $test_file } | complete)
                print $output.stdout

                # Save to log
                $"($output.stdout)\n" | save --append $log_file
                if $output.exit_code != 0 {
                    $"STDERR: ($output.stderr)\n" | save --append $log_file
                }

                if $output.exit_code == 0 {
                    {status: "✅ PASS", test: $test_name, error: null}
                } else {
                    {status: "❌ FAIL", test: $test_name, error: $"Exit code: ($output.exit_code)\nStderr: ($output.stderr)"}
                }
            } else {
                let output = (do { nu $test_file } | complete)

                # Log output
                $"Test: ($test_name)\nExit code: ($output.exit_code)\nStdout:\n($output.stdout)\n" | save --append $log_file
                if not ($output.stderr | is-empty) {
                    $"Stderr:\n($output.stderr)\n" | save --append $log_file
                }
                $"---\n" | save --append $log_file

                if $output.exit_code == 0 {
                    {status: "✅ PASS", test: $test_name, error: null}
                } else {
                    {status: "❌ FAIL", test: $test_name, error: $"Exit code: ($output.exit_code)\nStderr: ($output.stderr)"}
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

        $result
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
    }
}
