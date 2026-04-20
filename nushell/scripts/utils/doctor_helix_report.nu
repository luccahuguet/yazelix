#!/usr/bin/env nu

use common.nu require_yazelix_runtime_dir
use config_parser.nu [parse_yazelix_config run_yzx_core_json_command_with_error_surface]
use ../setup/helix_config_merger.nu [
    build_managed_helix_config
    get_generated_helix_config_path
    get_managed_helix_user_config_path
    get_managed_reveal_command
    get_native_helix_config_path
]

const DOCTOR_HELIX_EVALUATE_COMMAND = "doctor-helix.evaluate"

def doctor_helix_error_surface [] {
    {
        display_config_path: ""
        config_file: ""
    }
}

def get_hx_exe_path_for_report [] {
    try {
        (
            which hx
            | where type == "external"
            | get -o 0.path
            | default null
        )
    } catch {
        null
    }
}

export def evaluate_helix_doctor_report [] {
    let rd = require_yazelix_runtime_dir
    let home = ($env.HOME | path expand)
    let user_rt = ($home | path join ".config" "helix" "runtime")

    let parse_out = try {
        { ok: true, config: (parse_yazelix_config) }
    } catch {
        { ok: false, config: null }
    }

    mut editor_for_req = null
    mut expected_json = null
    mut expected_err = null
    if $parse_out.ok {
        let editor = (
            $parse_out.config
            | get -o editor_command
            | default ""
            | into string
            | str trim
        )
        $editor_for_req = $editor
        let cfg_build = try {
            let j = (build_managed_helix_config | to json -r | from json)
            { err: null, json: $j }
        } catch {|e|
            { err: $e.msg, json: null }
        }
        $expected_json = $cfg_build.json
        $expected_err = $cfg_build.err
    }

    let include_health = ($env.EDITOR? | default "" | str contains "hx")
    let hx_path = (get_hx_exe_path_for_report)

    let req = {
        home_dir: $home
        user_config_helix_runtime_dir: $user_rt
        hx_exe_path: $hx_path
        include_runtime_health: $include_health
        editor_command: $editor_for_req
        managed_helix_user_config_path: (get_managed_helix_user_config_path)
        native_helix_config_path: (get_native_helix_config_path)
        generated_helix_config_path: (get_generated_helix_config_path)
        expected_managed_config: $expected_json
        build_managed_config_error: $expected_err
        reveal_binding_expected: (get_managed_reveal_command)
    }

    let helper_args = [
        $DOCTOR_HELIX_EVALUATE_COMMAND
        "--request-json"
        ($req | to json -r)
    ]

    run_yzx_core_json_command_with_error_surface $rd (doctor_helix_error_surface) $helper_args "Yazelix Rust doctor-helix helper returned invalid JSON."
}

export def collect_helix_doctor_results [] {
    let data = (evaluate_helix_doctor_report)
    {
        runtime_conflicts: $data.runtime_conflicts
        runtime_health: ($data.runtime_health? | default null)
        managed_integration: ($data.managed_integration? | default [])
    }
}
