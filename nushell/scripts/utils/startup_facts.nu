#!/usr/bin/env nu

use common.nu [get_yazelix_runtime_dir]
use yzx_core_bridge.nu [
    build_default_yzx_core_error_surface
    run_yzx_core_json_command
]

const STARTUP_FACTS_COMPUTE_COMMAND = "startup-facts.compute"

export def load_startup_facts [] {
    let runtime_dir = (get_yazelix_runtime_dir)
    run_yzx_core_json_command $runtime_dir (build_default_yzx_core_error_surface) [
        $STARTUP_FACTS_COMPUTE_COMMAND
    ] "Yazelix Rust startup-facts helper returned invalid JSON."
}
