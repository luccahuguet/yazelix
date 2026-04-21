#!/usr/bin/env nu
# Runtime materialization bridge for startup and repair flows.
# Rust owns the materialization lifecycle; Nushell only invokes the helper and renders progress.

use ../utils/common.nu [get_materialized_state_path get_yazelix_state_dir require_yazelix_runtime_dir]
use ../utils/config_contract.nu MAIN_CONFIG_CONTRACT_RELATIVE_PATH
use ../utils/yzx_core_bridge.nu [build_record_yzx_core_error_surface run_yzx_core_request_json_command]
use ../utils/config_surfaces.nu load_active_config_surface
use ../utils/failure_classes.nu format_failure_classification
use ../utils/startup_profile.nu profile_startup_step

const RUNTIME_MATERIALIZATION_PLAN_COMMAND = "runtime-materialization.plan"
const RUNTIME_MATERIALIZATION_MATERIALIZE_COMMAND = "runtime-materialization.materialize"
const RUNTIME_MATERIALIZATION_REPAIR_COMMAND = "runtime-materialization.repair"

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
        null
    }
}

def build_runtime_materialization_context [runtime_dir: string] {
    let config_surface = (load_active_config_surface)
    let paths = (get_runtime_materialization_paths)

    {
        config_surface: $config_surface
        request: {
            config_path: $config_surface.config_file
            default_config_path: $config_surface.default_config_path
            contract_path: ($runtime_dir | path join $MAIN_CONFIG_CONTRACT_RELATIVE_PATH)
            runtime_dir: $runtime_dir
            state_path: $paths.state_path
            yazi_config_dir: $paths.yazi_config_dir
            zellij_config_dir: $paths.zellij_config_dir
            zellij_layout_dir: $paths.zellij_layout_dir
            layout_override: (get_runtime_materialization_layout_override)
        }
    }
}

export def compute_runtime_materialization_plan [runtime_dir: string] {
    let context = (build_runtime_materialization_context $runtime_dir)
    run_yzx_core_request_json_command $runtime_dir (
        build_record_yzx_core_error_surface {config_file: $context.config_surface.config_file}
    ) $RUNTIME_MATERIALIZATION_PLAN_COMMAND $context.request "Yazelix Rust runtime-materialization helper returned invalid JSON."
}

export def regenerate_runtime_configs [runtime_dir: string, --quiet] {
    let context = (build_runtime_materialization_context $runtime_dir)
    let result = (profile_startup_step "materialization_orchestrator" "materialize_runtime_state" {
        run_yzx_core_request_json_command $runtime_dir (
            build_record_yzx_core_error_surface {config_file: $context.config_surface.config_file}
        ) $RUNTIME_MATERIALIZATION_MATERIALIZE_COMMAND $context.request "Yazelix Rust runtime-materialization materialize helper returned invalid JSON."
    })

    if (not $quiet) and (($result.plan.status? | default "") != "noop") {
        print "✅ Generated runtime state materialized."
    }

    $result.plan
}

export def repair_generated_runtime_state [
    --force(-f)    # Force regeneration even when config/runtime inputs already match
    --verbose(-v)  # Print concise generated-state repair progress
] {
    let runtime_dir = (require_yazelix_runtime_dir)
    let context = (build_runtime_materialization_context $runtime_dir)
    let data = (run_yzx_core_request_json_command $runtime_dir (
        build_record_yzx_core_error_surface {config_file: $context.config_surface.config_file}
    ) $RUNTIME_MATERIALIZATION_REPAIR_COMMAND {
        plan: $context.request
        force: $force
    } "Yazelix Rust runtime-materialization repair helper returned invalid JSON.")
    let repair = ($data.repair? | default {})

    if (($data.status? | default "") == "noop") {
        for line in ($repair.lines? | default []) {
            print $line
        }
        return {
            status: "noop"
            applied_state: ($data.plan? | default {})
        }
    }

    if $verbose {
        let progress_message = ($repair.progress_message? | default "")
        if ($progress_message | is-not-empty) {
            print $progress_message
        }
        let detail = ($repair.missing_artifacts_detail_line? | default "")
        if ($detail | is-not-empty) {
            print $detail
        }
    }

    let materialization = ($data.materialization? | default null)
    if $materialization == null {
        let classification = (format_failure_classification "generated-state" "Run `yzx doctor` to inspect the generated-state contract, then rerun the repair after fixing the reported problem.")
        error make {msg: $"Rust runtime-materialization repair returned no materialization result for a non-noop repair.\n($classification)"}
    }

    for line in ($repair.success_lines? | default []) {
        print $line
    }

    {
        status: ($data.status? | default "repaired")
        applied_state: ($materialization.plan? | default {})
    }
}
