#!/usr/bin/env nu
# Modular terminal configuration generator for yazelix

use config_parser.nu parse_yazelix_config
use ./constants.nu *

# Get transparency value as opacity
def get_opacity_value [transparency: string] {
    match $transparency {
        "none" => "1.0",
        "low" => "0.95",
        "medium" => "0.9",
        "high" => "0.8",
        _ => "1.0" # Default to no transparency
    }
}

# Generate Ghostty configuration
export def generate_ghostty_config [] {
    let config = parse_yazelix_config
    let cursor_trail = $config.cursor_trail
    let transparency = $config.transparency

    # Base configuration template
    let base_config = "# This is the configuration file for Ghostty.
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
initial-command = \"bash -l -c 'nu ~/.config/yazelix/nushell/scripts/core/start_yazelix.nu'\"

# Yazelix branding for desktop environment recognition
class = com.yazelix.Yazelix
x11-instance-name = yazelix

# Theme and styling consistent with WezTerm config
theme = \"Abernathy\"
window-decoration = \"none\"
window-padding-y = 10,0

# Transparency \(configurable via yazelix.nix\)"

    # Add transparency configuration based on setting
    let opacity_value = (get_opacity_value $transparency)
    let transparency_config = if $transparency == "none" {
        "# background-opacity = 0.9"
    } else {
        $"background-opacity = ($opacity_value)"
    }

    # Add cursor trail configuration based on setting
    let cursor_config = match $cursor_trail {
        "blaze" => "custom-shader = ./shaders/cursor_smear.glsl",
        "snow" => "custom-shader = ./shaders/cursor_trail_white.glsl",
        "cosmic" => "custom-shader = ./shaders/cursor_trail_cosmic.glsl",
        "ocean" => "custom-shader = ./shaders/cursor_trail_ocean.glsl",
        "forest" => "custom-shader = ./shaders/cursor_trail_forest.glsl",
        "sunset" => "custom-shader = ./shaders/cursor_trail_sunset.glsl",
        "neon" => "custom-shader = ./shaders/cursor_trail_neon.glsl",
        "party" => "custom-shader = ./shaders/cursor_trail_party.glsl",
        "none" => "# custom-shader = ./shaders/cursor_smear.glsl",
        _ => "custom-shader = ./shaders/cursor_smear.glsl" # Default to blaze
    }

    [
        $base_config,
        "\n",
        $transparency_config,
        "\n\n# Cursor trail effect (configurable via yazelix.nix)\n",
        $cursor_config,
        "\n",
        "# Alternative presets (uncomment to try)\n",
        "# snow:  custom-shader = ./shaders/cursor_trail_white.glsl\n",
        "# blaze (fire):  custom-shader = ./shaders/cursor_smear.glsl\n",
        "# cosmic (violet): custom-shader = ./shaders/cursor_trail_cosmic.glsl\n",
        "# ocean (blue):  custom-shader = ./shaders/cursor_trail_ocean.glsl\n",
        "# forest (green): custom-shader = ./shaders/cursor_trail_forest.glsl\n",
        "# sunset (orange/pink): custom-shader = ./shaders/cursor_trail_sunset.glsl\n",
        "# neon (cyan/magenta): custom-shader = ./shaders/cursor_trail_neon.glsl\n",
        "# party (multi-hue): custom-shader = ./shaders/cursor_trail_party.glsl\n",
    ] | str join ""
}

# Generate WezTerm configuration
export def generate_wezterm_config [] {
    let config = parse_yazelix_config
    let transparency = $config.transparency

    let transparency_config = if $transparency == "none" {
        "-- config.window_background_opacity = 0.9"
    } else {
        let opacity_value = (get_opacity_value $transparency)
        $"config.window_background_opacity = ($opacity_value)"
    }

    $"-- WezTerm configuration for Yazelix
local wezterm = require 'wezterm'
local config = wezterm.config_builder()

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
config.color_scheme = 'Abernathy'

-- Window class for desktop integration
config.window_class = 'com.yazelix.Yazelix'

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

    let transparency_config = if $transparency == "none" {
        "# background_opacity 0.9"
    } else {
        let opacity_value = (get_opacity_value $transparency)
        $"background_opacity ($opacity_value)"
    }

    # Kitty cursor trail support (built-in animation)
    let cursor_config = match $cursor_trail {
        # Kitty supports a built-in white trail; map snow to it
        "snow" => "cursor_shape block\ncursor_trail 3\ncursor_trail_decay 0.1 0.4",
        "none" => "# cursor_trail 0",
        _ => "# cursor_trail 0  # Custom effects (blaze/ocean/forest/sunset/neon/cosmic) not supported in Kitty"
    }

    $"# Kitty configuration for Yazelix

# Basic Yazelix setup
shell bash -l -c \"nu ~/.config/yazelix/nushell/scripts/core/start_yazelix.nu\"

# Window styling to match other terminals
hide_window_decorations yes
window_padding_width 2

# Theme
include Abernathy.conf

# Window class for desktop integration
linux_display_server x11
x11_hide_window_decorations yes

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

    let transparency_config = if $transparency == "none" {
        "# opacity = 0.9"
    } else {
        let opacity_value = (get_opacity_value $transparency)
        $"opacity = ($opacity_value)"
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
class = { instance = \"yazelix\", general = \"com.yazelix.Yazelix\" }

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
    # Write generated configs to XDG state dir, not the user's terminal config
    let generated_dir = ($YAZELIX_GENERATED_CONFIGS_DIR | str replace "~" $env.HOME)
    let configs_dir = ($generated_dir | path join "terminal_emulators")

    print "Generating bundled terminal configurations..."

    # Generate Ghostty config (always bundled by default)
    let ghostty_dir = ($configs_dir | path join "ghostty")
    mkdir $ghostty_dir
    let ghostty_config = ($ghostty_dir | path join "config")
    save_config_with_backup $ghostty_config (generate_ghostty_config)

    # Generate Alacritty config (used by wrappers and system installs)
    let alacritty_dir = ($configs_dir | path join "alacritty")
    mkdir $alacritty_dir
    let alacritty_config = ($alacritty_dir | path join "alacritty.toml")
    save_config_with_backup $alacritty_config (generate_alacritty_config)

    print "✓ Generated terminal configurations (Ghostty, Alacritty)"
    print "📋 Static example configs for other terminals in configs/terminal_emulators/"
}
