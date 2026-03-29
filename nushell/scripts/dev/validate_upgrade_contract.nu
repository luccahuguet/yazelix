#!/usr/bin/env nu

use ../utils/constants.nu [YAZELIX_VERSION]
use ../utils/config_migrations.nu [get_config_migration_rules]

const REPO_ROOT = (path self | path dirname | path dirname | path dirname | path dirname)
const GUARDED_FILES = [
    "nushell/scripts/utils/constants.nu"
    "yazelix_default.toml"
    "yazelix_packs_default.toml"
    "home_manager/module.nix"
    "nushell/scripts/utils/config_schema.nu"
    "nushell/scripts/utils/config_migrations.nu"
    "docs/upgrade_notes.toml"
    "CHANGELOG.md"
]
const ACK_REQUIRED_FILES = [
    "yazelix_default.toml"
    "yazelix_packs_default.toml"
    "home_manager/module.nix"
    "nushell/scripts/utils/config_schema.nu"
    "nushell/scripts/utils/config_migrations.nu"
]
const IMPACT_VALUES = ["no_user_action", "migration_available", "manual_action_required"]

def load_notes [] {
    open (($REPO_ROOT | path join "docs" "upgrade_notes.toml"))
}

def load_notes_from_ref [ref: string] {
    if not (ref_exists $ref) {
        return null
    }

    let result = (^git show $"($ref):docs/upgrade_notes.toml" | complete)
    if $result.exit_code != 0 {
        return null
    }

    $result.stdout | from toml
}

def load_changelog [] {
    open --raw (($REPO_ROOT | path join "CHANGELOG.md"))
}

def get_release_entries [] {
    (load_notes).releases
}

def get_entry [entries: record, key: string] {
    $entries | get -o $key
}

def drop_acknowledged_guarded_changes [entry: record] {
    if ("acknowledged_guarded_changes" in ($entry | columns)) {
        $entry | reject acknowledged_guarded_changes
    } else {
        $entry
    }
}

def drop_optional_series [notes: record] {
    if ("series" in ($notes | columns)) {
        $notes | reject series
    } else {
        $notes
    }
}

def as_string_list [value: any] {
    if (($value | describe) | str contains "list") {
        $value | each {|item| $item | into string }
    } else {
        []
    }
}

def validate_entry [key: string, entry: record, migration_ids: list<string>] {
    let required_fields = [
        "version"
        "date"
        "headline"
        "summary"
        "upgrade_impact"
        "acknowledged_guarded_changes"
        "migration_ids"
        "manual_actions"
    ]
    mut errors = []

    for field in $required_fields {
        if not ($field in ($entry | columns)) {
            $errors = ($errors | append $"upgrade_notes.toml: entry `($key)` is missing required field `($field)`")
        }
    }

    if ($errors | is-not-empty) {
        return $errors
    }

    let headline = ($entry.headline | into string | str trim)
    let date = ($entry.date | into string | str trim)
    let summary = (as_string_list $entry.summary)
    let impact = ($entry.upgrade_impact | into string | str trim)
    let acknowledged = (as_string_list $entry.acknowledged_guarded_changes)
    let entry_migration_ids = (as_string_list $entry.migration_ids)
    let manual_actions = (as_string_list $entry.manual_actions)

    if ($entry.version | into string | str trim) != $key {
        $errors = ($errors | append $"upgrade_notes.toml: entry `($key)` must declare version = `($key)`")
    }

    if $key == "unreleased" {
        if $date != "" {
            $errors = ($errors | append "upgrade_notes.toml: `unreleased` must keep date empty until a real release exists")
        }
    } else if $date == "" {
        $errors = ($errors | append $"upgrade_notes.toml: release entry `($key)` must declare a real release date")
    }

    if $headline == "" {
        $errors = ($errors | append $"upgrade_notes.toml: entry `($key)` must have a non-empty headline")
    }

    if ($summary | is-empty) {
        $errors = ($errors | append $"upgrade_notes.toml: entry `($key)` must have a non-empty summary list")
    }

    if not ($impact in $IMPACT_VALUES) {
        $errors = ($errors | append $"upgrade_notes.toml: entry `($key)` has invalid upgrade_impact `($impact)`")
    }

    for migration_id in $entry_migration_ids {
        if not ($migration_id in $migration_ids) {
            $errors = ($errors | append $"upgrade_notes.toml: entry `($key)` references unknown migration id `($migration_id)`")
        }
    }

    match $impact {
        "no_user_action" => {
            if not ($entry_migration_ids | is-empty) {
                $errors = ($errors | append $"upgrade_notes.toml: entry `($key)` must keep migration_ids empty when upgrade_impact = no_user_action")
            }
            if not ($manual_actions | is-empty) {
                $errors = ($errors | append $"upgrade_notes.toml: entry `($key)` must keep manual_actions empty when upgrade_impact = no_user_action")
            }
        }
        "migration_available" => {
            if ($entry_migration_ids | is-empty) {
                $errors = ($errors | append $"upgrade_notes.toml: entry `($key)` must list migration_ids when upgrade_impact = migration_available")
            }
        }
        "manual_action_required" => {
            if ($manual_actions | is-empty) {
                $errors = ($errors | append $"upgrade_notes.toml: entry `($key)` must list manual_actions when upgrade_impact = manual_action_required")
            }
        }
    }

    for path in $acknowledged {
        if not ($path in $GUARDED_FILES) and not ($path in $ACK_REQUIRED_FILES) {
            $errors = ($errors | append $"upgrade_notes.toml: entry `($key)` acknowledges non-guarded path `($path)`")
        }
    }

    $errors
}

