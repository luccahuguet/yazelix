#!/usr/bin/env nu

export def get_transient_pane_contract [kind: string] {
    match ($kind | str trim) {
        "popup" => {
            kind: "popup"
            pane_title: "yzx_popup"
            wrapper_marker: "yzx_popup_program.nu"
            wrapper_relative_path: "nushell/scripts/zellij_wrappers/yzx_popup_program.nu"
            mode_env_key: "YAZELIX_POPUP_PANE"
            mode_env_value: "true"
        }
        "menu" => {
            kind: "menu"
            pane_title: "yzx_menu"
            wrapper_marker: "yzx_menu_popup.nu"
            wrapper_relative_path: "nushell/scripts/zellij_wrappers/yzx_menu_popup.nu"
            mode_env_key: "YAZELIX_MENU_POPUP"
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

export def resolve_transient_pane_geometry [config: record] {
    {
        width_percent: ($config.popup_width_percent? | default 90)
        height_percent: ($config.popup_height_percent? | default 90)
    }
}

export def build_transient_pane_open_contract [
    kind: string
    config: record
    runtime_dir: string
    workspace_root?: string
    current_dir?: string
    args: list<string> = []
] {
    let identity = (get_transient_pane_contract $kind)
    let geometry = (resolve_transient_pane_geometry $config)

    $identity | merge $geometry | merge {
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
