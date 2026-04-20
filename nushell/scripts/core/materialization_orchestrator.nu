#!/usr/bin/env nu
# Runtime materialization orchestration for startup and repair flows.
# Bridges between Rust plan evaluation and Nushell-side config generators.

use ../utils/common.nu require_yazelix_runtime_dir
use ../utils/failure_classes.nu format_failure_classification
use ../utils/generated_runtime_state.nu [
    apply_runtime_materialization
    compute_runtime_materialization_plan
    evaluate_runtime_materialization_repair
]
use ../utils/startup_profile.nu profile_startup_step
use ../setup/yazi_config_merger.nu generate_merged_yazi_config
use ../setup/zellij_config_merger.nu generate_merged_zellij_config

export def regenerate_runtime_configs [runtime_dir: string, --quiet, materialization_plan?: record] {
    let quiet_mode = $quiet
    let plan = if $materialization_plan == null {
        profile_startup_step "materialization_orchestrator" "compute_config_state" {
            compute_runtime_materialization_plan $runtime_dir
        }
    } else {
        $materialization_plan
    }
    let config_state = $plan

    try {
        profile_startup_step "materialization_orchestrator" "generate_yazi_config" {
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
        profile_startup_step "materialization_orchestrator" "generate_zellij_config" {
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
