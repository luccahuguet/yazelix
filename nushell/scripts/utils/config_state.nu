#!/usr/bin/env nu
# Config state tracking for Yazelix

use ./config_parser.nu parse_yazelix_config
use ./config_contract.nu [get_main_config_rebuild_required_paths]
use ./common.nu [get_yazelix_runtime_dir get_yazelix_state_dir]
use ./config_surfaces.nu [load_active_config_surface get_main_user_config_path normalize_config_surface_path]
use ./launch_state.nu [has_matching_launch_state]

# Extract a nested key from a record using dot notation (e.g., "core.recommended_deps")
def get_nested_key [record: record, key: string] {
    let parts = ($key | split row ".")
    mut value = $record
    for part in $parts {
        $value = ($value | get -o $part)
        if ($value == null) {
            return null
        }
    }
    $value
}

# Set a nested key in a record using dot notation
def set_nested_key [record: record, key: string, value: any] {
    let parts = ($key | split row ".")
    if ($parts | length) == 1 {
        return ($record | upsert ($parts | first) $value)
    }

    let first = ($parts | first)
    let rest = ($parts | skip 1 | str join ".")
    let nested = ($record | get -o $first | default {})

    $record | upsert $first (set_nested_key $nested $rest $value)
}

# Extract only rebuild-required keys from full config
def extract_rebuild_config [config: record] {
    mut rebuild_config = {}
    let rebuild_required_keys = (get_main_config_rebuild_required_paths)

    for key in $rebuild_required_keys {
        let value = (get_nested_key $config $key)
        if ($value != null) {
            $rebuild_config = (set_nested_key $rebuild_config $key $value)
        }
    }

    $rebuild_config
}

def get_materialized_state_path [] {
    (get_yazelix_state_dir | path join "state" "rebuild_hash")
}

def load_recorded_materialized_state [] {
    let materialized_state_path = (get_materialized_state_path)
    if not ($materialized_state_path | path exists) {
        return null
    }

    let raw_state = (open --raw $materialized_state_path | str trim)
    if ($raw_state | is-empty) {
        return null
    }

    try {
        $raw_state | from json
    } catch {
        $raw_state
    }
}

