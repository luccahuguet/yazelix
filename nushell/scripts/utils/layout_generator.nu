#!/usr/bin/env nu
# Copy Zellij layouts to merged config directory

const widget_tray_placeholder = "__YAZELIX_WIDGET_TRAY__"

def build_widget_tray [widget_tray: list<string>]: nothing -> string {
    let allowed = ["layout", "editor", "shell", "term", "cpu", "ram"]
    mut parts = []
    for widget in $widget_tray {
        if not ($widget in $allowed) {
            let allowed_str = ($allowed | str join ", ")
            print $"❌ Invalid zellij.widget_tray entry: ($widget) (allowed: ($allowed_str))"
            exit 1
        }
        let part = match $widget {
            "layout" => "{swap_layout}"
            "editor" => "#[fg=#00ff88,bold][editor: {command_editor}]"
            "shell" => "#[fg=#00ff88,bold][shell: {command_shell}]"
            "term" => "#[fg=#00ff88,bold][term: {command_term}]"
            "cpu" => "{command_cpu}"
            "ram" => "{command_ram}"
            _ => ""
        }
        $parts = ($parts | append $part)
    }
    $parts | str join " "
}

# Copy a layout file to the target directory
# Parameters:
#   source_layout: path to the template layout file
#   target_layout: path to the output layout file
export def generate_layout [
    source_layout: string
    target_layout: string
    widget_tray: list<string>
]: nothing -> nothing {
    let content = (open ($source_layout | path expand))
    if ($content | str contains $widget_tray_placeholder) {
        let tray = build_widget_tray $widget_tray
        let updated = ($content | str replace -a $widget_tray_placeholder $tray)
        $updated | save --force ($target_layout | path expand)
        return
    }

    if ($content | str contains "zjstatus.wasm") {
        print $"❌ Missing widget tray placeholder in: ($source_layout)"
        exit 1
    }

    $content | save --force ($target_layout | path expand)
}

# Copy all layout files to the target directory
export def generate_all_layouts [
    layouts_source_dir: string
    layouts_target_dir: string
    widget_tray: list<string>
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
            generate_layout $source $target $widget_tray
            print $"Generated layout: ($target)"
        }
    }
}
