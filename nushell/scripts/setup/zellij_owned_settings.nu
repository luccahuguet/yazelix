#!/usr/bin/env nu

def make_setting [name: string, value: string] {
    {name: $name, value: $value}
}

export def resolve_yazelix_owned_zellij_settings [
    config: record
    resolved_default_shell: string
    yazelix_layout_dir: string
    default_layout_name: string
] {
    let zellij_themes = [
        "ansi", "ao", "atelier-sulphurpool", "ayu_mirage", "ayu_dark",
        "catppuccin-frappe", "catppuccin-macchiato", "cyber-noir", "blade-runner",
        "retro-wave", "dracula", "everforest-dark", "gruvbox-dark", "iceberg-dark",
        "kanagawa", "lucario", "menace", "molokai-dark", "night-owl", "nightfox",
        "nord", "one-half-dark", "onedark", "solarized-dark", "tokyo-night-dark",
        "tokyo-night-storm", "tokyo-night", "vesper",
        "ayu_light", "catppuccin-latte", "everforest-light", "gruvbox-light",
        "iceberg-light", "dayfox", "pencil-light", "solarized-light", "tokyo-night-light"
    ]

    let theme_config = ($config | get -o zellij_theme | default "default")
    let theme = if $theme_config == "random" {
        $zellij_themes | shuffle | first
    } else {
        $theme_config
    }

    let pane_frames = ($config | get -o zellij_pane_frames | default "true")
    let pane_frames_value = if ($pane_frames | str starts-with "false") {
        "false"
    } else {
        "true"
    }

    let rounded = ($config | get -o zellij_rounded_corners | default "true")
    let rounded_value = if ($rounded | str starts-with "false") {
        "false"
    } else {
        "true"
    }

    let disable_tips = ($config | get -o disable_zellij_tips | default "true")
    let show_tips_value = if ($disable_tips | str starts-with "false") {
        "true"
    } else {
        "false"
    }

    let persistent_sessions = ($config | get -o persistent_sessions | default "false")
    let on_force_close_value = if ($persistent_sessions | str starts-with "true") {
        "detach"
    } else {
        "quit"
    }

    let kitty_protocol = ($config | get -o support_kitty_keyboard_protocol | default "true")
    let kitty_protocol_value = if ($kitty_protocol | str starts-with "false") {
        "false"
    } else {
        "true"
    }

    let default_mode = ($config.zellij_default_mode? | default "normal")
    let dynamic_top_level_settings = [
        (make_setting "theme" $"\"($theme)\"")
        (make_setting "show_startup_tips" $show_tips_value)
        (make_setting "show_release_notes" "false")
        (make_setting "on_force_close" $"\"($on_force_close_value)\"")
        (make_setting "pane_frames" $pane_frames_value)
    ]
    let enforced_top_level_settings = [
        (make_setting "session_serialization" "true")
        (make_setting "serialize_pane_viewport" "true")
        (make_setting "support_kitty_keyboard_protocol" $kitty_protocol_value)
        (make_setting "default_mode" $"\"($default_mode)\"")
        (make_setting "default_shell" $"\"($resolved_default_shell)\"")
        (make_setting "default_layout" $"\"($yazelix_layout_dir)/($default_layout_name).kdl\"")
        (make_setting "layout_dir" $"\"($yazelix_layout_dir)\"")
    ]

    {
        rounded_value: $rounded_value
        dynamic_top_level_settings: $dynamic_top_level_settings
        enforced_top_level_settings: $enforced_top_level_settings
        owned_top_level_setting_names: (
            $dynamic_top_level_settings
            | append $enforced_top_level_settings
            | get name
        )
    }
}

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
