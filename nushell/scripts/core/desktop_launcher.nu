#!/usr/bin/env nu

# Yazelix Desktop Launcher
# Launch through the same prepared devenv path as normal `yzx launch`.

use ../utils/environment_bootstrap.nu [prepare_environment run_in_devenv_shell_command]
use ../utils/common.nu [get_yazelix_runtime_dir]
use ../utils/failure_classes.nu [format_failure_classification]

def require_launch_script [script_path: string] {
    let resolved = ($script_path | path expand)
    if not ($resolved | path exists) {
        let classification = (format_failure_classification "generated-state" "Restore the missing launcher script, or reinstall/regenerate Yazelix and try again.")
        error make {msg: $"Missing Yazelix desktop launcher: ($resolved)\nYour runtime looks incomplete. Reinstall/regenerate Yazelix and try again.\n($classification)"}
    }

    $resolved
}

def main [] {
    let runtime_dir = (get_yazelix_runtime_dir)
    let launch_script = (require_launch_script ($runtime_dir | path join "nushell" "scripts" "core" "launch_yazelix.nu"))
    let env_prep = prepare_environment
    let config = $env_prep.config
    let max_jobs = ($config.max_jobs? | default "half" | into string)
    let build_cores = ($config.build_cores? | default "2" | into string)
    let refresh_output = ($config.refresh_output? | default "normal" | into string)

    run_in_devenv_shell_command "nu" $launch_script $env.HOME --cwd $env.HOME --runtime-dir $runtime_dir --skip-welcome --quiet --max-jobs $max_jobs --build-cores $build_cores --force-refresh=$env_prep.needs_refresh --refresh-output-mode $refresh_output
}
