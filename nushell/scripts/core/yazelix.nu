#!/usr/bin/env nu
# Yazelix Command Suite
# Consolidated commands for managing and interacting with yazelix

use ../utils/atomic_writes.nu write_text_atomic
use ../utils/constants.nu *
use ../utils/common.nu get_yazelix_runtime_dir
use ../utils/install_ownership_report.nu evaluate_install_ownership_report
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

def print_update_owner_warning [] {
    print "Choose one update owner for this Yazelix install."
    print ""
    print "  Use `yzx update upstream` if this install is owned by a Nix profile package."
    print "  Use `yzx update home_manager` if Home Manager owns this install."
    print ""
    print "Do not use both update paths for the same installed Yazelix runtime."
}

def print_update_path_confirmation [owner: string] {
    match $owner {
        "upstream" => {
            print "Requested update path: default Nix profile."
            print ""
            print "  Use this only when a Nix profile package owns the active Yazelix runtime."
        }
        "home_manager" => {
            print "Requested update path: Home Manager flake input."
            print ""
            print "  Use this only when Home Manager owns the active Yazelix runtime."
        }
        _ => {
            error make {msg: $"Unsupported update owner confirmation: ($owner)"}
        }
    }

    print ""
    print "Do not use both update paths for the same installed Yazelix runtime."
}

def fail_if_home_manager_owned_upstream_update [] {
    let install_report = (evaluate_install_ownership_report)
    if $install_report.install_owner != "home-manager" {
        return
    }

    print "❌ `yzx update upstream` is for default Nix profile installs, but this Yazelix runtime appears to be Home Manager-owned."
    print "   Run `yzx update home_manager` from the Home Manager flake that owns this install."
    print "   Then run `home-manager switch` to apply the updated input."
    print "   Do not use both update paths for the same installed Yazelix runtime."
    exit 1
}

def load_default_profile_elements [] {
    let result = (^nix profile list --json | complete)
    if $result.exit_code != 0 {
        print "❌ Failed to inspect the default Nix profile."
        print_completed_output $result
        exit 1
    }

    let profile_json = try {
        $result.stdout | from json
    } catch {|err|
        print $"❌ Failed to parse `nix profile list --json`: ($err.msg)"
        exit 1
    }

    $profile_json | get -o elements | default {}
}

def resolve_active_yazelix_profile_entry [] {
    let runtime_root = (get_yazelix_runtime_dir | path expand)
    let elements = (load_default_profile_elements)
    let matches = (
        $elements
        | transpose name entry
        | where {|row|
            let store_paths = ($row.entry | get -o storePaths | default [])
            $store_paths | any {|store_path| ($store_path | path expand) == $runtime_root }
        }
    )

    if (($matches | length) == 1) {
        return ($matches | first)
    }

    if (($matches | length) > 1) {
        let names = ($matches | get name | str join ", ")
        print $"❌ Multiple default-profile Yazelix entries point at the active runtime: ($names)"
        print "   Keep one clear profile owner, then rerun `yzx update upstream`."
        exit 1
    }

    print "❌ `yzx update upstream` could not find the active Yazelix runtime in the default Nix profile."
    print $"   Current runtime: ($runtime_root)"
    print "   This command now updates profile-installed Yazelix packages after the legacy flake installer was removed."
    print "   Recovery: Reinstall with `nix profile add github:luccahuguet/yazelix#yazelix`, or use `yzx update home_manager` if Home Manager owns this install."
    exit 1
}

def print_exact_command [command: string] {
    print "Running:"
    print $"  ($command)"
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

def require_current_working_flake [] {
    let flake_file = ((pwd) | path join "flake.nix")

    if not ($flake_file | path exists) {
        print "❌ yzx update home_manager must be run from the Home Manager flake directory that owns this install."
        print $"   Missing flake.nix in the current directory: ($flake_file)"
        exit 1
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

# Update dependencies and inputs
export def "yzx update" [] {
    print_update_owner_warning
    print ""
    print "Available update commands:"
    print "  yzx update upstream      Upgrade the active Yazelix package in the default Nix profile"
    print "  yzx update home_manager  Refresh the current Home Manager flake input, then print `home-manager switch`"
    print "  yzx update nix           Upgrade Determinate Nix \(if installed\)"
}

# Upgrade Determinate Nix through determinate-nixd
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

# Refresh the active Yazelix package in the default Nix profile
export def "yzx update upstream" [] {
    if not (has_external_command "nix") {
        print "❌ nix not found in PATH."
        print "   Install Nix first, then try again."
        exit 1
    }

    fail_if_home_manager_owned_upstream_update
    print_update_path_confirmation "upstream"
    print ""
    let profile_entry = (resolve_active_yazelix_profile_entry)
    let command = $"nix profile upgrade --refresh ($profile_entry.name)"
    print_exact_command $command

    let result = (^nix profile upgrade --refresh $profile_entry.name | complete)
    print_completed_output $result

    if $result.exit_code != 0 {
        print "❌ Upstream Yazelix update failed."
        exit 1
    }
}

# Refresh the current Home Manager flake input for Yazelix
export def "yzx update home_manager" [] {
    if not (has_external_command "nix") {
        print "❌ nix not found in PATH."
        print "   Install Nix first, then try again."
        exit 1
    }

    require_current_working_flake

    print_update_path_confirmation "home_manager"
    print ""
    print "⚠️  `yzx update home_manager` updates the `yazelix` input in the current flake directory."
    print "   Run it only from the Home Manager flake that owns this install."
    print "   If your Yazelix input uses a different name, run `nix flake update <your-input-name>` yourself."
    print ""
    let command = "nix flake update yazelix"
    print_exact_command $command

    let result = (^nix flake update yazelix | complete)
    print_completed_output $result

    if $result.exit_code != 0 {
        print "❌ Home Manager flake input update failed."
        exit 1
    }

    print ""
    print "Next step:"
    print "  home-manager switch"
}
