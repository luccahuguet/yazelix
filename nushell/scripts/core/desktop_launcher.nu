#!/usr/bin/env nu

# Yazelix Desktop Launcher
# Ensures we're in the yazelix environment and calls launch script directly

def main [] {
    # Set environment
    $env.YAZELIX_DIR = $"($nu.home-path)/.config/yazelix"

    # Change to yazelix directory
    cd $env.YAZELIX_DIR

    # Check if devenv is available (13x faster startup)
    let use_devenv = (which devenv | is-not-empty)

    # Call launch script within nix environment
    # Pass home directory as launch_cwd so desktop entry opens in ~/ instead of yazelix directory
    if $use_devenv {
        # Use devenv for instant shell startup (~0.3s)
        ^devenv shell nu $"($env.YAZELIX_DIR)/nushell/scripts/core/launch_yazelix.nu" $nu.home-path
    } else {
        # Fall back to nix develop (~4-5s)
        ^nix develop --impure --command nu $"($env.YAZELIX_DIR)/nushell/scripts/core/launch_yazelix.nu" $nu.home-path
    }
}

