#!/usr/bin/env nu
# Runtime environment helpers for the trimmed Yazelix entry surface.

use ./common.nu get_yazelix_runtime_dir
use ./config_parser.nu [parse_yazelix_config run_yzx_core_json_command]

const RUNTIME_ENV_COMPUTE_COMMAND = "runtime-env.compute"

def build_runtime_env_request [runtime_dir: string, config: record] {
    {
        runtime_dir: $runtime_dir
        home_dir: ($env.HOME? | default "")
        current_path: ($env.PATH? | default [])
        enable_sidebar: ($config.enable_sidebar? | default true)
        editor_command: ($config.editor_command? | default null)
        helix_runtime_path: ($config.helix_runtime_path? | default null)
    }
}

def runtime_env_error_surface [config: record] {
    let config_file = ($config.config_file? | default "")
    {
        display_config_path: $config_file
        config_file: $config_file
    }
}

export def get_runtime_env [config?: record] {
    let resolved_config = if $config == null {
        parse_yazelix_config
    } else {
        $config
    }
    let runtime_dir = (get_yazelix_runtime_dir)
    if $runtime_dir == null {
        error make {msg: "Could not resolve the Yazelix runtime directory for the runtime env contract."}
    }
    let request = (build_runtime_env_request $runtime_dir $resolved_config)
    let data = (run_yzx_core_json_command
        $runtime_dir
        (runtime_env_error_surface $resolved_config)
        [
            $RUNTIME_ENV_COMPUTE_COMMAND
            "--request-json"
            ($request | to json -r)
        ]
        "Yazelix Rust runtime-env helper returned invalid JSON.")
    $data.runtime_env
}

export def --env activate_runtime_env [config?: record] {
    load-env (get_runtime_env $config)
}

export def run_runtime_argv [
    argv: list<string>
    --cwd: string = ""
    --config: record
] {
    if ($argv | is-empty) {
        error make {msg: "No command provided"}
    }

    let command = ($argv | first)
    let args = ($argv | skip 1)
    let requested_cwd = $cwd
    let runtime_env = if $config == null {
        get_runtime_env
    } else {
        get_runtime_env $config
    }

    with-env $runtime_env {
        if ($requested_cwd | is-not-empty) {
            cd ($requested_cwd | path expand)
        }
        ^$command ...$args
    }
}
