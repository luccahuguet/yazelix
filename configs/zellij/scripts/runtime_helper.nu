#!/usr/bin/env nu

use ../../../nushell/scripts/utils/common.nu [get_yazelix_runtime_dir resolve_yazelix_nu_bin]

export def get_runtime_dir [] {
    let expanded_runtime_dir = (get_yazelix_runtime_dir | path expand)
    if not ($expanded_runtime_dir | path exists) {
        error make {msg: $"Resolved Yazelix runtime directory does not exist: ($expanded_runtime_dir)"}
    }

    $expanded_runtime_dir
}

export def get_runtime_script_path [relative_path: string] {
    (get_runtime_dir | path join $relative_path)
}

export def get_runtime_nu_path [] {
    let runtime_nu = (resolve_yazelix_nu_bin)
    if not ($runtime_nu | path exists) {
        error make {msg: $"Resolved Yazelix Nushell binary does not exist: ($runtime_nu)"}
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
