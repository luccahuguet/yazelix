#!/usr/bin/env nu
# Shared upgrade-note loading, rendering, and first-run suppression state.

use common.nu [get_yazelix_config_dir get_yazelix_runtime_dir get_yazelix_state_dir]
use constants.nu [YAZELIX_VERSION]
use config_migrations.nu [build_config_migration_plan_from_record]

def normalize_string_list [values: any] {
    if not (($values | describe) | str contains "list") {
        return []
    }

    $values
    | each {|value| $value | into string | str trim }
    | where {|value| $value | is-not-empty }
}

def get_upgrade_notes_path [] {
    (get_yazelix_runtime_dir | path join "docs" "upgrade_notes.toml")
}

def get_changelog_path [] {
    (get_yazelix_runtime_dir | path join "CHANGELOG.md")
}

def load_upgrade_notes_registry [] {
    let notes_path = (get_upgrade_notes_path)

    if not ($notes_path | path exists) {
        return null
    }

    let parsed = (try { open $notes_path } catch { null })
    if $parsed == null {
        return null
    }

    let releases = ($parsed.releases? | default {})
    if not (($releases | describe) | str contains "record") {
        return null
    }

    {
        notes_path: $notes_path
        changelog_path: (get_changelog_path)
        releases: $releases
    }
}

def load_current_release_entry [] {
    let registry = (load_upgrade_notes_registry)
    if $registry == null {
        return null
    }

    let release_keys = ($registry.releases | columns)
    if not ($YAZELIX_VERSION in $release_keys) {
        return null
    }

    let entry = ($registry.releases | get $YAZELIX_VERSION)
    if not (($entry | describe) | str contains "record") {
        return null
    }

    ($entry | merge {
        key: $YAZELIX_VERSION
        notes_path: $registry.notes_path
        changelog_path: $registry.changelog_path
    })
}

def resolve_raw_config_path [] {
    if ($env.YAZELIX_CONFIG_OVERRIDE? | is-not-empty) {
        return $env.YAZELIX_CONFIG_OVERRIDE
    }

    let config_dir = (get_yazelix_config_dir)
    let runtime_dir = (get_yazelix_runtime_dir)
    let user_config = ($config_dir | path join "yazelix.toml")
    let default_config = ($runtime_dir | path join "yazelix_default.toml")

    if ($user_config | path exists) {
        $user_config
    } else if ($default_config | path exists) {
        $default_config
    } else {
        null
    }
}

def load_raw_active_config [] {
    let config_path = (resolve_raw_config_path)
    if $config_path == null {
        return null
    }

    let parsed = (try { open $config_path } catch { null })
    if $parsed == null {
        return null
    }

    {
        config_path: $config_path
        config: $parsed
    }
}

def get_matching_current_migrations [entry: record] {
    let migration_ids = (normalize_string_list ($entry.migration_ids? | default []))
    if ($migration_ids | is-empty) {
        return []
    }

    let raw_config = (load_raw_active_config)
    if $raw_config == null {
        return []
    }

    let plan = (try { build_config_migration_plan_from_record $raw_config.config $raw_config.config_path } catch { null })
    if $plan == null {
        return []
    }

    $plan.results | where {|result| $result.id in $migration_ids }
}

def get_upgrade_summary_state_path [] {
    let summary_dir = (get_yazelix_state_dir | path join "state" "upgrade_summary")
    mkdir $summary_dir
    ($summary_dir | path join "last_seen_version.txt")
}

export def read_last_seen_upgrade_version [] {
    let state_path = (get_upgrade_summary_state_path)
    if not ($state_path | path exists) {
        return null
    }

    let raw_value = (open --raw $state_path | str trim)
    if ($raw_value | is-empty) {
        null
    } else {
        $raw_value
    }
}

export def write_last_seen_upgrade_version [version: string] {
    let state_path = (get_upgrade_summary_state_path)
    $version | save --force --raw $state_path
    $state_path
}

