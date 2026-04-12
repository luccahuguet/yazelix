#!/usr/bin/env nu

use ../utils/common.nu [get_yazelix_runtime_dir]
use ../utils/constants.nu [YAZELIX_VERSION]
use ../utils/upgrade_notes.nu [get_current_major_series_entry]

export const README_LATEST_SERIES_BEGIN = "<!-- BEGIN GENERATED README LATEST SERIES -->"
export const README_LATEST_SERIES_END = "<!-- END GENERATED README LATEST SERIES -->"

def get_readme_path [] {
    (get_yazelix_runtime_dir | path join "README.md")
}

def as_string_list [value: any] {
    if (($value | describe) | str contains "list") {
        $value | each {|item| $item | into string | str trim } | where {|item| $item | is-not-empty }
    } else {
        []
    }
}

export def render_readme_latest_series_section [version: string = $YAZELIX_VERSION] {
    let entry = (get_current_major_series_entry $version)
    let headline = ($entry.headline? | default "" | into string | str trim)
    let summary_items = (as_string_list ($entry.summary? | default []))

    mut lines = [
        $README_LATEST_SERIES_BEGIN
        $"## What's New In ($entry.key)"
        ""
    ]

    if ($headline | is-not-empty) {
        $lines = ($lines | append [$headline, ""])
    }

    for item in $summary_items {
        $lines = ($lines | append [$"- ($item)"])
    }

    $lines = ($lines | append [
        ""
        $"For exact ($version) upgrade notes, see [CHANGELOG]\(./CHANGELOG.md\) or run `yzx whats_new`."
        "For the longer project story, see [Version History](./docs/history.md)."
        $README_LATEST_SERIES_END
    ])

    $lines | str join "\n"
}

export def extract_readme_latest_series_section [contents: string] {
    let normalized = ($contents | str replace -a "\r\n" "\n")

    if (not ($normalized | str contains $README_LATEST_SERIES_BEGIN)) or (not ($normalized | str contains $README_LATEST_SERIES_END)) {
        error make {msg: "README is missing the generated latest-series markers"}
    }

    let before_and_rest = ($normalized | split row $README_LATEST_SERIES_BEGIN)
    if ($before_and_rest | length) != 2 {
        error make {msg: "README has an invalid generated latest-series start marker layout"}
    }

    let rest = ($before_and_rest | get -o 1)
    if $rest == null {
        error make {msg: "README is missing the generated latest-series remainder block"}
    }
    let block_and_after = ($rest | split row $README_LATEST_SERIES_END)
    if ($block_and_after | length) != 2 {
        error make {msg: "README has an invalid generated latest-series end marker layout"}
    }

    let block = ($block_and_after | get -o 0)
    if $block == null {
        error make {msg: "README is missing the generated latest-series block body"}
    }

    $"($README_LATEST_SERIES_BEGIN)($block)($README_LATEST_SERIES_END)"
}

def sync_readme_latest_series_section [readme_path: string] {
    let contents = (open --raw $readme_path)
    let normalized = ($contents | str replace -a "\r\n" "\n")
    let rendered = (render_readme_latest_series_section)
    let current_block = (extract_readme_latest_series_section $normalized)
    let updated = ($normalized | str replace $current_block $rendered)

    if $updated != $normalized {
        $updated | save --force --raw $readme_path
        return true
    }

    false
}

export def sync_readme_surface [readme_path?: string, version: string = $YAZELIX_VERSION] {
    let target_readme_path = if ($readme_path | default "" | is-empty) {
        get_readme_path
    } else {
        $readme_path
    }
    let contents = (open --raw $target_readme_path)
    let normalized = ($contents | str replace -a "\r\n" "\n")
    let expected_title = $"# Yazelix ($version)"
    let updated_title = ($normalized | str replace -r '^# Yazelix v[^\r\n]+' $expected_title)
    let title_changed = ($updated_title != $normalized)

    if $title_changed {
        $updated_title | save --force --raw $target_readme_path
    }

    let series_changed = (sync_readme_latest_series_section $target_readme_path)

    {
        readme_path: $target_readme_path
        title_changed: $title_changed
        series_changed: $series_changed
    }
}
