#!/usr/bin/env nu
# Modular terminal configuration generator for yazelix

use config_parser.nu parse_yazelix_config
use ./constants_with_helpers.nu *
use ./constants.nu [SUPPORTED_TERMINALS, CURSOR_TRAIL_COLOR_HEX]
use ./common.nu [get_yazelix_runtime_dir get_yazelix_user_config_dir]

# Helpers
def get_opacity_value [transparency: string] { $TRANSPARENCY_VALUES | get -o $transparency | default "1.0" }
def get_terminal_title [terminal: string] { $"Yazelix - ($TERMINAL_METADATA | get $terminal | get name)" }
def get_cursor_trail_shader [color: string] { $CURSOR_TRAIL_SHADERS | get -o $color | default $CURSOR_TRAIL_SHADERS.blaze }

def select_random_ghostty_trail_color [] {
    let pool = (get_cursor_trail_random_pool)
    if ($pool | is-empty) {
        null
    } else {
        let max_index = (($pool | length) - 1)
        let index = (random int 0..$max_index)
        $pool | get $index
    }
}

def resolve_ghostty_trail_color [color] {
    if ($color | is-empty) {
        null
    } else {
        match $color {
            "random" => (select_random_ghostty_trail_color),
            _ => $color
        }
    }
}

def get_ghostty_cursor_color_hex [selected_color: string] {
    $CURSOR_TRAIL_COLOR_HEX | get -o $selected_color | default $CURSOR_TRAIL_COLOR_HEX.blaze
}

def resolve_ghostty_trail_effect [effect] {
    if ($effect | is-empty) {
        null
    } else {
        match $effect {
            "random" => (select_random_ghostty_trail_effect),
            _ => $effect
        }
    }
}

def resolve_ghostty_mode_effect [effect] {
    if ($effect | is-empty) {
        null
    } else {
        match $effect {
            "random" => (select_random_ghostty_mode_effect),
            _ => $effect
        }
    }
}

def get_ghostty_cursor_effect_shader_path [effect: string] {
    $"./shaders/generated_effects/($effect).glsl"
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

def build_ghostty_transparency [transparency: string] {
    let opacity_line = (build_transparency $transparency "ini" "background-opacity")
    if $transparency == "none" {
        $opacity_line
    } else {
        $"($opacity_line)
background-opacity-cells = true"
    }
}

def build_ghostty_cursor_palette [selected_color: string] {
    if $selected_color == null {
        "# cursor-color = #ffb929"
    } else if $selected_color == "none" {
        "# Cursor color palette: none \(disable Yazelix color trail palette\)"
    } else {
        let cursor_hex = (get_ghostty_cursor_color_hex $selected_color)
        [
            $"# Cursor color palette: ($selected_color)"
            $"cursor-color = ($cursor_hex)"
            $"custom-shader = (get_cursor_trail_shader $selected_color)"
        ] | str join "\n"
    }
}

def build_ghostty_cursor_effects [trail_effect, mode_effect] {
    let selected_effects = (
        [$trail_effect, $mode_effect]
        | where {|effect| $effect != null and ($effect | str trim | is-not-empty)}
    )
    if ($selected_effects | is-empty) {
        "# custom-shader = ./shaders/generated_effects/tail.glsl"
    } else {
        let header_lines = [
            $"# Cursor effects: (($selected_effects | str join ', '))"
        ]
        let animation_lines = if ($mode_effect != null) and (ghostty_effect_requires_always_animation $mode_effect) {
            ["custom-shader-animation = always"]
        } else {
            []
        }
        let shader_lines = ($selected_effects | each {|effect|
            $"custom-shader = (get_ghostty_cursor_effect_shader_path $effect)"
        })
        ($header_lines | append $animation_lines | append $shader_lines | str join "\n")
    }
}

def build_kitty_cursor [ghostty_trail_color] {
    match $ghostty_trail_color {
        "snow" | "random" => "cursor_shape block\ncursor_trail 3\ncursor_trail_decay 0.1 0.4",
        "none" => "# cursor_trail 0  # ghostty_trail_color = none disables the fallback trail",
        null => "# cursor_trail 0",
        _ => "# cursor_trail 0  # Custom effects \(blaze/ocean/forest/sunset/neon/cosmic\) not supported"
    }
}

# Config generators
export def generate_ghostty_config [] {
    let config = parse_yazelix_config
    let selected_color = (resolve_ghostty_trail_color $config.ghostty_trail_color)
    let selected_trail_effect = (resolve_ghostty_trail_effect $config.ghostty_trail_effect)
    let selected_mode_effect = (resolve_ghostty_mode_effect $config.ghostty_mode_effect)
    let override_path = (get_terminal_override_path "ghostty")
    $"($GHOSTTY_CONFIG_HEADER)

# Yazelix branding for desktop environment recognition
(build_branding "ghostty" "ini")

# Theme and styling
theme = \"($YAZELIX_THEME)\"
window-decoration = \"none\"
window-padding-y = 10,0

# Transparency \(configurable via yazelix.toml\)
(build_ghostty_transparency $config.transparency)

# Ghostty cursor color + effects \(configurable via yazelix.toml\)
(build_ghostty_cursor_palette $selected_color)
(build_ghostty_cursor_effects $selected_trail_effect $selected_mode_effect)

# Personal Yazelix Ghostty overrides \(optional, user-owned\)
config-file = ?\"($override_path)\"
"
}

export def generate_wezterm_config [] {
    let config = parse_yazelix_config
    $"-- WezTerm configuration for Yazelix
local wezterm = require 'wezterm'
local config = wezterm.config_builder\(\)

config.window_decorations = \"NONE\"
config.window_padding = { left = 0, right = 0, top = 10, bottom = 0 }
config.color_scheme = '($YAZELIX_THEME)'

-- Hide tab bar \(Zellij handles tabs\)
config.enable_tab_bar = false

-- Transparency \(configurable via yazelix.toml\)
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
        $"# Personal Yazelix Kitty overrides \(optional, user-owned\)\n# Create ($override_path) if you want terminal-native Kitty tweaks."
    }
    $"# Kitty configuration for Yazelix

hide_window_decorations yes
window_padding_width 2
include ($YAZELIX_THEME).conf
window_title (get_terminal_title "kitty")

# Transparency \(configurable via yazelix.toml\)
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

# Cursor trail effect \(configurable via yazelix.toml\)
(build_kitty_cursor $config.ghostty_trail_color)

# Personal Yazelix Kitty overrides
($override_section)"
}

