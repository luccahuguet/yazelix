#!/usr/bin/env nu
# Bridge to Rust-owned Zellij render-plan computation.

use ./config_parser.nu [run_yzx_core_json_command]

const ZELLIJ_RENDER_PLAN_COMMAND = "zellij-render-plan.compute"

export def build_zellij_render_plan_request [
    config: record
    yazelix_layout_dir: string
    resolved_default_shell: string
] {
    {
        enable_sidebar: ($config.enable_sidebar? | default true)
        sidebar_width_percent: ($config.sidebar_width_percent? | default 20)
        popup_width_percent: ($config.popup_width_percent? | default 90)
        popup_height_percent: ($config.popup_height_percent? | default 90)
        zellij_widget_tray: ($config.zellij_widget_tray? | default null)
        zellij_custom_text: ($config.zellij_custom_text? | default null)
        zellij_theme: ($config.zellij_theme? | default "default")
        zellij_pane_frames: ($config.zellij_pane_frames? | default "true")
        zellij_rounded_corners: ($config.zellij_rounded_corners? | default "true")
        disable_zellij_tips: ($config.disable_zellij_tips? | default "true")
        persistent_sessions: ($config.persistent_sessions? | default "false")
        support_kitty_keyboard_protocol: ($config.support_kitty_keyboard_protocol? | default "true")
        zellij_default_mode: ($config.zellij_default_mode? | default "normal")
        yazelix_layout_dir: $yazelix_layout_dir
        resolved_default_shell: $resolved_default_shell
    }
}

def zellij_render_plan_error_surface [config: record] {
    {
        display_config_path: ($config.config_file? | default "")
        config_file: ($config.config_file? | default "")
    }
}

export def compute_zellij_render_plan [
    runtime_dir: string
    config: record
    yazelix_layout_dir: string
    resolved_default_shell: string
] {
    let request = (build_zellij_render_plan_request $config $yazelix_layout_dir $resolved_default_shell)
    run_yzx_core_json_command $runtime_dir (zellij_render_plan_error_surface $config) [
        $ZELLIJ_RENDER_PLAN_COMMAND
        "--request-json"
        ($request | to json -r)
    ] "Yazelix Rust zellij-render-plan helper returned invalid JSON."
}
