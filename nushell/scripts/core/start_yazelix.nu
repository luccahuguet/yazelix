#!/usr/bin/env nu
# ~/.config/yazelix/nushell/scripts/core/start_yazelix.nu

use ../utils/config_parser.nu parse_yazelix_config
use ../utils/constants.nu [ZELLIJ_CONFIG_PATHS, YAZI_CONFIG_PATHS]
use ../utils/nix_detector.nu ensure_nix_available
use ../setup/zellij_config_merger.nu generate_merged_zellij_config
use ../setup/yazi_config_merger.nu generate_merged_yazi_config

export def main [cwd_override?: string, --verbose] {
    # Try to set up Nix environment automatically when outside Yazelix/nix shells
    use ../utils/nix_env_helper.nu ensure_nix_in_environment

    let already_in_env = (
        ($env.IN_YAZELIX_SHELL? == "true")
        or ($env.IN_NIX_SHELL? | is-not-empty)
    )

    if not $already_in_env {
        # If automatic setup fails, fall back to the detector with user interaction
        if not (ensure_nix_in_environment) {
            ensure_nix_available
        }
    }

    let verbose_mode = $verbose or ($env.YAZELIX_VERBOSE? == "true")
    if $verbose_mode {
        print "🔍 start_yazelix: verbose mode enabled"
    }

    # Resolve HOME using Nushell's built-in
    let home = $env.HOME
    if ($home | is-empty) or (not ($home | path exists)) {
        print "Error: Cannot resolve HOME directory"
        exit 1
    }

    # Set absolute path for Yazelix directory
    let yazelix_dir = $"($home)/.config/yazelix"

    # Navigate to Yazelix directory
    # This is important for nix develop to find the flake.nix in the current directory
    if not ($yazelix_dir | path exists) {
        print $"Error: Cannot find Yazelix directory at ($yazelix_dir)"
        exit 1
    }

    cd $yazelix_dir

    # Parse configuration using the shared module
    let config = parse_yazelix_config

    # Generate merged Yazi configuration (doesn't need zellij)
    print "🔧 Preparing Yazi configuration..."
    let merged_yazi_dir = if $verbose_mode {
        generate_merged_yazi_config $yazelix_dir
    } else {
        generate_merged_yazi_config $yazelix_dir --quiet
    }
    
    # For Zellij config, create a placeholder for now - will be generated inside Nix environment
    let merged_zellij_dir = $"($env.HOME)/.local/share/yazelix/configs/zellij"

    # Determine which directory to use as default CWD
    # Priority: 1. cwd_override parameter 2. YAZELIX_LAUNCH_CWD env var 3. current directory 4. home
    let working_dir = if ($cwd_override | is-not-empty) {
        $cwd_override
    } else if ($env.YAZELIX_LAUNCH_CWD? | is-not-empty) {
        $env.YAZELIX_LAUNCH_CWD
    } else {
        pwd
    }

    # Build the command that first generates the zellij config, then starts zellij
    let zellij_merger_cmd = $"nu ($yazelix_dir)/nushell/scripts/setup/zellij_config_merger.nu ($yazelix_dir)"

    # Check for layout override (for testing)
    let layout = if ($env.ZELLIJ_DEFAULT_LAYOUT? | is-not-empty) {
        $env.ZELLIJ_DEFAULT_LAYOUT
    } else {
        "$ZELLIJ_DEFAULT_LAYOUT"  # Will be expanded by bash in nix shell
    }

    let cmd = if ($config.persistent_sessions == "true") {
        # Use zellij attach with create flag for persistent sessions
        [
            $zellij_merger_cmd "&&"
            "zellij"
            "--config-dir" $merged_zellij_dir
            "attach"
            "-c" $config.session_name
            "options"
            "--default-cwd" $working_dir
            "--default-layout" $layout
            "--default-shell" $config.default_shell
        ] | str join " "
    } else {
        # Use zellij options for new sessions (original behavior)
        [
            $zellij_merger_cmd "&&"
            "zellij"
            "--config-dir" $merged_zellij_dir
            "options"
            "--default-cwd" $working_dir
            "--default-layout" $layout
            "--default-shell" $config.default_shell
        ] | str join " "
    }

    if $verbose_mode {
        print $"🔁 zellij command: ($cmd)"
    }

    # Run nix develop with explicit HOME.
    # The default shell is dynamically read from yazelix.nix configuration
    # and passed directly to the zellij command.
    # Guard against recursive nix develop calls when already in a nix shell
    with-env {HOME: $home} {
        let in_nix_shell = ($env.IN_NIX_SHELL? | is-not-empty)
        let in_yazelix_shell = ($env.IN_YAZELIX_SHELL? == "true")

        if $verbose_mode {
            print $"🔁 IN_NIX_SHELL? ($in_nix_shell) | IN_YAZELIX_SHELL? ($in_yazelix_shell)"
            if ($in_nix_shell or $in_yazelix_shell) {
                print "⚙️ Executing zellij command directly"
            } else {
                print "⚙️ Entering nix develop before running zellij command"
            }
        }

        if ($in_nix_shell or $in_yazelix_shell) {
            # Already in nix shell, run command directly to avoid recursive nesting
            ^bash -c $cmd
        } else {
            # Not in nix shell, enter it first
            ^nix develop --impure --command bash -c $cmd
        }
    }
}
