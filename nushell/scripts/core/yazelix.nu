#!/usr/bin/env nu
# Yazelix Command Suite
# Consolidated commands for managing and interacting with yazelix

use ../utils/config_manager.nu *
use ../utils/constants.nu *
use ../utils/version_info.nu *
use ../utils/ascii_art.nu [get_yazelix_colors]
use ../utils/config_parser.nu parse_yazelix_config
use ../utils/config_state.nu [compute_config_state mark_config_state_applied]
use ../utils/common.nu [get_max_cores]
use ../utils/environment_bootstrap.nu prepare_environment
use ./start_yazelix.nu [start_yazelix_session]

# Import modularized commands (export use to properly re-export subcommands)
export use ../yzx/launch.nu *
export use ../yzx/env.nu *
export use ../yzx/run.nu *
export use ../yzx/packs.nu *
export use ../yzx/gc.nu *
export use ../yzx/dev.nu *
export use ../yzx/menu.nu *

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
def compare_versions [left: string, right: string] {
    let left_parts = ($left | split row "." | each { |part| $part | into int })
    let right_parts = ($right | split row "." | each { |part| $part | into int })

    for idx in 0..2 {
        let left_value = ($left_parts | get -o $idx | default 0)
        let right_value = ($right_parts | get -o $idx | default 0)
        if $left_value > $right_value {
            return 1
        }
        if $left_value < $right_value {
            return (-1)
        }
    }

    return (0)
}

def parse_semver [value: string] {
    $value | parse --regex '(\d+\.\d+\.\d+)' | get capture0 | last | default ""
}

def build_version_warning [tool: string, installed_raw: string, pinned: string] {
    if $installed_raw in ["not installed", "error"] {
        return null
    }

    let installed = (parse_semver $installed_raw)
    if ($installed | is-empty) or $installed == $pinned {
        return null
    }

    let status = if (compare_versions $installed $pinned) == 1 { "ahead" } else { "stale" }
    $"($tool) ($installed) is ($status) vs pinned ($pinned)"
}

export def yzx [
    --version (-V)  # Show version information
    --version-short (-v)  # Show version information
] {
    if $version or $version_short {
        print $"Yazelix ($YAZELIX_VERSION)"

        let devenv_version = if (which devenv | is-empty) {
            "not installed"
        } else {
            try { (devenv --version | lines | first) } catch { "error" }
        }

        let nix_version = if (which nix | is-empty) {
            "not installed"
        } else {
            try { (nix --version | lines | first) } catch { "error" }
        }

        let determinate_version = if (which determinate-nixd | is-empty) {
            "not installed"
        } else {
            try {
                let result = (^determinate-nixd version | complete)
                if $result.exit_code == 0 {
                    $result.stdout | lines | first
                } else {
                    "error"
                }
            } catch { "error" }
        }

        let warnings = ([
            (build_version_warning "devenv" $devenv_version $PINNED_DEVENV_VERSION)
            (build_version_warning "nix" $nix_version $PINNED_NIX_VERSION)
        ] | where ($it | is-not-empty))

        let colors = get_yazelix_colors
        let key_color = $colors.cyan
        let value_color = $colors.purple
        let warn_color = $colors.yellow
        let success_color = $colors.green
        let reset_color = $colors.reset

        print $"($key_color)devenv:($reset_color) ($value_color)($devenv_version)($reset_color)"
        print $"($key_color)nix:($reset_color) ($value_color)($nix_version)($reset_color)"
        print $"($key_color)determinate-nixd:($reset_color) ($value_color)($determinate_version)($reset_color)"

        if ($warnings | length) > 0 {
            print $"($warn_color)⚠️  Version drift detected:($reset_color)"
            $warnings | each { |warning| print $"   - ($warning)" }
        } else {
            print $"($success_color)✅ Versions match Yazelix pinned values.($reset_color)"
        }
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
export def "yzx versions" [
    --save(-s)
] {
    if $save {
        nu ~/.config/yazelix/nushell/scripts/utils/version_info.nu --save
    } else {
        nu ~/.config/yazelix/nushell/scripts/utils/version_info.nu
    }
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

# Helper: Resolve the current Zellij session from environment or CLI.
def get_current_zellij_session [] {
    if ($env.ZELLIJ_SESSION_NAME? | is-not-empty) {
        return $env.ZELLIJ_SESSION_NAME
    }

    try {
        let current_line = (
            zellij list-sessions
            | lines
            | where {|line| ($line =~ '\bcurrent\b')}
            | first
        )

        let clean_line = (
            $current_line
            | str replace -ra '\u001b\[[0-9;]*[A-Za-z]' ''
            | str replace -r '^>\s*' ''
            | str trim
        )

        if ($clean_line | is-empty) {
            return null
        }

        return (
            $clean_line
            | split row " "
            | where {|token| $token != ""}
            | first
        )
    } catch {
        return null
    }
}

# Helper: Kill a specific Zellij session
def kill_zellij_session [session_name?: string] {
    try {
        if ($session_name | is-empty) {
            print "⚠️  No Zellij session detected to close"
            return null
        }

        print $"Killing Zellij session: ($session_name)"
        zellij kill-session $session_name
        return $session_name
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
    let session_to_kill = get_current_zellij_session

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

    # Kill the originating session after launching the new window.
    # yzx launch clears inherited Zellij context vars so the new window starts independently.
    kill_zellij_session $session_to_kill
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
    print "  yzx update zjstatus  # Update bundled zjstatus.wasm plugin"
    print "  yzx update repo    # Pull latest Yazelix updates"
    print "  yzx update all     # Run safe update commands (excludes dev-only commands)"
    print ""
    print "Maintainer-only updates:"
    print "  yzx dev update_lock  # Refresh devenv.lock using devenv update"
    print "  yzx dev update_nix   # Upgrade Determinate Nix via determinate-nixd (sudo required)"
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


# Update zjstatus plugin
export def "yzx update zjstatus" [] {
    nu ~/.config/yazelix/nushell/scripts/dev/update_zjstatus.nu
}


# Run all available update commands
export def "yzx update all" [] {
    print "ℹ️  Note: update all skips maintainer-only commands (see 'yzx update')."
    yzx update devenv
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
