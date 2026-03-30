#!/usr/bin/env nu
# Copy Zellij layouts to merged config directory

const widget_tray_placeholder = "__YAZELIX_WIDGET_TRAY__"
const custom_text_placeholder = "__YAZELIX_CUSTOM_TEXT_SEGMENT__"
const pane_orchestrator_plugin_url_placeholder = "__YAZELIX_PANE_ORCHESTRATOR_PLUGIN_URL__"
const home_dir_placeholder = "__YAZELIX_HOME_DIR__"
const runtime_dir_placeholder = "__YAZELIX_RUNTIME_DIR__"
const sidebar_width_percent_placeholder = "__YAZELIX_SIDEBAR_WIDTH_PERCENT__"
const open_content_width_percent_placeholder = "__YAZELIX_OPEN_CONTENT_WIDTH_PERCENT__"
const open_primary_width_percent_placeholder = "__YAZELIX_OPEN_PRIMARY_WIDTH_PERCENT__"
const open_secondary_width_percent_placeholder = "__YAZELIX_OPEN_SECONDARY_WIDTH_PERCENT__"
const closed_content_width_percent_placeholder = "__YAZELIX_CLOSED_CONTENT_WIDTH_PERCENT__"
const closed_primary_width_percent_placeholder = "__YAZELIX_CLOSED_PRIMARY_WIDTH_PERCENT__"
const closed_secondary_width_percent_placeholder = "__YAZELIX_CLOSED_SECONDARY_WIDTH_PERCENT__"
const static_fragment_specs = [
    {placeholder: "__YAZELIX_ZJSTATUS_TAB_TEMPLATE__", file: "fragments/zjstatus_tab_template.kdl"}
    {placeholder: "__YAZELIX_KEYBINDS_COMMON__", file: "fragments/keybinds_common.kdl"}
    {placeholder: "__YAZELIX_SWAP_SIDEBAR_OPEN__", file: "fragments/swap_sidebar_open.kdl"}
    {placeholder: "__YAZELIX_SWAP_SIDEBAR_CLOSED__", file: "fragments/swap_sidebar_closed.kdl"}
]

