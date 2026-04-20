#!/usr/bin/env nu

use config_parser.nu parse_yazelix_config
use ./constants.nu *
use ./common.nu get_yazelix_user_config_dir

def get_opacity_value [transparency: string] {
    $TRANSPARENCY_VALUES | get -o $transparency | default "1.0"
}

def get_terminal_title [terminal: string] {
    $"Yazelix - (($TERMINAL_METADATA | get -o $terminal | default {} | get -o name | default $terminal))"
}

def get_terminal_override_dir [] {
    (get_yazelix_user_config_dir) | path join "terminal"
}

def get_terminal_override_path [terminal: string] {
    let override_dir = (get_terminal_override_dir)
    match $terminal {
        "ghostty" => ($override_dir | path join "ghostty")
        "kitty" => ($override_dir | path join "kitty.conf")
        "alacritty" => ($override_dir | path join "alacritty.toml")
        _ => null
    }
}

def build_branding [terminal: string, format: string] {
    let title = get_terminal_title $terminal
    match $format {
        "ini" => $"class = ($YAZELIX_WINDOW_CLASS)\nx11-instance-name = ($YAZELIX_X11_INSTANCE)\ntitle = ($title)",
        "toml" => $"class = { instance = \"($YAZELIX_X11_INSTANCE)\", general = \"($YAZELIX_WINDOW_CLASS)\" }\ntitle = \"($title)\"",
        _ => ""
    }
}

def build_transparency [transparency: string, format: string, key: string] {
    let opacity = get_opacity_value $transparency
    if $transparency == "none" {
        match $format {
            "ini" => $"# ($key) = 0.9",
            "ini-space" => $"# ($key) 0.9",
            "lua" => "-- config.window_background_opacity = 0.9",
            "toml" => "# opacity = 0.9",
            _ => ""
        }
    } else {
        match $format {
            "ini" => $"($key) = ($opacity)",
            "ini-space" => $"($key) ($opacity)",
            "lua" => $"config.window_background_opacity = ($opacity)",
            "toml" => $"opacity = ($opacity)",
            _ => ""
        }
    }
}

export def generate_wezterm_config [] {
    let config = parse_yazelix_config
    $"-- WezTerm configuration for Yazelix
local wezterm = require 'wezterm'
local config = wezterm.config_builder\(\)

config.window_decorations = \"NONE\"
config.window_padding = { left = 0, right = 0, top = 10, bottom = 0 }
config.color_scheme = '($YAZELIX_THEME)'

-- Hide tab bar (Zellij handles tabs)
config.enable_tab_bar = false

-- Transparency (configurable via yazelix.toml)
(build_transparency $config.transparency "lua" "")

-- Cursor trails: Not supported in WezTerm

return config"
}

export def generate_kitty_config [] {
    let config = parse_yazelix_config
    let override_path = (get_terminal_override_path "kitty")
    let override_section = if ($override_path | path exists) {
        $"# Personal Yazelix Kitty overrides\ninclude ($override_path)"
    } else {
        $"# Personal Yazelix Kitty overrides (optional, user-owned)\n# Create ($override_path) if you want terminal-native Kitty tweaks."
    }
    $"# Kitty configuration for Yazelix

hide_window_decorations yes
window_padding_width 2
include ($YAZELIX_THEME).conf
window_title (get_terminal_title "kitty")

# Transparency (configurable via yazelix.toml)
(build_transparency $config.transparency "ini-space" "background_opacity")

# Font settings
font_family      ($FONT_FIRACODE)
bold_font        auto
italic_font      auto
bold_italic_font auto

# Performance
repaint_delay 10
input_delay 3
sync_to_monitor yes

# Cursor trail effect (configurable via yazelix.toml)
(build_kitty_cursor $config.ghostty_trail_color)

# Personal Yazelix Kitty overrides
($override_section)"
}

def build_kitty_cursor [ghostty_trail_color] {
    match $ghostty_trail_color {
        "snow" | "random" => "cursor_shape block\ncursor_trail 3\ncursor_trail_decay 0.1 0.4",
        "none" => "# cursor_trail 0  # ghostty_trail_color = none disables the fallback trail",
        null => "# cursor_trail 0",
        _ => "# cursor_trail 0  # Custom effects (blaze/ocean/forest/sunset/neon/cosmic) not supported"
    }
}

export def generate_alacritty_base_config [] {
    let config = parse_yazelix_config
    $"# Alacritty base configuration for Yazelix

[env]
TERM = \"xterm-256color\"

[window]
decorations = \"None\"
padding = { x = 0, y = 10 }
(build_branding "alacritty" "toml")

# Transparency (configurable via yazelix.toml)
(build_transparency $config.transparency "toml" "")

# Cursor trails: Not supported in Alacritty

[font]
normal = { family = \"($FONT_FIRACODE)\", style = \"Regular\" }
bold = { family = \"($FONT_FIRACODE)\", style = \"Bold\" }
italic = { family = \"($FONT_FIRACODE)\", style = \"Italic\" }
bold_italic = { family = \"($FONT_FIRACODE)\", style = \"Bold Italic\" }
builtin_box_drawing = true
size = 12

[colors]
primary = { background = \"#000000\", foreground = \"#ffffff\" }"
}

export def generate_alacritty_config [] {
    let generated_dir = ($YAZELIX_GENERATED_CONFIGS_DIR | str replace "~" $env.HOME)
    let base_path = ($generated_dir | path join "terminal_emulators" "alacritty" "alacritty_base.toml")
    let override_path = (get_terminal_override_path "alacritty")
    let import_list = if ($override_path | path exists) {
        [$base_path, $override_path]
    } else {
        [$base_path]
    }
    let import_rendered = ($import_list | each {|path| $"\"($path)\"" } | str join ", ")
    $"# Alacritty configuration entrypoint for Yazelix

[general]
import = [($import_rendered)]

# Personal Yazelix Alacritty overrides (optional, user-owned)
# Create ($override_path) if you want terminal-native Alacritty tweaks.
"
}

export def generate_foot_config [] {
    let config = parse_yazelix_config
    $"# Foot configuration for Yazelix

[colors-dark]
# Transparency (configurable via yazelix.toml)
(build_transparency $config.transparency "ini" "alpha")

[main]
app-id=($YAZELIX_WINDOW_CLASS)
title=(get_terminal_title "foot")
locked-title=yes
font=($FONT_FIRACODE):size=12
pad=6x6 center

[csd]
preferred=client
size=0
border-width=0

[cursor]
style=block
blink=false"
}
