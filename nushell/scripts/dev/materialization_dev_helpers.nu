#!/usr/bin/env nu
# Direct Rust-owned materialization helpers for maintainer tooling and tests.

use ../utils/common.nu get_yazelix_state_dir
use ../utils/yzx_core_bridge.nu [build_record_yzx_core_error_surface run_yzx_core_json_command]
use ../utils/config_surfaces.nu load_active_config_surface

const YAZI_MATERIALIZATION_COMMAND = "yazi-materialization.generate"
const ZELLIJ_MATERIALIZATION_COMMAND = "zellij-materialization.generate"

def build_error_surface [config_surface: record] {
    build_record_yzx_core_error_surface {config_file: $config_surface.config_file}
}

export def generate_merged_yazi_config [
    yazelix_dir: string,
    --quiet,
    --sync-static-assets = true
] {
    let config_surface = (load_active_config_surface)
    let merged_config_dir = ((get_yazelix_state_dir) | path join "configs" "yazi")
    mut helper_args = [
        $YAZI_MATERIALIZATION_COMMAND
        "--config"
        $config_surface.config_file
        "--default-config"
        $config_surface.default_config_path
        "--contract"
        ($yazelix_dir | path join "config_metadata" "main_config_contract.toml")
        "--runtime-dir"
        $yazelix_dir
        "--yazi-config-dir"
        $merged_config_dir
    ]

    if $sync_static_assets {
        $helper_args = ($helper_args | append "--sync-static-assets")
    }

    let result = (run_yzx_core_json_command
        $yazelix_dir
        (build_error_surface $config_surface)
        $helper_args
        "Yazelix Rust yazi-materialization helper returned invalid JSON.")

    $result.merged_config_dir
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
    mut helper_args = [
        $ZELLIJ_MATERIALIZATION_COMMAND
        "--config"
        $config_surface.config_file
        "--default-config"
        $config_surface.default_config_path
        "--contract"
        ($yazelix_dir | path join "config_metadata" "main_config_contract.toml")
        "--runtime-dir"
        $yazelix_dir
        "--zellij-config-dir"
        $merged_config_dir
    ]

    if $seed_plugin_permissions {
        $helper_args = ($helper_args | append "--seed-plugin-permissions")
    }

    let result = (run_yzx_core_json_command
        $yazelix_dir
        (build_error_surface $config_surface)
        $helper_args
        "Yazelix Rust zellij-materialization helper returned invalid JSON.")

    $result.merged_config_path
}
