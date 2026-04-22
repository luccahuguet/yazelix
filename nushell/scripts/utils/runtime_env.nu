#!/usr/bin/env nu
# Runtime environment helpers for the trimmed Yazelix entry surface.

use ./yzx_core_bridge.nu compute_runtime_env_via_yzx_core

export def run_runtime_argv [
    argv: list<string>
    --cwd: string = ""
    --config: record
    --runtime-env: record
] {
    if ($argv | is-empty) {
        error make {msg: "No command provided"}
    }

    let command = ($argv | first)
    let args = ($argv | skip 1)
    let requested_cwd = $cwd
    let resolved_runtime_env = if $runtime_env != null {
        $runtime_env
    } else if $config == null {
        compute_runtime_env_via_yzx_core
    } else {
        compute_runtime_env_via_yzx_core $config
    }

    with-env $resolved_runtime_env {
        if ($requested_cwd | is-not-empty) {
            cd ($requested_cwd | path expand)
        }
        ^$command ...$args
    }
}
