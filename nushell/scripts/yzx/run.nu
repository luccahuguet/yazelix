#!/usr/bin/env nu
# yzx run command - Run a command inside Yazelix environment without UI

use ../utils/environment_bootstrap.nu [prepare_environment run_in_devenv_shell_command]
use ../utils/config_state.nu [mark_config_state_applied]

# Run a command in the Yazelix environment and exit
export def "yzx run" [
    --verbose          # Enable verbose logging
    command: string    # Command to run
    ...args: string    # Command arguments (quote args that start with '-')
] {
    use ../utils/nix_detector.nu ensure_nix_available
    ensure_nix_available

    let env_prep = prepare_environment
    let needs_refresh = $env_prep.needs_refresh
    let original_dir = (pwd)

    if ($command | is-empty) {
        print "Error: No command provided"
        print "Usage: yzx run <command> [args...]"
        exit 1
    }

    if $verbose {
        run_in_devenv_shell_command $command ...$args --cwd $original_dir --env-only --skip-welcome --verbose --force-refresh=$needs_refresh
    } else {
        run_in_devenv_shell_command $command ...$args --cwd $original_dir --env-only --skip-welcome --quiet --force-refresh=$needs_refresh
    }

    if $needs_refresh {
        mark_config_state_applied $env_prep.config_state
    }
}
