#!/usr/bin/env nu

export def get_runtime_dir [] {
    let runtime_dir = ($env.YAZELIX_RUNTIME_DIR? | default "" | str trim)
    if ($runtime_dir | is-empty) {
        error make {msg: "Missing YAZELIX_RUNTIME_DIR for Yazelix Zellij helper script."}
    }

    let expanded_runtime_dir = ($runtime_dir | path expand)
    if not ($expanded_runtime_dir | path exists) {
        error make {msg: $"Configured YAZELIX_RUNTIME_DIR does not exist: ($expanded_runtime_dir)"}
    }

    $expanded_runtime_dir
}

export def get_runtime_script_path [relative_path: string] {
    (get_runtime_dir | path join $relative_path)
}

export def get_runtime_nu_path [] {
    let runtime_nu = (get_runtime_script_path "bin/nu")
    if not ($runtime_nu | path exists) {
        error make {msg: $"Yazelix runtime-local nu is missing: ($runtime_nu)"}
    }

    $runtime_nu
}

export def run_runtime_nu_command [command: string, extra_env?: record] {
    let runtime_nu = (get_runtime_nu_path)
    let command_env = ($extra_env | default {})

    with-env $command_env {
        run-external $runtime_nu "-c" $command
    }
}
