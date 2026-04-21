#!/usr/bin/env nu
# Shared transport for structured doctor and install report evaluation.

use common.nu [
    get_generated_helix_config_path
    get_managed_helix_user_config_path
    get_native_helix_config_path
    get_runtime_platform_name
    get_yazelix_config_dir
    get_yazelix_runtime_dir
    get_yazelix_state_dir
    normalize_path_entries
    require_yazelix_runtime_dir
]
use config_parser.nu parse_yazelix_config
use config_surfaces.nu get_main_user_config_path
use constants.nu DEFAULT_TERMINAL
use ./yzx_core_bridge.nu [build_default_yzx_core_error_surface run_yzx_core_json_command run_yzx_core_request_json_command]

const INSTALL_OWNERSHIP_EVALUATE_COMMAND = "install-ownership.evaluate"
const DOCTOR_CONFIG_EVALUATE_COMMAND = "doctor-config.evaluate"
const DOCTOR_HELIX_EVALUATE_COMMAND = "doctor-helix.evaluate"
const DOCTOR_RUNTIME_EVALUATE_COMMAND = "doctor-runtime.evaluate"
const RUNTIME_MATERIALIZATION_PLAN_COMMAND = "runtime-materialization.plan"

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

def get_hx_exe_path_for_report [] {
    try {
        (
            which hx
            | where type == "external"
            | get -o 0.path
            | default null
        )
    } catch {
        null
    }
}

def get_command_search_paths_for_runtime_doctor [] {
    normalize_path_entries ($env.PATH? | default []) | each {|p| $p | path expand }
}

def evaluate_helix_doctor_report [] {
    let runtime_dir = (require_yazelix_runtime_dir)
    let home = ($env.HOME | path expand)
    let parse_out = try {
        { ok: true, config: (parse_yazelix_config) }
    } catch {
        { ok: false, config: null }
    }

    mut editor_for_req = null
    if $parse_out.ok {
        let editor = (
            $parse_out.config
            | get -o editor_command
            | default ""
            | into string
            | str trim
        )
        $editor_for_req = $editor
    }

    let req = {
        home_dir: $home
        runtime_dir: $runtime_dir
        config_dir: (get_yazelix_config_dir)
        user_config_helix_runtime_dir: ($home | path join ".config" "helix" "runtime")
        hx_exe_path: (get_hx_exe_path_for_report)
        include_runtime_health: (($env.EDITOR? | default "" | str contains "hx"))
        editor_command: $editor_for_req
        managed_helix_user_config_path: (get_managed_helix_user_config_path)
        native_helix_config_path: (get_native_helix_config_path)
        generated_helix_config_path: (get_generated_helix_config_path)
    }

    run_yzx_core_request_json_command $runtime_dir (
        build_default_yzx_core_error_surface
    ) $DOCTOR_HELIX_EVALUATE_COMMAND $req "Yazelix Rust doctor-helix helper returned invalid JSON."
}

def collect_helix_doctor_findings [] {
    let data = (evaluate_helix_doctor_report)
    mut findings = [($data.runtime_conflicts)]

    if ($data.runtime_health? | default null) != null {
        $findings = ($findings | append $data.runtime_health)
    }

    for finding in ($data.managed_integration? | default []) {
        $findings = ($findings | append $finding)
    }

    $findings
}

def collect_runtime_doctor_findings [install_report: record] {
    let runtime_dir = (require_yazelix_runtime_dir)
    let state_dir = (get_yazelix_state_dir)
    let parse_out = try {
        { ok: true, config: (parse_yazelix_config) }
    } catch {
        { ok: false, config: null }
    }

    let layout_bundle = if not $parse_out.ok {
        { extra: [], shared: null }
    } else {
        let config = $parse_out.config
        let active_runtime_dir = (get_yazelix_runtime_dir)
        let terminals = ($config.terminals? | default [$DEFAULT_TERMINAL] | uniq)

        let layout_result = try {
            let plan = (run_yzx_core_json_command
                $runtime_dir
                (build_default_yzx_core_error_surface)
                [$RUNTIME_MATERIALIZATION_PLAN_COMMAND "--from-env"]
                "Yazelix Rust runtime-materialization helper returned invalid JSON.")
            let candidate = (
                $plan.zellij_layout_path?
                | default ""
                | into string
                | str trim
            )

            if ($candidate | is-empty) {
                error make {msg: "Rust materialization plan omitted zellij_layout_path."}
            }

            { ok: true, path: $candidate }
        } catch {|err|
            { ok: false, msg: $err.msg }
        }

        if $layout_result.ok {
            {
                extra: []
                shared: {
                    zellij_layout_path: $layout_result.path
                    terminals: $terminals
                    startup_script_path: ($active_runtime_dir | path join "nushell" "scripts" "core" "start_yazelix_inner.nu")
                    launch_script_path: ($active_runtime_dir | path join "nushell" "scripts" "core" "launch_yazelix.nu")
                    command_search_paths: (get_command_search_paths_for_runtime_doctor)
                    platform_name: (get_runtime_platform_name)
                }
            }
        } else {
            {
                extra: [{
                    status: "error"
                    message: "Could not resolve the managed Zellij layout path from the Rust materialization plan"
                    details: $layout_result.msg
                    fix_available: false
                }]
                shared: null
            }
        }
    }

    let data = (run_yzx_core_request_json_command
        $runtime_dir
        (build_default_yzx_core_error_surface)
        $DOCTOR_RUNTIME_EVALUATE_COMMAND
        {
            runtime_dir: $runtime_dir
            yazelix_state_dir: $state_dir
            has_home_manager_managed_install: $install_report.has_home_manager_managed_install
            is_manual_runtime_reference_path: $install_report.is_manual_runtime_reference_path
            shared_runtime: $layout_bundle.shared
        }
        "Yazelix Rust doctor-runtime helper returned invalid JSON.")

    mut findings = [($data.distribution)]
    for finding in ($layout_bundle.extra | default []) {
        $findings = ($findings | append $finding)
    }
    for finding in ($data.shared_runtime_preflight? | default []) {
        $findings = ($findings | append $finding)
    }

    $findings
}

def collect_config_doctor_findings [] {
    let runtime_dir = (require_yazelix_runtime_dir)
    let data = (run_yzx_core_request_json_command
        $runtime_dir
        (build_default_yzx_core_error_surface)
        $DOCTOR_CONFIG_EVALUATE_COMMAND
        {
            config_dir: (get_yazelix_config_dir)
            runtime_dir: $runtime_dir
        }
        "Yazelix Rust doctor-config helper returned invalid JSON.")

    $data.findings
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

export def collect_structured_doctor_findings [] {
    let install_report = (evaluate_install_ownership_report)
    mut findings = []

    for finding in (collect_runtime_doctor_findings $install_report) {
        $findings = ($findings | append $finding)
    }
    for finding in (collect_helix_doctor_findings) {
        $findings = ($findings | append $finding)
    }
    for finding in (collect_config_doctor_findings) {
        $findings = ($findings | append $finding)
    }
    for finding in ($install_report.wrapper_shadowing? | default []) {
        $findings = ($findings | append $finding)
    }

    $findings = ($findings | append $install_report.desktop_entry_freshness)
    $findings
}
