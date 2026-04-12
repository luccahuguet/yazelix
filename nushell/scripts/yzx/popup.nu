#!/usr/bin/env nu

use ../integrations/zellij.nu [
    get_current_tab_workspace_root_including_bootstrap
    open_transient_pane
]
use ../utils/common.nu [get_yazelix_runtime_dir]
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

export def --wrapped "yzx popup" [
    ...program: string  # Optional command override, eg. `yzx popup lazygit`
] {
    let config = parse_yazelix_config
    let popup_program = (resolve_popup_command ($config.popup_program? | default ["lazygit"]) $program)

    if ($env.ZELLIJ? | is-empty) {
        error make {msg: "yzx popup only works inside Zellij. Start Yazelix first, then run it from the tab where you want the popup."}
    }

    let popup_cwd = (resolve_popup_cwd ((get_current_tab_workspace_root_including_bootstrap) | default "") (pwd))
    let runtime_dir = (get_yazelix_runtime_dir | path expand)
    let open_result = (open_transient_pane "popup" $popup_program $popup_cwd $runtime_dir)
    if $open_result.status != "ok" {
        error make {msg: $"Failed to open the Yazelix popup pane: ($open_result | to json -r)"}
    }
}
