#!/usr/bin/env nu
# Configuration parser for yazelix TOML files

use common.nu [require_yazelix_runtime_dir]
use ./yzx_core_bridge.nu [
    execute_yzx_core_command
    parse_yzx_core_envelope
    render_yzx_core_error
    resolve_active_config_surface_via_yzx_core
    run_yzx_core_json_command
]

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
    let runtime_dir = require_yazelix_runtime_dir
    let config_surface = (resolve_active_config_surface_via_yzx_core $runtime_dir)
    parse_yazelix_config_with_rust $runtime_dir $config_surface
}
