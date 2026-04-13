#!/usr/bin/env nu
# yzx run command - Run a command inside Yazelix environment without UI

use ../utils/environment_bootstrap.nu [prepare_environment]
use ../utils/runtime_env.nu run_runtime_argv

# Run a command in the Yazelix environment and exit
export def --wrapped "yzx run" [
    ...argv: string    # External argv; first token is the command and the rest pass through unchanged
] {
    let env_prep = prepare_environment
    let config = $env_prep.config
    let original_dir = (pwd)

    if ($argv | is-empty) {
        print "Error: No command provided"
        print "Usage: yzx run <command> [args...]"
        exit 1
    }

    run_runtime_argv $argv --cwd $original_dir --config $config
}
