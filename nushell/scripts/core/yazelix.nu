#!/usr/bin/env nu
# Yazelix Command Suite
# Consolidated commands for managing and interacting with yazelix

use ../utils/config_manager.nu *
use ../utils/constants.nu *
use ../utils/version_info.nu *
use ../utils/config_parser.nu parse_yazelix_config
use ../utils/config_state.nu [compute_config_state mark_config_state_applied]
use ../utils/common.nu [get_max_cores]
use ../utils/environment_bootstrap.nu prepare_environment
use ./start_yazelix.nu [start_yazelix_session]

# Import modularized commands (export use to properly re-export subcommands)
export use ../yzx/launch.nu *
export use ../yzx/env.nu *
export use ../yzx/run.nu *

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
#   yzx run       - Run a command inside the Yazelix environment
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
    let terminals = ($config.terminals? | default ["ghostty"])
    if ($terminals | is-empty) {
        print "Terminals: none"
    } else {
        print $"Terminals: (($terminals | str join ', '))"
    }
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
    let env_prep = prepare_environment
    let config = $env_prep.config
    let manage_terminals = ($config.manage_terminals? | default true)
    let needs_refresh = $env_prep.needs_refresh

    # Detect if we're in a Yazelix-controlled terminal (launched via wrapper)
    let is_yazelix_terminal = ($env.YAZELIX_TERMINAL_CONFIG_MODE? | is-not-empty)

    # Provide appropriate messaging
    if $manage_terminals and $needs_refresh {
        print "🔄 Configuration changed - rebuilding environment to install terminals..."
    }
    if $is_yazelix_terminal {
        print "🔄 Restarting Yazelix..."
    } else {
        print "🔄 Restarting Yazelix \(opening new window\)..."
    }

    # Launch new terminal window
    if $manage_terminals and $needs_refresh {
        with-env {YAZELIX_FORCE_REENTER: "true"} {
            yzx launch
        }
    } else {
        yzx launch
    }

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
    print "  yzx update devenv  # Update the devenv CLI in your Nix profile"
    print "  yzx update lock    # Refresh devenv.lock using devenv update"
    print "  yzx update zjstatus  # Update bundled zjstatus.wasm plugin"
    print "  yzx update repo    # Pull latest Yazelix updates"
    print "  yzx update all     # Run every update command"
}

export def "yzx update devenv" [
    --verbose  # Show the underlying devenv command
] {
    use ~/.config/yazelix/nushell/scripts/utils/nix_detector.nu ensure_nix_available
    ensure_nix_available --skip-devenv

    if $verbose {
        print "⚙️ Running: nix profile install github:cachix/devenv/latest"
        print "⚙️ Running: nix profile upgrade devenv"
    }

    let profile = try { ^nix profile list --json | from json } catch { null }
    let profile_entries = if $profile == null { [] } else { $profile.elements | columns }
    let profile_has_devenv = ($profile_entries | any { |name| $name == "devenv" })

    if not $profile_has_devenv {
        if (which devenv | is-not-empty) {
            print "ℹ️ devenv found in PATH but not managed by your Nix profile."
            print "   Installing into the profile so it can be updated with `yzx update devenv`."
        }

        print "🔄 Installing devenv CLI..."

        try {
            let result = (^nix profile install "github:cachix/devenv/latest" | complete)
            if $result.exit_code != 0 {
                print $"❌ devenv install failed: ($result.stderr | str trim)"
                print "   Check your Nix setup and try again."
                exit 1
            }
            print "✅ devenv CLI installed."
        } catch {|err|
            print $"❌ devenv install failed: ($err.msg)"
            print "   Check your Nix setup and try again."
            exit 1
        }
    } else {
        print "🔄 Updating devenv CLI..."

        try {
            let result = (^nix profile upgrade "devenv" | complete)
            if $result.exit_code != 0 {
                print $"❌ devenv update failed: ($result.stderr | str trim)"
                print "   Try: nix profile install github:cachix/devenv/latest"
                exit 1
            }

            let stderr = ($result.stderr | str trim)
            if ($stderr | str contains "No packages to upgrade") or ($stderr | str contains "does not match") {
                print "ℹ️ devenv CLI is already up to date."
            } else {
                print "✅ devenv CLI updated."
            }
        } catch {|err|
            print $"❌ devenv update failed: ($err.msg)"
            print "   Try: nix profile install github:cachix/devenv/latest"
            exit 1
        }
    }
}

