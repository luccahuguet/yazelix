#!/usr/bin/env nu

use ../utils/common.nu [get_yazelix_dir]
use ../utils/constants.nu [YAZELIX_VERSION]

export def main [] {
    let readme_path = ((get_yazelix_dir) | path join "README.md")
    let readme_title = (
        open --raw $readme_path
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
}
