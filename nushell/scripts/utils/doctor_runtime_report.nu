#!/usr/bin/env nu

use common.nu [get_yazelix_runtime_dir get_yazelix_state_dir require_yazelix_runtime_dir]
use config_parser.nu [parse_yazelix_config run_yzx_core_json_command_with_error_surface]
use constants.nu DEFAULT_TERMINAL
use generated_runtime_state.nu compute_runtime_materialization_plan

const DOCTOR_RUNTIME_EVALUATE_COMMAND = "doctor-runtime.evaluate"

def doctor_runtime_error_surface [] {
    {
        display_config_path: ""
        config_file: ""
    }
}

def normalize_path_entries [value: any] {
    let described = ($value | describe)

    if ($described | str starts-with "list") {
        $value | each {|entry| $entry | into string }
    } else {
        let text = ($value | into string | str trim)
        if ($text | is-empty) {
            []
        } else {
            $text | split row (char esep)
        }
    }
}

def get_command_search_paths_for_runtime_doctor [] {
    normalize_path_entries ($env.PATH? | default []) | each {|p| $p | path expand }
}

def get_runtime_doctor_platform_name [] {
    (
        $env.YAZELIX_TEST_OS?
        | default $nu.os-info.name
        | into string
        | str trim
        | str downcase
    )
}

export def evaluate_doctor_runtime_report [req: record] {
    let rd = require_yazelix_runtime_dir
    let helper_args = [
        $DOCTOR_RUNTIME_EVALUATE_COMMAND
        "--request-json"
        ($req | to json -r)
    ]

    run_yzx_core_json_command_with_error_surface $rd (doctor_runtime_error_surface) $helper_args "Yazelix Rust doctor-runtime helper returned invalid JSON."
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
                    platform_name: (get_runtime_doctor_platform_name)
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
