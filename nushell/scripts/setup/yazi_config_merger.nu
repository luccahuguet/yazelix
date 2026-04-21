#!/usr/bin/env nu

use ../utils/common.nu get_yazelix_state_dir
use ../utils/yzx_core_bridge.nu run_yzx_core_json_command
use ../utils/config_surfaces.nu load_active_config_surface

const YAZI_MATERIALIZATION_COMMAND = "yazi-materialization.generate"

def build_yazi_materialization_helper_args [
    runtime_dir: string
    config_surface: record
    merged_config_dir: string
    --sync-static-assets
] {
    mut helper_args = [
        $YAZI_MATERIALIZATION_COMMAND
        "--config"
        $config_surface.config_file
        "--default-config"
        $config_surface.default_config_path
        "--contract"
        ($runtime_dir | path join "config_metadata" "main_config_contract.toml")
        "--runtime-dir"
        $runtime_dir
        "--yazi-config-dir"
        $merged_config_dir
    ]

    if $sync_static_assets {
        $helper_args = ($helper_args | append "--sync-static-assets")
    }

    $helper_args
}

def print_yazi_materialization_summary [result: record] {
    let files = ($result.managed_files? | default [])
    for file_result in $files {
        let label = ($file_result.path | path basename)
        let status = if ($file_result.changed? | default false) { "✅" } else { "↺" }
        print $"   ($status) ($label)"
    }

    if ($result.synced_static_assets? | default false) {
        print "   📦 Synced bundled Yazi runtime assets"
    } else {
        print "   📦 Reusing existing bundled Yazi runtime assets"
    }

    let missing_plugins = ($result.missing_plugins? | default [])
    if ($missing_plugins | is-not-empty) {
        print $"⚠️  Warning: Missing plugins in yazelix.toml: ($missing_plugins | str join ', ')"
        print "   Install with: ya pkg add <owner/repo>"
        print "   Or remove from yazelix.toml [yazi] plugins list"
    }
}

export def generate_merged_yazi_config [
    yazelix_dir: string,
    --quiet,
    --sync-static-assets = true
] {
    let config_surface = (load_active_config_surface)
    let merged_config_dir = ((get_yazelix_state_dir) | path join "configs" "yazi")
    let helper_args = (build_yazi_materialization_helper_args
        $yazelix_dir
        $config_surface
        $merged_config_dir
        --sync-static-assets=$sync_static_assets)
    let result = (run_yzx_core_json_command
        $yazelix_dir
        $config_surface
        $helper_args
        "Yazelix Rust yazi-materialization helper returned invalid JSON.")

    if not $quiet {
        print "🔄 Generating Yazi configuration..."
        print_yazi_materialization_summary $result
        print "✅ Yazi configuration generated successfully!"
        print $"   📁 Config saved to: ($result.merged_config_dir)"
    }

    $result.merged_config_dir
}

export def main [yazelix_dir: string, --quiet] {
    generate_merged_yazi_config $yazelix_dir --quiet=$quiet | ignore
}
