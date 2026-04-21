#!/usr/bin/env nu

use common.nu [get_yazelix_state_dir require_yazelix_runtime_dir]
use config_surfaces.nu get_main_user_config_path
use ./yzx_core_bridge.nu [build_default_yzx_core_error_surface run_yzx_core_request_json_command]

const INSTALL_OWNERSHIP_EVALUATE_COMMAND = "install-ownership.evaluate"

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
    let resolved_runtime_dir = if $runtime_dir != null { $runtime_dir } else { require_yazelix_runtime_dir }

    run_yzx_core_request_json_command $resolved_runtime_dir (
        build_default_yzx_core_error_surface
    ) $INSTALL_OWNERSHIP_EVALUATE_COMMAND {
        runtime_dir: $resolved_runtime_dir
        home_dir: ($env.HOME | path expand)
        user: ($env.USER? | default null)
        xdg_config_home: (get_xdg_config_home)
        xdg_data_home: (get_xdg_data_home)
        yazelix_state_dir: (get_yazelix_state_dir)
        main_config_path: (get_main_user_config_path)
        invoked_yzx_path: ($env.YAZELIX_INVOKED_YZX_PATH? | default null)
        redirected_from_stale_yzx_path: ($env.YAZELIX_REDIRECTED_FROM_STALE_YZX_PATH? | default null)
        shell_resolved_yzx_path: (get_shell_resolved_yzx_path_for_report)
    } "Yazelix Rust install-ownership helper returned invalid JSON."
}

export def has_home_manager_managed_install [] {
    (evaluate_install_ownership_report).has_home_manager_managed_install
}
