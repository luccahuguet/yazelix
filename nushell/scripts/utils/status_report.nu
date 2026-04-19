#!/usr/bin/env nu

use constants.nu *
use version_info.nu [collect_version_info render_version_info]

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
        {field: "default_shell", value: ($summary.default_shell? | default "")}
        {field: "terminals", value: $terminal_label}
        {field: "helix_runtime", value: $helix_runtime_label}
        {field: "persistent_sessions", value: (($summary.persistent_sessions? | default false) | into string)}
        {field: "session_name", value: $session_name}
    ]
}

export def collect_status_report [
    config: record
    config_state: record
    yazelix_dir: string
    --include-versions
] {
    let persistent_sessions = (($config.persistent_sessions? | default "false") == "true")
    let summary = {
        version: $YAZELIX_VERSION
        description: $YAZELIX_DESCRIPTION
        config_file: $config_state.config_file
        runtime_dir: $yazelix_dir
        logs_dir: ($yazelix_dir | path join "logs")
        generated_state_repair_needed: ($config_state.needs_refresh? | default false)
        default_shell: ($config.default_shell? | default "")
        terminals: ($config.terminals? | default [$DEFAULT_TERMINAL])
        helix_runtime: ($config.helix_runtime_path? | default null)
        persistent_sessions: $persistent_sessions
        session_name: (if $persistent_sessions { $config.session_name? | default null } else { null })
    }

    mut report = {
        title: "Yazelix status"
        summary: $summary
    }

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