def validate_changelog_entry [key: string, entry: record, changelog: string] {
    mut errors = []
    let heading = if $key == "unreleased" {
        "## Unreleased"
    } else {
        $"## ($key) - ($entry.date)"
    }

    if not ($changelog | str contains $heading) {
        $errors = ($errors | append $"CHANGELOG.md: missing heading `($heading)`")
    }

    if not ($changelog | str contains ($entry.headline | into string)) {
        $errors = ($errors | append $"CHANGELOG.md: missing headline for `($key)`: (($entry.headline | into string))")
    }

    $errors
}

def get_diff_base [diff_base?: string] {
    if ($diff_base | default "" | is-not-empty) {
        return $diff_base
    }

    let github_base = ($env.GITHUB_BASE_REF? | default "" | str trim)
    if $github_base != "" {
        return $"origin/($github_base)"
    }

    "HEAD~1"
}

def ref_exists [ref: string] {
    let result = (^git rev-parse --verify $ref | complete)
    $result.exit_code == 0
}

def get_changed_files [base: string] {
    if not (ref_exists $base) {
        return []
    }

    (
        ^git diff --name-only $"($base)..HEAD"
        | lines
        | where {|line| ($line | str trim) != "" }
    )
}

def extract_version_from_constants [content: string] {
    (
        $content
        | parse --regex 'export const YAZELIX_VERSION = "(?<version>[^"]+)"'
        | get -o 0.version
        | default null
    )
}

def get_previous_version [base: string] {
    if not (ref_exists $base) {
        return null
    }

    let result = (^git show $"($base):nushell/scripts/utils/constants.nu" | complete)
    if $result.exit_code != 0 {
        return null
    }

    extract_version_from_constants $result.stdout
}

def notes_changed_only_acknowledgements [entries: record, diff_base: string] {
    let previous_notes = (load_notes_from_ref $diff_base)
    if $previous_notes == null {
        return false
    }

    let previous_entries = ($previous_notes.releases? | default null)
    if $previous_entries == null {
        return false
    }

    let current_keys = (($entries | columns) | sort)
    let previous_keys = (($previous_entries | columns) | sort)
    if $current_keys != $previous_keys {
        return false
    }

    let changed_keys = (
        $current_keys | where {|key|
            (get_entry $entries $key) != (get_entry $previous_entries $key)
        }
    )

    if ($changed_keys | is-empty) {
        return false
    }

    for key in $changed_keys {
        let current_entry = (get_entry $entries $key)
        let previous_entry = (get_entry $previous_entries $key)
        if (drop_acknowledged_guarded_changes $current_entry) != (drop_acknowledged_guarded_changes $previous_entry) {
            return false
        }
    }

    true
}

def notes_changed_only_series [diff_base: string] {
    let previous_notes = (load_notes_from_ref $diff_base)
    if $previous_notes == null {
        return false
    }

    let current_notes = (load_notes)
    let current_without_series = (drop_optional_series $current_notes)
    let previous_without_series = (drop_optional_series $previous_notes)

    if $current_without_series != $previous_without_series {
        return false
    }

    let current_series = ($current_notes.series? | default null)
    let previous_series = ($previous_notes.series? | default null)
    $current_series != $previous_series
}

