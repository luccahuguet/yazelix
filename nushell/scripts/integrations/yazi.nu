#!/usr/bin/env nu
# Yazi integration utilities for Yazelix

use ../utils/logging.nu log_to_file
use ../utils/runtime_paths.nu [get_yazelix_runtime_dir]
use ../utils/yzx_core_bridge.nu [build_default_yzx_core_error_surface run_yzx_core_json_command]

def resolve_optional_command [configured: any, fallback: string] {
    let raw = ($configured | default "" | into string | str trim)
    if ($raw | is-empty) {
        $fallback
    } else {
        $raw
    }
}

def command_is_available [command: string] {
    let normalized = ($command | str trim)
    if ($normalized | is-empty) {
        false
    } else if (($normalized | str contains "/") or ($normalized | str starts-with "~")) {
        (($normalized | path expand) | path exists)
    } else {
        (which $normalized | is-not-empty)
    }
}

export def get_yazi_command [] {
    let runtime_dir = (get_yazelix_runtime_dir)
    let facts = (run_yzx_core_json_command $runtime_dir (build_default_yzx_core_error_surface) [
        "integration-facts.compute"
    ] "Yazelix Rust integration-facts helper returned invalid JSON.")
    resolve_optional_command ($facts.yazi_command? | default null) "yazi"
}

export def get_ya_command [] {
    let runtime_dir = (get_yazelix_runtime_dir)
    let facts = (run_yzx_core_json_command $runtime_dir (build_default_yzx_core_error_surface) [
        "integration-facts.compute"
    ] "Yazelix Rust integration-facts helper returned invalid JSON.")
    resolve_optional_command ($facts.ya_command? | default null) "ya"
}

export def is_sidebar_enabled [] {
    let runtime_dir = (get_yazelix_runtime_dir)
    let facts = (run_yzx_core_json_command $runtime_dir (build_default_yzx_core_error_surface) [
        "integration-facts.compute"
    ] "Yazelix Rust integration-facts helper returned invalid JSON.")
    ($facts.enable_sidebar? | default true)
}

export def consume_bootstrap_sidebar_cwd [] {
    let cwd_file = ($env.YAZELIX_BOOTSTRAP_SIDEBAR_CWD_FILE? | default "" | str trim)
    if ($cwd_file | is-empty) {
        return null
    }

    let expanded_file = ($cwd_file | path expand)
    if not ($expanded_file | path exists) {
        return null
    }

    let requested_path = (open --raw $expanded_file | str trim)
    rm -f $expanded_file

    if ($requested_path | is-empty) {
        return null
    }

    let expanded_path = ($requested_path | path expand)
    if not ($expanded_path | path exists) {
        return null
    }

    if (($expanded_path | path type) == "dir") {
        $expanded_path
    } else {
        $expanded_path | path dirname
    }
}

export def sync_sidebar_yazi_state_to_directory [sidebar_state: record, target_path: path, log_file: string = "yazi_sync.log"] {
    let expanded_target_path = ($target_path | path expand)
    let target_dir = if (($expanded_target_path | path type) == "dir") {
        $expanded_target_path
    } else {
        $expanded_target_path | path dirname
    }

    let ya_command = (get_ya_command)
    let result = (^$ya_command "emit-to" $sidebar_state.yazi_id "cd" $target_dir | complete)
    if $result.exit_code != 0 {
        let details = ($result.stderr | default ($result.stdout | default "") | str trim)
        let reason = if ($details | is-empty) { $"exit code ($result.exit_code)" } else { $details }
        log_to_file $log_file $"Failed to sync active sidebar Yazi to directory '($target_dir)': ($reason)"
        {status: "error", reason: $reason, target_dir: $target_dir}
    } else {
        log_to_file $log_file $"Synced active sidebar Yazi to directory: ($target_dir)"
        {status: "ok", target_dir: $target_dir}
    }
}
