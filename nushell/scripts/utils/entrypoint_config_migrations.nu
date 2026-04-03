#!/usr/bin/env nu

use config_migrations.nu [
    apply_config_migration_plan
    get_config_migration_plan
    render_config_migration_plan
    validate_config_migration_rules
]
use config_migration_transactions.nu [recover_stale_managed_config_transactions]
use config_surfaces.nu [get_primary_config_paths reconcile_primary_config_surfaces]

def has_interactive_tty [] {
    let result = (^tty | complete)
    $result.exit_code == 0
}

def make_noop_report [entrypoint_label: string, status: string = "noop"] {
    {
        entrypoint_label: $entrypoint_label
        status: $status
        had_relocation: false
        applied_count: 0
        manual_count: 0
        config_path: null
        pack_config_path: null
        backup_path: null
        pack_backup_path: null
        remaining_plan: null
    }
}

def render_preflight_success_summary [report: record] {
    mut lines = []

    if $report.applied_count > 0 {
        $lines = ($lines | append $"ℹ️  Yazelix auto-applied ($report.applied_count) safe config migration\(s\) before ($report.entrypoint_label).")
    }

    if $report.had_relocation and ($report.config_path != null) {
        $lines = ($lines | append $"ℹ️  Yazelix relocated the managed config into user_configs before ($report.entrypoint_label).")
        $lines = ($lines | append $"   Main config: ($report.config_path)")
    }

    if ($report.backup_path | default "" | is-not-empty) {
        $lines = ($lines | append $"   Backup: ($report.backup_path)")
    }

    if ($report.pack_backup_path | default "" | is-not-empty) {
        $lines = ($lines | append $"   Pack backup: ($report.pack_backup_path)")
    }

    if ($report.pack_config_path | default "" | is-not-empty) and (($report.pack_config_path | path exists)) {
        $lines = ($lines | append $"   Pack config: ($report.pack_config_path)")
    }

    $lines
}

def render_preflight_manual_error [report: record] {
    let rendered_plan = (render_config_migration_plan $report.remaining_plan)
    mut lines = []

    if $report.applied_count > 0 {
        $lines = ($lines | append $"Yazelix auto-applied ($report.applied_count) safe config migration\(s\) before ($report.entrypoint_label).")
    } else {
        $lines = ($lines | append $"Yazelix found config migration follow-up that still needs manual review before ($report.entrypoint_label) can continue.")
    }

    if $report.had_relocation and ($report.config_path != null) {
        $lines = ($lines | append $"Managed config path: ($report.config_path)")
    }

    if ($report.backup_path | default "" | is-not-empty) {
        $lines = ($lines | append $"Backup: ($report.backup_path)")
    }

    $lines = ($lines | append ["", $rendered_plan, "", $"Finish the manual-only items above, then rerun ($report.entrypoint_label)."])
    $lines | str join "\n"
}

def build_preflight_context [] {
    if ($env.YAZELIX_CONFIG_OVERRIDE? | default "" | is-not-empty) {
        return {
            status: "skipped_override"
            paths: null
            had_relocation: false
            config_path: null
        }
    }

    let initial_paths = (get_primary_config_paths)
    let had_legacy = (
        ($initial_paths.legacy_user_config | path exists)
        or ($initial_paths.legacy_pack_config | path exists)
    )

    if $had_legacy {
        with-env {YAZELIX_ACCEPT_USER_CONFIG_RELOCATION: "true"} {
            reconcile_primary_config_surfaces | ignore
        }
    }

    let paths = (get_primary_config_paths)
    let config_path = if ($paths.user_config | path exists) { $paths.user_config } else { null }

    {
        status: "ready"
        paths: $paths
        had_relocation: $had_legacy
        config_path: $config_path
    }
}

export def run_entrypoint_config_migration_preflight [
    entrypoint_label: string
    --allow-noninteractive
] {
    if (not $allow_noninteractive) and (not (has_interactive_tty)) {
        return (make_noop_report $entrypoint_label "skipped_noninteractive")
    }

    let metadata_errors = (validate_config_migration_rules)
    if not ($metadata_errors | is-empty) {
        let details = ($metadata_errors | each {|line| $" - ($line)" } | str join "\n")
        error make {msg: $"Config migration rules are invalid:\n($details)"}
    }

    let context = (build_preflight_context)
    if $context.status == "skipped_override" {
        return (make_noop_report $entrypoint_label "skipped_override")
    }

    if $context.config_path == null {
        return ({
            entrypoint_label: $entrypoint_label
            status: (if $context.had_relocation { "relocated_only" } else { "noop" })
            had_relocation: $context.had_relocation
            applied_count: 0
            manual_count: 0
            config_path: null
            pack_config_path: null
            backup_path: null
            pack_backup_path: null
            remaining_plan: null
        })
    }

    let recovery = (recover_stale_managed_config_transactions $context.config_path)
    if $recovery.recovered_count > 0 {
        print ""
        print $"ℹ️  Recovered ($recovery.recovered_count) interrupted managed-config transaction\(s\) before ($entrypoint_label)."
    }

    let initial_plan = (get_config_migration_plan $context.config_path)
    if (not $initial_plan.has_auto_changes) and (not $initial_plan.has_manual_items) and (not $context.had_relocation) {
        return ({
            entrypoint_label: $entrypoint_label
            status: "noop"
            had_relocation: false
            applied_count: 0
            manual_count: 0
            config_path: $context.config_path
            pack_config_path: $initial_plan.pack_config_path
            backup_path: null
            pack_backup_path: null
            remaining_plan: $initial_plan
        })
    }

    let apply_result = if $initial_plan.has_auto_changes {
        apply_config_migration_plan $initial_plan
    } else {
        {
            status: "relocated"
            config_path: $context.config_path
            backup_path: null
            pack_config_path: $initial_plan.pack_config_path
            pack_backup_path: null
            applied_count: 0
            manual_count: $initial_plan.manual_count
        }
    }

    let remaining_plan = (get_config_migration_plan $context.config_path)
    let report = {
        entrypoint_label: $entrypoint_label
        status: (if $remaining_plan.has_manual_items { "manual_required" } else if ($apply_result.applied_count > 0) { "applied" } else if $context.had_relocation { "relocated_only" } else { "noop" })
        had_relocation: $context.had_relocation
        applied_count: ($apply_result.applied_count? | default 0)
        manual_count: $remaining_plan.manual_count
        config_path: $context.config_path
        pack_config_path: ($apply_result.pack_config_path? | default $remaining_plan.pack_config_path)
        backup_path: ($apply_result.backup_path? | default null)
        pack_backup_path: ($apply_result.pack_backup_path? | default null)
        remaining_plan: $remaining_plan
    }

    let success_lines = (render_preflight_success_summary $report)
    if not ($success_lines | is-empty) {
        print ""
        for line in $success_lines {
            print $line
        }
    }

    if $remaining_plan.has_manual_items {
        error make {msg: (render_preflight_manual_error $report)}
    }

    $report
}
