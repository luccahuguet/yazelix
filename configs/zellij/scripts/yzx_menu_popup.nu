#!/usr/bin/env nu
# Wrapper script for yzx menu popup (called from Zellij keybind)

def get_runtime_script_path [relative_path: string] {
    let runtime_dir = ($env.YAZELIX_RUNTIME_DIR? | default "" | str trim)
    if ($runtime_dir | is-empty) {
        error make {msg: "Missing YAZELIX_RUNTIME_DIR for Yazelix Zellij helper script."}
    }

    let expanded_runtime_dir = ($runtime_dir | path expand)
    if not ($expanded_runtime_dir | path exists) {
        error make {msg: $"Configured YAZELIX_RUNTIME_DIR does not exist: ($expanded_runtime_dir)"}
    }

    ($expanded_runtime_dir | path join $relative_path)
}

let core_script = (get_runtime_script_path "nushell/scripts/core/yazelix.nu")
let command = $"use '($core_script)' *; yzx menu"

with-env {YAZELIX_MENU_POPUP: "true"} {
    run-external nu "-c" $command
}
