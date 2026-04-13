#!/usr/bin/env nu

use ../integrations/zellij.nu [
    get_current_tab_workspace_root_including_bootstrap
    open_transient_pane_contract
]
use ../utils/common.nu [get_yazelix_runtime_dir]
use ../utils/config_parser.nu parse_yazelix_config
use ../utils/transient_pane_contract.nu [
    build_transient_pane_open_contract
    resolve_transient_pane_cwd
]

def resolve_popup_command [configured_program: list<string>, override_program: list<string>] {
    if ($override_program | is-empty) {
        $configured_program
    } else {
        $override_program
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
    resolve_transient_pane_cwd $workspace_root $current_dir
}

export def resolve_yzx_popup_contract [
    config: record
    runtime_dir: string
    workspace_root?: string
    current_dir?: string
    ...override_program: string
] {
    build_transient_pane_open_contract "popup" $config $runtime_dir $workspace_root $current_dir (
        resolve_popup_command ($config.popup_program? | default ["lazygit"]) $override_program
    )
}

# Open or toggle the configured Yazelix popup program in Zellij
export def --wrapped "yzx popup" [
    ...program: string  # Optional command override, eg. `yzx popup lazygit`
] {
    let config = parse_yazelix_config

    if ($env.ZELLIJ? | is-empty) {
        error make {msg: "yzx popup only works inside Zellij. Start Yazelix first, then run it from the tab where you want the popup."}
    }

    let runtime_dir = (get_yazelix_runtime_dir | path expand)
    let popup_contract = (resolve_yzx_popup_contract $config $runtime_dir ((get_current_tab_workspace_root_including_bootstrap) | default "") (pwd) ...$program)
    let open_result = (open_transient_pane_contract $popup_contract)
    if $open_result.status != "ok" {
        error make {msg: $"Failed to open the Yazelix popup pane: ($open_result | to json -r)"}
    }
}
