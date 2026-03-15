#!/usr/bin/env nu
# Copy Zellij layouts to merged config directory

const widget_tray_placeholder = "__YAZELIX_WIDGET_TRAY__"
const pane_orchestrator_plugin_url_placeholder = "__YAZELIX_PANE_ORCHESTRATOR_PLUGIN_URL__"
const home_dir_placeholder = "__YAZELIX_HOME_DIR__"
const static_fragment_specs = [
    {placeholder: "__YAZELIX_ZJSTATUS_TAB_TEMPLATE__", file: "fragments/zjstatus_tab_template.kdl"}
    {placeholder: "__YAZELIX_KEYBINDS_COMMON__", file: "fragments/keybinds_common.kdl"}
    {placeholder: "__YAZELIX_SWAP_SIDEBAR_OPEN__", file: "fragments/swap_sidebar_open.kdl"}
    {placeholder: "__YAZELIX_SWAP_SIDEBAR_CLOSED__", file: "fragments/swap_sidebar_closed.kdl"}
]

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

def load_static_fragments [layouts_source_dir: string]: nothing -> list<record> {
    let source_root = ($layouts_source_dir | path expand)
    $static_fragment_specs | each {|spec|
        let fragment_path = ($source_root | path join $spec.file)
        if not ($fragment_path | path exists) {
            print $"❌ Missing required layout fragment: ($fragment_path)"
            exit 1
        }
        {
            placeholder: $spec.placeholder
            value: (open $fragment_path)
        }
    }
}

def apply_static_fragments [
    content: string
    static_fragments: list<record>
]: nothing -> string {
    mut updated = $content
    for fragment in $static_fragments {
        if ($updated | str contains $fragment.placeholder) {
            let fragment_lines = ($fragment.value | lines)
            $updated = (
                $updated
                | lines
                | each {|line|
                    if ($line | str contains $fragment.placeholder) {
                        let indent = (($line | parse -r '^(?<indent>\s*).*') | get 0.indent)
                        $fragment_lines
                        | each {|fragment_line| $"($indent)($fragment_line)"}
                        | str join "\n"
                    } else {
                        $line
                    }
                }
                | str join "\n"
            )
        }
    }
    $updated
}

# Copy a layout file to the target directory
# Parameters:
#   source_layout: path to the template layout file
#   target_layout: path to the output layout file
export def generate_layout [
    source_layout: string
    target_layout: string
    widget_tray: list<string>
    static_fragments: list<record>
    pane_orchestrator_plugin_url: string
]: nothing -> nothing {
    let content = (open ($source_layout | path expand))
    mut updated = apply_static_fragments $content $static_fragments
    let home_dir = (
        if (($env.HOME? | default "") | is-not-empty) {
            $env.HOME | path expand
        } else {
            "~" | path expand
        }
    )

    if ($updated | str contains $widget_tray_placeholder) {
        let tray = build_widget_tray $widget_tray
        $updated = ($updated | str replace -a $widget_tray_placeholder $tray)
    }

    if ($updated | str contains $home_dir_placeholder) {
        $updated = ($updated | str replace -a $home_dir_placeholder $home_dir)
    }

    if ($updated | str contains $pane_orchestrator_plugin_url_placeholder) {
        $updated = ($updated | str replace -a $pane_orchestrator_plugin_url_placeholder $pane_orchestrator_plugin_url)
    }

    if ($updated | str contains "zjstatus.wasm") and not ($updated | str contains "{swap_layout}") {
        print $"❌ Missing widget tray placeholder in: ($source_layout)"
        exit 1
    }

    $updated | save --force ($target_layout | path expand)
}

# Copy all layout files to the target directory
export def generate_all_layouts [
    layouts_source_dir: string
    layouts_target_dir: string
    widget_tray: list<string>
    pane_orchestrator_plugin_url: string
]: nothing -> nothing {
    let source_root = ($layouts_source_dir | path expand)
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
    let static_fragments = load_static_fragments $source_root

    # Copy each layout file
    for file in $layout_files {
        let source = ($source_root | path join $file)
        let target = ($layouts_target_dir | path join $file)

        if ($source | path exists) {
            generate_layout $source $target $widget_tray $static_fragments $pane_orchestrator_plugin_url
            print $"Generated layout: ($target)"
        }
    }
}
