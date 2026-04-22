#!/usr/bin/env nu

use ../integrations/yazi.nu [refresh_active_sidebar_yazi]
use ../utils/transient_pane_facts.nu [load_transient_pane_facts]
use ../utils/runtime_env.nu [run_runtime_argv]
use ../utils/transient_pane_contract.nu [
    close_current_transient_pane
    get_transient_pane_mode_env
    rename_current_transient_pane
]
use ../utils/yzx_core_bridge.nu [compute_runtime_env_via_yzx_core]

def resolve_popup_program [popup_args: list<string>, transient_pane_facts: record] {
    if ($popup_args | is-not-empty) {
        return $popup_args
    }

    $transient_pane_facts.popup_program? | default ["lazygit"]
}

def require_popup_command_available [command: string, runtime_env: record] {
    let normalized = ($command | into string | str trim)

    if ($normalized | is-empty) {
        error make {msg: "Popup program command cannot be empty."}
    }

    if ($normalized | str contains "/") {
        if not (($normalized | path expand) | path exists) {
            error make {msg: $"Popup program path does not exist: ($normalized)"}
        }
        return
    }

    let command_exists = (with-env $runtime_env {
        not ((which $normalized) | is-empty)
    })

    if not $command_exists {
        error make {msg: $"Popup program not found in PATH: ($normalized)"}
    }
}

def resolve_popup_argv [popup_program: list<string>, runtime_env: record] {
    if ($popup_program | is-empty) {
        error make {msg: "No popup program was provided to the Yazelix popup runtime wrapper."}
    }

    let command = ($popup_program | first)
    let args = ($popup_program | skip 1)

    if $command == "editor" {
        let editor_command = ($runtime_env.EDITOR? | default "" | into string | str trim)

        if ($editor_command | is-empty) {
            error make {msg: "The configured Yazelix editor could not be resolved for popup_program = [\"editor\"]."}
        }

        return ([$editor_command] | append $args)
    }

    $popup_program
}

def resolve_popup_launch_context [popup_args: list<string>] {
    let transient_pane_facts = (load_transient_pane_facts)
    let runtime_env = (compute_runtime_env_via_yzx_core)
    let popup_program = (resolve_popup_program $popup_args $transient_pane_facts)
    let argv = (resolve_popup_argv $popup_program $runtime_env)
    let command = ($argv | first | default "")

    require_popup_command_available $command $runtime_env

    {
        runtime_env: $runtime_env
        argv: $argv
    }
}

def run_popup_program [popup_args: list<string>] {
    let launch_context = (resolve_popup_launch_context $popup_args)
    run_runtime_argv $launch_context.argv --runtime-env $launch_context.runtime_env
}

def --wrapped main [...popup_args: string] {
    rename_current_transient_pane "popup"

    let result = (try {
        with-env (get_transient_pane_mode_env "popup") {
            run_popup_program $popup_args
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
