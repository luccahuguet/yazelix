#!/usr/bin/env nu

use config_migrations.nu [build_config_migration_plan_from_record]

def format_release_context [rule: record] {
    if ($rule.introduced_in | is-not-empty) {
        $"($rule.introduced_in) on ($rule.introduced_on)"
    } else if ($rule.introduced_after_version | is-not-empty) {
        $"after ($rule.introduced_after_version) on ($rule.introduced_on)"
    } else {
        $rule.introduced_on
    }
}

export def get_config_migration_plan [config_path: string] {
    let config = open $config_path
    build_config_migration_plan_from_record $config $config_path
}

export def render_config_migration_plan [plan: record] {
    let lines = [
        "Yazelix config migration preview"
        $"Config: ($plan.config_path)"
        $"Known rule matches: ($plan.results | length)"
        $"Safe rewrites: ($plan.auto_count)"
        $"Manual follow-up items: ($plan.manual_count)"
    ]
    mut rendered = $lines

    if ($plan.results | is-empty) {
        $rendered = ($rendered | append [
            ""
            "No known config migrations detected."
        ])
        return ($rendered | str join "\n")
    }

    for result in $plan.results {
        let prefix = if $result.status == "auto" { "AUTO" } else { "MANUAL" }
        $rendered = ($rendered | append [
            ""
            $"[($prefix)] ($result.id)"
            $"  Title: ($result.title)"
            $"  Introduced: (format_release_context $result)"
            $"  Rationale: ($result.rationale)"
        ])

        if not ($result.matched_paths | is-empty) {
            let joined_paths = ($result.matched_paths | str join ", ")
            $rendered = ($rendered | append [$"  Matched paths: ($joined_paths)"])
        }

        if $result.status == "auto" {
            for change in $result.changes {
                $rendered = ($rendered | append [$"  Change: ($change)"])
            }
        } else {
            $rendered = ($rendered | append [$"  Manual fix: ($result.manual_fix)"])
        }
    }

    $rendered = ($rendered | append [
        ""
        "Preview only. Re-run with `yzx config migrate --apply` to write the safe rewrites."
    ])

    if $plan.has_manual_items {
        $rendered = ($rendered | append [
            "Manual-only items will stay untouched on apply."
        ])
    }

    $rendered | str join "\n"
}
