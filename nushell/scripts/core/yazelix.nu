#!/usr/bin/env nu
# Yazelix Command Suite
# Consolidated commands for managing and interacting with yazelix

use ../utils/config_manager.nu *
use ../utils/constants.nu *
use ../utils/version_info.nu *
use ../utils/config_parser.nu parse_yazelix_config
use ../utils/config_state.nu [compute_config_state mark_config_state_applied]
use ../utils/common.nu [get_max_cores]
use ./start_yazelix.nu [start_yazelix_session]

# Import modularized commands (export use to properly re-export subcommands)
export use ../yzx/launch.nu *
export use ../yzx/env.nu *

# =============================================================================
# YAZELIX COMMANDS WITH NATIVE SUBCOMMAND SUPPORT
# =============================================================================

# Yazelix Command Suite - Yazi + Zellij + Helix integrated terminal environment
#
# Manage yazelix sessions, run diagnostics, and configure your setup.
# Supports: bash, nushell, fish, zsh
#
# Common commands:
#   yzx launch    - Start a new yazelix session
#   yzx doctor    - Run health checks
#   yzx profile   - Profile launch performance
#   yzx test      - Run test suite
#   yzx lint      - Validate script syntax
#   yzx versions  - Show tool versions
export def yzx [
    --version (-V)  # Show version information
] {
    if $version {
        print $"Yazelix ($YAZELIX_VERSION)"
        return
    }
    help yzx
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

# Helper: Kill the current Zellij session
def kill_current_zellij_session [] {
    try {
        let current_session = (zellij list-sessions
            | lines
            | where $it =~ 'current'
            | first
            | split row " "
            | first)

        # Strip ANSI escape codes
        let clean_session = ($current_session | str replace -ra '\u001b\[[0-9;]*[A-Za-z]' '')

        if ($clean_session | is-empty) {
            print "⚠️  No current Zellij session detected"
            return null
        }

        print $"Killing Zellij session: ($clean_session)"
        zellij kill-session $clean_session
        return $clean_session
    } catch {|err|
        print $"❌ Failed to kill session: ($err.msg)"
        return null
    }
}

# Restart yazelix
export def "yzx restart" [] {
    # Detect if we're in a Yazelix-controlled terminal (launched via wrapper)
    let is_yazelix_terminal = ($env.YAZELIX_TERMINAL_CONFIG_MODE? | is-not-empty)

    # Provide appropriate messaging
    if $is_yazelix_terminal {
        print "🔄 Restarting Yazelix..."
    } else {
        print "🔄 Restarting Yazelix \(opening new window\)..."
    }

    # Launch new terminal window
    yzx launch

    # Wait for new session to spawn
    sleep 1sec

    # Kill old session (Yazelix terminals will close, vanilla stays open)
    kill_current_zellij_session
}

# Run health checks and diagnostics
export def "yzx doctor" [
    --verbose(-v)  # Show detailed information
    --fix(-f)      # Attempt to auto-fix issues
] {
    use ../utils/doctor.nu run_doctor_checks
    run_doctor_checks $verbose $fix
}

# Update dependencies and inputs
export def "yzx update" [] {
    print "Yazelix update commands:"
    print "  yzx update devenv  # Refresh devenv.lock using devenv update"
    print "  yzx update nix     # Alias for devenv update (refresh Yazelix dependencies)"
}

export def "yzx update devenv" [
    --verbose  # Show the underlying devenv command
] {
    use ~/.config/yazelix/nushell/scripts/utils/nix_detector.nu ensure_nix_available
    ensure_nix_available

    if (which devenv | is-empty) {
        print "❌ devenv command not found - install devenv to manage Yazelix dependencies."
        print "   See https://devenv.sh/getting-started/ for installation instructions."
        exit 1
    }

    let yazelix_dir = "~/.config/yazelix" | path expand
    let command = $"cd ($yazelix_dir) && devenv update"

    if $verbose {
        print $"⚙️ Running: ($command)"
    } else {
        print "🔄 Updating devenv inputs (this may take a moment)..."
    }

    try {
        ^bash -c $command
        print "✅ devenv.lock updated. Review and commit the changes if everything looks good."
    } catch {|err|
        print $"❌ devenv update failed: ($err.msg)"
        print "   Check your network connection and devenv.yaml inputs, then try again."
        exit 1
    }
}

export def "yzx update nix" [
    --verbose  # Show the underlying devenv command
] {
    if $verbose {
        yzx update devenv --verbose
    } else {
        yzx update devenv
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
    --all(-a)  # Include visual terminal sweep tests
] {
    use ../utils/test_runner.nu run_all_tests
    run_all_tests --verbose=$verbose --new-window=$new_window --all=$all
}

# Validate syntax of all Nushell scripts
export def "yzx lint" [
    --verbose(-v)  # Show detailed output for each file
] {
    if $verbose {
        nu $"($env.HOME)/.config/yazelix/nushell/scripts/dev/validate_syntax.nu" --verbose
    } else {
        nu $"($env.HOME)/.config/yazelix/nushell/scripts/dev/validate_syntax.nu"
    }
}

# Benchmark terminal launch performance
export def "yzx bench" [
    --iterations(-n): int = 1  # Number of iterations per terminal
    --terminal(-t): string     # Test only specific terminal
    --verbose(-v)              # Show detailed output
] {
    mut args = ["--iterations", $iterations]

    if ($terminal | is-not-empty) {
        $args = ($args | append ["--terminal", $terminal])
    }

    if $verbose {
        $args = ($args | append "--verbose")
    }

    nu $"($env.HOME)/.config/yazelix/nushell/scripts/dev/benchmark_terminals.nu" ...$args
}

# Profile launch sequence and identify bottlenecks
export def "yzx profile" [
    --cold(-c)        # Profile cold launch from vanilla terminal (emulates desktop entry or fresh terminal launch)
    --clear-cache     # Toggle yazelix.toml option and clear cache to force full Nix re-evaluation (simulates config change)
] {
    use ../utils/profile.nu *

    if $cold {
        profile_cold_launch --clear-cache=$clear_cache
    } else {
        profile_launch
    }
}