def generate_alacritty_base_config [] {
    let config = parse_yazelix_config
    $"# Alacritty base configuration for Yazelix

[env]
TERM = \"xterm-256color\"

[window]
decorations = \"None\"
padding = { x = 0, y = 10 }
(build_branding "alacritty" "toml")

# Transparency \(configurable via yazelix.toml\)
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

# Personal Yazelix Alacritty overrides \(optional, user-owned\)
# Create ($override_path) if you want terminal-native Alacritty tweaks.
"
}

export def generate_foot_config [] {
    let config = parse_yazelix_config
    $"# Foot configuration for Yazelix

[colors-dark]
# Transparency \(configurable via yazelix.toml)
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

# Config management
def save_config_with_backup [file_path: string, content: string] {
    if ($file_path | path exists) {
        print $"Backing up existing config: ($file_path) → ($file_path).yazelix-backup"
        cp $file_path $"($file_path).yazelix-backup"
    }
    $content | save $file_path --force
}

export def generate_all_terminal_configs [runtime_dir?: string] {
    let config = parse_yazelix_config
    let resolved_runtime_dir = (($runtime_dir | default (get_yazelix_runtime_dir)) | path expand)
    let manage_terminals = ($config.manage_terminals? | default true)
    mut terminals = ($config.terminals? | default ["ghostty"])
    if ($terminals | is-empty) {
        if $manage_terminals {
            error make {msg: "terminal.terminals must include at least one terminal"}
        } else {
            $terminals = $SUPPORTED_TERMINALS
        }
    }
    let should_generate_ghostty = ($terminals | any {|t| $t == "ghostty" })
    let should_generate_foot = ($terminals | any {|t| $t == "foot" })
    let should_generate_wezterm = ($terminals | any {|t| $t == "wezterm" })
    let should_generate_kitty = ($terminals | any {|t| $t == "kitty" })
    let generated_dir = ($YAZELIX_GENERATED_CONFIGS_DIR | str replace "~" $env.HOME)
    let configs_dir = ($generated_dir | path join "terminal_emulators")

    print "Generating bundled terminal configurations..."

    # Ghostty (optional)
    if $should_generate_ghostty {
        let ghostty_dir = ($configs_dir | path join "ghostty")
        mkdir $ghostty_dir
        save_config_with_backup ($ghostty_dir | path join "config") (generate_ghostty_config)

        let shaders_src = ($resolved_runtime_dir | path join "configs" "terminal_emulators" "ghostty" "shaders")
        let shaders_dest = ($ghostty_dir | path join "shaders")
        if ($shaders_dest | path exists) { rm --permanent --recursive $shaders_dest }
        mkdir $shaders_dest
        if ($shaders_src | path exists) {
            ls $shaders_src | get name | each {|file| cp -r $file $shaders_dest }
        }

        # Build cursor shader variants inside the generated config tree
        let build_script = ($shaders_src | path join "build_shaders.nu")
        if ($build_script | path exists) {
            let glow_level = ($config.ghostty_trail_glow? | default "medium")
            nu -c $"use '($build_script)' [build_cursor_trail_shaders build_ghostty_cursor_effect_shaders]; build_cursor_trail_shaders '($shaders_dest)' '($glow_level)'; build_ghostty_cursor_effect_shaders '($shaders_dest)' '($glow_level)'"
        }
    }

    # Alacritty (conditional)
    if ($terminals | any {|t| $t == "alacritty" }) {
        let alacritty_dir = ($configs_dir | path join "alacritty")
        mkdir $alacritty_dir
        save_config_with_backup ($alacritty_dir | path join "alacritty_base.toml") (generate_alacritty_base_config)
        save_config_with_backup ($alacritty_dir | path join "alacritty.toml") (generate_alacritty_config)
    }

    mut generated = []
    if $should_generate_ghostty { $generated = ($generated | append "Ghostty") }
    if ($terminals | any {|t| $t == "alacritty" }) { $generated = ($generated | append "Alacritty") }

    # WezTerm (conditional)
    if $should_generate_wezterm {
        let wezterm_dir = ($configs_dir | path join "wezterm")
        mkdir $wezterm_dir
        save_config_with_backup ($wezterm_dir | path join ".wezterm.lua") (generate_wezterm_config)
        $generated = ($generated | append "WezTerm")
    }

    # Kitty (conditional)
    if $should_generate_kitty {
        let kitty_dir = ($configs_dir | path join "kitty")
        mkdir $kitty_dir
        save_config_with_backup ($kitty_dir | path join "kitty.conf") (generate_kitty_config)
        $generated = ($generated | append "Kitty")
    }

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
