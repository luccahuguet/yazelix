#!/usr/bin/env nu
# Sweep Testing - Test Execution Utilities
# Handles individual test execution and validation

use ../../utils/terminal_assets.nu get_terminal_metadata
use ../../utils/runtime_paths.nu [get_yazelix_runtime_dir]

def command_exists [cmd: string]: nothing -> bool {
    (which $cmd | length) > 0
}

# Validate that environment setup works for a given config
export def validate_environment [config_path: string]: nothing -> record {
    try {
        let runtime_dir = (get_yazelix_runtime_dir)
        let yzx_cli = ($runtime_dir | path join "shells" "posix" "yzx_cli.sh")
        let validation_helper = ($runtime_dir | path join "shells" "posix" "sweep_validate_runtime_tools.sh")
        let validation_output = (do {
            with-env {YAZELIX_CONFIG_OVERRIDE: $config_path} {
                ^sh $yzx_cli run sh $validation_helper
            }
        } | complete)

        if $validation_output.exit_code != 0 {
            return {status: "fail", message: "Environment validation failed", details: $validation_output.stderr}
        }

        let stdout = $validation_output.stdout
        if not ($stdout | str contains "TOOLS_START") or not ($stdout | str contains "TOOLS_END") {
            return {status: "fail", message: "Tool availability incomplete", details: $stdout}
        }

        if not ($stdout | str contains "VERSION_START") or not ($stdout | str contains "VERSION_END") {
            return {status: "fail", message: "Version check incomplete", details: $stdout}
        }

        # Verify expected tools are mentioned in version output (case insensitive)
        let stdout_lower = ($stdout | str downcase)
        if not ($stdout_lower | str contains "zellij") or not ($stdout_lower | str contains "yazi") or not ($stdout_lower | str contains "helix") {
            return {status: "fail", message: "Missing expected tool versions", details: $stdout}
        }

        {status: "pass", message: "All environment tests passed", details: null}
    } catch { |err|
        {status: "error", message: $"Test execution failed: ($err.msg)", details: null}
    }
}

# Wait for verification file from sweep_verify.nu script running in launched session
export def run_demo_command [
    test_id: string
]: nothing -> record {
    try {
        let result_file = $"/tmp/yazelix_sweep_result_($test_id).json"

        print $"   Waiting for verification script in session to complete..."

        # Clean up any existing result file
        if ($result_file | path exists) {
            rm $result_file
        }

        # Wait for the verification file to be created by the pane in the layout
        mut attempts = 0
        let max_attempts = 20  # 20 * 500ms = 10 seconds
        mut file_found = false

        while $attempts < $max_attempts {
            if ($result_file | path exists) {
                $file_found = true
                break
            }
            sleep 500ms
            $attempts = $attempts + 1
        }

        if not $file_found {
            print $"   ✗ Verification timeout - script didn't create result file"
            return {status: "fail", output: "Verification script timeout", verified: false}
        }

        # Wait a moment for file to be completely written
        sleep 500ms

        # Read and parse the verification results
        let content = try {
            let raw = (open --raw $result_file)
            $raw | from json
        } catch { |err|
            print $"   ✗ Failed to parse verification file: ($err.msg)"
            print $"   File path: ($result_file)"
            return {status: "error", output: $"Parse error: ($err.msg)", verified: false}
        }

        # Check if all tools were found
        let all_tools_ok = try {
            ($content.tools.zellij.available and
             $content.tools.yazi.available and
             $content.tools.helix.available)
        } catch { |err|
            print $"   ✗ Failed to check tool availability: ($err.msg)"
            return {status: "error", output: $"Check error: ($err.msg)", verified: false}
        }

        if $all_tools_ok {
            print $"   ✓ Verification passed - all tools available in launched session"
            print $"     - Terminal: ($content.terminal)"
            print $"     - Zellij: ($content.tools.zellij.version)"
            print $"     - Yazi: ($content.tools.yazi.version)"
            print $"     - Helix: ($content.tools.helix.version)"
            rm $result_file
            {status: "pass", output: $content, verified: true}
        } else {
            print $"   ✗ Verification failed - some tools not available in session"
            rm $result_file
            {status: "fail", output: $content, verified: false}
        }
    } catch { |err|
        print $"   ✗ Verification error: ($err.msg)"
        {status: "error", output: $err.msg, verified: false}
    }
}

def terminal_available [terminal: string]: nothing -> bool {
    let term_meta = ((get_terminal_metadata) | get -o $terminal | default {})
    let wrapper_cmd = $term_meta.wrapper
    (command_exists $wrapper_cmd) or (command_exists $terminal)
}

# Launch Yazelix for visual testing with sweep layout
export def launch_visual_test [config_path: string, test_id: string, terminal: string]: nothing -> record {
    if not (terminal_available $terminal) {
        return {
            exit_code: 99
            stdout: ""
            stderr: $"Terminal not installed: ($terminal)"
        }
    }

    let launch_output = (do {
        with-env {
            YAZELIX_CONFIG_OVERRIDE: $config_path,
            YAZELIX_SHELLHOOK_SKIP_WELCOME: "true",
            YAZELIX_LAYOUT_OVERRIDE: "yzx_sweep_test",
            YAZELIX_SWEEP_TEST_ID: $test_id
        } {
            let runtime_dir = (get_yazelix_runtime_dir)
            let yzx_cli = ($runtime_dir | path join "shells" "posix" "yzx_cli.sh")
            ^sh $yzx_cli launch --terminal $terminal
        }
    } | complete)

    {
        exit_code: $launch_output.exit_code,
        stdout: $launch_output.stdout,
        stderr: $launch_output.stderr
    }
}

# Create a standardized test result record
export def create_test_result [
    test_id: string,
    shell: string,
    terminal: string,
    status: string,
    message: string,
    details?: any
]: nothing -> record {
    {
        test_id: $test_id,
        shell: $shell,
        terminal: $terminal,
        status: $status,
        message: $message,
        details: ($details | default null)
    }
}

# Create a standardized environment test result record
export def create_env_test_result [
    test_id: string,
    shell: string,
    terminal: string,
    features: record,
    config_status: string,
    config_message: string,
    config_details: any,
    env_status: string,
    env_message: string,
    env_details: any,
    overall: string
]: nothing -> record {
    {
        test_id: $test_id,
        shell: $shell,
        terminal: $terminal,
        features: $features,
        config_status: $config_status,
        config_message: $config_message,
        config_details: ($config_details | default null),
        env_status: $env_status,
        env_message: $env_message,
        env_details: ($env_details | default null),
        overall: $overall
    }
}
