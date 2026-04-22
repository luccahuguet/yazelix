#!/usr/bin/env nu
# Shared yzx_core helper transport and error-surface glue.

use config_report_rendering.nu render_startup_config_error
use failure_classes.nu [format_failure_classification]
use common.nu [require_yazelix_runtime_dir]

const YZX_CORE_HELPER_RELATIVE_PATH = ["libexec" "yzx_core"]
const CONFIG_SURFACE_RESOLVE_COMMAND = "config-surface.resolve"
const CONFIG_STATE_COMPUTE_COMMAND = "config-state.compute"
const CONFIG_STATE_RECORD_COMMAND = "config-state.record"

def get_runtime_yzx_core_helper_path [runtime_dir: string] {
    $YZX_CORE_HELPER_RELATIVE_PATH | prepend $runtime_dir | path join
}

def get_explicit_yzx_core_helper_path [] {
    let explicit = (
        $env.YAZELIX_YZX_CORE_BIN?
        | default ""
        | into string
        | str trim
    )

    if ($explicit | is-empty) {
        return null
    }

    let expanded = ($explicit | path expand)
    if not ($expanded | path exists) {
        error make {
            msg: (
                [
                    $"YAZELIX_YZX_CORE_BIN points to a missing helper: ($expanded)"
                    ""
                    (format_failure_classification "host-dependency" "Enter the Yazelix maintainer shell, rebuild the local yzx_core helper, or unset YAZELIX_YZX_CORE_BIN.")
                ] | str join "\n"
            )
        }
    }

    $expanded
}

def get_source_checkout_yzx_core_helper_path [runtime_dir: string] {
    mut candidates = []

    for candidate in [
        ($runtime_dir | path join "rust_core" "target" "release" "yzx_core")
        ($runtime_dir | path join "rust_core" "target" "debug" "yzx_core")
    ] {
        if ($candidate | path exists) {
            $candidates = ($candidates | append {
                path: $candidate
                modified: (ls $candidate | get 0.modified)
            })
        }
    }

    if ($candidates | is-empty) {
        return null
    }

    # Prefer the freshest local helper build so a newer debug artifact wins over
    # an older stale release binary during source-checkout work.
    (
        $candidates
        | reduce -f null {|candidate, best|
            if ($best == null) or ($candidate.modified > $best.modified) {
                $candidate
            } else {
                $best
            }
        }
        | get path
    )
}

export def resolve_yzx_core_helper_path [runtime_dir: string] {
    let runtime_helper_path = (get_runtime_yzx_core_helper_path $runtime_dir)
    if ($runtime_helper_path | path exists) {
        return $runtime_helper_path
    }

    let explicit_helper_path = (get_explicit_yzx_core_helper_path)
    if $explicit_helper_path != null {
        return $explicit_helper_path
    }

    let source_helper_path = (get_source_checkout_yzx_core_helper_path $runtime_dir)
    if $source_helper_path != null {
        return $source_helper_path
    }

    error make {
        msg: (
            [
                $"Yazelix runtime is missing the Rust config helper at ($runtime_helper_path)."
                ""
                (format_failure_classification "host-dependency" "Reinstall Yazelix so packaged runtimes include libexec/yzx_core. For source checkouts, enter the maintainer shell or set YAZELIX_YZX_CORE_BIN to a built yzx_core helper.")
            ] | str join "\n"
        )
    }
}

export def build_default_yzx_core_error_surface [] {
    {
        display_config_path: ""
        config_file: ""
    }
}

export def build_record_yzx_core_error_surface [config: record] {
    let config_file = ($config.config_file? | default "")
    {
        display_config_path: $config_file
        config_file: $config_file
    }
}

export def render_yzx_core_error [error_surface: record, stderr: string] {
    let trimmed_stderr = ($stderr | default "" | str trim)
    let envelope = (
        try {
            $trimmed_stderr | from json
        } catch {
            null
        }
    )

    if $envelope == null {
        return (
            [
                "Yazelix Rust helper execution failed before it could report a structured error."
                $"Raw helper stderr: ($trimmed_stderr)"
                ""
                (format_failure_classification "host-dependency" "Reinstall Yazelix so the runtime includes a working yzx_core helper, then retry.")
            ] | str join "\n"
        )
    }

    let error = ($envelope.error? | default {})
    let error_class = ($error.class? | default "runtime")
    let code = ($error.code? | default "unknown")
    let message = ($error.message? | default "Yazelix Rust helper execution failed.")
    let remediation = ($error.remediation? | default "Fix the reported Yazelix issue and retry.")
    let details = ($error.details? | default {})

    if ($error_class == "config") and ($code == "unsupported_config") and (($details | describe) | str contains "record") {
        let display_config_path = (
            $error_surface.display_config_path?
            | default ""
            | into string
            | str trim
        )
        let report_details = if ($display_config_path | is-empty) {
            $details
        } else {
            $details | upsert config_path $display_config_path
        }
        render_startup_config_error $report_details
    } else {
        let failure_class = if $error_class == "config" { "config" } else { "host-dependency" }
        [
            $message
            $"Helper code: ($code)"
            ""
            (format_failure_classification $failure_class $remediation)
        ] | str join "\n"
    }
}

