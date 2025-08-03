#!/usr/bin/env nu
# ~/.config/yazelix/nushell/scripts/core/start_yazelix.nu

use ../utils/config_parser.nu parse_yazelix_config

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

    # Build the appropriate zellij command based on persistent session setting
    let cmd = if ($config.persistent_sessions == "true") {
        # Use zellij attach with create flag for persistent sessions
        [
            "zellij"
            "--config-dir" $"($yazelix_dir)/configs/zellij"
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
            "zellij"
            "--config-dir" $"($yazelix_dir)/configs/zellij"
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