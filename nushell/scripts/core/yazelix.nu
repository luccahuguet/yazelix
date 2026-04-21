#!/usr/bin/env nu
# Yazelix root help/version surface plus re-exported command families.

use ../utils/constants.nu *
use ../utils/common.nu get_yazelix_runtime_dir
use ../utils/config_parser.nu resolve_yzx_core_helper_path

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
export use ./yzx_doctor.nu *
export use ./yzx_session.nu *
export use ./yzx_support.nu *
export use ./yzx_workspace.nu *

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
