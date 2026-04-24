#!/usr/bin/env nu

use ../utils/runtime_paths.nu [get_yazelix_runtime_dir]
use ../utils/yzx_core_bridge.nu [build_default_yzx_core_error_surface run_yzx_core_json_command]

def consume_bootstrap_sidebar_cwd [] {
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

def main [] {
    let bootstrap_dir = (consume_bootstrap_sidebar_cwd)
    let target_dir = if ($bootstrap_dir | is-not-empty) {
        $bootstrap_dir
    } else {
        pwd | path expand
    }
    let runtime_dir = (get_yazelix_runtime_dir)
    let facts = (run_yzx_core_json_command $runtime_dir (build_default_yzx_core_error_surface) [
        "integration-facts.compute"
    ] "Yazelix Rust integration-facts helper returned invalid JSON.")
    let yazi_command = (
        $facts.yazi_command?
        | default "yazi"
        | into string
        | str trim
    )
    run-external $yazi_command $target_dir
}
