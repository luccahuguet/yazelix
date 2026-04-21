#!/usr/bin/env nu

use ../utils/common.nu get_yazelix_state_dir
use ../utils/yzx_core_bridge.nu run_yzx_core_json_command
use ../utils/config_surfaces.nu load_active_config_surface

const ZELLIJ_MATERIALIZATION_COMMAND = "zellij-materialization.generate"

def build_zellij_materialization_helper_args [
    runtime_dir: string
    config_surface: record
    merged_config_dir: string
    --seed-plugin-permissions
] {
    mut helper_args = [
        $ZELLIJ_MATERIALIZATION_COMMAND
        "--config"
        $config_surface.config_file
        "--default-config"
        $config_surface.default_config_path
        "--contract"
        ($runtime_dir | path join "config_metadata" "main_config_contract.toml")
        "--runtime-dir"
        $runtime_dir
        "--zellij-config-dir"
        $merged_config_dir
    ]

    if $seed_plugin_permissions {
        $helper_args = ($helper_args | append "--seed-plugin-permissions")
    }

    $helper_args
}

def print_zellij_materialization_summary [result: record] {
    let layout_count = ($result.generated_layouts? | default [] | length)
    let reuse_label = if ($result.reused? | default false) { "reused" } else { "regenerated" }
    print $"   ↺ State: ($reuse_label)"
    print $"   📁 Base config source: ($result.base_config_source? | default 'unknown')"
    print $"   🧩 Layouts: ($layout_count)"
    print $"   🔌 Pane orchestrator: ($result.pane_orchestrator_runtime_path)"
    print $"   🔌 zjstatus: ($result.zjstatus_runtime_path)"
}

export def generate_merged_zellij_config [
    yazelix_dir: string
    merged_config_dir_override?: string
    --quiet
    --seed-plugin-permissions
] {
    let config_surface = (load_active_config_surface)
    let merged_config_dir = if ($merged_config_dir_override | is-not-empty) {
        $merged_config_dir_override | path expand
    } else {
        (get_yazelix_state_dir) | path join "configs" "zellij"
    }
    let helper_args = (build_zellij_materialization_helper_args
        $yazelix_dir
        $config_surface
        $merged_config_dir
        --seed-plugin-permissions=$seed_plugin_permissions)
    let result = (run_yzx_core_json_command
        $yazelix_dir
        $config_surface
        $helper_args
        "Yazelix Rust zellij-materialization helper returned invalid JSON.")

    if not $quiet {
        print "🔄 Generating Zellij configuration..."
        print_zellij_materialization_summary $result
        print "✅ Zellij configuration generated successfully!"
        print $"   📁 Config saved to: ($result.merged_config_path)"
    }

    $result.merged_config_path
}

export def main [yazelix_dir: string, --quiet] {
    generate_merged_zellij_config $yazelix_dir --quiet=$quiet | ignore
}
