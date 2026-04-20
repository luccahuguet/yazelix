#!/usr/bin/env nu

use common.nu [get_yazelix_config_dir require_yazelix_runtime_dir]
use config_parser.nu [run_yzx_core_json_command_with_error_surface]

const DOCTOR_CONFIG_EVALUATE_COMMAND = "doctor-config.evaluate"

def doctor_config_error_surface [] {
    {
        display_config_path: ""
        config_file: ""
    }
}

export def evaluate_doctor_config_report [req: record] {
    let rd = require_yazelix_runtime_dir
    let helper_args = [
        $DOCTOR_CONFIG_EVALUATE_COMMAND
        "--request-json"
        ($req | to json -r)
    ]

    run_yzx_core_json_command_with_error_surface $rd (doctor_config_error_surface) $helper_args "Yazelix Rust doctor-config helper returned invalid JSON."
}

export def collect_config_doctor_results [] {
    let data = (evaluate_doctor_config_report {
        config_dir: (get_yazelix_config_dir)
        runtime_dir: (require_yazelix_runtime_dir)
    })

    $data.findings
}
