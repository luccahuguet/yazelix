#!/usr/bin/env nu

use config_parser.nu [run_yzx_core_command run_yzx_core_json_command]
use config_surfaces.nu [get_main_user_config_path load_active_config_surface]
use config_contract.nu MAIN_CONFIG_CONTRACT_RELATIVE_PATH
use common.nu [get_materialized_state_path get_yazelix_state_dir require_yazelix_runtime_dir]
use failure_classes.nu format_failure_classification
use startup_profile.nu profile_startup_step
use ../setup/yazi_config_merger.nu generate_merged_yazi_config
use ../setup/zellij_config_merger.nu generate_merged_zellij_config

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

def apply_runtime_materialization [state: record] {
    let config_file = ($state.config_file? | default "")
    let runtime_dir = require_yazelix_runtime_dir
    let helper_args = (build_runtime_materialization_apply_args $state)
    run_yzx_core_command $runtime_dir {display_config_path: $config_file} $helper_args | ignore
}

export def regenerate_runtime_configs [runtime_dir: string, --quiet, materialization_plan?: record] {
    let quiet_mode = $quiet
    let plan = if $materialization_plan == null {
        profile_startup_step "generated_runtime_state" "compute_config_state" {
            compute_runtime_materialization_plan $runtime_dir
        }
    } else {
        $materialization_plan
    }
    let config_state = $plan

    try {
        profile_startup_step "generated_runtime_state" "generate_yazi_config" {
            if $quiet_mode {
                generate_merged_yazi_config $runtime_dir --quiet --sync-static-assets=($plan.should_sync_static_assets? | default true) | ignore
            } else {
                print "🔧 Regenerating managed Yazi configuration..."
                generate_merged_yazi_config $runtime_dir --sync-static-assets=($plan.should_sync_static_assets? | default true) | ignore
            }
        } {
            inputs_require_refresh: ($config_state.inputs_require_refresh? | default false)
            refresh_reason: ($config_state.refresh_reason? | default "")
        }
    } catch {|err|
        error make {msg: $"Failed to regenerate Yazi configuration: ($err.msg)"}
    }

    try {
        profile_startup_step "generated_runtime_state" "generate_zellij_config" {
            if not $quiet_mode {
                print "🔧 Regenerating managed Zellij configuration..."
            }
            generate_merged_zellij_config $runtime_dir ($plan.zellij_config_dir? | default "") --quiet | ignore
        } {
            inputs_require_refresh: ($config_state.inputs_require_refresh? | default false)
            refresh_reason: ($config_state.refresh_reason? | default "")
        }
    } catch {|err|
        error make {msg: $"Failed to regenerate Zellij configuration: ($err.msg)"}
    }

    $plan
}

export def record_current_materialized_state [applied_state?: record] {
    let applied_state = if $applied_state == null {
        let runtime_dir = require_yazelix_runtime_dir
        compute_runtime_materialization_plan $runtime_dir
    } else {
        $applied_state
    }
    apply_runtime_materialization $applied_state
    $applied_state
}

export def repair_generated_runtime_state [
    --force(-f)    # Force regeneration even when config/runtime inputs already match
    --verbose(-v)  # Print concise generated-state repair progress
] {
    let runtime_dir = (require_yazelix_runtime_dir)
    let show_progress = $verbose
    let evaluation = if $force {
        evaluate_runtime_materialization_repair $runtime_dir --force
    } else {
        evaluate_runtime_materialization_repair $runtime_dir
    }
    let plan = ($evaluation.plan)
    let repair = ($evaluation.repair)

    if ($repair.action? | default "") == "noop" {
        for line in ($repair.lines? | default []) {
            print $line
        }
        return {
            status: "noop"
            applied_state: $plan
        }
    }

    if $show_progress {
        print ($repair.progress_message? | default "")
        let detail = ($repair.missing_artifacts_detail_line? | default "")
        if ($detail | is-not-empty) {
            print $detail
        }
    }

    let applied_state = (regenerate_runtime_configs $runtime_dir --quiet=(not $show_progress) $plan)
    try {
        record_current_materialized_state $applied_state | ignore
    } catch {|err|
        let classification = (format_failure_classification "generated-state" "Run `yzx doctor` to inspect the generated-state contract, then rerun the repair after fixing the reported problem.")
        error make {msg: $"Failed to finalize the generated runtime state repair: ($err.msg)\n($classification)"}
    }

    for line in ($repair.success_lines? | default []) {
        print $line
    }

    {
        status: ($repair.result_status? | default "repaired")
        applied_state: $applied_state
    }
}
