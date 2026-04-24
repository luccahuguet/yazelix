#!/usr/bin/env nu
# Shared yzx_core helper transport and error-surface glue.

use runtime_paths.nu [get_yazelix_runtime_dir require_yazelix_runtime_dir]

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
const CONFIG_SURFACE_RESOLVE_COMMAND = "config-surface.resolve"
const RUNTIME_ENV_COMPUTE_COMMAND = "runtime-env.compute"

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

def resolve_bridge_runtime_dir [runtime_dir?: string] {
    if $runtime_dir == null {
        require_yazelix_runtime_dir
    } else {
        $runtime_dir | path expand
    }
}

export def resolve_active_config_surface_via_yzx_core [runtime_dir?: string] {
    let resolved_runtime_dir = (resolve_bridge_runtime_dir $runtime_dir)

    run_yzx_core_json_command $resolved_runtime_dir (build_default_yzx_core_error_surface) [
        $CONFIG_SURFACE_RESOLVE_COMMAND
        "--runtime-dir"
        $resolved_runtime_dir
    ] "Yazelix Rust active-config-surface helper returned invalid JSON."
}

export def compute_runtime_env_via_yzx_core [config?: record, runtime_dir?: string] {
    let helper_args = if $config == null {
        []
    } else {
        ["--config-json", ($config | to json -r)]
    }

    let error_surface = if $config == null { null } else { build_record_yzx_core_error_surface $config }
    let resolved_runtime_dir = (resolve_bridge_runtime_dir $runtime_dir)
    let resolved_helper_args = [$RUNTIME_ENV_COMPUTE_COMMAND, "--from-env"] | append $helper_args

    run_yzx_core_json_command $resolved_runtime_dir (
        $error_surface | default (build_default_yzx_core_error_surface)
    ) $resolved_helper_args "Yazelix Rust runtime-env helper returned invalid JSON."
    | get runtime_env
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

    let release = ($resolved | path join "rust_core" "target" "release" "yzx_control")
    let debug = ($resolved | path join "rust_core" "target" "debug" "yzx_control")
    mut candidates = []
    if ($release | path exists) {
        $candidates = ($candidates | append {
            path: $release
            modified: (ls $release | get 0.modified)
        })
    }
    if ($debug | path exists) {
        $candidates = ($candidates | append {
            path: $debug
            modified: (ls $debug | get 0.modified)
        })
    }

    if ($candidates | is-empty) {
        error make {
            msg: (
                [
                    "Yazelix runtime is missing the Rust control helper."
                    ""
                    "Tried:"
                    $"  - explicit: ($explicit)"
                    $"  - libexec:  ($libexec)"
                    $"  - release:  ($release)"
                    $"  - debug:    ($debug)"
                    ""
                    (format_failure_classification "host-dependency" "Reinstall Yazelix so the runtime includes a compatible yzx_control helper, then retry.")
                ] | str join "\n"
            )
        }
    }

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

export def run_zellij_retarget [target_path: path, editor_kind: string = ""] {
    let yzx_control = (resolve_yzx_control_path)
    mut args = [zellij retarget $target_path]
    if ($editor_kind | is-not-empty) {
        $args = ($args | append [--editor $editor_kind])
    }
    let result = (^$yzx_control ...$args | complete)
    if ($result.stdout | is-not-empty) {
        try {
            $result.stdout | from json
        } catch {
            {status: "error", reason: ($result.stdout | str trim)}
        }
    } else {
        {status: "error", reason: ($result.stderr | default "" | str trim)}
    }
}
