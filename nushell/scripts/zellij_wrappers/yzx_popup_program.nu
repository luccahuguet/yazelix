#!/usr/bin/env nu

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

def --wrapped main [...popup_args: string] {
    if ($env.ZELLIJ? | is-not-empty) {
        ^zellij action rename-pane "yzx_popup" | complete | ignore
    }

    run_popup_program $popup_args
}
