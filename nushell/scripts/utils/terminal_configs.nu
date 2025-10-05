#!/usr/bin/env nu
# Modular terminal configuration generator for yazelix

use config_parser.nu parse_yazelix_config
use ./constants.nu *

# Helpers
def get_opacity_value [transparency: string] { $TRANSPARENCY_VALUES | get -o $transparency | default "1.0" }
def get_terminal_title [terminal: string] { $"Yazelix - ($TERMINAL_METADATA | get $terminal | get name)" }
def get_cursor_trail_shader [cursor_trail: string] { $CURSOR_TRAIL_SHADERS | get -o $cursor_trail | default $CURSOR_TRAIL_SHADERS.blaze }

# Section builders
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
        match $format { "ini" => $"# ($key) = 0.9", "lua" => "-- config.window_background_opacity = 0.9", "toml" => "# opacity = 0.9", _ => "" }
    } else {
        match $format { "ini" => $"($key) = ($opacity)", "lua" => $"config.window_background_opacity = ($opacity)", "toml" => $"opacity = ($opacity)", _ => "" }
    }
}

def build_cursor_trail [cursor_trail: string] {
    let shader = get_cursor_trail_shader $cursor_trail
    if $cursor_trail == "none" { "# custom-shader = ./shaders/cursor_smear.glsl" } else { $"custom-shader = ($shader)" }
}

def build_kitty_cursor [cursor_trail: string] {
    match $cursor_trail {
        "snow" => "cursor_shape block\ncursor_trail 3\ncursor_trail_decay 0.1 0.4",
        "none" => "# cursor_trail 0",
        _ => "# cursor_trail 0  # Custom effects \(blaze/ocean/forest/sunset/neon/cosmic\) not supported"
    }
}

# Config generators
export def generate_ghostty_config [] {
    let config = parse_yazelix_config
    $"($GHOSTTY_CONFIG_HEADER)

# Start Yazelix via login shell to ensure Nix environment is loaded
initial-command = \"($YAZELIX_SHELL_COMMAND)\"

# Yazelix branding for desktop environment recognition
(build_branding "ghostty" "ini")

# Theme and styling
theme = \"($YAZELIX_THEME)\"
window-decoration = \"none\"
window-padding-y = 10,0

# Transparency \(configurable via yazelix.nix\)
(build_transparency $config.transparency "ini" "background-opacity")

# Cursor trail effect \(configurable via yazelix.nix\)
(build_cursor_trail $config.cursor_trail)

($CURSOR_TRAIL_PRESETS_COMMENT)
"
}

export def generate_wezterm_config [] {
    let config = parse_yazelix_config
    $"-- WezTerm configuration for Yazelix
local wezterm = require 'wezterm'
local config = wezterm.config_builder\(\)

config.default_prog = ($SHELL_ARGS_BASH)
config.window_decorations = \"NONE\"
config.window_padding = { left = 0, right = 0, top = 10, bottom = 0 }
config.color_scheme = '($YAZELIX_THEME)'
config.window_class = '($YAZELIX_WINDOW_CLASS)'
config.window_title = '(get_terminal_title "wezterm")'

-- Transparency \(configurable via yazelix.nix\)
(build_transparency $config.transparency "lua" "")

-- Cursor trails: Not supported in WezTerm

return config"
}

export def generate_kitty_config [] {
    let config = parse_yazelix_config
    $"# Kitty configuration for Yazelix

shell ($YAZELIX_SHELL_COMMAND)
hide_window_decorations yes
window_padding_width 2
include ($YAZELIX_THEME).conf
linux_display_server x11
x11_hide_window_decorations yes
window_title (get_terminal_title "kitty")

# Transparency \(configurable via yazelix.nix\)
(build_transparency $config.transparency "ini" "background_opacity")

# Font settings
font_family      ($FONT_FIRACODE)
bold_font        auto
italic_font      auto
bold_italic_font auto

# Performance
repaint_delay 10
input_delay 3
sync_to_monitor yes

# Cursor trail effect \(configurable via yazelix.nix\)
(build_kitty_cursor $config.cursor_trail)"
}

export def generate_alacritty_config [] {
    let config = parse_yazelix_config
    $"# Alacritty configuration for Yazelix

[general]
import = []

[env]
TERM = \"xterm-256color\"

[terminal.shell]
program = \"bash\"
args = ($SHELL_ARGS_STRING)

[window]
decorations = \"None\"
padding = { x = 0, y = 10 }
(build_branding "alacritty" "toml")

# Transparency \(configurable via yazelix.nix\)
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

export def generate_foot_config [] {
    let config = parse_yazelix_config
    $"# Foot configuration for Yazelix
shell=($YAZELIX_SHELL_COMMAND)

[colors]
# Transparency \(configurable via yazelix.nix)
(build_transparency $config.transparency "ini" "alpha")

[main]
app-id=($YAZELIX_WINDOW_CLASS)
title=(get_terminal_title "foot")
font=($FONT_FIRACODE):size=12

[cursor]
style=block
blink=false"
}

# Config management
def save_config_with_backup [file_path: string, content: string] {
    if ($file_path | path exists) {
        print $"Backing up existing config: ($file_path) → ($file_path).yazelix-backup"
        cp $file_path $"($file_path).yazelix-backup"
    }
    $content | save $file_path --force
}

export def generate_all_terminal_configs [] {
    let config = parse_yazelix_config
    let extra_terminals = ($config.extra_terminals | str replace -a '["\] ' '' | split row ' ' | where {|t| ($t | str length) > 0 })
    let should_generate_foot = ($config.preferred_terminal == "foot") or ($extra_terminals | any {|t| $t == "foot" })
    let generated_dir = ($YAZELIX_GENERATED_CONFIGS_DIR | str replace "~" $env.HOME)
    let configs_dir = ($generated_dir | path join "terminal_emulators")

    print "Generating bundled terminal configurations..."

    # Ghostty (always bundled)
    let ghostty_dir = ($configs_dir | path join "ghostty")
    mkdir $ghostty_dir
    save_config_with_backup ($ghostty_dir | path join "config") (generate_ghostty_config)
    let shaders_src = $"($env.HOME)/.config/yazelix/configs/terminal_emulators/ghostty/shaders"
    if ($shaders_src | path exists) { cp -r $shaders_src ($ghostty_dir | path join "shaders") }

    # Alacritty (always bundled)
    let alacritty_dir = ($configs_dir | path join "alacritty")
    mkdir $alacritty_dir
    save_config_with_backup ($alacritty_dir | path join "alacritty.toml") (generate_alacritty_config)

    mut generated = ["Ghostty", "Alacritty"]

    # Foot (conditional)
    if $should_generate_foot {
        let foot_dir = ($configs_dir | path join "foot")
        mkdir $foot_dir
        save_config_with_backup ($foot_dir | path join "foot.ini") (generate_foot_config)
        $generated = ($generated | append "Foot")
    }

    let generated_list = ($generated | str join ", ")
    print $"✓ Generated terminal configurations ($generated_list)"
    print "📋 Static example configs for other terminals in configs/terminal_emulators/"
}