export def "yzx update lock" [
    --verbose  # Show the underlying devenv command
    --yes      # Skip confirmation prompt
] {
    use ~/.config/yazelix/nushell/scripts/utils/nix_detector.nu ensure_nix_available
    ensure_nix_available

    let yazelix_dir = "~/.config/yazelix" | path expand

    if not $yes {
        print "⚠️  This updates Yazelix inputs (devenv.lock) to latest upstream versions."
        print "   If upstream changes are broken, you may hit bugs before fixes land."
        print "   Prefer a safer path? The Yazelix maintainer updates the project at least once a month."
        let confirm = (input "Continue? [y/N]: " | str downcase)
        if $confirm not-in ["y", "yes"] {
            print "Aborted."
            exit 0
        }
    }

    if $verbose {
        print $"⚙️ Running: devenv update \(cwd: ($yazelix_dir)\)"
    } else {
        print "🔄 Updating Yazelix inputs (devenv.lock)..."
    }

    try {
        do {
            cd $yazelix_dir
            ^devenv update
        }
        print "✅ devenv.lock updated. Review and commit the changes if everything looks good."
    } catch {|err|
        print $"❌ devenv update failed: ($err.msg)"
        print "   Check your network connection and devenv.yaml inputs, then try again."
        exit 1
    }
}

# Update zjstatus plugin
export def "yzx update zjstatus" [] {
    nu ~/.config/yazelix/nushell/scripts/dev/update_zjstatus.nu
}

# Run all available update commands
export def "yzx update all" [] {
    yzx update devenv
    yzx update lock --yes
    yzx update zjstatus
}

export def "yzx update repo" [
    --stash  # Stash local changes, pull updates, then re-apply the stash
    --verbose  # Show git commands
] {
    if (which git | is-empty) {
        print "❌ git not found in PATH."
        exit 1
    }

    let yazelix_dir = "~/.config/yazelix" | path expand
    let status = (do {
        cd $yazelix_dir
        ^git status --porcelain
    } | complete)

    if $status.exit_code != 0 {
        print $"❌ Failed to check git status: ($status.stderr | str trim)"
        exit 1
    }

    let is_dirty = ($status.stdout | str trim | is-not-empty)
    let dirty_files = ($status.stdout | lines | each { |line| $line | str trim | split row " " | last })
    let only_lock_dirty = ($dirty_files | length) == 1 and ($dirty_files | first) == "devenv.lock"

    if $is_dirty and (not $stash) {
        if $only_lock_dirty {
            print "❌ Local devenv.lock changes detected."
            print "   If you want upstream updates, delete it and rerun 'yzx update repo':"
            print "   rm ~/.config/yazelix/devenv.lock"
            exit 1
        }
        print "❌ Working tree is dirty. Please commit or stash changes first."
        print "   Tip: rerun with 'yzx update repo --stash' to stash automatically."
        exit 1
    }

    if $verbose {
        print "⚙️ Running: git pull --rebase"
    } else {
        print "🔄 Updating Yazelix repository..."
    }

    if $stash {
        let stash_result = (do {
            cd $yazelix_dir
            ^git stash push -u -m "yzx update repo"
        } | complete)

        if $stash_result.exit_code != 0 {
            print $"❌ git stash failed: ($stash_result.stderr | str trim)"
            exit 1
        }
    }

    let pull_result = (do {
        cd $yazelix_dir
        ^git pull --rebase
    } | complete)

    if $pull_result.exit_code != 0 {
        print $"❌ git pull failed: ($pull_result.stderr | str trim)"
        if $stash {
            print "   Your stash is still available. Run 'git stash list' to recover."
        }
        exit 1
    }

    if $stash {
        let pop_result = (do {
            cd $yazelix_dir
            ^git stash pop
        } | complete)

        if $pop_result.exit_code != 0 {
            print $"⚠️ git stash pop reported conflicts: ($pop_result.stderr | str trim)"
            print "   Resolve conflicts and run 'git status' to verify."
            exit 1
        }
    }

    print "✅ Yazelix repository updated."
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
