#!/usr/bin/env nu
# ~/.config/yazelix/nushell/scripts/start_yazelix.nu

def main [] {
    # Resolve HOME using Nushell's built-in
    let home = $env.HOME
    if ($home | is-empty) or (not ($home | path exists)) {
        print "Error: Cannot resolve HOME directory"
        exit 1
    }

    print $"Resolved HOME=($home)"

    # Set absolute path for Yazelix directory
    let yazelix_dir = $"($home)/.config/yazelix"

    # Navigate to Yazelix directory
    # This is important for nix develop to find the flake.nix in the current directory
    if not ($yazelix_dir | path exists) {
        print $"Error: Cannot find Yazelix directory at ($yazelix_dir)"
        exit 1
    }

    cd $yazelix_dir

    # Run nix develop with explicit HOME.
    # The YAZELIX_DEFAULT_SHELL variable will be set by the shellHook of the flake
    # and used by the inner zellij command.
    # We use bash -c '...' to ensure $YAZELIX_DEFAULT_SHELL is expanded after nix develop sets it.
    let cmd = $"zellij --config-dir \"($yazelix_dir)/zellij\" options --default-cwd \"($home)\" --default-layout yazelix --default-shell \"$YAZELIX_DEFAULT_SHELL\""
    
    with-env {HOME: $home} {
        ^nix develop --impure --command bash -c $cmd
    }
}

# Export the main function so it can be called
export def start_yazelix [] {
    main
} 