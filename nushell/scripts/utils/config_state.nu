#!/usr/bin/env nu
# Config state tracking for Yazelix

use ./config_parser.nu parse_yazelix_config
use ./config_metadata.nu REBUILD_REQUIRED_KEYS

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

    for key in $REBUILD_REQUIRED_KEYS {
        let value = (get_nested_key $config $key)
        if ($value != null) {
            $rebuild_config = (set_nested_key $rebuild_config $key $value)
        }
    }

    $rebuild_config
}

# Normalize TOML by parsing and re-serializing
# This ignores comments, whitespace, and key ordering
def normalize_toml [toml_string: string] {
    $toml_string | from toml | to toml
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
    let config = parse_yazelix_config
    let config_file = $config.config_file

    let cache_dir = "~/.local/share/yazelix/state" | path expand
    if not ($cache_dir | path exists) {
        mkdir $cache_dir
    }

    let cache_file = ($cache_dir | path join "rebuild_hash")
    let config_hash = if ($config_file | is-empty) or (not ($config_file | path exists)) {
        ""
    } else {
        # Load full TOML, extract rebuild-required keys, normalize, and hash
        let full_config = (open $config_file)
        let rebuild_config = (extract_rebuild_config $full_config)
        let normalized = ($rebuild_config | to toml)
        $normalized | hash sha256
    }

    # Include devenv inputs so updates trigger refresh on restart
    let yazelix_dir = "~/.config/yazelix" | path expand
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

    let combined_hash = [$config_hash, $lock_hash, $devenv_nix_hash, $devenv_yaml_hash]
        | str join ":"
        | hash sha256

    let cached_state = if ($cache_file | path exists) {
        let raw_cache = (open --raw $cache_file | str trim)
        if ($raw_cache | is-empty) {
            null
        } else {
            try {
                $raw_cache | from json
            } catch {
                $raw_cache
            }
        }
    } else {
        null
    }

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
        )
    } else {
        false
    }

    let needs_refresh = if $has_structured_cache {
        $config_changed or $inputs_changed
    } else if ($cached_hash | is-not-empty) {
        $combined_hash != $cached_hash
    } else {
        true
    }

    let refresh_reason = if not $needs_refresh {
        ""
    } else if not $has_structured_cache {
        "config or devenv inputs changed since last launch"
    } else if $config_changed and $inputs_changed {
        "config and devenv inputs changed since last launch"
    } else if $config_changed {
        "config changed since last launch"
    } else if $inputs_changed {
        "devenv inputs changed since last launch"
    } else {
        "config or devenv inputs changed since last launch"
    }

    {
        config: $config
        config_file: $config_file
        needs_refresh: $needs_refresh
        refresh_reason: $refresh_reason
        config_hash: $config_hash
        lock_hash: $lock_hash
        devenv_nix_hash: $devenv_nix_hash
        devenv_yaml_hash: $devenv_yaml_hash
        combined_hash: $combined_hash
        cached_hash: $cached_hash
        cache_file: $cache_file
    }
}

# Mark the current config hash as applied
export def mark_config_state_applied [state: record] {
    let config_file = ($state.config_file? | default "")
    let default_config = "~/.config/yazelix/yazelix.toml" | path expand
    if ($config_file | is-not-empty) and ($config_file | path expand) != $default_config {
        return
    }

    let cache_file = ($state.cache_file? | default null)
    if ($cache_file == null) {
        return
    }
    let cache_dir = ($cache_file | path dirname)
    if not ($cache_dir | path exists) {
        mkdir $cache_dir
    }
    let cache_record = {
        config_hash: ($state.config_hash? | default "")
        lock_hash: ($state.lock_hash? | default "")
        devenv_nix_hash: ($state.devenv_nix_hash? | default "")
        devenv_yaml_hash: ($state.devenv_yaml_hash? | default "")
    }
    $cache_record | to json | save --force $cache_file
}
