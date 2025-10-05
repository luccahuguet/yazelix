#!/usr/bin/env nu
# Modular terminal configuration generator for yazelix

use config_parser.nu parse_yazelix_config
use ./constants.nu *

# Helper: Get opacity value for transparency setting
def get_opacity_value [transparency: string] {
    $TRANSPARENCY_VALUES | get -o $transparency | default "1.0"
}

# Helper: Get terminal title
def get_terminal_title [terminal: string] {
    let name = $TERMINAL_METADATA | get $terminal | get name
    $"Yazelix - ($name)"
}

# Helper: Get cursor trail shader path
def get_cursor_trail_shader [cursor_trail: string] {
    $CURSOR_TRAIL_SHADERS | get -o $cursor_trail | default $CURSOR_TRAIL_SHADERS.blaze
}

# Generate Ghostty configuration
export def generate_ghostty_config [] {
    let config = parse_yazelix_config
    let cursor_trail = $config.cursor_trail
    let transparency = $config.transparency
    let opacity = get_opacity_value $transparency
    let title = get_terminal_title "ghostty"

    # Transparency config
    let transparency_config = if $transparency == "none" {
        "# background-opacity = 0.9"
    } else {
        $"background-opacity = ($opacity)"
    }

    # Cursor trail config
    let shader_path = get_cursor_trail_shader $cursor_trail
    let cursor_config = if $cursor_trail == "none" {
        "# custom-shader = ./shaders/cursor_smear.glsl"
    } else {
        $"custom-shader = ($shader_path)"
    }

    $"# This is the configuration file for Ghostty.
#
# This template file has been automatically created at the following
# path since Ghostty couldn't find any existing config files on your system:
#
#   /home/lucca/.config/ghostty/config
#
# The template does not set any default options, since Ghostty ships
# with sensible defaults for all options. Users should only need to set
# options that they want to change from the default.
#
# Run `ghostty +show-config --default --docs` to view a list of
# all available config options and their default values.
#
# Additionally, each config option is also explained in detail
# on Ghostty's website, at https://ghostty.org/docs/config.

# Config syntax crash course
# ==========================
# # The config file consists of simple key-value pairs,
# # separated by equals signs.
# font-family = Iosevka
# window-padding-x = 2
#
# # Spacing around the equals sign does not matter.
# # All of these are identical:
# key=value
# key= value
# key =value
# key = value
#
# # Any line beginning with a # is a comment. It's not possible to put
# # a comment after a config option, since it would be interpreted as a
# # part of the value. For example, this will have a value of \"#123abc\":
# background = #123abc
#
# # Empty values are used to reset config keys to default.
# key =
#
# # Some config options have unique syntaxes for their value,
# # which is explained in the docs for that config option.
# # Just for example:
# resize-overlay-duration = 4s 200ms

# Start Yazelix via login shell to ensure Nix environment is loaded
initial-command = \"($YAZELIX_SHELL_COMMAND)\"

# Yazelix branding for desktop environment recognition
class = ($YAZELIX_WINDOW_CLASS)
x11-instance-name = ($YAZELIX_X11_INSTANCE)
title = ($title)

# Theme and styling consistent with WezTerm config
theme = \"($YAZELIX_THEME)\"
window-decoration = \"none\"
window-padding-y = 10,0

# Transparency \(configurable via yazelix.nix\)
($transparency_config)

# Cursor trail effect \(configurable via yazelix.nix\)
($cursor_config)

# Alternative presets \(uncomment to try\)
# snow:  custom-shader = ./shaders/cursor_trail_white.glsl
# blaze \(fire\):  custom-shader = ./shaders/cursor_smear.glsl
# cosmic \(violet\): custom-shader = ./shaders/cursor_trail_cosmic.glsl
# ocean \(blue\):  custom-shader = ./shaders/cursor_trail_ocean.glsl
# forest \(green\): custom-shader = ./shaders/cursor_trail_forest.glsl
# sunset \(orange/pink\): custom-shader = ./shaders/cursor_trail_sunset.glsl
# neon \(cyan/magenta\): custom-shader = ./shaders/cursor_trail_neon.glsl
# party \(multi-hue\): custom-shader = ./shaders/cursor_trail_party.glsl
"
}

