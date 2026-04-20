#!/usr/bin/env nu

use config_parser.nu [
    build_default_yzx_core_error_surface
    run_yzx_core_request_json_command
]
use common.nu [normalize_path_entries require_yazelix_runtime_dir get_runtime_platform_name]
use failure_classes.nu [format_failure_classification]

const RUNTIME_CONTRACT_EVALUATE_COMMAND = "runtime-contract.evaluate"
const STARTUP_LAUNCH_PREFLIGHT_EVALUATE_COMMAND = "startup-launch-preflight.evaluate"

def get_command_search_paths [] {
    normalize_path_entries ($env.PATH? | default [])
}

def evaluate_runtime_contract_checks [request: record] {
    let runtime_dir = (require_yazelix_runtime_dir)
    let data = (run_yzx_core_request_json_command
        $runtime_dir
        (build_default_yzx_core_error_surface)
        $RUNTIME_CONTRACT_EVALUATE_COMMAND
        $request
        "Yazelix Rust runtime-contract helper returned invalid JSON.")

    $data.checks? | default []
}

def evaluate_startup_launch_preflight_data [request: record] {
    let runtime_dir = (require_yazelix_runtime_dir)
    let data = (run_yzx_core_request_json_command
        $runtime_dir
        (build_default_yzx_core_error_surface)
        $STARTUP_LAUNCH_PREFLIGHT_EVALUATE_COMMAND
        $request
        "Yazelix Rust startup-launch-preflight helper returned invalid JSON.")

    $data
}

def first_runtime_check [checks: list<record>, id: string] {
    let check = ($checks | where {|candidate| ($candidate.id? | default "") == $id } | get -o 0)
    if $check == null {
        error make {msg: $"Missing runtime-contract check result for `($id)`."}
    }
    $check
}

def build_runtime_check_detail_lines [check: record] {
    mut detail_lines = []

    if (($check.details? | default "") | is-not-empty) {
        $detail_lines = ($detail_lines | append $check.details)
    }

    let recovery = ($check.recovery? | default "" | into string | str trim)
    let failure_class = ($check.failure_class? | default "" | into string | str trim)
    if ($recovery | is-not-empty) and ($failure_class | is-not-empty) {
        $detail_lines = ($detail_lines | append (format_failure_classification $failure_class $recovery))
    } else if ($recovery | is-not-empty) {
        $detail_lines = ($detail_lines | append $recovery)
    }

    $detail_lines
}

def runtime_check_to_error [check: record] {
    ([ $check.message ] | append (build_runtime_check_detail_lines $check) | str join "\n")
}

export def require_runtime_check [check: record] {
    if ($check.status == "ok") {
        return $check
    }

    error make {msg: (runtime_check_to_error $check)}
}

export def runtime_check_to_doctor_result [check: record] {
    let detail_lines = (build_runtime_check_detail_lines $check)

    {
        status: (if ($check.status == "ok") { "ok" } else { $check.severity })
        message: $check.message
        details: (if ($detail_lines | is-empty) { null } else { $detail_lines | str join "\n" })
        fix_available: false
        runtime_contract_check: $check.id
        owner_surface: $check.owner_surface
    }
}

def build_terminal_support_request [owner_surface: string, requested_terminal: string, terminals: list<string>] {
    {
        owner_surface: $owner_surface
        requested_terminal: $requested_terminal
        terminals: $terminals
        command_search_paths: (get_command_search_paths)
    }
}

def build_linux_ghostty_graphics_request [owner_surface: string, terminals: list<string>] {
    {
        owner_surface: $owner_surface
        terminals: $terminals
        runtime_dir: (require_yazelix_runtime_dir)
        command_search_paths: (get_command_search_paths)
        platform_name: (get_runtime_platform_name)
    }
}

export def run_startup_preflight [working_dir: string, script_path: string, label: string] {
    let data = (evaluate_startup_launch_preflight_data {
        startup: {
            working_dir: ($working_dir | path expand)
            runtime_script: {
                id: "startup_runtime_script"
                label: $label
                owner_surface: "startup"
                path: ($script_path | path expand)
            }
        }
    })
    if ($data.kind? | default "") != "startup" {
        error make {msg: "Unexpected startup-launch-preflight response \(expected startup\)."}
    }

    {
        working_dir: $data.working_dir
        script_path: ($data.script_path? | default "")
    }
}

export def run_launch_preflight [working_dir: string, requested_terminal: string, terminals: list<string>] {
    let data = (evaluate_startup_launch_preflight_data {
        launch: {
            working_dir: ($working_dir | path expand)
            requested_terminal: $requested_terminal
            terminals: $terminals
            command_search_paths: (get_command_search_paths)
        }
    })
    if ($data.kind? | default "") != "launch" {
        error make {msg: "Unexpected startup-launch-preflight response \(expected launch\)."}
    }

    {
        working_dir: $data.working_dir
        terminal_candidates: ($data.terminal_candidates? | default [])
    }
}

export def check_doctor_shared_runtime_preflight [
    layout_path: string
    terminals: list<string>
    runtime_scripts: list<record>
] {
    let runtime_script_requests = (
        $runtime_scripts
        | each {|script|
            {
                id: $script.id
                label: $script.label
                owner_surface: ($script.owner_surface? | default "doctor")
                path: ($script.path | path expand)
            }
        }
    )

    evaluate_runtime_contract_checks {
        runtime_scripts: $runtime_script_requests
        generated_layout: {
            owner_surface: "doctor"
            path: ($layout_path | path expand)
        }
        terminal_support: (build_terminal_support_request "launch" "" $terminals)
        linux_ghostty_desktop_graphics_support: (build_linux_ghostty_graphics_request "doctor" $terminals)
    }
}

export def check_startup_working_dir [working_dir: string] {
    first_runtime_check (evaluate_runtime_contract_checks {
        working_dir: {
            kind: "startup"
            path: ($working_dir | path expand)
        }
    }) "startup_working_dir"
}

export def check_launch_working_dir [working_dir: string] {
    first_runtime_check (evaluate_runtime_contract_checks {
        working_dir: {
            kind: "launch"
            path: ($working_dir | path expand)
        }
    }) "launch_working_dir"
}

export def check_runtime_script [script_path: string, id: string, label: string, owner_surface: string] {
    first_runtime_check (evaluate_runtime_contract_checks {
        runtime_scripts: [
            {
                id: $id
                label: $label
                owner_surface: $owner_surface
                path: ($script_path | path expand)
            }
        ]
    }) $id
}

export def check_generated_layout [layout_path: string, owner_surface: string] {
    first_runtime_check (evaluate_runtime_contract_checks {
        generated_layout: {
            owner_surface: $owner_surface
            path: ($layout_path | path expand)
        }
    }) "generated_layout"
}

export def check_launch_terminal_support [requested_terminal: string, terminals: list<string>] {
    first_runtime_check (evaluate_runtime_contract_checks {
        terminal_support: (build_terminal_support_request "launch" $requested_terminal $terminals)
    }) "launch_terminal_support"
}

export def check_linux_ghostty_desktop_graphics_support [terminals: list<string>] {
    evaluate_runtime_contract_checks {
        linux_ghostty_desktop_graphics_support: (build_linux_ghostty_graphics_request "doctor" $terminals)
    } | where {|check| ($check.id? | default "") == "linux_ghostty_desktop_graphics_support" } | get -o 0
}
