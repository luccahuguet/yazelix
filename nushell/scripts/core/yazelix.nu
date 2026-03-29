#!/usr/bin/env nu
# Yazelix Command Suite
# Consolidated commands for managing and interacting with yazelix

use ../utils/config_manager.nu *
use ../utils/constants.nu *
use ../utils/environment_bootstrap.nu [prepare_environment rebuild_yazelix_environment get_refresh_output_mode]
use ../utils/common.nu [describe_build_parallelism get_yazelix_dir require_yazelix_dir]
use ../setup/zellij_plugin_paths.nu [seed_yazelix_plugin_permissions]
use ../integrations/yazi.nu [sync_active_sidebar_yazi_to_directory sync_managed_editor_cwd]
use ./start_yazelix.nu [start_yazelix_session]
use ../integrations/zellij.nu [set_tab_cwd resolve_tab_cwd_target]

# Import modularized commands (export use to properly re-export subcommands)
export use ../yzx/launch.nu *
export use ../yzx/env.nu *
export use ../yzx/refresh.nu *
export use ../yzx/run.nu *
export use ../yzx/packs.nu *
export use ../yzx/popup.nu *
export use ../yzx/screen.nu *
export use ../yzx/gc.nu *
export use ../yzx/dev.nu *
export use ../yzx/desktop.nu *
export use ../yzx/menu.nu *
export use ../yzx/keys.nu *
export use ../yzx/tutor.nu *
export use ../yzx/whats_new.nu *

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

def has_external_command [command_name: string] {
    (which $command_name | where type == "external" | is-not-empty)
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

    if (has_external_command "xdg-open") {
        let result = (^xdg-open $sponsor_url | complete)
        if $result.exit_code == 0 {
            print "Opened sponsor page."
            return
        }
    }

    if (has_external_command "open") {
        let result = (^open $sponsor_url | complete)
        if $result.exit_code == 0 {
            print "Opened sponsor page."
            return
        }
    }

    print "Support Yazelix:"
    print $sponsor_url
}

export def resolve_yzx_cwd_target [
    target?: string  # Directory path or zoxide query for the current tab (defaults to the current directory)
] {
    resolve_tab_cwd_target $target
}

export def "yzx cwd" [
    target?: string  # Directory path or zoxide query for the current tab (defaults to the current directory)
] {
    if ($env.ZELLIJ? | is-empty) {
        print "❌ yzx cwd only works inside Zellij."
        print "   Start Yazelix first, then run this command from the tab you want to update."
        exit 1
    }

    let resolved_dir = try {
        resolve_yzx_cwd_target $target
    } catch {|err|
        print $"❌ ($err.msg)"
        exit 1
    }

    let result = try {
        set_tab_cwd $resolved_dir "yzx_cwd.log"
    } catch {|err|
        {
            status: "error"
            reason: $err.msg
        }
    }

    match $result.status {
        "ok" => {
            let editor_sync_result = (sync_managed_editor_cwd $result.workspace_root "yzx_cwd.log")
            let sidebar_sync_result = (sync_active_sidebar_yazi_to_directory $result.workspace_root "yzx_cwd.log")
            print $"✅ Updated current tab workspace directory to: ($result.workspace_root)"
            print $"   Tab renamed to: ($result.tab_name)"
            print "   The current pane will switch after this command returns."
            print "   Other existing panes keep their current working directories."
            print "   New managed actions will use the updated tab directory."
            if $editor_sync_result.status == "ok" {
                print "   Managed editor cwd synced to the updated directory."
            }
            if $sidebar_sync_result.status == "ok" {
                print "   Sidebar Yazi synced to the updated directory."
            }
        }
        "not_ready" => {
            print "❌ Yazelix tab state is not ready yet."
            print "   Wait a moment for the pane orchestrator plugin to finish loading, then try again."
            exit 1
        }
        "permissions_denied" => {
            print "❌ The Yazelix pane orchestrator plugin is missing required Zellij permissions."
            print "   Run `yzx repair zellij-permissions`, then restart Yazelix."
            exit 1
        }
        _ => {
            let reason = ($result.reason? | default "unknown error")
            print $"❌ Failed to update the current tab workspace directory: ($reason)"
            exit 1
        }
    }
}

