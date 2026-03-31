#!/usr/bin/env nu

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

def main [...popup_args: string] {
    let core_script = (get_runtime_script_path "nushell/scripts/core/yazelix.nu")
    let popup_args_json = ($popup_args | to json -r)
    let command = ([
        $"use '($core_script)' *"
        "let popup_args = ($env.YAZELIX_POPUP_ARGS_JSON | from json)"
        "yzx popup ...$popup_args"
    ] | str join "\n")
    if ($env.ZELLIJ? | is-not-empty) {
        ^zellij action rename-pane "yzx_popup" | complete | ignore
    }
    with-env {
        YAZELIX_POPUP_PANE: "true"
        YAZELIX_POPUP_ARGS_JSON: $popup_args_json
    } {
        run-external nu "-c" $command
    }
}
