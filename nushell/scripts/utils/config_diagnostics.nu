#!/usr/bin/env nu
# Shared config diagnostics for startup, refresh, and doctor.

use config_schema.nu [apply_main_contract_to_reference_config compare_configs validate_enum_values]
use config_surfaces.nu [load_config_surface_from_main]
use failure_classes.nu [format_failure_classification]

def get_schema_findings [default_config: record, user_config: record, include_missing: bool] {
    let schema_reference = (apply_main_contract_to_reference_config $default_config)
    let schema_findings = (compare_configs $schema_reference $user_config)
    let filtered_schema = if $include_missing {
        $schema_findings
    } else {
        $schema_findings | where kind != "missing_field"
    }
    let enum_findings = (validate_enum_values $user_config)
    [$filtered_schema $enum_findings] | flatten
}

def make_schema_diagnostic [finding: record] {
    let base = {
        category: "schema"
        path: $finding.path
        status: $finding.kind
        blocking: ($finding.kind != "missing_field")
        fix_available: false
    }

    match $finding.kind {
        "unknown_field" => (
            $base | merge {
                headline: $"Unknown config field at ($finding.path)"
                detail_lines: [
                    $finding.message
                    "Next: Remove or rename this field manually."
                    "Next: Run `yzx doctor --verbose` to review the full config report."
                    "Next: Use `yzx config reset` only as a blunt fallback."
                ]
            }
        )
        "type_mismatch" => (
            $base | merge {
                headline: $"Wrong config type at ($finding.path)"
                detail_lines: [
                    $finding.message
                    "Next: Update the value to the expected type manually."
                    "Next: Run `yzx doctor --verbose` to review the full config report."
                ]
            }
        )
        "invalid_enum" => (
            $base | merge {
                headline: $"Unsupported config value at ($finding.path)"
                detail_lines: [
                    $finding.message
                    "Next: Replace this value with one of the supported options."
                    "Next: Run `yzx doctor --verbose` to review the full config report."
                    "Next: Use `yzx config reset` only as a blunt fallback."
                ]
            }
        )
        "missing_field" => (
            $base | merge {
                headline: $"Missing config field at ($finding.path)"
                detail_lines: [
                    $finding.message
                    "Next: Add the field from the current template if you want your config to stay fully in sync."
                ]
            }
        )
        _ => (
            $base | merge {
                headline: $"Config issue at ($finding.path)"
                detail_lines: [$finding.message]
            }
        )
    }
}

def format_diagnostic_lines [diagnostics: list<record>] {
    mut lines = []

    for diagnostic in $diagnostics {
        $lines = ($lines | append ["", $diagnostic.headline])
        for detail in $diagnostic.detail_lines {
            $lines = ($lines | append [$"  ($detail)"])
        }
    }

    $lines
}

export def build_config_diagnostic_report_from_records [
    user_config: record
    default_config: record
    config_path: string
    --include-missing
] {
    let schema_findings = (get_schema_findings $default_config $user_config $include_missing)
    let schema_diagnostics = ($schema_findings | each {|finding| make_schema_diagnostic $finding })
    let doctor_diagnostics = $schema_diagnostics
    let blocking_diagnostics = ($doctor_diagnostics | where {|diagnostic| $diagnostic.blocking })

    {
        config_path: $config_path
        schema_diagnostics: $schema_diagnostics
        doctor_diagnostics: $doctor_diagnostics
        blocking_diagnostics: $blocking_diagnostics
        issue_count: ($doctor_diagnostics | length)
        blocking_count: ($blocking_diagnostics | length)
        fixable_count: 0
        has_blocking: (not ($blocking_diagnostics | is-empty))
        has_fixable_config_issues: false
    }
}

export def build_config_diagnostic_report [
    config_path: string
    default_path: string
    --include-missing
] {
    let config_surface = (load_config_surface_from_main $config_path)
    let user_config = $config_surface.merged_config
    let default_config = ((load_config_surface_from_main $default_path).merged_config)
    (
        build_config_diagnostic_report_from_records
            $user_config
            $default_config
            $config_path
            --include-missing=$include_missing
        | upsert config_path $config_surface.display_config_path
    )
}

export def render_startup_config_error [report: record] {
    let detail_lines = (format_diagnostic_lines $report.blocking_diagnostics)
    let recovery_hint = "Update the reported config fields manually, then retry. Use `yzx config reset` only as a blunt fallback."

    (
        [
            $"Yazelix found stale or unsupported config entries in ($report.config_path)."
            $"Blocking issues: ($report.blocking_count)"
            ...$detail_lines
            ""
            (format_failure_classification "config" $recovery_hint)
        ] | str join "\n"
    )
}

export def render_doctor_config_details [report: record] {
    if ($report.issue_count == 0) {
        return "No stale or unsupported config issues detected."
    }

    let guidance = [
        ""
        "Review the listed fields manually."
        "Blunt fallback: `yzx config reset`"
    ]

    (
        [
            $"Config report for: ($report.config_path)"
            $"Issues: ($report.issue_count)"
            ...(
                format_diagnostic_lines $report.doctor_diagnostics
            )
            ...$guidance
        ] | str join "\n"
    )
}