# Compute active config hash and track whether devenv needs cache refresh.
# Only hashes rebuild-required keys (ignoring comments and runtime settings).
# Returns a record with:
#   config: parsed Yazelix configuration
#   config_file: path to the active config file
#   needs_refresh: true when the hash changed since last launch
#   current_hash: sha256 of rebuild-required config (empty string if missing)
#   cached_hash: previously stored hash (empty if none)
export def compute_config_state [] {
    let config_surface = (load_active_config_surface)
    let config = parse_yazelix_config
    let config_file = $config.config_file

    let materialized_state_path = (get_materialized_state_path)
    let materialized_state_dir = ($materialized_state_path | path dirname)
    if not ($materialized_state_dir | path exists) {
        mkdir $materialized_state_dir
    }

    let config_hash = if ($config_file | is-empty) or (not ($config_file | path exists)) {
        ""
    } else {
        let rebuild_config = (extract_rebuild_config $config_surface.merged_config)
        let normalized = ($rebuild_config | to toml)
        $normalized | hash sha256
    }

    # Include devenv inputs so updates trigger refresh on restart
    let yazelix_dir = get_yazelix_runtime_dir
    let lock_path = ($yazelix_dir | path join "devenv.lock")
    let devenv_nix_path = ($yazelix_dir | path join "devenv.nix")
    let devenv_yaml_path = ($yazelix_dir | path join "devenv.yaml")

    let lock_hash = if ($lock_path | path exists) {
        open --raw $lock_path | hash sha256
    } else {
        ""
    }
    let devenv_nix_hash = if ($devenv_nix_path | path exists) {
        open --raw $devenv_nix_path | hash sha256
    } else {
        ""
    }
    let devenv_yaml_hash = if ($devenv_yaml_path | path exists) {
        open --raw $devenv_yaml_path | hash sha256
    } else {
        ""
    }
    let runtime_hash = (
        $yazelix_dir
        | path expand
        | hash sha256
    )

    let combined_hash = [$config_hash, $lock_hash, $devenv_nix_hash, $devenv_yaml_hash, $runtime_hash]
        | str join ":"
        | hash sha256

    let cached_state = (load_recorded_materialized_state)

    let cached_state_type = ($cached_state | describe)
    let cached_hash = if $cached_state_type == "string" {
        $cached_state
    } else {
        ""
    }

    let has_structured_cache = ($cached_state_type | str starts-with "record")
    let cached_config_hash = if $has_structured_cache {
        $cached_state | get -o config_hash | default ""
    } else {
        ""
    }
    let cached_lock_hash = if $has_structured_cache {
        $cached_state | get -o lock_hash | default ""
    } else {
        ""
    }
    let cached_devenv_nix_hash = if $has_structured_cache {
        $cached_state | get -o devenv_nix_hash | default ""
    } else {
        ""
    }
    let cached_devenv_yaml_hash = if $has_structured_cache {
        $cached_state | get -o devenv_yaml_hash | default ""
    } else {
        ""
    }
    let cached_runtime_hash = if $has_structured_cache {
        $cached_state | get -o runtime_hash | default ""
    } else {
        ""
    }
    let config_changed = if $has_structured_cache {
        $config_hash != $cached_config_hash
    } else {
        false
    }
    let inputs_changed = if $has_structured_cache {
        (
            ($lock_hash != $cached_lock_hash)
            or ($devenv_nix_hash != $cached_devenv_nix_hash)
            or ($devenv_yaml_hash != $cached_devenv_yaml_hash)
            or ($runtime_hash != $cached_runtime_hash)
        )
    } else {
        false
    }

    let inputs_require_refresh = if $has_structured_cache {
        $config_changed or $inputs_changed
    } else if ($cached_hash | is-not-empty) {
        $combined_hash != $cached_hash
    } else {
        true
    }
    let has_verified_launch_profile = (has_matching_launch_state {combined_hash: $combined_hash} --allow-stale=false)
    let needs_refresh = $inputs_require_refresh or (not $has_verified_launch_profile)

    let refresh_reason = if not $needs_refresh {
        ""
    } else if $inputs_require_refresh and (not $has_structured_cache) {
        "config, runtime, or devenv inputs changed since last launch"
    } else if $inputs_require_refresh and $config_changed and $inputs_changed {
        "config and runtime/devenv inputs changed since last launch"
    } else if $inputs_require_refresh and $config_changed {
        "config changed since last launch"
    } else if $inputs_require_refresh and $inputs_changed {
        "runtime or devenv inputs changed since last launch"
    } else if not $has_verified_launch_profile {
        "verified launch profile missing for current config"
    } else {
        "config, runtime, or devenv inputs changed since last launch"
    }

    {
        config: $config
        config_file: $config_file
        needs_refresh: $needs_refresh
        refresh_reason: $refresh_reason
        config_changed: $config_changed
        inputs_changed: $inputs_changed
        inputs_require_refresh: $inputs_require_refresh
        has_verified_launch_profile: $has_verified_launch_profile
        config_hash: $config_hash
        lock_hash: $lock_hash
        devenv_nix_hash: $devenv_nix_hash
        devenv_yaml_hash: $devenv_yaml_hash
        runtime_hash: $runtime_hash
        combined_hash: $combined_hash
        cached_hash: $cached_hash
    }
}

# Record that the current config/runtime inputs have been materialized into the
# canonical Yazelix build state for the default managed config surface.
export def record_materialized_state [state: record] {
    let config_file = ($state.config_file? | default "")
    let default_config = (normalize_config_surface_path (get_main_user_config_path))
    if ($config_file | is-not-empty) and ((normalize_config_surface_path $config_file) != $default_config) {
        return
    }

    let materialized_state_path = (get_materialized_state_path)
    let materialized_state_dir = ($materialized_state_path | path dirname)
    if not ($materialized_state_dir | path exists) {
        mkdir $materialized_state_dir
    }
    let cache_record = {
        config_hash: ($state.config_hash? | default "")
        lock_hash: ($state.lock_hash? | default "")
        devenv_nix_hash: ($state.devenv_nix_hash? | default "")
        devenv_yaml_hash: ($state.devenv_yaml_hash? | default "")
        runtime_hash: ($state.runtime_hash? | default "")
    }
    $cache_record | to json | save --force $materialized_state_path
}
