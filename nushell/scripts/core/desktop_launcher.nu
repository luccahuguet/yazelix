#!/usr/bin/env nu

# Yazelix Desktop Launcher
# Delegate to the same `yzx launch` command path used everywhere else.

use ../utils/environment_bootstrap.nu [prepare_environment run_in_devenv_shell_command]
use ../utils/common.nu [require_installed_yazelix_runtime_dir]

export def main [] {
    let runtime_dir = (require_installed_yazelix_runtime_dir)
    let core_script = ($runtime_dir | path join "nushell" "scripts" "core" "yazelix.nu")
    let env_prep = prepare_environment
    let config = $env_prep.config
    let max_jobs = ($config.max_jobs? | default "half" | into string)
    let build_cores = ($config.build_cores? | default "2" | into string)
    let refresh_output = ($config.refresh_output? | default "normal" | into string)
    let launch_command = $"use '($core_script)' *; yzx launch --home"

    run_in_devenv_shell_command "nu" "-c" $launch_command --cwd $env.HOME --runtime-dir $runtime_dir --skip-welcome --quiet --max-jobs $max_jobs --build-cores $build_cores --force-shell --force-refresh=$env_prep.needs_refresh --refresh-output-mode $refresh_output
}
