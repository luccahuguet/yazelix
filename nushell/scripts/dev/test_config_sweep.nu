#!/usr/bin/env nu
# Configuration Sweep Testing Framework
# Tests shell/terminal combinations and configuration variations

use ../utils/config_parser.nu parse_yazelix_config
use ../utils/constants.nu *

# Test sweep definitions - using supported shells and terminals from constants
const SHELLS = ["nu", "bash", "fish", "zsh"]
const PRIMARY_SHELL = $DEFAULT_SHELL
const PRIMARY_TERMINAL = $DEFAULT_TERMINAL
const TERMINALS = $SUPPORTED_TERMINALS

# Configuration variations to test
const HELIX_MODES = ["release", "source"]
const BOOLEAN_FEATURES = [
    "enable_sidebar",
    "persistent_sessions",
    "recommended_deps",
    "yazi_extensions"
]

# Generate temporary yazelix.nix config for testing
export def generate_sweep_config [
    shell: string,
    terminal: string,
    features: record,
    test_id: string
] {
    let temp_dir = $"($env.HOME)/.local/share/yazelix/sweep_tests"
    mkdir $temp_dir

    let config_path = $"($temp_dir)/yazelix_test_($test_id).nix"

    let config_content = $"{ pkgs }:
{
  # Sweep test configuration - ($test_id)
  # Shell: ($shell), Terminal: ($terminal)

  # Core settings
  default_shell = \"($shell)\";
  preferred_terminal = \"($terminal)\";
  helix_mode = \"($features.helix_mode)\";

  # Feature flags
  enable_sidebar = ($features.enable_sidebar);
  persistent_sessions = ($features.persistent_sessions);
  recommended_deps = ($features.recommended_deps);
  yazi_extensions = ($features.yazi_extensions);
  yazi_media = false;  # Keep minimal for testing

  # Disable features that might cause issues in testing
  debug_mode = false;
  skip_welcome_screen = true;  # Suppress output for clean testing
  enable_atuin = false;
  disable_zellij_tips = true;  # Prevent tips popup during visual testing

  # Minimal extras for testing
  extra_shells = [];
  extra_terminals = [];
  packs = [];
  user_packages = with pkgs; [];

  # Terminal config mode
  terminal_config_mode = \"yazelix\";

  # Session settings
  session_name = \"sweep_test_($test_id)\";

  # Appearance \(minimal\)
  cursor_trail = \"none\";
  transparency = \"none\";
  ascii_art_mode = \"static\";
  show_macchina_on_welcome = false;
}
"

    $config_content | save --force $config_path
    $config_path
}

# Clean up temporary test configs
def cleanup_sweep_configs [] {
    let temp_dir = $"($env.HOME)/.local/share/yazelix/sweep_tests"
    if ($temp_dir | path exists) {
        rm -rf $temp_dir
    }
}

