#!/usr/bin/env nu
# Copy Zellij layouts to merged config directory

use constants.nu *

# Copy a layout file to the target directory
# Parameters:
#   source_layout: path to the template layout file
#   target_layout: path to the output layout file
export def generate_layout [
    source_layout: string
    target_layout: string
]: nothing -> nothing {
    cp $source_layout $target_layout
}

# Copy all layout files to the target directory
export def generate_all_layouts [
    layouts_source_dir: string
    layouts_target_dir: string
]: nothing -> nothing {
    # Ensure target directory exists
    mkdir $layouts_target_dir

    # List of layout files to process
    let layout_files = [
        "yzx_side.kdl"
        "yzx_no_side.kdl"
        "yzx_side.swap.kdl"
        "yzx_no_side.swap.kdl"
        "yzx_sweep_test.kdl"
    ]

    # Copy each layout file
    for file in $layout_files {
        let source = ($layouts_source_dir | path join $file)
        let target = ($layouts_target_dir | path join $file)

        if ($source | path exists) {
            generate_layout $source $target
            print $"Generated layout: ($target)"
        }
    }
}
