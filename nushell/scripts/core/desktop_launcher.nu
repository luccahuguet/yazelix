#!/usr/bin/env nu

# Yazelix Desktop Launcher
# Launch the terminal window immediately, but keep the resolved Yazelix
# profile environment so wrapper binaries and terminal runtime selection
# behave the same as normal launches.

use ../utils/environment_bootstrap.nu [prepare_environment]
use ../utils/launch_state.nu [get_launch_env get_launch_profile resolve_built_profile]

def require_launch_script [script_path: string] {
    let resolved = ($script_path | path expand)
    if not ($resolved | path exists) {
        error make {msg: $"Missing Yazelix desktop launcher: ($resolved)\nYour runtime looks incomplete. Reinstall/regenerate Yazelix and try again."}
    }

    $resolved
}

def main [] {
    let launch_script = (require_launch_script ($env.HOME | path join ".config" "yazelix" "nushell" "scripts" "core" "launch_yazelix.nu"))
    let env_prep = prepare_environment
    let config = $env_prep.config
    let cached_profile = (get_launch_profile $env_prep.config_state)
    let bootstrap_profile = if $cached_profile != null {
        $cached_profile
    } else {
        resolve_built_profile
    }

    if ($bootstrap_profile | is-not-empty) {
        with-env (get_launch_env $config $bootstrap_profile) {
            ^nu $launch_script $env.HOME
        }
    } else {
        ^nu $launch_script $env.HOME
    }
}
