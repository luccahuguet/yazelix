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
    let current_hash = if ($config_file | is-empty) or (not ($config_file | path exists)) {
        ""
    } else {
        # Load full TOML, extract rebuild-required keys, normalize, and hash
        let full_config = (open $config_file)
        let rebuild_config = (extract_rebuild_config $full_config)
        let normalized = ($rebuild_config | to toml)
        $normalized | hash sha256
    }

    let cached_hash = if ($cache_file | path exists) {
        try {
            open $cache_file | str trim
        } catch {
            ""
        }
    } else {
        ""
    }

    let needs_refresh = ($current_hash != $cached_hash)

    {
        config: $config
        config_file: $config_file
        needs_refresh: $needs_refresh
        current_hash: $current_hash
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
    ($state.current_hash? | default "") | save --force $cache_file
}
