#!/usr/bin/env nu
# Yazelix Command Suite
# Consolidated commands for managing and interacting with yazelix

use ../utils/config_manager.nu *
use ../utils/constants.nu *
use ../utils/version_info.nu *
use ../utils/config_parser.nu parse_yazelix_config

# =============================================================================
# YAZELIX COMMANDS WITH NATIVE SUBCOMMAND SUPPORT
# =============================================================================

# Main yzx command - default shows help
export def yzx [] {
    yzx help
}

# Help subcommand
export def "yzx help" [] {
    print "=== Yazelix Command Suite ==="
    print ""
    print "DIAGNOSTICS:"
    print "  yzx doctor [--verbose] [--fix] - Run health checks and diagnostics"
    print "  yzx test [--verbose] [--filter] [--new-window] - Run test suite"
    print "  yzx sweep [--verbose] [--visual] - Test shell/terminal combinations"
    print ""
    print "CONFIGURATION MANAGEMENT:"
    print "  yzx config_status [shell]      - Show status of all shell configurations"
    print ""
    print "VERSION AND SYSTEM:"
    print "  yzx versions                   - Show version info for all tools"
    print "  yzx info                       - Show yazelix system information"
    print "  yzx why                        - Why Yazelix (elevator pitch)"
    print ""
    print "LAUNCHER:"
    print "  yzx launch [--here] [--path DIR] [--home] - Launch yazelix (--here: current terminal, --path: specific dir, --home: home dir)"
    print "  yzx env [--no-shell] [--command CMD] - Load yazelix environment without UI (configured shell)"
    print "  yzx restart                    - Restart yazelix (preserves persistent sessions)"
    print ""
    print "MAINTENANCE:"
    print "  yzx update                     - Run 'nix flake update' for Yazelix"
    print ""
    print "HELP:"
    print "  yzx help                       - Show this help message"
    print ""
    print "Supported shells: bash, nushell, fish, zsh"
    print "=========================================="
}

# Elevator pitch: Why Yazelix
export def "yzx why" [] {
    print "Yazelix is a reproducible terminal IDE (Yazi + Zellij + Helix) with:"
    print "• Zero‑conflict keybindings, zjstatus, smooth Yazi↔editor flows"
    print "• Top terminals (Ghostty/WezTerm/Kitty/Alacritty) and shells (Bash/Zsh/Fish/Nushell)"
    print "• One‑file config (Nix) with sane defaults and curated packs"
    print "• Remote‑ready over SSH; same superterminal on barebones hosts"
    print "• Git and tooling preconfigured (lazygit, starship, zoxide, carapace)"
    print "Get everything running in <10 minutes. No extra deps, only Nix."
    print "Install once, get the same environment everywhere."
}

# Show configuration status (canonical, no aliases)
export def "yzx config_status" [shell?: string] {
    if ($shell | is-empty) {
        show_config_status ~/.config/yazelix
    } else {
        let config_file = ($SHELL_CONFIGS | get $shell | str replace "~" $env.HOME)
        if not ($config_file | path exists) {
            print $"❌ Config file not found: ($config_file)"
            return
        }
        let section = extract_yazelix_section $config_file
        if $section.exists {
            print $"=== Yazelix Section in ($shell) ==="
            print $section.content
            print "=================================="
        } else {
            print $"❌ No yazelix section found in ($config_file)"
        }
        $section
    }
}

# List available versions
export def "yzx versions" [] {
    nu ~/.config/yazelix/nushell/scripts/utils/version_info.nu
}

# Show system info
export def "yzx info" [] {
    # Parse configuration using the shared module
    let config = parse_yazelix_config

    print "=== Yazelix Information ==="
    print $"Version: ($YAZELIX_VERSION)"
    print $"Description: ($YAZELIX_DESCRIPTION)"
    print $"Directory: ($YAZELIX_CONFIG_DIR | str replace "~" $env.HOME)"
    print $"Logs: ($YAZELIX_LOGS_DIR | str replace "~" $env.HOME)"
    print $"Default Shell: ($config.default_shell)"
    print $"Preferred Terminal: ($config.preferred_terminal)"
    print $"Helix Mode: ($config.helix_mode)"
    print $"Persistent Sessions: ($config.persistent_sessions)"
    if ($config.persistent_sessions == "true") {
        print $"Session Name: ($config.session_name)"
    }
    print "=========================="
}

