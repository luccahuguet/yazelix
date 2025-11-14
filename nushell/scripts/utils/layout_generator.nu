#!/usr/bin/env nu
# Generate Zellij layouts with icon or ASCII widgets

use constants.nu *

# Icon format strings (Nerd Fonts)
const ICON_CPU_FORMAT = '#[fg=#ff6600]󰚂{stdout}'
const ICON_RAM_FORMAT = '#[fg=#ff6600]󰇾{stdout}'
const ICON_FLOATING = '⬚ '

# ASCII format strings (universal compatibility)
const ASCII_CPU_FORMAT = '#[fg=#ff6600][CPU {stdout}]'
const ASCII_RAM_FORMAT = '#[fg=#ff6600][RAM {stdout}]'
const ASCII_FLOATING = '[F] '

# Generate a layout file with icon or ASCII widgets
# Parameters:
#   source_layout: path to the template layout file
#   target_layout: path to the output layout file
#   use_icons: whether to use icon widgets (true) or ASCII (false)
export def generate_layout [
    source_layout: string
    target_layout: string
    use_icons: bool
]: nothing -> nothing {
    # If using icons, just copy the source file directly (source has icons)
    if $use_icons {
        cp $source_layout $target_layout
        return
    }

    # Otherwise, generate ASCII variant by replacing icon strings
    let content = (open $source_layout)

    # Replace icon format strings with ASCII variants
    let updated_content = ($content
        | str replace --all $ICON_CPU_FORMAT $ASCII_CPU_FORMAT
        | str replace --all $ICON_RAM_FORMAT $ASCII_RAM_FORMAT
        | str replace --all $ICON_FLOATING $ASCII_FLOATING
    )

    # Write the generated layout
    $updated_content | save --force $target_layout
}

# Generate all layout files based on icon preference
export def generate_all_layouts [
    layouts_source_dir: string
    layouts_target_dir: string
    use_icons: bool
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

    # Generate each layout file
    for file in $layout_files {
        let source = ($layouts_source_dir | path join $file)
        let target = ($layouts_target_dir | path join $file)

        if ($source | path exists) {
            generate_layout $source $target $use_icons
            print $"Generated layout: ($target) \(icons: ($use_icons)\)"
        }
    }
}