# Generate WezTerm configuration
export def generate_wezterm_config [] {
    let config = parse_yazelix_config
    let transparency = $config.transparency
    let opacity = get_opacity_value $transparency
    let title = get_terminal_title "wezterm"

    let transparency_config = if $transparency == "none" {
        "-- config.window_background_opacity = 0.9"
    } else {
        $"config.window_background_opacity = ($opacity)"
    }

    $"-- WezTerm configuration for Yazelix
local wezterm = require 'wezterm'
local config = wezterm.config_builder\(\)

-- Basic Yazelix setup
config.default_prog = { 'bash', '-l', '-c', 'nu ~/.config/yazelix/nushell/scripts/core/start_yazelix.nu' }

-- Window styling to match Ghostty
config.window_decorations = \"NONE\"
config.window_padding = {
  left = 0,
  right = 0,
  top = 10,
  bottom = 0,
}

-- Theme
config.color_scheme = '($YAZELIX_THEME)'

-- Window class for desktop integration
config.window_class = '($YAZELIX_WINDOW_CLASS)'

-- Window title
config.window_title = '($title)'

-- Transparency \(configurable via yazelix.nix\)
($transparency_config)

-- Cursor trails: Not supported in WezTerm

return config"
}

# Generate Kitty configuration
export def generate_kitty_config [] {
    let config = parse_yazelix_config
    let transparency = $config.transparency
    let cursor_trail = $config.cursor_trail
    let opacity = get_opacity_value $transparency
    let title = get_terminal_title "kitty"

    let transparency_config = if $transparency == "none" {
        "# background_opacity 0.9"
    } else {
        $"background_opacity ($opacity)"
    }

    # Kitty cursor trail support (built-in animation)
    let cursor_config = match $cursor_trail {
        # Kitty supports a built-in white trail; map snow to it
        "snow" => "cursor_shape block\ncursor_trail 3\ncursor_trail_decay 0.1 0.4",
        "none" => "# cursor_trail 0",
        _ => "# cursor_trail 0  # Custom effects \(blaze/ocean/forest/sunset/neon/cosmic\) not supported in Kitty"
    }

    $"# Kitty configuration for Yazelix

# Basic Yazelix setup
shell ($YAZELIX_SHELL_COMMAND)

# Window styling to match other terminals
hide_window_decorations yes
window_padding_width 2

# Theme
include ($YAZELIX_THEME).conf

# Window class for desktop integration
linux_display_server x11
x11_hide_window_decorations yes

# Window title
window_title ($title)

# Transparency \(configurable via yazelix.nix\)
($transparency_config)

# Font settings
font_family      FiraCode Nerd Font
bold_font        auto
italic_font      auto
bold_italic_font auto

# Performance
repaint_delay 10
input_delay 3
sync_to_monitor yes

# Cursor trail effect \(configurable via yazelix.nix\)
($cursor_config)"
}

