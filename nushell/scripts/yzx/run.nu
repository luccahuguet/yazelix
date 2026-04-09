#!/usr/bin/env nu
# yzx run command - Run a command inside Yazelix environment without UI

use ../utils/environment_bootstrap.nu [prepare_environment]
use ../utils/devenv_backend.nu [run_in_devenv_shell_command]
use ../utils/config_state.nu [record_materialized_state]

# Run a command in the Yazelix environment and exit
export def "yzx run" [
    --verbose          # Enable verbose logging
    command: string    # Command to run
    ...args: string    # Command arguments (quote args that start with '-')
] {
    use ../utils/nix_detector.nu ensure_nix_available
    ensure_nix_available

    let env_prep = prepare_environment
    let config = $env_prep.config
    let needs_refresh = $env_prep.needs_refresh
    let max_jobs = ($config.max_jobs? | default "half" | into string)
    let build_cores = ($config.build_cores? | default "2" | into string)
    let original_dir = (pwd)

    if ($command | is-empty) {
        print "Error: No command provided"
        print "Usage: yzx run <command> [args...]"
        exit 1
    }

    if $verbose {
        run_in_devenv_shell_command $command ...$args --max-jobs $max_jobs --build-cores $build_cores --cwd $original_dir --env-only --skip-welcome --verbose --force-refresh=$needs_refresh
    } else {
        run_in_devenv_shell_command $command ...$args --max-jobs $max_jobs --build-cores $build_cores --cwd $original_dir --env-only --skip-welcome --quiet --force-refresh=$needs_refresh
    }

    if $needs_refresh {
        record_materialized_state $env_prep.config_state
    }
}
