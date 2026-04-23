#!/usr/bin/env nu
# Dev-only helpers for probing the Rust-owned active-config and normalize path.

use ../utils/runtime_paths.nu [require_yazelix_runtime_dir]
use ../utils/yzx_core_bridge.nu [
    resolve_active_config_surface_via_yzx_core
    run_yzx_core_json_command
]

def get_yzx_core_contract_path [runtime_dir: string] {
    $runtime_dir | path join "config_metadata" "main_config_contract.toml"
}

def build_config_normalize_helper_args [
    runtime_dir: string
    config_surface: record
    --include-missing
] {
    mut helper_args = [
        "config.normalize"
        "--config"
        $config_surface.config_file
        "--default-config"
        $config_surface.default_config_path
        "--contract"
        (get_yzx_core_contract_path $runtime_dir)
    ]

    if $include_missing {
        $helper_args = ($helper_args | append "--include-missing")
    }

    $helper_args
}

export def load_active_config_normalize_data [
    runtime_dir?: string
    --include-missing
] {
    let resolved_runtime_dir = if $runtime_dir == null {
        require_yazelix_runtime_dir
    } else {
        $runtime_dir | path expand
    }
    let config_surface = (resolve_active_config_surface_via_yzx_core $resolved_runtime_dir)

    (run_yzx_core_json_command
        $resolved_runtime_dir
        $config_surface
        (build_config_normalize_helper_args $resolved_runtime_dir $config_surface --include-missing=$include_missing)
        "Yazelix Rust config helper returned invalid JSON.")
}

export def load_normalized_active_config [runtime_dir?: string] {
    load_active_config_normalize_data $runtime_dir | get normalized_config
}
