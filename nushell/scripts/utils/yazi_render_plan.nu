#!/usr/bin/env nu
# Bridge to Rust-owned Yazi render-plan computation.

use ./config_parser.nu [run_yzx_core_json_command]

const YAZI_RENDER_PLAN_COMMAND = "yazi-render-plan.compute"

export def build_yazi_render_plan_request [config: record] {
    {
        yazi_theme: ($config.yazi_theme? | default "default")
        yazi_sort_by: ($config.yazi_sort_by? | default "alphabetical")
        yazi_plugins: ($config.yazi_plugins? | default null)
    }
}

def yazi_render_plan_error_surface [config: record] {
    {
        display_config_path: ($config.config_file? | default "")
        config_file: ($config.config_file? | default "")
    }
}

export def compute_yazi_render_plan [runtime_dir: string, config: record] {
    let request = (build_yazi_render_plan_request $config)
    run_yzx_core_json_command $runtime_dir (yazi_render_plan_error_surface $config) [
        $YAZI_RENDER_PLAN_COMMAND
        "--request-json"
        ($request | to json -r)
    ] "Yazelix Rust yazi-render-plan helper returned invalid JSON."
}
