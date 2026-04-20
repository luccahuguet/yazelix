#!/usr/bin/env nu

use config_parser.nu [run_yzx_core_command run_yzx_core_json_command]
use config_surfaces.nu [get_main_user_config_path load_active_config_surface]
use config_contract.nu MAIN_CONFIG_CONTRACT_RELATIVE_PATH
use common.nu [get_materialized_state_path get_yazelix_state_dir require_yazelix_runtime_dir]

def get_runtime_materialization_paths [] {
    let state_dir = (get_yazelix_state_dir)
    let zellij_config_dir = ($state_dir | path join "configs" "zellij")

    {
        state_path: (get_materialized_state_path)
        yazi_config_dir: ($state_dir | path join "configs" "yazi")
        zellij_config_dir: $zellij_config_dir
        zellij_layout_dir: ($zellij_config_dir | path join "layouts")
    }
}

def get_runtime_materialization_layout_override [] {
    if ($env.YAZELIX_LAYOUT_OVERRIDE? | is-not-empty) {
        $env.YAZELIX_LAYOUT_OVERRIDE
    } else if ($env.YAZELIX_SWEEP_TEST_ID? | is-not-empty) and ($env.ZELLIJ_DEFAULT_LAYOUT? | is-not-empty) {
        $env.ZELLIJ_DEFAULT_LAYOUT
    } else {
        ""
    }
}

def build_runtime_materialization_helper_argv [command: string, runtime_dir: string] {
    let config_surface = (load_active_config_surface)
    let paths = (get_runtime_materialization_paths)
    mut helper_args = [
        $command
        "--config"
        $config_surface.config_file
        "--default-config"
        $config_surface.default_config_path
        "--contract"
        ($runtime_dir | path join $MAIN_CONFIG_CONTRACT_RELATIVE_PATH)
        "--runtime-dir"
        $runtime_dir
        "--state-path"
        $paths.state_path
        "--yazi-config-dir"
        $paths.yazi_config_dir
        "--zellij-config-dir"
        $paths.zellij_config_dir
        "--zellij-layout-dir"
        $paths.zellij_layout_dir
    ]
    let layout_override = (get_runtime_materialization_layout_override)

    if ($layout_override | is-not-empty) {
        $helper_args | append "--layout-override" | append $layout_override
    } else {
        $helper_args
    }
}

export def build_runtime_materialization_plan_helper_argv [runtime_dir: string] {
    build_runtime_materialization_helper_argv "runtime-materialization.plan" $runtime_dir
}

export def build_runtime_materialization_repair_evaluate_helper_argv [runtime_dir: string, --force] {
    mut argv = (build_runtime_materialization_helper_argv "runtime-materialization.repair-evaluate" $runtime_dir)
    if $force {
        $argv = ($argv | append "--force")
    }

    $argv
}

def build_runtime_materialization_apply_args [state: record] {
    [
        "runtime-materialization.apply"
        "--config-file"
        ($state.config_file? | default "")
        "--managed-config"
        (get_main_user_config_path)
        "--state-path"
        (get_materialized_state_path)
        "--config-hash"
        ($state.config_hash? | default "")
        "--runtime-hash"
        ($state.runtime_hash? | default "")
        "--expected-artifacts-json"
        (($state.expected_artifacts? | default []) | to json -r)
    ]
}

export def compute_runtime_materialization_plan [runtime_dir: string] {
    let config_surface = (load_active_config_surface)
    let helper_args = (build_runtime_materialization_plan_helper_argv $runtime_dir)

    run_yzx_core_json_command $runtime_dir $config_surface $helper_args "Yazelix Rust runtime-materialization helper returned invalid JSON."
}

export def evaluate_runtime_materialization_repair [runtime_dir: string, --force] {
    let config_surface = (load_active_config_surface)
    let helper_args = if $force {
        (build_runtime_materialization_repair_evaluate_helper_argv $runtime_dir --force)
    } else {
        (build_runtime_materialization_repair_evaluate_helper_argv $runtime_dir)
    }

    run_yzx_core_json_command $runtime_dir $config_surface $helper_args "Yazelix Rust runtime-materialization repair-evaluate helper returned invalid JSON."
}

export def apply_runtime_materialization [state: record] {
    let config_file = ($state.config_file? | default "")
    let runtime_dir = require_yazelix_runtime_dir
    let helper_args = (build_runtime_materialization_apply_args $state)
    run_yzx_core_command $runtime_dir {display_config_path: $config_file} $helper_args | ignore
}
