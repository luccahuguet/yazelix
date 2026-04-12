#!/usr/bin/env nu

use ../integrations/yazi.nu [refresh_active_sidebar_yazi]
use ../utils/config_parser.nu [parse_yazelix_config]

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

    if (which $command | is-empty) {
        error make {msg: $"Popup program not found in PATH: ($command)"}
    }

    run-external $command ...$args
}

def rename_transient_pane [name: string] {
    if ($env.ZELLIJ? | is-not-empty) {
        ^zellij action rename-pane $name | complete | ignore
    }
}

def close_transient_pane [] {
    if ($env.ZELLIJ? | is-not-empty) {
        ^zellij action close-pane | complete | ignore
    }
}

def --wrapped main [...popup_args: string] {
    rename_transient_pane "yzx_popup"

    let result = (try {
        run_popup_program (resolve_popup_program $popup_args)
        {ok: true}
    } catch {|err|
        {ok: false, msg: $err.msg}
    })

    if $result.ok {
        refresh_active_sidebar_yazi | ignore
        close_transient_pane
        return
    }

    error make {msg: $result.msg}
}
