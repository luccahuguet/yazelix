#!/usr/bin/env nu

# Yazelix Desktop Launcher
# Ensures we're in the yazelix environment and calls launch script directly

def main [] {
    # Set environment
    $env.YAZELIX_DIR = $"($nu.home-path)/.config/yazelix"

    # Change to yazelix directory
    cd $env.YAZELIX_DIR

    # Call launch script directly within nix environment
    # Pass home directory as launch_cwd so desktop entry opens in ~/ instead of yazelix directory
    ^nix develop --impure --command nu $"($env.YAZELIX_DIR)/nushell/scripts/core/launch_yazelix.nu" $nu.home-path
}

