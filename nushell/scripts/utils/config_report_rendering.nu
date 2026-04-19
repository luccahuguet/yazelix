#!/usr/bin/env nu

use failure_classes.nu [format_failure_classification]

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

export def render_startup_config_error [report: record] {
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

export def render_doctor_config_details [report: record] {
    if (($report.issue_count? | default 0) == 0) {
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
            $"Issues: ($report.issue_count? | default 0)"
            ...(format_diagnostic_lines ($report.doctor_diagnostics? | default []))
            ...$guidance
        ] | str join "\n"
    )
}
