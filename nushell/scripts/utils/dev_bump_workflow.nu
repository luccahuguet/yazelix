#!/usr/bin/env nu

def fail [message: string] {
    error make {msg: $message}
}

def run_git [
    repo_root: string
    ...args: string
] {
    let result = (^git -C $repo_root ...$args | complete)
    if $result.exit_code != 0 {
        let stderr = ($result.stderr | default "" | str trim)
        let rendered_args = ($args | str join " ")
        fail $"Git command failed: git -C ($repo_root) ($rendered_args)\n($stderr)"
    }

    {
        stdout: ($result.stdout | default "" | str trim)
        stderr: ($result.stderr | default "" | str trim)
    }
}

def render_default_unreleased_summary [released_version: string] {
    [$"Reserved for post-release changes after ($released_version) lands."]
}

def build_default_unreleased_entry [released_version: string] {
    {
        version: "unreleased"
        date: ""
        headline: $"Post-($released_version) work in progress"
        summary: (render_default_unreleased_summary $released_version)
        upgrade_impact: "no_user_action"
        acknowledged_guarded_changes: []
        migration_ids: []
        manual_actions: []
    }
}

def render_default_unreleased_changelog [released_version: string] {
    [
        "## Unreleased"
        ""
        $"Post-($released_version) work in progress"
        ""
        "Upgrade impact: no user action required"
        ""
        "Highlights:"
        $"- Reserved for post-release changes after ($released_version) lands."
    ] | str join "\n"
}

def ensure_clean_git_worktree [repo_root: string] {
    let status = (run_git $repo_root "status" "--porcelain").stdout
    if ($status | is-not-empty) {
        fail "yzx dev bump requires a clean git worktree."
    }
}

def validate_target_version [target_version: string] {
    let normalized = ($target_version | into string | str trim)
    if not ($normalized =~ '^v\d+(?:\.\d+)+$') {
        fail $"Invalid version `($target_version)`. Expected a git tag like v13.14 or v13.14.1"
    }
    $normalized
}

def get_current_version [repo_root: string] {
    let constants_path = ($repo_root | path join "nushell" "scripts" "utils" "constants.nu")
    let constants = (open --raw $constants_path)
    let version = (
        $constants
        | parse --regex 'export const YAZELIX_VERSION = "(?<version>v[^"]+)"'
        | get -o version.0
        | default ""
    )

    if ($version | is-empty) {
        fail $"Failed to read YAZELIX_VERSION from ($constants_path)"
    }

    $version
}

def ensure_target_tag_absent [repo_root: string, target_version: string] {
    let existing = (run_git $repo_root "tag" "--list" $target_version).stdout
    if ($existing | is-not-empty) {
        fail $"Tag already exists: ($target_version)"
    }
}

def ensure_releasable_unreleased_entry [entry: record, current_version: string] {
    if $entry == (build_default_unreleased_entry $current_version) {
        fail $"Refusing to bump version while docs/upgrade_notes.toml still has the untouched unreleased placeholder for ($current_version)."
    }
}

def ensure_releasable_unreleased_changelog [unreleased_section: string, current_version: string] {
    if (($unreleased_section | str trim) == ((render_default_unreleased_changelog $current_version) | str trim)) {
        fail $"Refusing to bump version while CHANGELOG.md still has the untouched unreleased placeholder for ($current_version)."
    }
}

def update_version_constant [repo_root: string, target_version: string] {
    let constants_path = ($repo_root | path join "nushell" "scripts" "utils" "constants.nu")
    let contents = (open --raw $constants_path)
    let updated = (
        $contents
        | str replace -ra 'export const YAZELIX_VERSION = "v[^"]+"' $"export const YAZELIX_VERSION = \"($target_version)\""
    )
    $updated | save --force --raw $constants_path
}

def rotate_upgrade_notes [repo_root: string, current_version: string, target_version: string, release_date: string] {
    let notes_path = ($repo_root | path join "docs" "upgrade_notes.toml")
    let notes = (open $notes_path)
    let releases = ($notes.releases? | default {})
    let unreleased = ($releases.unreleased? | default null)

    if $unreleased == null {
        fail $"docs/upgrade_notes.toml is missing releases.unreleased"
    }
    if ($releases | columns | any {|column| $column == $target_version }) {
        fail $"docs/upgrade_notes.toml already contains release entry `($target_version)`"
    }

    ensure_releasable_unreleased_entry $unreleased $current_version

    let released_entry = (
        $unreleased
        | upsert version $target_version
        | upsert date $release_date
    )
    let rotated_entries = (
        [
            {key: "unreleased", value: (build_default_unreleased_entry $target_version)}
            {key: $target_version, value: $released_entry}
        ]
        | append (
            $releases
            | transpose key value
            | where {|entry| ($entry.key != "unreleased") and ($entry.key != $target_version) }
        )
    )
    let rotated_releases = (
        $rotated_entries
        | reduce -f {} {|entry, acc| $acc | upsert $entry.key $entry.value }
    )
    let updated_notes = ($notes | upsert releases $rotated_releases)
    $updated_notes | to toml | save --force --raw $notes_path
}

