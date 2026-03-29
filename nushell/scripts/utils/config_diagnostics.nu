#!/usr/bin/env nu
# Shared config diagnostics for startup, refresh, and doctor.

use common.nu [get_yazelix_config_dir get_yazelix_runtime_dir]
use config_migrations.nu [apply_config_migration_plan build_config_migration_plan_from_record]
use config_schema.nu [compare_configs validate_enum_values]
use config_surfaces.nu [get_pack_sidecar_path load_config_surface_from_main get_main_user_config_path]
use failure_classes.nu [format_failure_classification]

def format_release_context [result: record] {
    if ($result.introduced_in | is-not-empty) {
        $"($result.introduced_in) on ($result.introduced_on)"
    } else if ($result.introduced_after_version | is-not-empty) {
        $"after ($result.introduced_after_version) on ($result.introduced_on)"
    } else {
        $result.introduced_on
    }
}

def get_schema_findings [default_config: record, user_config: record, include_missing: bool] {
    let schema_findings = (compare_configs $default_config $user_config)
    let filtered_schema = if $include_missing {
        $schema_findings
    } else {
        $schema_findings | where kind != "missing_field"
    }
    let enum_findings = (validate_enum_values $user_config)
    [$filtered_schema $enum_findings] | flatten
}

def make_migration_diagnostic [result: record] {
    let path_label = ($result.matched_paths | get -o 0 | default "<config>")
    let next_steps = if $result.status == "auto" {
        [
            "Run `yzx config migrate` to preview the known safe rewrite."
            "Run `yzx config migrate --apply` to apply the safe rewrite with backup."
            "Run `yzx doctor --fix` to apply the same safe rewrite from the doctor flow."
        ]
    } else {
        [$result.manual_fix]
    }

    {
        category: "migration"
        path: $path_label
        status: $result.status
        blocking: true
        fix_available: ($result.status == "auto")
        headline: $"Known migration at ($path_label)"
        detail_lines: (
            [
                $"What changed: ($result.title)"
                $"Introduced: (format_release_context $result)"
                $"Why: ($result.rationale)"
                ...($next_steps | each {|line| $"Next: ($line)" })
            ]
        )
    }
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
                    "Next: Use `yzx config reset --yes` only as a blunt fallback."
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
                    "Next: Use `yzx config reset --yes` only as a blunt fallback."
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
    migration_config?: any
    pack_config?: any
    --include-missing
] {
    let migration_source = ($migration_config | default $user_config)
    let migration_plan = (
        build_config_migration_plan_from_record
            $migration_source
            $config_path
            $pack_config
            (get_pack_sidecar_path $config_path)
    )
    let migration_paths = ($migration_plan.results | get -o matched_paths | default [] | flatten | uniq)
    let schema_findings = (
        get_schema_findings $default_config $user_config $include_missing
        | where {|finding| not ($finding.path in $migration_paths) }
    )
    let migration_diagnostics = ($migration_plan.results | each {|result| make_migration_diagnostic $result })
    let schema_diagnostics = ($schema_findings | each {|finding| make_schema_diagnostic $finding })
    let doctor_diagnostics = [$migration_diagnostics $schema_diagnostics] | flatten
    let blocking_diagnostics = ($doctor_diagnostics | where blocking == true)

    {
        config_path: $config_path
        migration_plan: $migration_plan
        migration_diagnostics: $migration_diagnostics
        schema_diagnostics: $schema_diagnostics
        doctor_diagnostics: $doctor_diagnostics
        blocking_diagnostics: $blocking_diagnostics
        issue_count: ($doctor_diagnostics | length)
        blocking_count: ($blocking_diagnostics | length)
        fixable_count: ($migration_diagnostics | where fix_available == true | length)
        has_blocking: (not ($blocking_diagnostics | is-empty))
        has_fixable_migrations: (($migration_diagnostics | where fix_available == true | is-not-empty))
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
            $config_surface.main_config
            $config_surface.pack_config
            --include-missing=$include_missing
        | upsert config_path $config_surface.display_config_path
    )
}

export def build_active_config_diagnostic_report [--include-missing] {
    let config_dir = (get_yazelix_config_dir)
    let runtime_dir = (get_yazelix_runtime_dir)
    let config_path = (get_main_user_config_path $config_dir)
    let default_path = ($runtime_dir | path join "yazelix_default.toml")

    if not ($default_path | path exists) {
        error make {msg: $"yazelix_default.toml not found at ($default_path)"}
    }

    if not ($config_path | path exists) {
        return null
    }

    build_config_diagnostic_report $config_path $default_path --include-missing=$include_missing
}

export def render_startup_config_error [report: record] {
    let detail_lines = (format_diagnostic_lines $report.blocking_diagnostics)
    let recovery_hint = if $report.has_fixable_migrations {
        "Run `yzx config migrate` to preview known safe rewrites, `yzx config migrate --apply` to apply them with backup, or `yzx doctor --fix` to apply the same safe rewrites from the doctor flow."
    } else {
        "Update the reported config fields manually, then retry. Use `yzx config reset --yes` only as a blunt fallback."
    }

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

    let guidance = if $report.has_fixable_migrations {
        [
            ""
            "Safe preview: `yzx config migrate`"
            "Safe apply: `yzx config migrate --apply` or `yzx doctor --fix`"
            "Blunt fallback: `yzx config reset --yes`"
        ]
    } else {
        [
            ""
            "Review the listed fields manually."
            "Blunt fallback: `yzx config reset --yes`"
        ]
    }

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

export def apply_doctor_config_fixes [report: record] {
    if not $report.has_fixable_migrations {
        return {
            status: "noop"
            backup_path: null
            applied_count: 0
        }
    }

    apply_config_migration_plan $report.migration_plan
}