def validate_ci_rules [entries: record, diff_base: string] {
    let changed_files = (get_changed_files $diff_base)
    let current_entry = (get_entry $entries $YAZELIX_VERSION)
    let unreleased_entry = (get_entry $entries "unreleased")
    let previous_version = (get_previous_version $diff_base)
    let version_bumped = ($previous_version != null) and ($previous_version != $YAZELIX_VERSION)
    let docs_changed = ("docs/upgrade_notes.toml" in $changed_files) and ("CHANGELOG.md" in $changed_files)
    let one_doc_changed = (("docs/upgrade_notes.toml" in $changed_files) or ("CHANGELOG.md" in $changed_files)) and (not $docs_changed)
    let changed_ack_required = ($changed_files | where {|path| $path in $ACK_REQUIRED_FILES })
    let ack_only_notes_change = (
        ("docs/upgrade_notes.toml" in $changed_files)
        and (not ("CHANGELOG.md" in $changed_files))
        and (notes_changed_only_acknowledgements $entries $diff_base)
    )
    let series_only_notes_change = (
        ("docs/upgrade_notes.toml" in $changed_files)
        and (not ("CHANGELOG.md" in $changed_files))
        and (notes_changed_only_series $diff_base)
    )
    let target_key = if $version_bumped { $YAZELIX_VERSION } else { "unreleased" }
    let target_entry = if $target_key == "unreleased" { $unreleased_entry } else { $current_entry }
    let acknowledged = (as_string_list $target_entry.acknowledged_guarded_changes)
    mut errors = []

    if $one_doc_changed and (not $ack_only_notes_change) and (not $series_only_notes_change) {
        $errors = ($errors | append "CI: CHANGELOG.md and docs/upgrade_notes.toml must change together")
    }

    if $version_bumped and not $docs_changed {
        $errors = ($errors | append $"CI: version bump from ($previous_version) to ($YAZELIX_VERSION) must update both CHANGELOG.md and docs/upgrade_notes.toml")
    }

    if (not $version_bumped) and (not ($changed_ack_required | is-empty)) and (not $docs_changed) and (not $ack_only_notes_change) {
        $errors = ($errors | append "CI: guarded config-contract changes must update both CHANGELOG.md and docs/upgrade_notes.toml in the same diff")
    }

    for path in $changed_ack_required {
        if not ($path in $acknowledged) {
            $errors = ($errors | append $"CI: entry `($target_key)` must acknowledge guarded change `($path)`")
        }
    }

    if (not $version_bumped) and ("nushell/scripts/utils/constants.nu" in $changed_files) and (not $docs_changed) {
        $errors = ($errors | append "CI: changes to nushell/scripts/utils/constants.nu must update both CHANGELOG.md and docs/upgrade_notes.toml")
    }

    if not ($errors | is-empty) {
        print $"Upgrade contract diff base: ($diff_base)"
        print $"Changed files: (($changed_files | str join ', '))"
        print $"Target upgrade-notes entry: ($target_key)"
        print $"Acknowledged guarded changes: (($acknowledged | str join ', '))"
    }

    $errors
}

export def main [
    --ci
    --diff-base: string
] {
    let changelog_path = ($REPO_ROOT | path join "CHANGELOG.md")
    let notes_path = ($REPO_ROOT | path join "docs" "upgrade_notes.toml")
    let changelog = (load_changelog)
    let notes = (load_notes)
    let entries = (get_release_entries)
    let migration_ids = (get_config_migration_rules | get id)
    mut errors = []

    if not ($changelog_path | path exists) {
        $errors = ($errors | append "CHANGELOG.md is missing")
    }
    if not ($notes_path | path exists) {
        $errors = ($errors | append "docs/upgrade_notes.toml is missing")
    }

    let current_entry = (get_entry $entries $YAZELIX_VERSION)
    let unreleased_entry = (get_entry $entries "unreleased")

    if $current_entry == null {
        $errors = ($errors | append $"docs/upgrade_notes.toml is missing the current release entry `($YAZELIX_VERSION)`")
    }
    if $unreleased_entry == null {
        $errors = ($errors | append "docs/upgrade_notes.toml is missing the `unreleased` entry")
    }

    if $current_entry != null {
        $errors = ($errors | append (validate_entry $YAZELIX_VERSION $current_entry $migration_ids))
        $errors = ($errors | append (validate_changelog_entry $YAZELIX_VERSION $current_entry $changelog))
    }
    if $unreleased_entry != null {
        $errors = ($errors | append (validate_entry "unreleased" $unreleased_entry $migration_ids))
        $errors = ($errors | append (validate_changelog_entry "unreleased" $unreleased_entry $changelog))
    }

    if $ci {
        $errors = ($errors | append (validate_ci_rules $entries (get_diff_base $diff_base)))
    }

    if not ($errors | is-empty) {
        $errors | flatten | each {|line| print $"❌ ($line)" }
        error make {msg: "Upgrade contract validation failed"}
    }

    if $ci {
        print "✅ Upgrade contract is valid in CI mode"
    } else {
        print "✅ Upgrade contract is valid"
    }
}
