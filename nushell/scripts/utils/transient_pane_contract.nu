#!/usr/bin/env nu

export def get_transient_pane_contract [kind: string] {
    match ($kind | str trim) {
        "popup" => {
            kind: "popup"
            pane_title: "yzx_popup"
            mode_env_key: "YAZELIX_POPUP_PANE"
            mode_env_value: "true"
        }
        "menu" => {
            kind: "menu"
            pane_title: "yzx_menu"
            mode_env_key: "YAZELIX_MENU_POPUP"
            mode_env_value: "true"
        }
        "config" => {
            kind: "config"
            pane_title: "yzx_config"
            mode_env_key: "YAZELIX_CONFIG_POPUP"
            mode_env_value: "true"
        }
        _ => {
            error make {msg: $"Unsupported transient pane kind: ($kind)"}
        }
    }
}

export def resolve_transient_pane_cwd [
    workspace_root?: string
    current_dir?: string
] {
    let normalized_workspace_root = ($workspace_root | default "" | str trim)
    let fallback_dir = ($current_dir | default (pwd))

    if ($normalized_workspace_root | is-empty) {
        $fallback_dir
    } else {
        $normalized_workspace_root
    }
}

export def build_transient_pane_open_contract [
    kind: string
    runtime_dir: string
    width_percent: int = 90
    height_percent: int = 90
    workspace_root?: string
    current_dir?: string
    args: list<string> = []
] {
    let identity = (get_transient_pane_contract $kind)

    $identity | merge {
        width_percent: $width_percent
        height_percent: $height_percent
        args: $args
        cwd: (resolve_transient_pane_cwd $workspace_root $current_dir)
        runtime_dir: ($runtime_dir | path expand)
    }
}

export def get_transient_pane_mode_env [kind: string] {
    let contract = (get_transient_pane_contract $kind)
    {} | upsert $contract.mode_env_key $contract.mode_env_value
}

export def is_transient_pane_mode_active [kind: string] {
    let contract = (get_transient_pane_contract $kind)
    (($env | get -o $contract.mode_env_key | default "") == $contract.mode_env_value)
}

export def rename_current_transient_pane [kind: string] {
    if ($env.ZELLIJ? | is-not-empty) {
        let contract = (get_transient_pane_contract $kind)
        ^zellij action rename-pane $contract.pane_title | complete | ignore
    }
}

export def close_current_transient_pane [] {
    if ($env.ZELLIJ? | is-not-empty) {
        ^zellij action close-pane | complete | ignore
    }
}
