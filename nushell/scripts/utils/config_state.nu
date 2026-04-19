#!/usr/bin/env nu
# Config state tracking for Yazelix

use ./config_parser.nu [render_yzx_core_error resolve_yzx_core_helper_path]
use ./common.nu [get_yazelix_state_dir require_yazelix_runtime_dir]
use ./config_contract.nu MAIN_CONFIG_CONTRACT_RELATIVE_PATH
use ./config_surfaces.nu [load_active_config_surface get_main_user_config_path]

def get_materialized_state_path [] {
    (get_yazelix_state_dir | path join "state" "rebuild_hash")
}

# Compute active config hash and track whether generated runtime state needs repair.
# Only hashes rebuild-required keys (ignoring comments and runtime settings).
# Returns a record with:
#   config: parsed Yazelix configuration
#   config_file: path to the active config file
#   needs_refresh: true when the materialized generated state is stale
#   config_hash: sha256 of rebuild-required config
#   runtime_hash: sha256 of the active runtime identity
#   combined_hash: sha256 of config_hash + runtime_hash
#   cached_hash: previously stored hash (empty if none)
export def compute_config_state [] {
    let config_surface = (load_active_config_surface)
    let materialized_state_path = (get_materialized_state_path)
    let materialized_state_dir = ($materialized_state_path | path dirname)
    if not ($materialized_state_dir | path exists) {
        mkdir $materialized_state_dir
    }
    let runtime_dir = require_yazelix_runtime_dir
    let helper_path = resolve_yzx_core_helper_path $runtime_dir
    let config_path = $config_surface.config_file
    let default_config_path = $config_surface.default_config_path
    let contract_path = ($runtime_dir | path join $MAIN_CONFIG_CONTRACT_RELATIVE_PATH)
    let helper_args = [
        "config-state.compute"
        "--config"
        $config_path
        "--default-config"
        $default_config_path
        "--contract"
        $contract_path
        "--runtime-dir"
        $runtime_dir
        "--state-path"
        $materialized_state_path
    ]
    let result = (
        do {
            ^$helper_path ...$helper_args
        } | complete
    )

    if $result.exit_code != 0 {
        error make {msg: (render_yzx_core_error $config_surface $result.stderr)}
    }

    let envelope = (
        try {
            $result.stdout | from json
        } catch {|err|
            error make {msg: $"Yazelix Rust config-state helper returned invalid JSON.\n($err.msg)"}
        }
    )

    if (($envelope.status? | default "") != "ok") {
        error make {msg: (render_yzx_core_error $config_surface ($result.stdout | default ""))}
    }

    $envelope.data
}

# Record that the current config/runtime inputs have been materialized into the
# canonical Yazelix build state for the default managed config surface.
export def record_materialized_state [state: record] {
    let config_file = ($state.config_file? | default "")
    let runtime_dir = require_yazelix_runtime_dir
    let helper_path = resolve_yzx_core_helper_path $runtime_dir
    let materialized_state_path = (get_materialized_state_path)
    let helper_args = [
        "config-state.record"
        "--config-file"
        $config_file
        "--managed-config"
        (get_main_user_config_path)
        "--state-path"
        $materialized_state_path
        "--config-hash"
        ($state.config_hash? | default "")
        "--runtime-hash"
        ($state.runtime_hash? | default "")
    ]
    let result = (
        do {
            ^$helper_path ...$helper_args
        } | complete
    )

    if $result.exit_code != 0 {
        error make {msg: (render_yzx_core_error {display_config_path: $config_file} $result.stderr)}
    }
}
