#!/usr/bin/env nu
# Yazelix Command Suite
# Consolidated commands for managing and interacting with yazelix

use ../utils/atomic_writes.nu write_text_atomic
use ../utils/constants.nu *
use ../utils/common.nu get_yazelix_runtime_dir
use ../utils/config_parser.nu resolve_yzx_core_helper_path
use ../utils/launcher_resolution.nu resolve_stable_yzx_wrapper_path
use ../utils/status_report.nu [collect_status_report render_status_report]
use ../integrations/managed_editor.nu get_managed_editor_kind
use ../integrations/yazi.nu [reveal_in_yazi sync_sidebar_yazi_state_to_directory]
use ../integrations/zellij.nu [retarget_tab_cwd resolve_tab_cwd_target]

# Import modularized commands (export use to properly re-export subcommands)
export use ../yzx/launch.nu *
export use ../yzx/enter.nu *
export use ../yzx/import.nu *
export use ../yzx/popup.nu *
export use ../yzx/screen.nu *
export use ../yzx/dev.nu *
export use ../yzx/desktop.nu *
export use ../yzx/menu.nu *
export use ../yzx/config.nu *
export use ../yzx/edit.nu *
export use ../yzx/keys.nu *
export use ../yzx/tutor.nu *
export use ../yzx/whats_new.nu *
export use ../yzx/home_manager.nu *

# =============================================================================
# YAZELIX COMMANDS WITH NATIVE SUBCOMMAND SUPPORT
# =============================================================================

# Yazelix Command Suite - Yazi + Zellij + Helix integrated terminal environment
#
# Manage yazelix sessions, run diagnostics, and configure your setup.
# Supports: bash, nushell, fish, zsh
#
# Common commands:
#   yzx launch    - Start a new yazelix window
#   yzx enter     - Start Yazelix in the current terminal
#   yzx run       - Run a command inside the Yazelix environment
#   yzx status    - Show current Yazelix status
#   yzx doctor    - Run health checks

def has_external_command [command_name: string] {
    (which $command_name | where type == "external" | is-not-empty)
}

def print_completed_output [result: record] {
    let stdout_text = ($result.stdout | default "")
    let stderr_text = ($result.stderr | default "")

    if ($stdout_text | is-not-empty) {
        print --raw $stdout_text
    }

    if ($stderr_text | is-not-empty) {
        print --stderr --raw $stderr_text
    }
}

def print_rust_yzx_help [] {
    let helper_path = (resolve_yzx_core_helper_path (get_yazelix_runtime_dir))
    let result = (^$helper_path yzx-command-metadata.help | complete)
    print_completed_output $result
    if $result.exit_code != 0 {
        exit $result.exit_code
    }
}

# Show Yazelix help or version information
export def yzx [
    --version (-V)  # Show Yazelix version
    --version-short (-v)  # Show Yazelix version
] {
    if $version or $version_short {
        print $"Yazelix ($YAZELIX_VERSION)"
        return
    }
    print_rust_yzx_help
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

# Open the Yazelix sponsor page or print its URL
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

# Retarget the current Yazelix tab workspace directory
export def "yzx cwd" [
    target?: string  # Directory path or zoxide query for the current tab workspace root (defaults to the current directory)
] {
    if ($env.ZELLIJ? | is-empty) {
        print "❌ yzx cwd only works inside Zellij."
        print "   Start Yazelix first, then run this command from the tab you want to update."
        exit 1
    }

    let resolved_dir = try {
        resolve_tab_cwd_target $target
    } catch {|err|
        print $"❌ ($err.msg)"
        exit 1
    }

    let editor_kind = ((get_managed_editor_kind) | default "")
    let result = try {
        retarget_tab_cwd $resolved_dir $editor_kind "yzx_cwd.log"
    } catch {|err|
        {
            status: "error"
            reason: $err.msg
        }
    }

    match $result.status {
        "ok" => {
            let sidebar_sync_result = if ($result.sidebar_state? | is-not-empty) {
                sync_sidebar_yazi_state_to_directory $result.sidebar_state $result.workspace_root "yzx_cwd.log"
            } else {
                {status: "skipped", reason: "sidebar_yazi_missing"}
            }
            print $"✅ Updated current tab workspace directory to: ($result.workspace_root)"
            print $"   Tab renamed to: ($result.tab_name)"
            print "   The current pane will switch after this command returns."
            print "   Other existing panes keep their current working directories."
            print "   New managed actions will use the updated tab directory."
            if (($result.editor_status? | default "") == "ok") {
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
            print "   Run `yzx doctor --fix`, then restart Yazelix."
            exit 1
        }
        _ => {
            let reason = ($result.reason? | default "unknown error")
            print $"❌ Failed to update the current tab workspace directory: ($reason)"
            exit 1
        }
    }
}

# Reveal a file or directory in the managed Yazi sidebar
export def "yzx reveal" [
    target: string  # File or directory to reveal in the managed Yazi sidebar
] {
    reveal_in_yazi $target
}

# Canonical inspection command
export def "yzx status" [
    --versions(-V)  # Include tool version matrix
    --json          # Emit machine-readable status data
] {
    let yazelix_dir = (get_yazelix_runtime_dir)
    let report = (collect_status_report $yazelix_dir --include-versions=$versions)

    if $json {
        print ($report | to json -r)
    } else {
        render_status_report $report
    }
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
    write_text_atomic $bootstrap_file ($target_dir | path expand) --raw | ignore
    $bootstrap_file
}

# Restart yazelix
export def "yzx restart" [
] {
    let session_to_kill = get_current_zellij_session
    let restart_sidebar_cwd_file = (create_restart_sidebar_bootstrap_file (pwd))
    let restart_env = {
        YAZELIX_BOOTSTRAP_SIDEBAR_CWD_FILE: $restart_sidebar_cwd_file
    }

    # Detect if we're in a Yazelix-controlled terminal.
    let is_yazelix_terminal = ($env.YAZELIX_TERMINAL? | is-not-empty)

    if $is_yazelix_terminal {
        print "🔄 Restarting Yazelix..."
    } else {
        print "🔄 Restarting Yazelix \(opening new window\)..."
    }

    let stable_wrapper = (resolve_stable_yzx_wrapper_path)
    if $stable_wrapper != null {
        let launch_output = (with-env $restart_env {
            ^$stable_wrapper launch | complete
        })
        if $launch_output.exit_code != 0 {
            print_completed_output $launch_output
            print "❌ Failed to relaunch Yazelix through the stable owner wrapper."
            exit $launch_output.exit_code
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
    --json         # Emit machine-readable doctor data
] {
    use ../utils/doctor.nu [collect_doctor_report run_doctor_checks]
    if $json and $fix {
        error make {msg: "`yzx doctor --json` does not support `--fix` yet. Run `yzx doctor --json` for machine-readable diagnostics or `yzx doctor --fix` for the current interactive repair flow."}
    }

    if $json {
        print ((collect_doctor_report) | to json -r)
    } else if $fix {
        with-env { YAZELIX_ACCEPT_USER_CONFIG_RELOCATION: "true" } {
            run_doctor_checks $verbose $fix
        }
    } else {
        run_doctor_checks $verbose $fix
    }
}
