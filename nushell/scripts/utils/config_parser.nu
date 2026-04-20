#!/usr/bin/env nu
# Configuration parser for yazelix TOML files

use config_report_rendering.nu render_startup_config_error
use failure_classes.nu [format_failure_classification]
use config_surfaces.nu load_active_config_surface
use common.nu [require_yazelix_runtime_dir]

const YZX_CORE_HELPER_RELATIVE_PATH = ["libexec" "yzx_core"]

def get_runtime_yzx_core_helper_path [runtime_dir: string] {
    $YZX_CORE_HELPER_RELATIVE_PATH | prepend $runtime_dir | path join
}

def get_yzx_core_contract_path [runtime_dir: string] {
    $runtime_dir | path join "config_metadata" "main_config_contract.toml"
}

def build_config_normalize_helper_args [
    runtime_dir: string
    config_surface: record
    --include-missing
] {
    mut helper_args = [
        "config.normalize"
        "--config"
        $config_surface.config_file
        "--default-config"
        $config_surface.default_config_path
        "--contract"
        (get_yzx_core_contract_path $runtime_dir)
    ]

    if $include_missing {
        $helper_args = ($helper_args | append "--include-missing")
    }

    $helper_args
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
    for candidate in [
        ($runtime_dir | path join "rust_core" "target" "release" "yzx_core")
        ($runtime_dir | path join "rust_core" "target" "debug" "yzx_core")
    ] {
        if ($candidate | path exists) {
            return $candidate
        }
    }

    null
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

export def render_yzx_core_error [config_surface: record, stderr: string] {
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
                "Yazelix Rust config normalization failed before it could report a structured error."
                $"Raw helper stderr: ($trimmed_stderr)"
                ""
                (format_failure_classification "host-dependency" "Reinstall Yazelix so the runtime includes a working yzx_core helper, then retry.")
            ] | str join "\n"
        )
    }

    let error = ($envelope.error? | default {})
    let error_class = ($error.class? | default "runtime")
    let code = ($error.code? | default "unknown")
    let message = ($error.message? | default "Yazelix Rust config normalization failed.")
    let remediation = ($error.remediation? | default "Fix the reported Yazelix config issue and retry.")
    let details = ($error.details? | default {})

    if ($error_class == "config") and ($code == "unsupported_config") and (($details | describe) | str contains "record") {
        render_startup_config_error ($details | upsert config_path $config_surface.display_config_path)
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

def build_single_error_config_diagnostic_report [config_surface: record, envelope: record] {
    let error = ($envelope.error? | default {})
    let details = ($error.details? | default {})
    let path = ($details.field? | default "<root>")
    let message = ($error.message? | default "Yazelix config validation failed.")
    let remediation = ($error.remediation? | default "Fix the reported config issue and retry.")
    let status = ($error.code? | default "config_error")

    let diagnostic = {
        category: "config"
        path: $path
        status: $status
        blocking: true
        fix_available: false
        headline: $"Invalid config value at ($path)"
        detail_lines: [
            $message
            $"Next: ($remediation)"
            "Next: Run `yzx doctor --verbose` to review the full config report."
        ]
    }

    {
        config_path: $config_surface.display_config_path
        schema_diagnostics: [$diagnostic]
        doctor_diagnostics: [$diagnostic]
        blocking_diagnostics: [$diagnostic]
        issue_count: 1
        blocking_count: 1
        fixable_count: 0
        has_blocking: true
        has_fixable_config_issues: false
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

export def run_yzx_core_json_command_with_error_surface [
    runtime_dir: string
    error_surface: record
    helper_args
    invalid_json_message: string
] {
    let result = (execute_yzx_core_command $runtime_dir $helper_args)
    if $result.exit_code != 0 {
        error make {msg: (render_yzx_core_error $error_surface $result.stderr)}
    }

    let envelope = (
        try {
            $result.stdout | from json
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
    )

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
    run_yzx_core_json_command_with_error_surface $runtime_dir $error_surface [
        $command
        "--request-json"
        ($request | to json -r)
    ] $invalid_json_message
}

export def run_yzx_core_json_command [
    runtime_dir: string
    config_surface: record
    helper_args
    invalid_json_message: string
] {
    run_yzx_core_json_command_with_error_surface $runtime_dir $config_surface $helper_args $invalid_json_message
}

export def run_yzx_core_command_with_error_surface [
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

export def run_yzx_core_command [
    runtime_dir: string
    config_surface: record
    helper_args
] {
    run_yzx_core_command_with_error_surface $runtime_dir $config_surface $helper_args
}

export def collect_config_diagnostic_report [
    runtime_dir: string
    config_surface: record
    --include-missing
] {
    let helper_args = (build_config_normalize_helper_args $runtime_dir $config_surface --include-missing=$include_missing)
    let result = (execute_yzx_core_command $runtime_dir $helper_args)

    if $result.exit_code == 0 {
        let envelope = (parse_yzx_core_envelope ($result.stdout | default "") "Yazelix Rust config helper returned invalid JSON.")
        let status = ($envelope | get -o status | default "")
        if $status != "ok" {
            error make {msg: (render_yzx_core_error $config_surface ($result.stdout | default ""))}
        }

        return (
            $envelope
            | get data
            | get diagnostic_report
            | upsert config_path $config_surface.display_config_path
        )
    }

    let envelope = (parse_yzx_core_envelope (($result.stderr | default "") | str trim) "Yazelix Rust config helper returned invalid JSON.")
    let error = ($envelope.error? | default {})
    let error_class = ($error.class? | default "")
    let code = ($error.code? | default "")

    if ($error_class == "config") and ($code == "unsupported_config") and ((($error.details? | default {}) | describe) | str contains "record") {
        return (
            ($error.details? | default {})
            | upsert config_path $config_surface.display_config_path
        )
    }

    if $error_class == "config" {
        return (build_single_error_config_diagnostic_report $config_surface $envelope)
    }

    error make {msg: (render_yzx_core_error $config_surface $result.stderr)}
}

def parse_yazelix_config_with_rust [
    runtime_dir: string
    config_surface: record
] {
    run_yzx_core_json_command $runtime_dir $config_surface (build_config_normalize_helper_args $runtime_dir $config_surface) "Yazelix Rust config helper returned invalid JSON."
    | get normalized_config
}

# Parse yazelix configuration file and extract settings
export def parse_yazelix_config [] {
    let config_surface = load_active_config_surface
    let runtime_dir = require_yazelix_runtime_dir
    parse_yazelix_config_with_rust $runtime_dir $config_surface
}