# Launch yazelix
export def "yzx launch" [
    --here             # Start in current terminal instead of launching new terminal
    --path(-p): string # Start in specific directory
    --home             # Start in home directory
    --terminal(-t): string  # Override terminal selection (for sweep testing)
] {
    use ~/.config/yazelix/nushell/scripts/utils/nix_detector.nu ensure_nix_available
    ensure_nix_available

    if $here {
        # Start in current terminal (like old yzx start)
        let start_script = ~/.config/yazelix/nushell/scripts/core/start_yazelix.nu
        if $home {
            nu $start_script $env.HOME
        } else if ($path | is-not-empty) {
            nu $start_script $path
        } else {
            nu $start_script
        }
    } else {
        # Launch new terminal (original behavior)
        let launch_cwd = if $home {
            $env.HOME
        } else if ($path | is-not-empty) {
            $path
        } else {
            pwd
        }

        # Run launch_yazelix.nu inside nix develop so all terminal wrappers are available
        # Pass through sweep test environment variables if present
        let launch_script = "~/.config/yazelix/nushell/scripts/core/launch_yazelix.nu"
        let launch_cmd = if ($terminal | is-not-empty) {
            $"nu ($launch_script) ($launch_cwd) --terminal ($terminal)"
        } else {
            $"nu ($launch_script) ($launch_cwd)"
        }

        # Build environment variable exports for bash
        let env_exports = [
            (if ($env.YAZELIX_CONFIG_OVERRIDE? | is-not-empty) { $"export YAZELIX_CONFIG_OVERRIDE='($env.YAZELIX_CONFIG_OVERRIDE)'; " } else { "" })
            (if ($env.ZELLIJ_DEFAULT_LAYOUT? | is-not-empty) { $"export ZELLIJ_DEFAULT_LAYOUT='($env.ZELLIJ_DEFAULT_LAYOUT)'; " } else { "" })
            (if ($env.YAZELIX_SWEEP_TEST_ID? | is-not-empty) { $"export YAZELIX_SWEEP_TEST_ID='($env.YAZELIX_SWEEP_TEST_ID)'; " } else { "" })
            (if ($env.YAZELIX_SKIP_WELCOME? | is-not-empty) { $"export YAZELIX_SKIP_WELCOME='($env.YAZELIX_SKIP_WELCOME)'; " } else { "" })
            (if ($env.YAZELIX_TERMINAL? | is-not-empty) { $"export YAZELIX_TERMINAL='($env.YAZELIX_TERMINAL)'; " } else { "" })
        ] | str join ""

        let full_cmd = $"($env_exports)($launch_cmd)"
        ^nix develop --impure ~/.config/yazelix --command bash -c $full_cmd
    }
}

# Load yazelix environment without UI
export def "yzx env" [
    --no-shell(-n)  # Keep current shell instead of launching configured shell
    --command(-c): string  # Run a command in the Yazelix environment
] {
    use ~/.config/yazelix/nushell/scripts/utils/nix_detector.nu ensure_nix_available
    ensure_nix_available
    if ($command | is-not-empty) {
        # Run command in Yazelix environment (skip welcome screen for automation)
        with-env {YAZELIX_ENV_ONLY: "true", YAZELIX_SKIP_WELCOME: "true"} {
            ^nix develop --impure ~/.config/yazelix --command bash -c $command
        }
    } else if $no_shell {
        with-env {YAZELIX_ENV_ONLY: "true"} {
            ^nix develop --impure ~/.config/yazelix
        }
    } else {
        let config = (try { parse_yazelix_config } catch {|err|
            print $"❌ Failed to parse Yazelix configuration: ($err.msg)"
            exit 1
        })
        let shell_name = ($config.default_shell? | default "nu" | str downcase)
        let shell_command = match $shell_name {
            "nu" => ["nu" "--login"]
            "bash" => ["bash" "--login"]
            "fish" => ["fish" "-l"]
            "zsh" => ["zsh" "-l"]
            _ => [$shell_name]
        }
        let shell_exec = ($shell_command | first)
        let command_str = ($shell_command | str join " ")
        let exec_command = $"exec ($command_str)"
        with-env {YAZELIX_ENV_ONLY: "true", SHELL: $shell_exec} {
            try {
                ^nix develop --impure ~/.config/yazelix --command bash "-lc" $exec_command
            } catch {|err|
                print $"❌ Failed to launch configured shell: ($err.msg)"
                print "   Tip: rerun with 'yzx env --no-shell' to stay in your current shell."
                exit 1
            }
        }
    }
}

