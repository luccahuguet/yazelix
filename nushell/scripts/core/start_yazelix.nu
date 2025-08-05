#!/usr/bin/env nu
# ~/.config/yazelix/nushell/scripts/core/start_yazelix.nu

use ../utils/config_parser.nu parse_yazelix_config
use ../utils/constants.nu [ZELLIJ_CONFIG_PATHS, YAZI_CONFIG_PATHS]
use ../setup/zellij_config_merger.nu generate_merged_zellij_config
use ../setup/yazi_config_merger.nu generate_merged_yazi_config

export def main [] {
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
    let merged_yazi_dir = generate_merged_yazi_config $yazelix_dir
    
    # For Zellij config, create a placeholder for now - will be generated inside Nix environment
    let merged_zellij_dir = $"($env.HOME)/.local/share/yazelix/configs/zellij"

    # Build the command that first generates the zellij config, then starts zellij
    let zellij_merger_cmd = $"nu ($yazelix_dir)/nushell/scripts/setup/zellij_config_merger.nu ($yazelix_dir)"
    
    let cmd = if ($config.persistent_sessions == "true") {
        # Use zellij attach with create flag for persistent sessions
        [
            $zellij_merger_cmd "&&"
            "zellij"
            "--config-dir" $merged_zellij_dir
            "attach"
            "-c" $config.session_name
            "options"
            "--default-cwd" $home
            "--default-layout" "$ZELLIJ_DEFAULT_LAYOUT"
            "--default-shell" "$YAZELIX_DEFAULT_SHELL"
        ] | str join " "
    } else {
        # Use zellij options for new sessions (original behavior)
        [
            $zellij_merger_cmd "&&"
            "zellij"
            "--config-dir" $merged_zellij_dir
            "options"
            "--default-cwd" $home
            "--default-layout" "$ZELLIJ_DEFAULT_LAYOUT"
            "--default-shell" "$YAZELIX_DEFAULT_SHELL"
        ] | str join " "
    }

    # Run nix develop with explicit HOME.
    # The YAZELIX_DEFAULT_SHELL variable will be set by the shellHook of the flake
    # and used by the inner zellij command.
    # We use bash -c '...' to ensure $YAZELIX_DEFAULT_SHELL is expanded after nix develop sets it.
    with-env {HOME: $home} {
        ^nix develop --impure --command bash -c $cmd
    }
}