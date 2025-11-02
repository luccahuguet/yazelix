#!/usr/bin/env nu

# Yazelix Desktop Launcher
# Ensures we're in the yazelix environment and calls launch script directly

def main [] {
    # Set environment
    let yazelix_dir = $"($nu.home-path)/.config/yazelix"
    $env.YAZELIX_DIR = $yazelix_dir

    if (which devenv | is-empty) {
        print "❌ devenv command not found - install devenv to launch Yazelix."
        print "   See https://devenv.sh/getting-started/ for installation instructions."
        exit 1
    }

    # Call launch script within devenv environment
    # Pass home directory as launch_cwd so desktop entry opens in ~/ instead of yazelix directory
    let devenv_cmd = $"cd ($yazelix_dir) && devenv shell -- nu ($yazelix_dir)/nushell/scripts/core/launch_yazelix.nu ($nu.home-path)"
    ^bash -c $devenv_cmd
}
