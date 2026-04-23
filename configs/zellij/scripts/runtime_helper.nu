#!/usr/bin/env nu

use ../../../nushell/scripts/utils/runtime_paths.nu [get_yazelix_runtime_dir]
use ../../../nushell/scripts/utils/runtime_commands.nu [resolve_yazelix_nu_bin]

export def get_runtime_dir [] {
    let expanded_runtime_dir = (get_yazelix_runtime_dir | path expand)
    if not ($expanded_runtime_dir | path exists) {
        error make {msg: $"Resolved Yazelix runtime directory does not exist: ($expanded_runtime_dir)"}
    }

    $expanded_runtime_dir
}

export def get_runtime_script_path [relative_path: string] {
    let script_path = (get_runtime_dir | path join $relative_path)
    let expanded_script_path = ($script_path | path expand)

    if not ($expanded_script_path | path exists) {
        error make {msg: $"Resolved Yazelix runtime script does not exist: ($expanded_script_path)"}
    }

    if (($expanded_script_path | path type) != "file") {
        error make {msg: $"Resolved Yazelix runtime script is not a file: ($expanded_script_path)"}
    }

    $expanded_script_path
}

export def get_runtime_nu_path [] {
    let runtime_nu = (resolve_yazelix_nu_bin)
    if not ($runtime_nu | path exists) {
        error make {msg: $"Resolved Yazelix Nushell binary does not exist: ($runtime_nu)"}
    }

    $runtime_nu
}

export def run_runtime_nu_script [
    relative_script_path: string
    ...script_args: string
    --extra-env: record = {}
] {
    let runtime_nu = (get_runtime_nu_path)
    let runtime_script = (get_runtime_script_path $relative_script_path)

    with-env $extra_env {
        run-external $runtime_nu $runtime_script ...$script_args
    }
}