export def render_upgrade_summary [entry: record, matching_migrations: list<record> = []] {
    let release_date = ($entry.date? | default "" | into string | str trim)
    let headline = ($entry.headline? | default "" | into string | str trim)
    let summary_items = (normalize_string_list ($entry.summary? | default []))
    let impact = ($entry.upgrade_impact? | default "no_user_action" | into string)
    let migration_ids = (normalize_string_list ($entry.migration_ids? | default []))
    let manual_actions = (normalize_string_list ($entry.manual_actions? | default []))
    let changelog_path = ($entry.changelog_path? | default (get_changelog_path))

    mut lines = [
        ""
        $"=== What's New In Yazelix ($entry.version) ==="
        $"Released: ($release_date)"
    ]

    if ($headline | is-not-empty) {
        $lines = ($lines | append [$headline])
    }

    if not ($summary_items | is-empty) {
        $lines = ($lines | append ["", "Highlights:"])
        for item in $summary_items {
            $lines = ($lines | append [$"- ($item)"])
        }
    }

    match $impact {
        "migration_available" => {
            $lines = ($lines | append [
                ""
                "Upgrade impact: this release includes known config migrations."
            ])
            if ($matching_migrations | is-empty) {
                $lines = ($lines | append [
                    "Run `yzx config migrate` to preview safe rewrites if your config predates this release."
                    "Run `yzx doctor` for migration-aware diagnostics if startup fails."
                ])
            } else {
                $lines = ($lines | append [
                    "Detected matching migration candidates in your current config:"
                ])
                for result in $matching_migrations {
                    let matched_paths = ($result.matched_paths | default [] | str join ", ")
                    $lines = ($lines | append [$"- ($result.id): ($result.title)"])
                    if ($matched_paths | is-not-empty) {
                        $lines = ($lines | append [$"  Matched paths: ($matched_paths)"])
                    }
                }
                $lines = ($lines | append [
                    "Next: run `yzx config migrate` to preview safe rewrites."
                    "Next: run `yzx config migrate --apply` or `yzx doctor --fix` to apply only deterministic rewrites."
                ])
            }
        }
        "manual_action_required" => {
            $lines = ($lines | append [
                ""
                "Upgrade impact: manual follow-up is required."
            ])
            for action in $manual_actions {
                $lines = ($lines | append [$"- ($action)"])
            }
        }
        _ => {
            $lines = ($lines | append [
                ""
                "Upgrade impact: no user action required."
            ])
            if not ($migration_ids | is-empty) {
                $lines = ($lines | append [
                    $"Recorded migration ids: ($migration_ids | str join ', ')"
                ])
            }
        }
    }

    $lines = ($lines | append [
        ""
        "Reopen later: `yzx whats_new`"
        $"Full notes: ($changelog_path)"
    ])

    $lines | str join "\n"
}

export def build_current_upgrade_summary_report [] {
    let entry = (load_current_release_entry)
    let state_path = (get_upgrade_summary_state_path)
    let last_seen_version = (read_last_seen_upgrade_version)

    if $entry == null {
        return {
            found: false
            version: $YAZELIX_VERSION
            notes_path: (get_upgrade_notes_path)
            changelog_path: (get_changelog_path)
            state_path: $state_path
            last_seen_version: $last_seen_version
            matching_migrations: []
            matching_migration_ids: []
            output: ""
        }
    }

    let matching_migrations = (get_matching_current_migrations $entry)
    let rendered = (render_upgrade_summary $entry $matching_migrations)

    {
        found: true
        version: $YAZELIX_VERSION
        entry: $entry
        notes_path: $entry.notes_path
        changelog_path: $entry.changelog_path
        state_path: $state_path
        last_seen_version: $last_seen_version
        matching_migrations: $matching_migrations
        matching_migration_ids: ($matching_migrations | get -o id | default [])
        output: $rendered
    }
}

export def maybe_show_first_run_upgrade_summary [] {
    let report = (build_current_upgrade_summary_report)

    if not $report.found {
        return ($report | merge { shown: false, reason: "missing_release_entry" })
    }

    if (($report.last_seen_version? | default "") == $report.version) {
        return ($report | merge { shown: false, reason: "already_seen" })
    }

    print $report.output
    let state_path = (write_last_seen_upgrade_version $report.version)

    $report | merge {
        shown: true
        reason: "displayed"
        state_path: $state_path
        last_seen_version: $report.version
    }
}

export def show_current_upgrade_summary [--mark-seen] {
    let report = (build_current_upgrade_summary_report)

    if not $report.found {
        error make {msg: $"No upgrade notes found for ($report.version). Expected an entry in ($report.notes_path)."}
    }

    print $report.output

    if $mark_seen {
        let state_path = (write_last_seen_upgrade_version $report.version)
        return ($report | merge {
            shown: true
            reason: "displayed"
            state_path: $state_path
            last_seen_version: $report.version
        })
    }

    $report | merge {
        shown: true
        reason: "displayed"
    }
}
