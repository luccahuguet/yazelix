#!/usr/bin/env nu

use runtime_paths.nu [get_yazelix_runtime_dir]
use yzx_core_bridge.nu [
    build_default_yzx_core_error_surface
    run_yzx_core_json_command
]

const TRANSIENT_PANE_FACTS_COMPUTE_COMMAND = "transient-pane-facts.compute"

export def load_transient_pane_facts [] {
    let runtime_dir = (get_yazelix_runtime_dir)
    run_yzx_core_json_command $runtime_dir (build_default_yzx_core_error_surface) [
        $TRANSIENT_PANE_FACTS_COMPUTE_COMMAND
    ] "Yazelix Rust transient-pane-facts helper returned invalid JSON."
}
