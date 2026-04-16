#!/usr/bin/env nu

use common.nu [get_yazelix_runtime_dir]
use constants.nu [YAZELIX_VERSION]

def get_upgrade_notes_path [] {
    (get_yazelix_runtime_dir | path join "docs" "upgrade_notes.toml")
}

def load_upgrade_notes [] {
    let notes_path = (get_upgrade_notes_path)
    if not ($notes_path | path exists) {
        error make {msg: $"upgrade notes not found at ($notes_path)"}
    }

    open $notes_path
}

export def find_release_entry [version: string = $YAZELIX_VERSION] {
    let release_key = ($version | into string | str trim)
    let notes = (load_upgrade_notes)
    let releases = ($notes.releases? | default {})
    if not (($releases | describe) | str contains "record") {
        error make {msg: "upgrade notes are missing the `releases` table"}
    }

    let entry = ($releases | get -o $release_key)
    if $entry == null {
        return null
    }

    if not (($entry | describe) | str contains "record") {
        error make {msg: $"upgrade notes release entry `($release_key)` is not a record"}
    }

    ($entry | merge {key: $release_key, version: $release_key})
}

export def get_current_major_series_entry [version: string = $YAZELIX_VERSION] {
    let series_key = (
        $version
        | parse --regex '^(?<major>v\d+)'
        | get -o 0.major
        | default null
    )
    if $series_key == null {
        error make {msg: $"failed to derive a major series key from version `($version)`"}
    }

    let notes = (load_upgrade_notes)
    let series = ($notes.series? | default {})
    if not (($series | describe) | str contains "record") {
        error make {msg: "upgrade notes are missing the `series` table"}
    }

    let entry = ($series | get -o $series_key)
    if $entry == null {
        error make {msg: $"upgrade notes are missing the current major series entry `($series_key)`"}
    }

    if not (($entry | describe) | str contains "record") {
        error make {msg: $"upgrade notes series entry `($series_key)` is not a record"}
    }

    ($entry | merge {key: $series_key, version: $version})
}
