#!/usr/bin/env nu
# yzx run command - Run a command inside Yazelix environment without UI

use ../utils/environment_bootstrap.nu [prepare_environment run_in_devenv_shell_command]
use ../utils/config_state.nu [mark_config_state_applied]
use ../utils/system_mode.nu [assert_no_packs require_command]

# Run a command in the Yazelix environment and exit
export def "yzx run" [
    --verbose          # Enable verbose logging
    command: string    # Command to run
    ...args: string    # Command arguments (quote args that start with '-')
] {
    let env_prep = prepare_environment
    let config = $env_prep.config
    let env_mode = ($config.environment_mode? | default "nix")
    let needs_refresh = $env_prep.needs_refresh
    let original_dir = (pwd)

    if $env_mode == "system" {
        assert_no_packs $config
        if (which env | is-empty) {
            print "Error: env command not found - cannot run command in system mode"
            exit 1
        }
    } else {
        use ../utils/nix_detector.nu ensure_nix_available
        ensure_nix_available
    }

    if ($command | is-empty) {
        print "Error: No command provided"
        print "Usage: yzx run <command> [args...]"
        exit 1
    }

    if $env_mode == "system" {
        require_command $command "command"
        ^env -C $original_dir $command ...$args
    } else if $verbose {
        run_in_devenv_shell_command $command ...$args --cwd $original_dir --env-only --skip-welcome --verbose --force-refresh=$needs_refresh
    } else {
        run_in_devenv_shell_command $command ...$args --cwd $original_dir --env-only --skip-welcome --quiet --force-refresh=$needs_refresh
    }

    if ($env_mode != "system") and $needs_refresh {
        mark_config_state_applied $env_prep.config_state
    }
}
