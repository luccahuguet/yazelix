#!/usr/bin/env nu
# Configuration Sweep Testing Framework (Refactored)
# Tests shell/terminal combinations and configuration variations

use ../utils/config_parser.nu parse_yazelix_config
use sweep/sweep_config_generator.nu *
use sweep/sweep_process_manager.nu *
use sweep/sweep_test_executor.nu *
use sweep/sweep_test_combinations.nu *

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
    $results | where ($it | get $field) == $status_value | length
}

# Helper: Get status value from a result
def get_result_status [result: record, visual: bool]: nothing -> string {
    let field = get_status_field $visual
    $result | get $field
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

        # Launch Yazelix with the test config and test_id (terminal from config's preferred_terminal)
        let launch_result = launch_visual_test $config_path $test_id

        if $launch_result.exit_code != 0 {
            print $"❌ Failed to launch ($shell) + ($terminal): ($launch_result.stderr)"
            create_test_result $test_id $shell $terminal "fail" "Launch failed" $launch_result.stderr
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
                    parse_yazelix_config
                }
            })

            if ($parsed.default_shell == $shell) and ($parsed.preferred_terminal == $terminal) {
                {status: "pass", message: "Config parsing successful"}
            } else {
                {status: "fail", message: "Config parsing mismatch"}
            }
        } catch { |err|
            {status: "error", message: $"Config parsing failed: ($err.msg)"}
        }

        if $config_test.status != "pass" {
            return (create_env_test_result $test_id $shell $terminal $features $config_test.status $config_test.message "skipped" "Skipped due to config failure" "fail")
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

        create_env_test_result $test_id $shell $terminal $features $config_test.status $config_test.message $env_result.status $env_result.message $overall_status
    } catch { |err|
        create_env_test_result $test_id $shell $terminal $features "error" $"Test failed: ($err.msg)" "error" "Test execution error" "error"
    }

    cleanup_test_config $config_path
    $result
}

# Main sweep test runner
export def run_all_sweep_tests [
    --verbose(-v)           # Show detailed output
    --visual(-w)            # Launch visual Yazelix windows for each test
    --visual-delay: int     # Delay between visual launches in seconds (default: 3)
]: nothing -> nothing {
    let visual_delay = (($visual_delay | default 3) * 1sec)

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
            print $"  Progress: ($completed)/($total_tests) - ($status | str upcase) ($combo.shell)+($combo.terminal)"
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
        0  # Visual tests don't have skip status
    } else {
        ($results | where env_status == "skip" | length)
    }

    # Show detailed results
    for $result in $results {
        let status = get_result_status $result $visual
        let status_icon = match $status {
            "pass" => "✅",
            "fail" => "❌",
            "error" => "💥"
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
                print $"   Environment: ($result.env_status) - ($result.env_message)"
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