#!/usr/bin/env nu

export def build_yazelix_ui_block [existing_ui_lines: list<string>, rounded_value: string] {
    let existing_ui_text = ($existing_ui_lines | str join "\n")
    let hide_session_name = ($existing_ui_text | str contains "hide_session_name true")

    [
        "ui {"
        "    pane_frames {"
        $"        rounded_corners ($rounded_value)"
        ...(if $hide_session_name { ["        hide_session_name true"] } else { [] })
        "    }"
        "}"
    ] | str join "\n"
}

export def render_yazelix_top_level_settings_block [header: string, settings: list<record>] {
    [
        $header
        ...($settings | each {|setting| $"($setting.name) ($setting.value)" })
    ] | str join "\n"
}

export def strip_yazelix_owned_top_level_settings [config_content: string, owned_setting_names: list<string>] {
    (
        $config_content
        | lines
        | where {|line|
            let trimmed = ($line | str trim)
            not ($owned_setting_names | any {|name| $trimmed | str starts-with $"($name) " })
        }
        | str join "\n"
    )
}
