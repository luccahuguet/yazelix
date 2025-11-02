#!/usr/bin/env nu
# Config state tracking for Yazelix

use ./config_parser.nu parse_yazelix_config

# Compute active config hash and track whether devenv needs cache refresh.
# Returns a record with:
#   config: parsed Yazelix configuration
#   config_file: path to the active config file
#   needs_refresh: true when the hash changed since last launch
#   current_hash: sha256 of the active config file (empty string if missing)
#   cached_hash: previously stored hash (empty if none)
export def compute_config_state [] {
    let config = parse_yazelix_config
    let config_file = $config.config_file

    let cache_dir = "~/.local/share/yazelix/state" | path expand
    if not ($cache_dir | path exists) {
        mkdir $cache_dir
    }

    let cache_file = ($cache_dir | path join "config_hash")
    let current_hash = if ($config_file | is-empty) or (not ($config_file | path exists)) {
        ""
    } else {
        open --raw $config_file | hash sha256
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
