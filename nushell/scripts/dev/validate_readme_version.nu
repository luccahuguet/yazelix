#!/usr/bin/env nu

use ../maintainer/repo_checkout.nu [require_yazelix_repo_root]
use ../utils/constants.nu [YAZELIX_VERSION]
use ../maintainer/readme_surface.nu [extract_readme_latest_series_section render_readme_latest_series_section_for_root]

export def main [] {
    let repo_root = (require_yazelix_repo_root)
    let readme_path = ($repo_root | path join "README.md")
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

    let expected_block = (render_readme_latest_series_section_for_root $repo_root)
    let actual_block = (extract_readme_latest_series_section $readme_contents)

    if $actual_block != $expected_block {
        error make {
            msg: "README generated latest-series block drift detected. Regenerate the managed block from docs/upgrade_notes.toml."
        }
    }

    let release_heading = (
        $actual_block
        | lines
        | get -o 1
        | default ""
        | str trim
    )
    let expected_release_heading = $"## Latest Tagged Release: ($YAZELIX_VERSION)"

    if $release_heading != $expected_release_heading {
        error make {
            msg: $"README latest tagged release drift detected. Expected '($expected_release_heading)' but found '($release_heading)'."
        }
    }
}
