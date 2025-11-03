#!/usr/bin/env nu

# Yazelix Desktop Launcher
# Ensures we're in the yazelix environment and calls launch script directly

use ../utils/config_state.nu [compute_config_state mark_config_state_applied]
use ../utils/common.nu [get_max_cores]

def main [] {
    # Set environment
    let yazelix_dir = $"($nu.home-path)/.config/yazelix"
    $env.YAZELIX_DIR = $yazelix_dir

    if (which devenv | is-empty) {
        print "❌ devenv command not found - install devenv to launch Yazelix."
        print "   See https://devenv.sh/getting-started/ for installation instructions."
        exit 1
    }

    let config_state = compute_config_state
    let needs_refresh = $config_state.needs_refresh

    if $needs_refresh {
        $env.YAZELIX_FORCE_REFRESH = "true"
    }

    # Call launch script within devenv environment
    # Pass home directory as launch_cwd so desktop entry opens in ~/ instead of yazelix directory
    let max_cores = get_max_cores
    let devenv_cmd = $"cd ($yazelix_dir) && devenv --impure --cores ($max_cores) shell -- nu ($yazelix_dir)/nushell/scripts/core/launch_yazelix.nu ($nu.home-path)"
    ^bash -c $devenv_cmd
    if $needs_refresh {
        mark_config_state_applied $config_state
    }
}
