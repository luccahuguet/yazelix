#!/usr/bin/env nu

use config_migrations.nu [
    apply_config_migration_plan
    build_config_migration_plan_from_record
]
use config_migration_preview.nu [get_config_migration_plan render_config_migration_plan]
use config_migration_rules.nu validate_config_migration_rules
use config_migration_transactions.nu [
    apply_managed_config_relocation_transaction
    recover_stale_managed_config_transactions
]
use config_surfaces.nu [ensure_no_duplicate_primary_config_surfaces get_primary_config_paths]

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
    } else if $report.had_relocation and (($report.pack_config_path | default "" | is-not-empty)) {
        $lines = ($lines | append $"ℹ️  Yazelix relocated the managed pack config into user_configs before ($report.entrypoint_label).")
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
        }
    }

    let initial_paths = (get_primary_config_paths)
    let had_legacy = (
        ($initial_paths.legacy_user_config | path exists)
        or ($initial_paths.legacy_pack_config | path exists)
    )

    {
        status: "ready"
        paths: $initial_paths
        had_relocation: $had_legacy
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

    let recovery = (recover_stale_managed_config_transactions $context.paths.user_config)
    if $recovery.recovered_count > 0 {
        print ""
        print $"ℹ️  Recovered ($recovery.recovered_count) interrupted managed-config transaction\(s\) before ($entrypoint_label)."
    }

    let refreshed_paths = (get_primary_config_paths)
    ensure_no_duplicate_primary_config_surfaces $refreshed_paths
    let refreshed_had_legacy = (
        ($refreshed_paths.legacy_user_config | path exists)
        or ($refreshed_paths.legacy_pack_config | path exists)
    )
    let context = (
        $context
        | upsert paths $refreshed_paths
        | upsert had_relocation $refreshed_had_legacy
    )

    let relocation_result = if $context.had_relocation {
        let has_legacy_main = ($context.paths.legacy_user_config | path exists)
        let has_legacy_pack = ($context.paths.legacy_pack_config | path exists)

        if $has_legacy_main {
            let legacy_main_config = (open $context.paths.legacy_user_config)
            let legacy_pack_config = if $has_legacy_pack {
                open $context.paths.legacy_pack_config
            } else {
                null
            }
            let relocation_plan = (
                build_config_migration_plan_from_record
                    $legacy_main_config
                    $context.paths.user_config
                    $legacy_pack_config
                    $context.paths.user_pack_config
            )
            let rewritten_main_toml = ($relocation_plan.migrated_config | to toml)
            let rewritten_pack_toml = if $has_legacy_pack or $relocation_plan.has_pack_config_change {
                $relocation_plan.migrated_pack_config | to toml
            } else {
                null
            }

            {
                apply_result: (
                    apply_managed_config_relocation_transaction
                        "entrypoint_preflight"
                        ($context.paths | merge {
                            rewritten_main_toml: $rewritten_main_toml
                            rewritten_pack_toml: $rewritten_pack_toml
                        })
                )
                plan: $relocation_plan
            }
        } else {
            {
                apply_result: (apply_managed_config_relocation_transaction "entrypoint_preflight" $context.paths)
                plan: null
            }
        }
    } else {
        {
            apply_result: null
            plan: null
        }
    }

    let effective_config_path = if ($context.paths.user_config | path exists) { $context.paths.user_config } else { null }

    if $effective_config_path == null {
        let report = {
            entrypoint_label: $entrypoint_label
            status: (if $context.had_relocation { "relocated_only" } else { "noop" })
            had_relocation: $context.had_relocation
            applied_count: ($relocation_result.plan.auto_count? | default 0)
            manual_count: ($relocation_result.plan.manual_count? | default 0)
            config_path: null
            pack_config_path: ($relocation_result.apply_result.pack_config_path? | default null)
            backup_path: ($relocation_result.apply_result.backup_path? | default null)
            pack_backup_path: ($relocation_result.apply_result.pack_backup_path? | default null)
            remaining_plan: null
        }

        let success_lines = (render_preflight_success_summary $report)
        if not ($success_lines | is-empty) {
            print ""
            for line in $success_lines {
                print $line
            }
        }

        return $report
    }

    let initial_plan = if $relocation_result.plan != null {
        $relocation_result.plan
    } else {
        get_config_migration_plan $effective_config_path
    }
    if (not $initial_plan.has_auto_changes) and (not $initial_plan.has_manual_items) and (not $context.had_relocation) {
        return ({
            entrypoint_label: $entrypoint_label
            status: "noop"
            had_relocation: false
            applied_count: 0
            manual_count: 0
            config_path: $effective_config_path
            pack_config_path: $initial_plan.pack_config_path
            backup_path: null
            pack_backup_path: null
            remaining_plan: $initial_plan
        })
    }

    let apply_result = if $relocation_result.apply_result != null {
        $relocation_result.apply_result | merge {
            applied_count: ($initial_plan.auto_count? | default 0)
            manual_count: ($initial_plan.manual_count? | default 0)
        }
    } else if $initial_plan.has_auto_changes {
        apply_config_migration_plan $initial_plan "entrypoint_preflight"
    } else {
        {
            status: "relocated"
            config_path: $effective_config_path
            backup_path: null
            pack_config_path: $initial_plan.pack_config_path
            pack_backup_path: null
            applied_count: 0
            manual_count: $initial_plan.manual_count
        }
    }

    let remaining_plan = (get_config_migration_plan $effective_config_path)
    let report = {
        entrypoint_label: $entrypoint_label
        status: (if $remaining_plan.has_manual_items { "manual_required" } else if ($apply_result.applied_count > 0) { "applied" } else if $context.had_relocation { "relocated_only" } else { "noop" })
        had_relocation: $context.had_relocation
        applied_count: ($apply_result.applied_count? | default 0)
        manual_count: $remaining_plan.manual_count
        config_path: $effective_config_path
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
