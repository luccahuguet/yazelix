#!/usr/bin/env nu

use runtime_helper.nu [get_runtime_script_path run_runtime_nu_command]

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
        run_runtime_nu_command $command
    }
}