def rotate_changelog [repo_root: string, current_version: string, target_version: string, release_date: string] {
    let changelog_path = ($repo_root | path join "CHANGELOG.md")
    let lines = (open --raw $changelog_path | lines)
    let unreleased_index = (
        $lines
        | enumerate
        | where item == "## Unreleased"
        | get -o index.0
        | default null
    )

    if $unreleased_index == null {
        fail "CHANGELOG.md is missing the `## Unreleased` heading."
    }

    let next_heading_index = (
        $lines
        | enumerate
        | where {|row| ($row.index > $unreleased_index) and ($row.item | str starts-with "## ") }
        | get -o index.0
        | default ($lines | length)
    )
    let prefix = ($lines | first $unreleased_index)
    let unreleased_lines = ($lines | skip $unreleased_index | first ($next_heading_index - $unreleased_index))
    let suffix = ($lines | skip $next_heading_index)
    let unreleased_section = ($unreleased_lines | str join "\n")

    ensure_releasable_unreleased_changelog $unreleased_section $current_version

    let released_lines = (
        [$"## ($target_version) - ($release_date)"]
        | append ($unreleased_lines | skip 1)
    )
    mut final_lines = []
    $final_lines = ($final_lines | append $prefix)
    $final_lines = ($final_lines | append ((render_default_unreleased_changelog $target_version) | lines))
    $final_lines = ($final_lines | append "")
    $final_lines = ($final_lines | append $released_lines)
    if ($suffix | is-not-empty) {
        $final_lines = ($final_lines | append "")
        $final_lines = ($final_lines | append $suffix)
    }

    (($final_lines | str join "\n") + "\n") | save --force --raw $changelog_path
}

def sync_readme_version [repo_root: string, target_version: string] {
    let readme_path = ($repo_root | path join "README.md")
    if not ($readme_path | path exists) {
        fail $"README.md not found under ($repo_root)"
    }
    let contents = (open --raw $readme_path)
    let updated = ($contents | str replace -r '^# Yazelix v[^\r\n]+' $"# Yazelix ($target_version)")
    $updated | save --force --raw $readme_path
}

export def perform_version_bump [
    repo_root: string
    target_version: string
] {
    let resolved_repo_root = ($repo_root | path expand)
    let resolved_target_version = (validate_target_version $target_version)
    let current_version = (get_current_version $resolved_repo_root)

    if $current_version == $resolved_target_version {
        fail $"YAZELIX_VERSION already matches ($resolved_target_version)"
    }

    ensure_clean_git_worktree $resolved_repo_root
    ensure_target_tag_absent $resolved_repo_root $resolved_target_version

    let release_date = (date now | format date "%Y-%m-%d")

    rotate_upgrade_notes $resolved_repo_root $current_version $resolved_target_version $release_date
    rotate_changelog $resolved_repo_root $current_version $resolved_target_version $release_date
    update_version_constant $resolved_repo_root $resolved_target_version
    sync_readme_version $resolved_repo_root $resolved_target_version

    run_git $resolved_repo_root "add" "nushell/scripts/utils/constants.nu" "docs/upgrade_notes.toml" "CHANGELOG.md" "README.md" | ignore
    let commit_message = $"Bump version to ($resolved_target_version)"
    run_git $resolved_repo_root "commit" "--quiet" "-m" $commit_message | ignore
    run_git $resolved_repo_root "tag" "-a" $resolved_target_version "-m" $"Release ($resolved_target_version)" | ignore

    let final_version = (get_current_version $resolved_repo_root)
    if $final_version != $resolved_target_version {
        fail $"Version mismatch after bump: constants declare ($final_version), expected ($resolved_target_version)"
    }

    let commit_sha = (run_git $resolved_repo_root "rev-parse" "HEAD").stdout
    let created_tag = (run_git $resolved_repo_root "tag" "--list" $resolved_target_version).stdout
    if $created_tag != $resolved_target_version {
        fail $"Failed to verify created git tag ($resolved_target_version)"
    }

    {
        repo_root: $resolved_repo_root
        previous_version: $current_version
        target_version: $resolved_target_version
        release_date: $release_date
        commit_message: $commit_message
        commit_sha: $commit_sha
        tag: $resolved_target_version
    }
}
