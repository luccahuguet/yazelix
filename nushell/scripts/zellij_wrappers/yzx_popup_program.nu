#!/usr/bin/env nu

use ../integrations/yazi.nu [refresh_active_sidebar_yazi]
use ../utils/config_parser.nu [parse_yazelix_config]
use ../utils/runtime_env.nu [get_runtime_env run_runtime_argv]
use ../utils/transient_pane_contract.nu [
    close_current_transient_pane
    get_transient_pane_mode_env
    rename_current_transient_pane
]

def resolve_popup_program [popup_args: list<string>] {
    if ($popup_args | is-not-empty) {
        return $popup_args
    }

    let config = (parse_yazelix_config)
    $config.popup_program? | default ["lazygit"]
}

def run_popup_program [popup_program: list<string>] {
    if ($popup_program | is-empty) {
        error make {msg: "No popup program was provided to the Yazelix popup runtime wrapper."}
    }

    let command = ($popup_program | first)
    let args = ($popup_program | skip 1)

    if $command == "editor" {
        let config = (parse_yazelix_config)
        let runtime_env = (get_runtime_env $config)
        let editor_command = ($runtime_env.EDITOR? | default "" | into string | str trim)

        if ($editor_command | is-empty) {
            error make {msg: "The configured Yazelix editor could not be resolved for popup_program = [\"editor\"]."}
        }

        run_runtime_argv ([$editor_command] | append $args) --config $config
        return
    }

    if (which $command | is-empty) {
        error make {msg: $"Popup program not found in PATH: ($command)"}
    }

    run-external $command ...$args
}

def --wrapped main [...popup_args: string] {
    rename_current_transient_pane "popup"

    let result = (try {
        with-env (get_transient_pane_mode_env "popup") {
            run_popup_program (resolve_popup_program $popup_args)
        }
        {ok: true}
    } catch {|err|
        {ok: false, msg: $err.msg}
    })

    if $result.ok {
        refresh_active_sidebar_yazi | ignore
        close_current_transient_pane
        return
    }

    error make {msg: $result.msg}
}
