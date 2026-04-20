#!/usr/bin/env nu
# Thin wrapper for Yazelix-managed Helix config generation.
# The real owner is helix-materialization.generate in yzx_core.

use ../utils/common.nu [get_yazelix_runtime_dir get_yazelix_state_dir get_yazelix_user_config_dir get_yazelix_config_dir]
use ../utils/config_parser.nu [run_yzx_core_json_command resolve_yzx_core_helper_path]

const MANAGED_REVEAL_COMMAND = ':sh yzx reveal "%{buffer_name}"'

export def get_managed_reveal_command [] {
    $MANAGED_REVEAL_COMMAND
}

export def get_managed_helix_user_config_dir [] {
    (get_yazelix_user_config_dir) | path join "helix"
}

export def get_managed_helix_user_config_path [] {
    (get_managed_helix_user_config_dir) | path join "config.toml"
}

export def get_native_helix_config_path [] {
    let xdg_config_home = (
        $env.XDG_CONFIG_HOME?
        | default ($env.HOME | path join ".config")
        | into string
        | str trim
        | path expand
    )
    ($xdg_config_home | path join "helix" "config.toml")
}

export def get_generated_helix_config_dir [] {
    (get_yazelix_state_dir) | path join "configs" "helix"
}

export def get_generated_helix_config_path [] {
    (get_generated_helix_config_dir) | path join "config.toml"
}

export def generate_managed_helix_config [] {
    let runtime_dir = (get_yazelix_runtime_dir)
    let config_dir = (get_yazelix_config_dir)
    let state_dir = (get_yazelix_state_dir)

    let result = (run_yzx_core_json_command
        $runtime_dir
        {display_config_path: "" config_file: ""}
        [
            "helix-materialization.generate"
            "--runtime-dir" $runtime_dir
            "--config-dir" $config_dir
            "--state-dir" $state_dir
        ]
        "Yazelix Rust helix-materialization helper returned invalid JSON.")

    if ($result.import_notice? | is-not-empty) {
        for line in ($result.import_notice.lines? | default []) {
            print --stderr $line
        }
    }

    $result.generated_path
}

export def build_managed_helix_config [] {
    generate_managed_helix_config
    open (get_generated_helix_config_path)
}

export def main [--print-path] {
    let output_path = (generate_managed_helix_config)
    if $print_path {
        print $output_path
    }
}
