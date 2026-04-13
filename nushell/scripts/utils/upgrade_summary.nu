#!/usr/bin/env nu
# Shared upgrade-note loading, rendering, and first-run suppression state.

use atomic_writes.nu write_text_atomic
use common.nu [get_yazelix_runtime_dir get_yazelix_state_dir]
use constants.nu [YAZELIX_VERSION]

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

export def get_upgrade_note_entry [version: string = $YAZELIX_VERSION] {
    let registry = (load_upgrade_notes_registry)
    if $registry == null {
        return null
    }

    let release_keys = ($registry.releases | columns)
    if not ($version in $release_keys) {
        return null
    }

    let entry = ($registry.releases | get -o $version)
    if $entry == null {
        return null
    }
    if not (($entry | describe) | str contains "record") {
        return null
    }

    ($entry | merge {
        key: $version
        notes_path: $registry.notes_path
        changelog_path: $registry.changelog_path
    })
}

def get_upgrade_summary_state_path [] {
    let summary_dir = (get_yazelix_state_dir | path join "state" "upgrade_summary")
    mkdir $summary_dir
    ($summary_dir | path join "last_seen_version.txt")
}

def read_last_seen_upgrade_version [] {
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

def write_last_seen_upgrade_version [version: string] {
    let state_path = (get_upgrade_summary_state_path)
    write_text_atomic $state_path $version --raw | ignore
    $state_path
}

def render_upgrade_summary [entry: record] {
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
                "Upgrade impact: this historical release included config-shape changes."
                "Yazelix v15 no longer ships an automatic config migration engine."
                "If you are jumping from this release era, compare your config manually with the current template or run `yzx config reset` to start fresh."
            ])
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

export def build_upgrade_summary_report [version: string = $YAZELIX_VERSION] {
    let entry = (get_upgrade_note_entry $version)
    let state_path = (get_upgrade_summary_state_path)
    let last_seen_version = (read_last_seen_upgrade_version)

    if $entry == null {
        return {
            found: false
            version: $version
            notes_path: (get_upgrade_notes_path)
            changelog_path: (get_changelog_path)
            state_path: $state_path
            last_seen_version: $last_seen_version
            matching_migrations: []
            matching_migration_ids: []
            output: ""
        }
    }

    let rendered = (render_upgrade_summary $entry)

    {
        found: true
        version: $version
        entry: $entry
        notes_path: $entry.notes_path
        changelog_path: $entry.changelog_path
        state_path: $state_path
        last_seen_version: $last_seen_version
        matching_migrations: []
        matching_migration_ids: []
        output: $rendered
    }
}

export def maybe_show_first_run_upgrade_summary [] {
    let report = (build_upgrade_summary_report $YAZELIX_VERSION)

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
    let report = (build_upgrade_summary_report $YAZELIX_VERSION)

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