# Validate that environment setup works for a given config
export def validate_environment [config_path: string, timeout: duration = 30sec] {
    let result = try {
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

    $result
}

# Run a visual sweep test by launching actual Yazelix
export def run_visual_sweep_test [
    shell: string,
    terminal: string,
    features: record,
    test_id: string,
    delay: duration = 3sec
] {
    print $"🖥️  Launching visual test: ($shell) + ($terminal) \(($test_id)\)"

    let config_path = generate_sweep_config $shell $terminal $features $test_id
    let session_name = $"sweep_test_($test_id)"

    let result = try {
        # Get terminal process count before launch (to identify new processes)
        let before_pids = try {
            ps | where name =~ $terminal | get pid
        } catch {
            []
        }

        # Launch Yazelix with the test config and specific session name
        let launch_output = (do {
            with-env {YAZELIX_CONFIG_OVERRIDE: $config_path, YAZELIX_SKIP_WELCOME: "true"} {
                nu -c "use ~/.config/yazelix/nushell/scripts/core/yazelix.nu *; yzx launch"
            }
        } | complete)

        if $launch_output.exit_code != 0 {
            print $"❌ Failed to launch ($shell) + ($terminal): ($launch_output.stderr)"
            {
                test_id: $test_id,
                shell: $shell,
                terminal: $terminal,
                status: "fail",
                message: "Launch failed",
                details: $launch_output.stderr
            }
        } else {
            print $"✅ Launched ($shell) + ($terminal) successfully"
            print $"   Running demo command to show functionality..."

            # Execute a demo command to show the environment works and bypass any tips
            try {
                with-env {YAZELIX_CONFIG_OVERRIDE: $config_path} {
                    nu -c $"use ~/.config/yazelix/nushell/scripts/core/yazelix.nu *; yzx env --command 'echo \"($shell) + ($terminal) environment ready\" && zellij --version && echo \"Demo complete\"'"
                }
            } catch {
                print "   Demo command skipped"
            }

            print $"   Waiting ($delay) before cleanup..."
            sleep $delay

            # Kill the zellij session after demonstration
            try {
                let sessions = (zellij list-sessions | lines | where $it =~ $session_name)
                if not ($sessions | is-empty) {
                    let session_line = ($sessions | first)
                    let session_id = ($session_line | split row " " | first | str replace -ra '\u001b\[[0-9;]*[A-Za-z]' '')
                    print $"   Cleaning up session: ($session_id)"
                    zellij kill-session $session_id
                }
            } catch {
                print "   Session cleanup skipped"
            }

            # Kill terminal processes associated with this test
            try {
                # Wait for session cleanup to complete
                sleep 1sec

                # Find terminal processes that were started after our baseline and kill them
                let after_pids = try {
                    ps | where name =~ $terminal | get pid
                } catch {
                    []
                }

                let new_pids = $after_pids | where $it not-in $before_pids

                if not ($new_pids | is-empty) {
                    for $pid in $new_pids {
                        print $"   Terminating terminal process: ($pid)"
                        try {
                            # Graceful termination first (SIGTERM = 15)
                            kill --signal 15 $pid
                            sleep 300ms
                            # Force kill if still running
                            let still_running = try {
                                (ps | where pid == $pid | length) > 0
                            } catch { false }
                            if $still_running {
                                kill --force $pid
                            }
                        } catch {
                            print $"   Failed to kill process ($pid)"
                        }
                    }
                } else {
                    print $"   No new terminal processes detected for cleanup"
                }
            } catch {
                print $"   Terminal cleanup failed"
            }

            {
                test_id: $test_id,
                shell: $shell,
                terminal: $terminal,
                status: "pass",
                message: "Visual launch successful",
                details: null
            }
        }
    } catch { |err|
        print $"💥 Error launching ($shell) + ($terminal): ($err.msg)"
        {
            test_id: $test_id,
            shell: $shell,
            terminal: $terminal,
            status: "error",
            message: $"Launch error: ($err.msg)",
            details: null
        }
    }

    # Clean up config
    if ($config_path | path exists) {
        rm $config_path
    }

    $result
}

# Run a single sweep test
def run_sweep_test [
    shell: string,
    terminal: string,
    features: record,
    test_id: string,
    verbose: bool = false
] {
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
            return {
                test_id: $test_id,
                shell: $shell,
                terminal: $terminal,
                features: $features,
                config_status: $config_test.status,
                config_message: $config_test.message,
                env_status: "skipped",
                env_message: "Skipped due to config failure",
                overall: "fail"
            }
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

        {
            test_id: $test_id,
            shell: $shell,
            terminal: $terminal,
            features: $features,
            config_status: $config_test.status,
            config_message: $config_test.message,
            env_status: $env_result.status,
            env_message: $env_result.message,
            overall: $overall_status
        }
    } catch { |err|
        {
            test_id: $test_id,
            shell: $shell,
            terminal: $terminal,
            features: $features,
            config_status: "error",
            config_message: $"Test failed: ($err.msg)",
            env_status: "error",
            env_message: "Test execution error",
            overall: "error"
        }
    }

    # Clean up individual test config
    if ($config_path | path exists) {
        rm $config_path
    }

    $result
}