export def render_widget_tray_segment [widget_tray: list<string>]: nothing -> string {
    let allowed = ["editor", "shell", "term", "cpu", "ram"]
    mut parts = []
    for widget in $widget_tray {
        if not ($widget in $allowed) {
            let allowed_str = ($allowed | str join ", ")
            print $"❌ Invalid zellij.widget_tray entry: ($widget) \(allowed: ($allowed_str)\)"
            exit 1
        }
        let part = match $widget {
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

export def render_custom_text_segment [custom_text: string]: nothing -> string {
    let trimmed = ($custom_text | str trim)
    if ($trimmed | is-empty) {
        ""
    } else {
        $"#[fg=#ffff00,bold][($trimmed)] "
    }
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

def compute_sidebar_layout_percentages [sidebar_width_percent: int]: nothing -> record {
    if ($sidebar_width_percent < 10) or ($sidebar_width_percent > 40) {
        print $"❌ Invalid sidebar width percent: ($sidebar_width_percent) \(expected 10 to 40\)"
        exit 1
    }

    let open_content_width_percent = (100 - $sidebar_width_percent)
    let open_primary_width_percent = (($open_content_width_percent * 3) // 5)
    let open_secondary_width_percent = ($open_content_width_percent - $open_primary_width_percent)
    let closed_content_width_percent = 99
    let closed_primary_width_percent = (($closed_content_width_percent * 3) // 5)
    let closed_secondary_width_percent = ($closed_content_width_percent - $closed_primary_width_percent)

    {
        sidebar_width_percent: $"($sidebar_width_percent)%"
        open_content_width_percent: $"($open_content_width_percent)%"
        open_primary_width_percent: $"($open_primary_width_percent)%"
        open_secondary_width_percent: $"($open_secondary_width_percent)%"
        closed_content_width_percent: $"($closed_content_width_percent)%"
        closed_primary_width_percent: $"($closed_primary_width_percent)%"
        closed_secondary_width_percent: $"($closed_secondary_width_percent)%"
    }
}

# Copy a layout file to the target directory
# Parameters:
#   source_layout: path to the template layout file
#   target_layout: path to the output layout file
export def generate_layout [
    source_layout: string
    target_layout: string
    widget_tray: list<string>
    custom_text: string
    static_fragments: list<record>
    pane_orchestrator_plugin_url: string
    runtime_dir: string
    sidebar_width_percent: int
]: nothing -> nothing {
    let content = (open ($source_layout | path expand))
    mut updated = apply_static_fragments $content $static_fragments
    let layout_percentages = (compute_sidebar_layout_percentages $sidebar_width_percent)
    let home_dir = (
        if (($env.HOME? | default "") | is-not-empty) {
            $env.HOME | path expand
        } else {
            "~" | path expand
        }
    )

    if ($updated | str contains $widget_tray_placeholder) {
        let tray = render_widget_tray_segment $widget_tray
        $updated = ($updated | str replace -a $widget_tray_placeholder $tray)
    }

    if ($updated | str contains $custom_text_placeholder) {
        let segment = render_custom_text_segment $custom_text
        $updated = ($updated | str replace -a $custom_text_placeholder $segment)
    }

    if ($updated | str contains $home_dir_placeholder) {
        $updated = ($updated | str replace -a $home_dir_placeholder $home_dir)
    }

    if ($updated | str contains $runtime_dir_placeholder) {
        $updated = ($updated | str replace -a $runtime_dir_placeholder ($runtime_dir | path expand))
    }

    if ($updated | str contains $pane_orchestrator_plugin_url_placeholder) {
        $updated = ($updated | str replace -a $pane_orchestrator_plugin_url_placeholder $pane_orchestrator_plugin_url)
    }

    for placeholder in [
        {name: $sidebar_width_percent_placeholder, value: $layout_percentages.sidebar_width_percent}
        {name: $open_content_width_percent_placeholder, value: $layout_percentages.open_content_width_percent}
        {name: $open_primary_width_percent_placeholder, value: $layout_percentages.open_primary_width_percent}
        {name: $open_secondary_width_percent_placeholder, value: $layout_percentages.open_secondary_width_percent}
        {name: $closed_content_width_percent_placeholder, value: $layout_percentages.closed_content_width_percent}
        {name: $closed_primary_width_percent_placeholder, value: $layout_percentages.closed_primary_width_percent}
        {name: $closed_secondary_width_percent_placeholder, value: $layout_percentages.closed_secondary_width_percent}
    ] {
        if ($updated | str contains $placeholder.name) {
            $updated = ($updated | str replace -a $placeholder.name $placeholder.value)
        }
    }

    if ($updated | str contains $widget_tray_placeholder) {
        print $"❌ Failed to expand widget tray placeholder in: ($source_layout)"
        exit 1
    }

    $updated | save --force ($target_layout | path expand)
}

# Copy all layout files to the target directory
export def generate_all_layouts [
    layouts_source_dir: string
    layouts_target_dir: string
    widget_tray: list<string>
    custom_text: string
    pane_orchestrator_plugin_url: string
    runtime_dir: string
    sidebar_width_percent: int
]: nothing -> nothing {
    let source_root = ($layouts_source_dir | path expand)
    # Ensure target directory exists
    mkdir $layouts_target_dir

    let layout_files = (
        ls $source_root
        | where type == file
        | get name
        | where { |file| ($file | path parse | get extension | default "") == "kdl" }
        | each { |file| $file | path basename }
        | sort
    )
    let expected_targets = ($layout_files | each { |file| ($layouts_target_dir | path join $file | path expand) })
    let static_fragments = load_static_fragments $source_root

    let stale_targets = (
        ls ($layouts_target_dir | path expand)
        | where type == file
        | get name
        | where { |file| (($file | path parse | get extension | default "") == "kdl") and ($file not-in $expected_targets) }
    )
    for stale_target in $stale_targets {
        rm --force $stale_target
    }

    # Copy each layout file
    for file in $layout_files {
        let source = ($source_root | path join $file)
        let target = ($layouts_target_dir | path join $file)

        if ($source | path exists) {
            generate_layout $source $target $widget_tray $custom_text $static_fragments $pane_orchestrator_plugin_url $runtime_dir $sidebar_width_percent
            print $"Generated layout: ($target)"
        }
    }
}
