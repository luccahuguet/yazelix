#!/usr/bin/env nu

use common.nu [get_yazelix_runtime_dir]
use yzx_core_bridge.nu [
    build_default_yzx_core_error_surface
    run_yzx_core_json_command
]

const INTEGRATION_FACTS_COMPUTE_COMMAND = "integration-facts.compute"

export def load_integration_facts [] {
    let runtime_dir = (get_yazelix_runtime_dir)
    run_yzx_core_json_command $runtime_dir (build_default_yzx_core_error_surface) [
        $INTEGRATION_FACTS_COMPUTE_COMMAND
    ] "Yazelix Rust integration-facts helper returned invalid JSON."
}
