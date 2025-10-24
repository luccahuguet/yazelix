#!/usr/bin/env nu
# Sweep Testing - Test Execution Utilities
# Handles individual test execution and validation

use config_parser.nu parse_yazelix_config

# Validate that environment setup works for a given config
export def validate_environment [config_path: string]: nothing -> record {
    try {
        # Test 1: Tool availability check using yzx env --command
        let tools_cmd = "echo 'TOOLS_START' && which zellij && which yazi && which hx && echo 'TOOLS_END'"
        let tools_output = (do {
            with-env {YAZELIX_CONFIG_OVERRIDE: $config_path} {
                nu -c $"use ~/.config/yazelix/nushell/scripts/core/yazelix.nu *; yzx env --command '($tools_cmd)'"
            }
        } | complete)

        if $tools_output.exit_code != 0 {
            return {status: "fail", message: "Tool availability check failed", details: $tools_output.stderr}
        }

        let stdout = $tools_output.stdout
        if not ($stdout | str contains "TOOLS_START") or not ($stdout | str contains "TOOLS_END") {
            return {status: "fail", message: "Tool availability incomplete", details: $stdout}
        }

        # Test 2: Version commands using yzx env --command
        let version_cmd = "echo 'VERSION_START' && zellij --version && yazi --version && hx --version && echo 'VERSION_END'"
        let version_output = (do {
            with-env {YAZELIX_CONFIG_OVERRIDE: $config_path} {
                nu -c $"use ~/.config/yazelix/nushell/scripts/core/yazelix.nu *; yzx env --command '($version_cmd)'"
            }
        } | complete)

        if $version_output.exit_code != 0 {
            return {status: "fail", message: "Version commands failed", details: $version_output.stderr}
        }

        let version_stdout = $version_output.stdout
        if not ($version_stdout | str contains "VERSION_START") or not ($version_stdout | str contains "VERSION_END") {
            return {status: "fail", message: "Version check incomplete", details: $version_stdout}
        }

        # Verify expected tools are mentioned in version output (case insensitive)
        let stdout_lower = ($version_stdout | str downcase)
        if not ($stdout_lower | str contains "zellij") or not ($stdout_lower | str contains "yazi") or not ($stdout_lower | str contains "helix") {
            return {status: "fail", message: "Missing expected tool versions", details: $version_stdout}
        }

        {status: "pass", message: "All environment tests passed", details: null}
    } catch { |err|
        {status: "error", message: $"Test execution failed: ($err.msg)", details: null}
    }
}

# Run a demo command in the visual test environment
export def run_demo_command [
    config_path: string,
    shell: string,
    terminal: string
]: nothing -> nothing {
    try {
        with-env {YAZELIX_CONFIG_OVERRIDE: $config_path} {
            nu -c $"use ~/.config/yazelix/nushell/scripts/core/yazelix.nu *; yzx env --command 'echo \\\"($shell) + ($terminal) environment ready\\\" && zellij --version && echo \\\"Demo complete\\\"'"
        }
    } catch {
        print "   Demo command skipped"
    }
}

# Launch Yazelix for visual testing
export def launch_visual_test [config_path: string]: nothing -> record {
    let launch_output = (do {
        with-env {YAZELIX_CONFIG_OVERRIDE: $config_path, YAZELIX_SKIP_WELCOME: "true"} {
            nu -c "use ~/.config/yazelix/nushell/scripts/core/yazelix.nu *; yzx launch"
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
    env_status: string,
    env_message: string,
    overall: string
]: nothing -> record {
    {
        test_id: $test_id,
        shell: $shell,
        terminal: $terminal,
        features: $features,
        config_status: $config_status,
        config_message: $config_message,
        env_status: $env_status,
        env_message: $env_message,
        overall: $overall
    }
}