#!/usr/bin/env nu

# Yazelix Desktop Launcher
# Ensures we're in the yazelix environment and calls launch script directly

def main [] {
    # Set environment
    let yazelix_dir = $"($nu.home-path)/.config/yazelix"
    $env.YAZELIX_DIR = $yazelix_dir

    # Check if devenv is available (13x faster startup)
    let use_devenv = (which devenv | is-not-empty)

    # Call launch script within nix environment
    # Pass home directory as launch_cwd so desktop entry opens in ~/ instead of yazelix directory
    if $use_devenv {
        # Use devenv for instant shell startup (~0.3s)
        # Must run devenv from the directory containing devenv.nix
        let devenv_cmd = $"cd ($yazelix_dir) && devenv shell nu ($yazelix_dir)/nushell/scripts/core/launch_yazelix.nu ($nu.home-path)"
        ^bash -c $devenv_cmd
    } else {
        # Fall back to nix develop (~4-5s)
        cd $yazelix_dir
        ^nix develop --impure --command nu $"($yazelix_dir)/nushell/scripts/core/launch_yazelix.nu" $nu.home-path
    }
}

