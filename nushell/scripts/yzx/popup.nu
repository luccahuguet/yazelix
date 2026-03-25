#!/usr/bin/env nu

use ../integrations/zellij.nu [
    get_current_tab_workspace_root_including_bootstrap
    open_floating_runtime_wrapper
]
use ../utils/config_parser.nu parse_yazelix_config

def resolve_popup_command [configured_program: list<string>, override_program: list<string>] {
    if ($override_program | is-empty) {
        $configured_program
    } else {
        $override_program
    }
}

def resolve_popup_cwd [workspace_root: string, current_dir: string] {
    if ($workspace_root | str trim | is-empty) {
        $current_dir
    } else {
        $workspace_root
    }
}

def run_popup_program_inline [popup_program: list<string>] {
    if ($popup_program | is-empty) {
        error make {msg: "No popup program configured. Set zellij.popup_program in yazelix.toml or pass an explicit command to `yzx popup`."}
    }

    let command = ($popup_program | first)
    let args = ($popup_program | skip 1)

    if (which $command | is-empty) {
        error make {msg: $"Popup program not found in PATH: ($command)"}
    }

    run-external $command ...$args
}

export def resolve_yzx_popup_command [
    configured_program: list<string>
    ...override_program: string
] {
    resolve_popup_command $configured_program $override_program
}

export def resolve_yzx_popup_cwd [
    workspace_root?: string
    current_dir?: string
] {
    resolve_popup_cwd ($workspace_root | default "") ($current_dir | default (pwd))
}

export def "yzx popup" [
    ...program: string  # Optional command override, eg. `yzx popup lazygit`
] {
    let config = parse_yazelix_config
    let popup_program = (resolve_popup_command ($config.popup_program? | default ["lazygit"]) $program)

    let in_popup = (($env.YAZELIX_POPUP_PANE? | default "false") == "true")
    if $in_popup {
        run_popup_program_inline $popup_program
        return
    }

    if ($env.ZELLIJ? | is-empty) {
        error make {msg: "yzx popup only works inside Zellij. Start Yazelix first, then run it from the tab where you want the popup."}
    }

    let popup_cwd = (resolve_popup_cwd ((get_current_tab_workspace_root_including_bootstrap) | default "") (pwd))
    open_floating_runtime_wrapper "yzx_popup" "yzx_popup_program.nu" $popup_cwd { } $popup_program $config.popup_width_percent $config.popup_height_percent
}
