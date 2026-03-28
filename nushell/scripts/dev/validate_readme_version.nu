#!/usr/bin/env nu

use ../utils/common.nu [get_yazelix_dir]
use ../utils/constants.nu [YAZELIX_VERSION]
use ../utils/readme_release_block.nu [extract_readme_latest_series_section render_readme_latest_series_section]

export def main [] {
    let readme_path = ((get_yazelix_dir) | path join "README.md")
    let readme_contents = (open --raw $readme_path)
    let readme_title = (
        $readme_contents
        | lines
        | first
        | str trim
    )
    let expected_title = $"# Yazelix ($YAZELIX_VERSION)"

    if $readme_title != $expected_title {
        error make {
            msg: $"README title/version drift detected. Expected '($expected_title)' but found '($readme_title)'."
        }
    }

    let expected_block = (render_readme_latest_series_section)
    let actual_block = (extract_readme_latest_series_section $readme_contents)

    if $actual_block != $expected_block {
        error make {
            msg: "README generated latest-series block drift detected. Regenerate the managed block from docs/upgrade_notes.toml."
        }
    }
}
