#!/usr/bin/env nu
# Shared yzx_core helper transport and error-surface glue.

use runtime_paths.nu [get_yazelix_runtime_dir]

def format_failure_classification [failure_class: string, recovery_hint: string] {
    let label = if ($failure_class | str downcase | str trim) == "config" {
        "config problem"
    } else if ($failure_class | str downcase | str trim) == "generated-state" {
        "generated-state problem"
    } else if ($failure_class | str downcase | str trim) == "host-dependency" {
        "host-dependency problem"
    } else {
        error make {msg: $"Unsupported failure class: ($failure_class)"}
    }
    [
        $"Failure class: ($label)."
        $"Recovery: ($recovery_hint)"
    ] | str join "\n"
}

def format_diagnostic_lines [diagnostics: list<record>] {
    mut lines = []

    for diagnostic in $diagnostics {
        $lines = ($lines | append ["", $diagnostic.headline])
        for detail in ($diagnostic.detail_lines? | default []) {
            $lines = ($lines | append [$"  ($detail)"])
        }
    }

    $lines
}

def render_startup_config_error [report: record] {
    let detail_lines = (format_diagnostic_lines ($report.blocking_diagnostics? | default []))
    let recovery_hint = "Update the reported config fields manually, then retry. Use `yzx config reset` only as a blunt fallback."

    (
        [
            $"Yazelix found stale or unsupported config entries in ($report.config_path)."
            $"Blocking issues: ($report.blocking_count? | default 0)"
            ...$detail_lines
            ""
            (format_failure_classification "config" $recovery_hint)
        ] | str join "\n"
    )
}

const YZX_CORE_HELPER_RELATIVE_PATH = ["libexec" "yzx_core"]

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
        error make {msg: $"YAZELIX_YZX_CORE_BIN points to a missing helper: ($expanded)"}
    }

    $expanded
}

export def resolve_yzx_core_helper_path [runtime_dir: string] {
    let explicit_helper_path = (get_explicit_yzx_core_helper_path)
    if $explicit_helper_path != null {
        return $explicit_helper_path
    }

    let runtime_helper_path = (get_runtime_yzx_core_helper_path $runtime_dir)
    if ($runtime_helper_path | path exists) {
        return $runtime_helper_path
    }

    error make {
        msg: (
            [
                $"Yazelix runtime is missing the Rust config helper at ($runtime_helper_path)."
                ""
                (format_failure_classification "host-dependency" "Reinstall Yazelix so packaged runtimes include libexec/yzx_core, or export YAZELIX_YZX_CORE_BIN before running the helper from a source checkout.")
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

def execute_yzx_core_command [runtime_dir: string, helper_args] {
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

def parse_yzx_core_envelope [raw: string, invalid_json_message: string] {
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

export def resolve_yzx_control_path [runtime_dir?: string] {
    let resolved = if $runtime_dir != null {
        $runtime_dir | path expand
    } else {
        get_yazelix_runtime_dir
    }

    let explicit = ($env.YAZELIX_YZX_CONTROL_BIN? | default "" | into string | str trim)
    if ($explicit | is-not-empty) {
        let expanded = ($explicit | path expand)
        if ($expanded | path exists) {
            return $expanded
        }
    }

    let libexec = ($resolved | path join "libexec" "yzx_control")
    if ($libexec | path exists) {
        return $libexec
    }

    error make {
        msg: (
            [
                "Yazelix runtime is missing the Rust control helper."
                ""
                "Tried:"
                $"  - explicit: ($explicit)"
                $"  - libexec:  ($libexec)"
                ""
                (format_failure_classification "host-dependency" "Reinstall Yazelix so the runtime includes a compatible yzx_control helper, or export YAZELIX_YZX_CONTROL_BIN before running from a source checkout.")
            ] | str join "\n"
        )
    }
}

export def profile_startup_step [component: string, step: string, code: closure, metadata?: record] {
    if not ($env.YAZELIX_STARTUP_PROFILE? == "true") {
        return (do $code)
    }

    let started_ns = (date now | into int)
    let result = (do $code)
    let ended_ns = (date now | into int)

    let report_path = ($env.YAZELIX_STARTUP_PROFILE_REPORT? | default "" | into string | str trim)
    if ($report_path | is-empty) {
        return $result
    }

    let meta = ($metadata | default {} | to json -r)
    let yzx_control_bin = (resolve_yzx_control_path)
    try {
        ^$yzx_control_bin profile record-step $component $step $started_ns $ended_ns --metadata $meta | ignore
    } catch {
        # Silently ignore recording failures so profiling never breaks startup
    }

    $result
}

export def run_zellij_pipe [command: string, payload: string = ""] {
    let yzx_control = (resolve_yzx_control_path)
    mut args = [zellij pipe $command]
    if ($payload | is-not-empty) {
        $args = ($args | append [--payload $payload])
    }
    let result = (^$yzx_control ...$args | complete)
    if $result.exit_code != 0 {
        error make {msg: ($result.stderr | default "" | str trim)}
    }
    $result.stdout | default "" | str trim
}

export def get_current_tab_workspace_root [--include-bootstrap] {
    let yzx_control = (resolve_yzx_control_path)
    mut args = [zellij get-workspace-root]
    if $include_bootstrap {
        $args = ($args | append "--include-bootstrap")
    }
    let result = (^$yzx_control ...$args | complete)
    if $result.exit_code != 0 {
        return null
    }
    $result.stdout | default "" | str trim
}