# Canonical inspection command
export def "yzx status" [
    --versions(-V)  # Include tool version matrix
    --verbose(-v)   # Include detailed shell hook status
] {
    let env_prep = prepare_environment
    let config = $env_prep.config
    let config_state = $env_prep.config_state
    let yazelix_dir = (get_yazelix_dir)
    let shell_status = check_config_versions $yazelix_dir

    print "=== Yazelix Status ==="
    print $"Version: ($YAZELIX_VERSION)"
    print $"Description: ($YAZELIX_DESCRIPTION)"
    print $"Config File: ($config_state.config_file)"
    print $"Directory: ($yazelix_dir)"
    print $"Logs: ($yazelix_dir | path join "logs")"
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
    if $versions {
        print ""
        let version_info_script = ($yazelix_dir | path join "nushell" "scripts" "utils" "version_info.nu")
        let version_info_command = $"source \"($version_info_script)\"; main"
        ^nu -c $version_info_command
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

def create_restart_sidebar_bootstrap_file [target_dir: string] {
    let state_dir = ($env.HOME | path join ".local" "share" "yazelix" "state" "restart")
    mkdir $state_dir

    let bootstrap_file = (^mktemp ($state_dir | path join "sidebar_cwd_XXXXXX") | str trim)
    ($target_dir | path expand) | save --force --raw $bootstrap_file
    $bootstrap_file
}

# Restart yazelix
export def "yzx restart" [
    --reuse         # Reuse the last built profile without rebuilding
    --skip-refresh(-s) # Skip explicit refresh trigger and allow potentially stale environment
] {
    let env_prep = prepare_environment
    let config = $env_prep.config
    let manage_terminals = ($config.manage_terminals? | default true)
    let needs_refresh = $env_prep.needs_refresh
    let reuse_mode = $reuse
    let should_refresh = ($needs_refresh and (not $skip_refresh) and (not $reuse_mode))
    let refresh_output = get_refresh_output_mode $config
    let max_jobs = ($config.max_jobs? | default "half" | into string)
    let build_cores = ($config.build_cores? | default "2" | into string)
    let build_parallelism_description = (describe_build_parallelism $build_cores $max_jobs)
    let session_to_kill = get_current_zellij_session
    let restart_sidebar_cwd_file = (create_restart_sidebar_bootstrap_file (pwd))
    let restart_env = {
        YAZELIX_BOOTSTRAP_SIDEBAR_CWD_FILE: $restart_sidebar_cwd_file
    }

    # Detect if we're in a Yazelix-controlled terminal (launched via wrapper)
    let is_yazelix_terminal = ($env.YAZELIX_TERMINAL_CONFIG_MODE? | is-not-empty)

    # Provide appropriate messaging
    if $reuse_mode and $needs_refresh {
        print "⚡ Reuse mode enabled - using the last built Yazelix profile without rebuild."
        print "   Local config/input changes since the last refresh are not applied."
    } else if $skip_refresh and $needs_refresh {
        print "⚠️  Skipping explicit refresh trigger; environment may be stale."
        print "   If tools/env vars look outdated, rerun without --skip-refresh or run 'yzx refresh'."
    } else if $manage_terminals and $should_refresh and ($refresh_output != "quiet") {
        print $"🔄 Configuration changed - rebuilding environment using ($build_parallelism_description)..."
    }
    if $is_yazelix_terminal {
        print "🔄 Restarting Yazelix..."
    } else {
        print "🔄 Restarting Yazelix \(opening new window\)..."
    }

    # Launch new terminal window
    if $manage_terminals and $should_refresh {
        with-env $restart_env {
            rebuild_yazelix_environment --max-jobs $max_jobs --build-cores $build_cores --refresh-eval-cache --output-mode $refresh_output
            yzx launch --force-reenter
        }
    } else if $reuse_mode {
        with-env $restart_env {
            yzx launch --reuse
        }
    } else if $skip_refresh {
        with-env $restart_env {
            yzx launch --skip-refresh
        }
    } else {
        with-env $restart_env {
            yzx launch
        }
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

export def "yzx repair" [] {
    print "Available recovery commands:"
    print "  yzx repair zellij-permissions   Seed Yazelix plugin grants in ~/.cache/zellij/permissions.kdl"
}

export def "yzx repair zellij-permissions" [] {
    let result = (seed_yazelix_plugin_permissions)
    print $"✅ Seeded Yazelix plugin permissions at: ($result.permissions_cache_path)"
    print "   Restart Yazelix so Zellij reloads the plugin permission state."
}

# Update dependencies and inputs
export def "yzx update" [
    --verbose  # Show verbose output for update commands
] {
    print "User-facing updates:"
    print "  yzx update all        Update both the devenv CLI and the Yazelix repo"
    print "  yzx update devenv     Update the devenv CLI in your Nix profile"
    print "  yzx update repo       Pull latest Yazelix repo changes"
    print "  yzx update nix        Upgrade Determinate Nix \(if installed\)"
    print ""
    print "Maintainer update:"
    print "  yzx dev update        Refresh devenv.lock (optionally one input), run canaries, sync pins, and refresh vendored zjstatus"
}

export def "yzx update all" [
    --stash  # Stash local changes when updating the repo
    --verbose  # Show verbose output for update commands
] {
    yzx update devenv --verbose=$verbose
    yzx update repo --stash=$stash --verbose=$verbose
}

export def "yzx update devenv" [
    --verbose  # Show the underlying devenv command
] {
    use ../utils/nix_detector.nu ensure_nix_available
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

    let yazelix_dir = try {
        require_yazelix_dir
    } catch {|err|
        print $"❌ ($err.msg)"
        exit 1
    }
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
            print $"   rm ($yazelix_dir | path join "devenv.lock")"
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
