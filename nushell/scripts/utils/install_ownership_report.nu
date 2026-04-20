#!/usr/bin/env nu

use common.nu [get_yazelix_state_dir require_yazelix_runtime_dir]
use config_parser.nu run_yzx_core_json_command_with_error_surface
use config_surfaces.nu get_main_user_config_path

const INSTALL_OWNERSHIP_EVALUATE_COMMAND = "install-ownership.evaluate"

def install_ownership_error_surface [] {
    {
        display_config_path: ""
        config_file: ""
    }
}

def get_xdg_config_home [] {
    let configured = (
        $env.XDG_CONFIG_HOME?
        | default ""
        | into string
        | str trim
    )

    if ($configured | is-not-empty) {
        $configured | path expand
    } else if (($env.HOME? | default "" | into string | str trim) | is-not-empty) {
        $env.HOME | path join ".config"
    } else {
        "~/.config" | path expand
    }
}

def get_xdg_data_home [] {
    let configured = (
        $env.XDG_DATA_HOME?
        | default ""
        | into string
        | str trim
    )

    if ($configured | is-not-empty) {
        $configured | path expand
    } else if (($env.HOME? | default "" | into string | str trim) | is-not-empty) {
        $env.HOME | path join ".local" "share"
    } else {
        "~/.local/share" | path expand
    }
}

def get_shell_resolved_yzx_path_for_report [] {
    let invoked = (
        $env.YAZELIX_INVOKED_YZX_PATH?
        | default ""
        | into string
        | str trim
    )

    if ($invoked | is-not-empty) {
        return ($invoked | path expand --no-symlink)
    }

    let resolved = (
        which yzx
        | where type == "external"
        | get -o 0.path
        | default null
    )

    if $resolved == null {
        null
    } else {
        $resolved | path expand --no-symlink
    }
}

export def evaluate_install_ownership_report [--runtime-dir: string] {
    let rd = if $runtime_dir != null { $runtime_dir } else { require_yazelix_runtime_dir }

    let req = {
        runtime_dir: $rd
        home_dir: ($env.HOME | path expand)
        user: ($env.USER? | default null)
        xdg_config_home: (get_xdg_config_home)
        xdg_data_home: (get_xdg_data_home)
        yazelix_state_dir: (get_yazelix_state_dir)
        main_config_path: (get_main_user_config_path)
        invoked_yzx_path: ($env.YAZELIX_INVOKED_YZX_PATH? | default null)
        redirected_from_stale_yzx_path: ($env.YAZELIX_REDIRECTED_FROM_STALE_YZX_PATH? | default null)
        shell_resolved_yzx_path: (get_shell_resolved_yzx_path_for_report)
    }

    let helper_args = [
        $INSTALL_OWNERSHIP_EVALUATE_COMMAND
        "--request-json"
        ($req | to json -r)
    ]

    run_yzx_core_json_command_with_error_surface $rd (install_ownership_error_surface) $helper_args "Yazelix Rust install-ownership helper returned invalid JSON."
}