# Restart yazelix
export def "yzx restart" [] {
    # Parse configuration using the shared module
    let config = parse_yazelix_config

    if ($config.persistent_sessions == "true") {
        print $"Persistent sessions are enabled \(session: ($config.session_name)\)"
        print "yzx restart is disabled when persistent sessions are enabled."
        print "Your session will persist automatically - no restart needed."
        print ""
        print "To start a new session, use: yzx start"
        print $"To kill the current session, use: zellij kill-session ($config.session_name)"
    } else {
        print "Attempting to kill the current Zellij session..."
        let current_session = (zellij list-sessions | lines | where $it =~ 'current' | first | split row " " | first)
        let clean_session = ($current_session | str replace -ra '\u001b\[[0-9;]*[A-Za-z]' '')
        print "Restarting Yazelix..."
        yzx launch
        print "Waiting for Zellij to shut down..."
        sleep 1sec
        if ($clean_session | is-empty) {
            print "No current Zellij session detected. Skipping kill step."
        } else {
            print $"Killing Zellij session: ($clean_session)"
            try { zellij kill-session $clean_session } catch { print $"Failed to kill session: ($clean_session)" }
        }
    }
}

# Run health checks and diagnostics
export def "yzx doctor" [
    --verbose(-v)  # Show detailed information
    --fix(-f)      # Attempt to auto-fix issues
] {
    use ../utils/doctor.nu run_doctor_checks
    run_doctor_checks $verbose $fix
}

# Update flake inputs for Yazelix
export def "yzx update" [] {
    use ~/.config/yazelix/nushell/scripts/utils/nix_detector.nu ensure_nix_available
    ensure_nix_available
    let dir = $"($env.HOME)/.config/yazelix"
    if not ($dir | path exists) {
        print $"Error: Yazelix directory not found at ($dir)"
        exit 1
    }
    print "Running: nix flake update (this may take a while)"
    cd $dir
    try {
        ^nix flake update
        print "Done: flake inputs updated. Review and commit flake.lock changes."
    } catch {|err|
        print $"flake update failed: ($err.msg)"
        exit 1
    }
}

# Run configuration sweep tests across shell/terminal combinations
export def "yzx sweep" [] {
    print "Run 'yzx sweep --help' to see available subcommands"
}

# Test all shell combinations
export def "yzx sweep shells" [
    --verbose(-v)  # Show detailed output
] {
    use ../dev/test_config_sweep.nu run_all_sweep_tests

    if $verbose {
        run_all_sweep_tests --verbose
    } else {
        run_all_sweep_tests
    }
}

# Test all terminal launches
export def "yzx sweep terminals" [
    --verbose(-v)       # Show detailed output
    --delay: int = 3    # Delay between terminal launches in seconds
] {
    use ../dev/test_config_sweep.nu run_all_sweep_tests

    run_all_sweep_tests --visual --verbose=$verbose --visual-delay $delay
}

# Run all sweep tests (shells + terminals)
export def "yzx sweep all" [
    --verbose(-v)       # Show detailed output
    --delay: int = 3    # Delay between terminal launches in seconds
] {
    print "=== Running All Sweep Tests ==="
    print "Phase 1: Shell combinations (fast)"
    print ""

    yzx sweep shells --verbose=$verbose

    print ""
    print "=== Phase 2: Terminal launches (slow) ==="
    print ""

    yzx sweep terminals --verbose=$verbose --delay $delay
}

# Run Yazelix test suite
export def "yzx test" [
    --verbose(-v)  # Show detailed test output
    --new-window(-n)  # Run tests in a new Yazelix window
] {
    use ../utils/test_runner.nu run_all_tests
    run_all_tests --verbose=$verbose --new-window=$new_window
}
