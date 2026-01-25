#!/usr/bin/env nu

# Yazelix Desktop Launcher
# Ensures we're in the yazelix environment and calls launch script directly

use ../utils/environment_bootstrap.nu *
use ../utils/config_state.nu [mark_config_state_applied]
use ../utils/system_mode.nu [assert_no_packs]

def main [] {
    # Ensure Nix environment is available (shared with yzx commands)
    ensure_environment_available

    # Prepare environment and get config state (shared logic)
    let env_prep = prepare_environment
    let config = $env_prep.config
    let needs_refresh = $env_prep.needs_refresh
    let env_mode = ($config.environment_mode? | default "nix")

    # Build launch command - open in home directory instead of yazelix directory
    let yazelix_dir = $"($env.HOME)/.config/yazelix"
    let launch_command = $"nu ($yazelix_dir)/nushell/scripts/core/launch_yazelix.nu ($env.HOME)"

    if $env_mode == "system" {
        assert_no_packs $config
        ^nu $"($yazelix_dir)/nushell/scripts/core/launch_yazelix.nu" $env.HOME
        return
    }

    # Run launch script in devenv environment (shared devenv runner)
    run_in_devenv_shell $launch_command --force-refresh=$needs_refresh

    if $needs_refresh {
        mark_config_state_applied $env_prep.config_state
    }
}