# Generate all test combinations
def generate_test_combinations [] {
    mut combinations = []

    # 1. Cross-shell testing (each shell with primary terminal)
    for $shell in $SHELLS {
        $combinations = ($combinations | append {
            type: "cross_shell",
            shell: $shell,
            terminal: $PRIMARY_TERMINAL,
            features: {
                helix_mode: "release",
                enable_sidebar: true,
                persistent_sessions: false,
                recommended_deps: true,
                yazi_extensions: true
            }
        })
    }

    # 2. Cross-terminal testing (primary shell with each terminal)
    for $terminal in $TERMINALS {
        if $terminal != $PRIMARY_TERMINAL {  # Avoid duplicate
            $combinations = ($combinations | append {
                type: "cross_terminal",
                shell: $PRIMARY_SHELL,
                terminal: $terminal,
                features: {
                    helix_mode: "release",
                    enable_sidebar: true,
                    persistent_sessions: false,
                    recommended_deps: true,
                    yazi_extensions: true
                }
            })
        }
    }

    # 3. Feature variation testing (primary shell/terminal with different features)
    for $helix_mode in $HELIX_MODES {
        $combinations = ($combinations | append {
            type: "feature_variation",
            shell: $PRIMARY_SHELL,
            terminal: $PRIMARY_TERMINAL,
            features: {
                helix_mode: $helix_mode,
                enable_sidebar: true,
                persistent_sessions: false,
                recommended_deps: true,
                yazi_extensions: true
            }
        })
    }

    # 4. Boolean feature combinations (test key features on/off)
    $combinations = ($combinations | append {
        type: "minimal_config",
        shell: $PRIMARY_SHELL,
        terminal: $PRIMARY_TERMINAL,
        features: {
            helix_mode: "release",
            enable_sidebar: false,
            persistent_sessions: false,
            recommended_deps: false,
            yazi_extensions: false
        }
    })

    $combinations = ($combinations | append {
        type: "maximal_config",
        shell: $PRIMARY_SHELL,
        terminal: $PRIMARY_TERMINAL,
        features: {
            helix_mode: "source",
            enable_sidebar: true,
            persistent_sessions: true,
            recommended_deps: true,
            yazi_extensions: true
        }
    })

    $combinations
}

# Main sweep test runner
export def run_all_sweep_tests [
    --verbose(-v)           # Show detailed output
    --visual(-w)            # Launch visual Yazelix windows for each test
    --visual-delay: int     # Delay between visual launches in seconds (default: 3)
] {
    let visual_delay = (($visual_delay | default 3) * 1sec)

    if $visual {
        print "=== Visual Configuration Sweep Testing ==="
        print "🖥️  Each configuration will launch in a new window"
        print $"⏱️  Delay between launches: ($visual_delay)"
    } else {
        print "=== Configuration Sweep Testing ==="
    }
    print ""

    # Generate test combinations
    let combinations = generate_test_combinations

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
            let status = if $visual { $result.status } else { $result.overall }
            print $"  Progress: ($completed)/($total_tests) - ($status | str upcase) ($combo.shell)+($combo.terminal)"
        }
    }

    # Generate summary report
    print ""
    print "=== Sweep Test Results ==="

    # Handle both visual and regular test result formats
    let passed = if $visual {
        ($results | where status == "pass" | length)
    } else {
        ($results | where overall == "pass" | length)
    }
    let failed = if $visual {
        ($results | where status == "fail" | length)
    } else {
        ($results | where overall == "fail" | length)
    }
    let errors = if $visual {
        ($results | where status == "error" | length)
    } else {
        ($results | where overall == "error" | length)
    }
    let skipped = if $visual {
        0  # Visual tests don't have skip status
    } else {
        ($results | where env_status == "skip" | length)
    }

    # Show detailed results
    for $result in $results {
        let status_field = if $visual { $result.status } else { $result.overall }
        let status_icon = match $status_field {
            "pass" => "✅",
            "fail" => "❌",
            "error" => "💥"
        }

        if $visual {
            print $"($status_icon) ($result | get test_id? | default "unknown"): ($result | get shell? | default "unknown") + ($result | get terminal? | default "unknown")"
            if $verbose or ($status_field != "pass") {
                print $"   Message: ($result.message)"
                if ($result.details | is-not-empty) {
                    print $"   Details: ($result.details)"
                }
            }
        } else {
            print $"($status_icon) ($result.test_id): ($result.shell) + ($result.terminal)"
            if $verbose or ($result.overall != "pass") {
                print $"   Config: ($result.config_status) - ($result.config_message)"
                print $"   Environment: ($result.env_status) - ($result.env_message)"
                if ($result.overall != "pass") {
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