export def execute_yzx_core_command [runtime_dir: string, helper_args] {
    let helper_path = resolve_yzx_core_helper_path $runtime_dir
    let command_name = ($helper_args | get -o 0 | default "unknown")

    try {
        do { ^$helper_path ...$helper_args } | complete
    } catch {|err|
        error make {
            msg: (
                [
                    $"Could not execute Yazelix Rust helper command `($command_name)` at ($helper_path)."
                    $err.msg
                    ""
                    (format_failure_classification "host-dependency" "Reinstall Yazelix so the runtime includes an executable yzx_core helper, then retry.")
                ] | str join "\n"
            )
        }
    }
}

export def parse_yzx_core_envelope [raw: string, invalid_json_message: string] {
    try {
        $raw | from json
    } catch {|err|
        error make {
            msg: (
                [
                    $invalid_json_message
                    $err.msg
                    ""
                    (format_failure_classification "host-dependency" "Reinstall Yazelix so the runtime includes a compatible yzx_core helper, then retry.")
                ] | str join "\n"
            )
        }
    }
}

export def run_yzx_core_json_command [
    runtime_dir: string
    error_surface: record
    helper_args
    invalid_json_message: string
] {
    let result = (execute_yzx_core_command $runtime_dir $helper_args)
    if $result.exit_code != 0 {
        error make {msg: (render_yzx_core_error $error_surface $result.stderr)}
    }

    let envelope = (parse_yzx_core_envelope ($result.stdout | default "") $invalid_json_message)
    let status = ($envelope | get -o status | default "")
    if $status != "ok" {
        error make {msg: (render_yzx_core_error $error_surface ($result.stdout | default ""))}
    }

    $envelope | get data
}

export def run_yzx_core_request_json_command [
    runtime_dir: string
    error_surface: record
    command: string
    request: any
    invalid_json_message: string
] {
    run_yzx_core_json_command $runtime_dir $error_surface [
        $command
        "--request-json"
        ($request | to json -r)
    ] $invalid_json_message
}

export def run_yzx_core_runtime_request_json_command [
    command: string
    request: any
    invalid_json_message: string
] {
    let runtime_dir = (require_yazelix_runtime_dir)
    run_yzx_core_request_json_command $runtime_dir (build_default_yzx_core_error_surface) $command $request $invalid_json_message
}

export def resolve_active_config_surface_via_yzx_core [runtime_dir?: string] {
    let resolved_runtime_dir = if $runtime_dir == null {
        require_yazelix_runtime_dir
    } else {
        $runtime_dir | path expand
    }

    run_yzx_core_json_command $resolved_runtime_dir (build_default_yzx_core_error_surface) [
        $CONFIG_SURFACE_RESOLVE_COMMAND
        "--runtime-dir"
        $resolved_runtime_dir
    ] "Yazelix Rust active-config-surface helper returned invalid JSON."
}

export def compute_config_state_via_yzx_core [runtime_dir?: string] {
    let resolved_runtime_dir = if $runtime_dir == null {
        require_yazelix_runtime_dir
    } else {
        $runtime_dir | path expand
    }

    run_yzx_core_json_command $resolved_runtime_dir (build_default_yzx_core_error_surface) [
        $CONFIG_STATE_COMPUTE_COMMAND
        "--from-env"
    ] "Yazelix Rust config-state helper returned invalid JSON."
}

export def record_materialized_state_via_yzx_core [state: record, runtime_dir?: string] {
    let resolved_runtime_dir = if $runtime_dir == null {
        require_yazelix_runtime_dir
    } else {
        $runtime_dir | path expand
    }
    let config_file = ($state.config_file? | default "")

    run_yzx_core_command $resolved_runtime_dir {display_config_path: $config_file} [
        $CONFIG_STATE_RECORD_COMMAND
        "--from-env"
        "--config-file"
        $config_file
        "--config-hash"
        ($state.config_hash? | default "")
        "--runtime-hash"
        ($state.runtime_hash? | default "")
    ] | ignore
}

export def run_yzx_core_command [
    runtime_dir: string
    error_surface: record
    helper_args
] {
    let result = (execute_yzx_core_command $runtime_dir $helper_args)

    if $result.exit_code != 0 {
        error make {msg: (render_yzx_core_error $error_surface $result.stderr)}
    }

    $result
}
