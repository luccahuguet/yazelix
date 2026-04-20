#!/usr/bin/env nu

use constants.nu *
use version_info.nu [collect_version_info render_version_info]
use config_parser.nu run_yzx_core_json_command
use config_surfaces.nu load_active_config_surface
use generated_runtime_state.nu build_runtime_materialization_plan_helper_argv

def build_status_rows [summary: record] {
    let terminal_label = if (($summary.terminals? | default []) | is-empty) {
        "none"
    } else {
        $summary.terminals | str join ", "
    }
    let helix_runtime_label = ($summary.helix_runtime? | default "runtime default")
    let session_name = if ($summary.persistent_sessions? | default false) {
        $summary.session_name? | default "unknown"
    } else {
        "disabled"
    }

    [
        {field: "version", value: ($summary.version? | default "")}
        {field: "description", value: ($summary.description? | default "")}
        {field: "config_file", value: ($summary.config_file? | default "")}
        {field: "runtime_dir", value: ($summary.runtime_dir? | default "")}
        {field: "logs_dir", value: ($summary.logs_dir? | default "")}
        {field: "generated_state_repair_needed", value: (($summary.generated_state_repair_needed? | default false) | into string)}
        {field: "generated_state_materialization_status", value: ($summary.generated_state_materialization_status? | default "")}
        {field: "generated_state_materialization_reason", value: ($summary.generated_state_materialization_reason? | default "")}
        {field: "default_shell", value: ($summary.default_shell? | default "")}
        {field: "terminals", value: $terminal_label}
        {field: "helix_runtime", value: $helix_runtime_label}
        {field: "persistent_sessions", value: (($summary.persistent_sessions? | default false) | into string)}
        {field: "session_name", value: $session_name}
    ]
}

export def collect_status_report [
    runtime_dir: string
    --include-versions
] {
    let config_surface = (load_active_config_surface)
    let plan_tail = (build_runtime_materialization_plan_helper_argv $runtime_dir | skip 1)
    let helper_args = [
        "status.compute"
        ...$plan_tail
        "--yazelix-version"
        $YAZELIX_VERSION
        "--yazelix-description"
        $YAZELIX_DESCRIPTION
    ]

    mut report = (
        run_yzx_core_json_command $runtime_dir $config_surface $helper_args "Yazelix Rust status helper returned invalid JSON."
    )

    if $include_versions {
        $report = ($report | upsert versions (collect_version_info))
    }

    $report
}

export def render_status_report [report: record] {
    print ($report.title? | default "Yazelix status")
    print ((build_status_rows $report.summary) | table)

    let versions = ($report.versions? | default null)
    if $versions != null {
        print ""
        render_version_info $versions
    }
}
