#!/usr/bin/env nu
# Configuration Sweep Testing Framework (Refactored)
# Tests shell/terminal combinations and configuration variations
# Test lane: sweep

use ./config_normalize_test_helpers.nu [load_normalized_active_config]
use sweep/sweep_config_generator.nu *
use sweep/sweep_process_manager.nu *
use sweep/sweep_test_executor.nu *
use sweep/sweep_test_combinations.nu *

def append_progress [line: string]: nothing -> nothing {
    let progress_file = ($env.YAZELIX_SWEEP_PROGRESS_FILE? | default "")
    if ($progress_file | is-not-empty) {
        $"($line)\n" | save --append $progress_file
    }
}

# Helper: Get the status field name based on test mode
def get_status_field [visual: bool]: nothing -> string {
    if $visual { "status" } else { "overall" }
}

# Helper: Count results by status value
def count_results_by_status [
    results: list,
    visual: bool,
    status_value: string
]: nothing -> int {
    let field = get_status_field $visual
    $results | where {|result| (($result | get -o $field) == $status_value) } | length
}

# Helper: Get status value from a result
def get_result_status [result: record, visual: bool]: nothing -> string {
    let field = get_status_field $visual
    $result | get -o $field
}

def get_launch_error_details [stdout: string, stderr: string]: nothing -> string {
    let combined = [$stderr, $stdout] | where ($it | is-not-empty) | str join "\n"
    if ($combined | is-empty) {
        return "No output captured"
    }

    let lines = ($combined | lines)
    let lower = ($combined | str downcase)
    let missing_terminal = (
        ($lower | str contains "specified terminal")
        or ($lower | str contains "none of the supported terminals")
        or ($lower | str contains "terminal.terminals must include")
    )

    if $missing_terminal {
        let hint = "Missing terminal. Install one of the configured host terminals, or update [terminal].terminals and rerun yzx launch."
        let matched_line = (
            $lines
            | where {|line|
                let lower_line = ($line | str downcase)
                let matches = [
                    ($lower_line | str contains "specified terminal")
                    ($lower_line | str contains "none of the supported terminals")
                    ($lower_line | str contains "terminal.terminals must include")
                ]
                $matches | any {|match| $match }
            }
            | first
        )
        if ($matched_line | is-empty) {
            $hint
        } else {
            $"($hint) ($matched_line)"
        }
    } else {
        $lines | first
    }
}

# Run a visual sweep test by launching actual Yazelix
export def run_visual_sweep_test [
    shell: string,
    terminal: string,
    features: record,
    test_id: string,
    delay: duration = 3sec
]: nothing -> record {
    print $"🖥️  Launching visual test: ($shell) + ($terminal) \(($test_id)\)"

    let config_path = generate_sweep_config $shell $terminal $features $test_id
    let session_name = $"sweep_test_($test_id)"

    let result = try {
        # Get terminal process baseline before launch
        let before_pids = get_terminal_pids $terminal

        # Launch Yazelix with the test config and test_id (pass terminal explicitly)
        let launch_result = launch_visual_test $config_path $test_id $terminal

        if $launch_result.exit_code == 99 {
            print $"⏭️  Skipped ($shell) + ($terminal) - terminal not installed"
            create_test_result $test_id $shell $terminal "skip" "Terminal not installed"
        } else if $launch_result.exit_code != 0 {
            let details = get_launch_error_details $launch_result.stdout $launch_result.stderr
            print $"❌ Failed to launch ($shell) + ($terminal)"
            create_test_result $test_id $shell $terminal "fail" "Launch failed" $details
        } else {
            print $"✅ Launched ($shell) + ($terminal) successfully"

            # Wait for verification from sweep_verify.nu script in launched session
            let demo_result = run_demo_command $test_id

            # Clean up after demo period
            cleanup_visual_test $session_name $terminal $before_pids $delay

            # Mark test as pass only if both launch and verification succeeded
            if $demo_result.verified {
                create_test_result $test_id $shell $terminal "pass" "Visual launch and verification successful"
            } else {
                create_test_result $test_id $shell $terminal "fail" $"Launch succeeded but verification failed: ($demo_result.status)" $demo_result.output
            }
        }
    } catch { |err|
        print $"💥 Error launching ($shell) + ($terminal): ($err.msg)"
        create_test_result $test_id $shell $terminal "error" $"Launch error: ($err.msg)"
    }

    cleanup_test_config $config_path
    $result
}

