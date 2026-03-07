#!/usr/bin/env nu
# Yazelix Command Suite
# Consolidated commands for managing and interacting with yazelix

use ../utils/config_manager.nu *
use ../utils/constants.nu *
use ../utils/environment_bootstrap.nu [prepare_environment rebuild_yazelix_environment]
use ./start_yazelix.nu [start_yazelix_session]

# Import modularized commands (export use to properly re-export subcommands)
export use ../yzx/launch.nu *
export use ../yzx/env.nu *
export use ../yzx/refresh.nu *
export use ../yzx/run.nu *
export use ../yzx/packs.nu *
export use ../yzx/gc.nu *
export use ../yzx/dev.nu *
export use ../yzx/menu.nu *
export use ../yzx/gen_config.nu *

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
#   yzx status    - Show current Yazelix status
#   yzx doctor    - Run health checks

def format_shell_hook_summary [shell_status] {
    let current = ($shell_status | where status == "current" | length)
    let outdated = ($shell_status | where status == "outdated" | length)
    let missing = ($shell_status | where status == "missing" | length)
    $"($current) current, ($outdated) outdated, ($missing) missing"
}

export def yzx [
    --version (-V)  # Show Yazelix version
    --version-short (-v)  # Show Yazelix version
] {
    if $version or $version_short {
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

export def "yzx sponsor" [] {
    let sponsor_url = "https://github.com/sponsors/luccahuguet"

    if (which xdg-open | is-not-empty) {
        let result = (^xdg-open $sponsor_url | complete)
        if $result.exit_code == 0 {
            print "Opened sponsor page."
            return
        }
    }

    if (which open | is-not-empty) {
        let result = (^open $sponsor_url | complete)
        if $result.exit_code == 0 {
            print "Opened sponsor page."
            return
        }
    }

    print "Support Yazelix:"
    print $sponsor_url
}

# Canonical inspection command
export def "yzx status" [
    --versions(-V)  # Include tool version matrix
    --verbose(-v)   # Include detailed shell hook status
    --save          # Save version matrix to docs/version_table.md (implies --versions)
] {
    let env_prep = prepare_environment
    let config = $env_prep.config
    let config_state = $env_prep.config_state
    let shell_status = check_config_versions ~/.config/yazelix

    print "=== Yazelix Status ==="
    print $"Version: ($YAZELIX_VERSION)"
    print $"Description: ($YAZELIX_DESCRIPTION)"
    print $"Config File: ($config_state.config_file)"
    print $"Directory: ($YAZELIX_CONFIG_DIR | str replace "~" $env.HOME)"
    print $"Logs: ($YAZELIX_LOGS_DIR | str replace "~" $env.HOME)"
    print $"Environment Refresh Needed: ($config_state.needs_refresh)"
    print $"Shell Hooks: (format_shell_hook_summary $shell_status)"
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
    if $verbose {
        print ""
        print "Shell Hook Details:"
        print ($shell_status | table)
    }
    if $versions or $save {
        print ""
        if $save {
            nu ~/.config/yazelix/nushell/scripts/utils/version_info.nu --save
        } else {
            nu ~/.config/yazelix/nushell/scripts/utils/version_info.nu
        }
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
export def "yzx restart" [
    --skip-refresh(-s) # Skip explicit refresh trigger and allow potentially stale environment
] {
    let env_prep = prepare_environment
    let config = $env_prep.config
    let manage_terminals = ($config.manage_terminals? | default true)
    let needs_refresh = $env_prep.needs_refresh
    let should_refresh = ($needs_refresh and (not $skip_refresh))
    let session_to_kill = get_current_zellij_session

    # Detect if we're in a Yazelix-controlled terminal (launched via wrapper)
    let is_yazelix_terminal = ($env.YAZELIX_TERMINAL_CONFIG_MODE? | is-not-empty)

    # Provide appropriate messaging
    if $skip_refresh and $needs_refresh {
        print "⚠️  Skipping explicit refresh trigger; environment may be stale."
        print "   If tools/env vars look outdated, rerun without --skip-refresh or run 'yzx refresh'."
    } else if $manage_terminals and $should_refresh {
        print "🔄 Configuration changed - rebuilding environment..."
    }
    if $is_yazelix_terminal {
        print "🔄 Restarting Yazelix..."
    } else {
        print "🔄 Restarting Yazelix \(opening new window\)..."
    }

    # Launch new terminal window
    if $manage_terminals and $should_refresh {
        rebuild_yazelix_environment --refresh-eval-cache
        yzx launch --force-reenter
    } else if $skip_refresh {
        yzx launch --skip-refresh
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
export def "yzx update" [
    --verbose  # Show verbose output for default updates
] {
    yzx update devenv --verbose=$verbose
    yzx update zjstatus
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

export def "yzx update nix" [
    --yes      # Skip confirmation prompt
    --verbose  # Show the underlying command
] {
    if (which determinate-nixd | is-empty) {
        print "❌ determinate-nixd not found in PATH."
        print "   Install Determinate Nix or check your PATH, then try again."
        exit 1
    }

    if not $yes {
        print "⚠️  This upgrades Determinate Nix using determinate-nixd."
        print "   If your Nix install is not based on Determinate Nix, this will not work."
        print "   It requires sudo and may prompt for your password."
        let confirm = try {
            (input "Continue? [y/N]: " | str downcase)
        } catch { "n" }
        if $confirm not-in ["y", "yes"] {
            print "Aborted."
            return
        }
    }

    if $verbose {
        print "⚙️ Running: sudo determinate-nixd upgrade"
    } else {
        print "🔄 Upgrading Determinate Nix..."
    }

    try {
        let result = (^sudo determinate-nixd upgrade | complete)
        if $result.exit_code != 0 {
            print $"❌ Determinate Nix upgrade failed: ($result.stderr | str trim)"
            exit 1
        }
        print "✅ Determinate Nix upgraded."
    } catch {|err|
        print $"❌ Determinate Nix upgrade failed: ($err.msg)"
        exit 1
    }
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
