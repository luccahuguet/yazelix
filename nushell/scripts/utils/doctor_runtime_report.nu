#!/usr/bin/env nu

use common.nu [get_yazelix_runtime_dir get_yazelix_state_dir normalize_path_entries require_yazelix_runtime_dir get_runtime_platform_name]
use config_parser.nu [
    build_default_yzx_core_error_surface
    parse_yazelix_config
    run_yzx_core_request_json_command
]
use constants.nu DEFAULT_TERMINAL
use generated_runtime_state.nu compute_runtime_materialization_plan

const DOCTOR_RUNTIME_EVALUATE_COMMAND = "doctor-runtime.evaluate"

def get_command_search_paths_for_runtime_doctor [] {
    normalize_path_entries ($env.PATH? | default []) | each {|p| $p | path expand }
}

export def evaluate_doctor_runtime_report [req: record] {
    let rd = require_yazelix_runtime_dir
    run_yzx_core_request_json_command $rd (build_default_yzx_core_error_surface) $DOCTOR_RUNTIME_EVALUATE_COMMAND $req "Yazelix Rust doctor-runtime helper returned invalid JSON."
}

export def collect_runtime_doctor_results [install_io: record] {
    let rd = require_yazelix_runtime_dir
    let state_dir = (get_yazelix_state_dir)

    let parse_out = try {
        { ok: true, config: (parse_yazelix_config) }
    } catch {
        { ok: false, config: null }
    }

    let layout_bundle = if not $parse_out.ok {
        { extra: [], shared: null }
    } else {
        let config = $parse_out.config
        let runtime_dir = (get_yazelix_runtime_dir)
        let terminals = ($config.terminals? | default [$DEFAULT_TERMINAL] | uniq)

        let layout_result = try {
            let plan = (compute_runtime_materialization_plan (require_yazelix_runtime_dir))
            let candidate = (
                $plan.zellij_layout_path?
                | default ""
                | into string
                | str trim
            )
            if ($candidate | is-empty) {
                error make {msg: "Rust materialization plan omitted zellij_layout_path."}
            }
            { ok: true, path: $candidate }
        } catch {|e|
            { ok: false, msg: $e.msg }
        }

        if $layout_result.ok {
            {
                extra: []
                shared: {
                    zellij_layout_path: $layout_result.path
                    terminals: $terminals
                    startup_script_path: ($runtime_dir | path join "nushell" "scripts" "core" "start_yazelix_inner.nu")
                    launch_script_path: ($runtime_dir | path join "nushell" "scripts" "core" "launch_yazelix.nu")
                    command_search_paths: (get_command_search_paths_for_runtime_doctor)
                    platform_name: (get_runtime_platform_name)
                }
            }
        } else {
            {
                extra: [{
                    status: "error"
                    message: "Could not resolve the managed Zellij layout path from the Rust materialization plan"
                    details: $layout_result.msg
                    fix_available: false
                }]
                shared: null
            }
        }
    }

    let req = {
        runtime_dir: $rd
        yazelix_state_dir: $state_dir
        has_home_manager_managed_install: $install_io.has_home_manager_managed_install
        is_manual_runtime_reference_path: $install_io.is_manual_runtime_reference_path
        shared_runtime: $layout_bundle.shared
    }

    let data = (evaluate_doctor_runtime_report $req)
    let from_rust = ($data.shared_runtime_preflight? | default [])

    {
        distribution: $data.distribution
        shared_runtime_preflight: ($layout_bundle.extra | append $from_rust)
    }
}