# Run a single sweep test (non-visual)
def run_sweep_test [
    shell: string,
    terminal: string,
    features: record,
    test_id: string,
    verbose: bool = false
]: nothing -> record {
    if $verbose {
        print $"🧪 Testing: ($shell) + ($terminal) \(($test_id)\)"
    }

    let config_path = generate_sweep_config $shell $terminal $features $test_id

    let result = try {
        # Validate configuration parsing
        let config_test = try {
            let parsed = (do {
                with-env {YAZELIX_CONFIG_OVERRIDE: $config_path} {
                    load_normalized_active_config
                }
            })

            let parsed_terminal = ($parsed.terminals? | default [] | first)
            if ($parsed.default_shell == $shell) and ($parsed_terminal == $terminal) {
                {status: "pass", message: "Config parsing successful"}
            } else {
                {status: "fail", message: "Config parsing mismatch"}
            }
        } catch { |err|
            {status: "error", message: $"Config parsing failed: ($err.msg)", details: $err.msg}
        }

        if $config_test.status != "pass" {
            return (create_env_test_result $test_id $shell $terminal $features $config_test.status $config_test.message ($config_test.details? | default null) "skipped" "Skipped due to config failure" null "fail")
        }

        # Validate environment setup (only on Linux for foot, skip others on unsupported platforms)
        let env_result = if ($terminal == "foot") and ((uname).kernel-name != "Linux") {
            {status: "skip", message: "Foot only supported on Linux", details: null}
        } else {
            validate_environment $config_path
        }

        let overall_status = if ($config_test.status == "pass") and ($env_result.status in ["pass", "skip"]) {
            "pass"
        } else {
            "fail"
        }

        create_env_test_result $test_id $shell $terminal $features $config_test.status $config_test.message ($config_test.details? | default null) $env_result.status $env_result.message ($env_result.details? | default null) $overall_status
    } catch { |err|
        create_env_test_result $test_id $shell $terminal $features "error" $"Test failed: ($err.msg)" $err.msg "error" "Test execution error" null "error"
    }

    cleanup_test_config $config_path
    $result
}

# Main sweep test runner
export def run_all_sweep_tests [
    --verbose(-v)           # Show detailed output
    --visual(-w)            # Launch visual Yazelix windows for each test
    --visual-delay: int = 3 # Delay between visual launches in seconds
]: nothing -> nothing {
    let visual_delay_seconds = $visual_delay
    let visual_delay = ($visual_delay_seconds * 1sec)

    if $visual {
        print "=== Visual Configuration Sweep Testing ==="
        print "🖥️  Each configuration will launch in a new window"
        print $"⏱️  Delay between launches: ($visual_delay)"
    } else {
        print "=== Configuration Sweep Testing ==="
    }
    print ""

    # Generate test combinations based on mode
    # Visual mode: tests terminal launches (cross-terminal tests)
    # Non-visual mode: tests environment/shell setup (cross-shell, feature tests)
    let combinations = if $visual {
        generate_visual_test_combinations
    } else {
        generate_test_combinations
    }

    print $"Running ($combinations | length) sweep test combinations..."
    print ""

    # Clean up any existing test configs
    cleanup_sweep_configs

    # Run tests
    mut results = []
    let total_tests = ($combinations | length)

    for $combo in $combinations {
        let test_id = $"($combo.type)_($combo.shell)_($combo.terminal)"
        let completed = ($results | length)

        if not $verbose and not $visual {
            let start_line = $"  Starting (($completed + 1))/($total_tests): ($combo.shell)+($combo.terminal)"
            print $start_line
            append_progress $start_line
        }

        let result = if $visual {
            run_visual_sweep_test $combo.shell $combo.terminal $combo.features $test_id $visual_delay
        } else {
            run_sweep_test $combo.shell $combo.terminal $combo.features $test_id $verbose
        }
        $results = ($results | append $result)

        # Progress indicator
        if not $verbose and not $visual {
            let completed = ($results | length)
            let status = get_result_status $result $visual
            let progress_line = $"  Progress: ($completed)/($total_tests) - ($status | str upcase) ($combo.shell)+($combo.terminal)"
            print $progress_line
            append_progress $progress_line
        }
    }

    # Generate summary report
    print ""
    print "=== Sweep Test Results ==="

    # Handle both visual and regular test result formats
    let passed = count_results_by_status $results $visual "pass"
    let failed = count_results_by_status $results $visual "fail"
    let errors = count_results_by_status $results $visual "error"
    let skipped = if $visual {
        count_results_by_status $results $visual "skip"
    } else {
        ($results | where env_status == "skip" | length)
    }

    # Show detailed results
    for $result in $results {
        let status = get_result_status $result $visual
        let status_icon = match $status {
            "pass" => "✅",
            "fail" => "❌",
            "error" => "💥",
            "skip" => "⏭️"
        }

        if $visual {
            let test_name = ($result | get test_id? | default "unknown")
            let shell_name = ($result | get shell? | default "unknown")
            let terminal_name = ($result | get terminal? | default "unknown")
            print $"($status_icon) ($test_name): ($shell_name) + ($terminal_name)"
            if $verbose or ($status != "pass") {
                print $"   Message: ($result.message)"
                if ($result.details | is-not-empty) {
                    print $"   Details: ($result.details)"
                }
            }
        } else {
            print $"($status_icon) ($result.test_id): ($result.shell) + ($result.terminal)"
            if $verbose or ($status != "pass") {
                print $"   Config: ($result.config_status) - ($result.config_message)"
                let config_details = ($result.config_details? | default null)
                if ($config_details != null) and (($config_details | into string | str trim) | is-not-empty) {
                    print $"   Config details: ($config_details)"
                }
                print $"   Environment: ($result.env_status) - ($result.env_message)"
                let env_details = ($result.env_details? | default null)
                if ($env_details != null) and (($env_details | into string | str trim) | is-not-empty) {
                    print $"   Environment details: ($env_details)"
                }
                if ($status != "pass") {
                    print ""
                }
            }
        }
    }

    print ""
    print $"Summary: ($passed) passed, ($failed) failed, ($errors) errors, ($skipped) skipped"

    # Clean up
    cleanup_sweep_configs

    if ($failed + $errors) > 0 {
        print ""
        print "❌ Some sweep tests failed"
        error make { msg: "Sweep test failures detected" }
    } else {
        print ""
        print "✅ All sweep tests passed!"
    }
}

def main [] {
    run_all_sweep_tests
}