# Generate Alacritty configuration
export def generate_alacritty_config [] {
    let config = parse_yazelix_config
    let transparency = $config.transparency
    let opacity = get_opacity_value $transparency
    let title = get_terminal_title "alacritty"

    let transparency_config = if $transparency == "none" {
        "# opacity = 0.9"
    } else {
        $"opacity = ($opacity)"
    }

    $"# Alacritty configuration for Yazelix

[general]
import = []

[env]
TERM = \"xterm-256color\"

[terminal.shell]
program = \"bash\"
args = [\"-l\", \"-c\", \"nu ~/.config/yazelix/nushell/scripts/core/start_yazelix.nu\"]

[window]
decorations = \"None\"
padding = { x = 0, y = 10 }
class = { instance = \"($YAZELIX_X11_INSTANCE)\", general = \"($YAZELIX_WINDOW_CLASS)\" }
title = \"($title)\"

# Transparency \(configurable via yazelix.nix\)
($transparency_config)

# Cursor trails: Not supported in Alacritty

[font]
normal = { family = \"FiraCode Nerd Font\", style = \"Regular\" }
bold = { family = \"FiraCode Nerd Font\", style = \"Bold\" }
italic = { family = \"FiraCode Nerd Font\", style = \"Italic\" }
bold_italic = { family = \"FiraCode Nerd Font\", style = \"Bold Italic\" }
builtin_box_drawing = true
size = 12

[colors]
# Abernathy theme colors would go here
primary = { background = \"#000000\", foreground = \"#ffffff\" }"
}

# Generate Foot configuration
export def generate_foot_config [] {
    let config = parse_yazelix_config
    let transparency = $config.transparency
    let opacity = get_opacity_value $transparency
    let title = get_terminal_title "foot"

    let transparency_config = if $transparency == "none" {
        "# alpha=0.9"
    } else {
        $"alpha=($opacity)"
    }

    $"# Foot configuration for Yazelix
shell=($YAZELIX_SHELL_COMMAND)

[colors]
# Transparency \(configurable via yazelix.nix)
($transparency_config)

[main]
# Window class
app-id=($YAZELIX_WINDOW_CLASS)
# Window title
title=($title)
# Font configuration
font=FiraCode Nerd Font:size=12

# Foot does not support cursor trails
[cursor]
style=block
blink=false"
}

# Safely save config with backup
def save_config_with_backup [file_path: string, content: string] {
    if ($file_path | path exists) {
        let backup_path = ($file_path + ".yazelix-backup")
        print $"Backing up existing config: ($file_path) → ($backup_path)"
        cp $file_path $backup_path
    }
    $content | save $file_path --force
}

# Write terminal configurations (bundled terminals only)
export def generate_all_terminal_configs [] {
    let config = parse_yazelix_config

    # Helper to parse extra terminals from string representation like "[ wezterm kitty ]"
    let extra_terminals = (
        $config.extra_terminals
        | str replace '[' ''
        | str replace ']' ''
        | str replace '"' ''
        | str trim
        | split row ' '
        | where {|t| ($t | str length) > 0 }
    )

    let should_generate_foot = (
        ($config.preferred_terminal == "foot") or ($extra_terminals | any {|t| $t == "foot" })
    )

    # Write generated configs to XDG state dir, not the user's terminal config
    let generated_dir = ($YAZELIX_GENERATED_CONFIGS_DIR | str replace "~" $env.HOME)
    let configs_dir = ($generated_dir | path join "terminal_emulators")

    print "Generating bundled terminal configurations..."

    # Generate Ghostty config (always bundled by default)
    let ghostty_dir = ($configs_dir | path join "ghostty")
    mkdir $ghostty_dir
    let ghostty_config = ($ghostty_dir | path join "config")
    save_config_with_backup $ghostty_config (generate_ghostty_config)

    # Copy shader files to bundled config directory so relative paths work
    let shaders_src = $"($env.HOME)/.config/yazelix/configs/terminal_emulators/ghostty/shaders"
    let shaders_dest = ($ghostty_dir | path join "shaders")
    if ($shaders_src | path exists) {
        cp -r $shaders_src $shaders_dest
    }

    # Generate Alacritty config (used by wrappers and system installs)
    let alacritty_dir = ($configs_dir | path join "alacritty")
    mkdir $alacritty_dir
    let alacritty_config = ($alacritty_dir | path join "alacritty.toml")
    save_config_with_backup $alacritty_config (generate_alacritty_config)

    mut generated = ["Ghostty", "Alacritty"]

    if $should_generate_foot {
        let foot_dir = ($configs_dir | path join "foot")
        mkdir $foot_dir
        let foot_config = ($foot_dir | path join "foot.ini")
        save_config_with_backup $foot_config (generate_foot_config)
        $generated = ($generated | append "Foot")
    }

    let generated_list = ($generated | str join ", ")
    print $"✓ Generated terminal configurations ($generated_list)"
    print "📋 Static example configs for other terminals in configs/terminal_emulators/"
}
