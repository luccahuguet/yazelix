#!/usr/bin/env nu
# Modular terminal configuration generator for yazelix

use config_parser.nu parse_yazelix_config

# Generate Ghostty configuration
export def generate_ghostty_config [] {
    let config = parse_yazelix_config
    let cursor_trail = $config.cursor_trail

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

# Transparency consistent with WezTerm
# background-opacity = 0.9

# Cursor trail effect (configurable via yazelix.nix)"

    # Add cursor trail configuration based on setting
    let cursor_config = match $cursor_trail {
        "blaze" => "custom-shader = ./shaders/cursor_smear.glsl",
        "white" => "custom-shader = ./shaders/cursor_trail_white.glsl",
        "none" => "# custom-shader = ./shaders/cursor_smear.glsl",
        _ => "custom-shader = ./shaders/cursor_smear.glsl" # Default to blaze
    }

    $base_config + "\n" + $cursor_config + "\n"
}

# Generate WezTerm configuration
export def generate_wezterm_config [] {
    let config = parse_yazelix_config

    "-- WezTerm configuration for Yazelix
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

-- Transparency (commented out by default)
-- config.window_background_opacity = 0.9

return config"
}

# Generate Kitty configuration
export def generate_kitty_config [] {
    let config = parse_yazelix_config

    "# Kitty configuration for Yazelix

# Basic Yazelix setup
shell bash -l -c \"nu ~/.config/yazelix/nushell/scripts/core/start_yazelix.nu\"

# Window styling to match other terminals
hide_window_decorations yes
window_padding_width 0 10

# Theme
include Abernathy.conf

# Window class for desktop integration
linux_display_server x11
x11_hide_window_decorations yes

# Transparency (commented out by default)
# background_opacity 0.9

# Font settings
font_family      JetBrains Mono
bold_font        auto
italic_font      auto
bold_italic_font auto

# Performance
repaint_delay 10
input_delay 3
sync_to_monitor yes"
}

# Generate Alacritty configuration
export def generate_alacritty_config [] {
    let config = parse_yazelix_config

    "# Alacritty configuration for Yazelix

[general]
import = []

[env]
TERM = \"xterm-256color\"

[shell]
program = \"bash\"
args = [\"-l\", \"-c\", \"nu ~/.config/yazelix/nushell/scripts/core/start_yazelix.nu\"]

[window]
decorations = \"None\"
padding = { x = 0, y = 10 }
class = { instance = \"yazelix\", general = \"com.yazelix.Yazelix\" }

# Transparency (commented out by default)
# opacity = 0.9

[font]
normal = { family = \"JetBrains Mono\", style = \"Regular\" }
bold = { family = \"JetBrains Mono\", style = \"Bold\" }
italic = { family = \"JetBrains Mono\", style = \"Italic\" }
bold_italic = { family = \"JetBrains Mono\", style = \"Bold Italic\" }
size = 12

[colors]
# Abernathy theme colors would go here
primary = { background = \"#000000\", foreground = \"#ffffff\" }"
}

# Write all terminal configurations
export def generate_all_terminal_configs [] {
    let yazelix_dir = "~/.config/yazelix" | path expand
    let configs_dir = ($yazelix_dir | path join "configs" "terminal_emulators")

    print "Generating terminal configurations..."

    # Generate Ghostty config
    let ghostty_dir = ($configs_dir | path join "ghostty")
    mkdir $ghostty_dir
    generate_ghostty_config | save ($ghostty_dir | path join "config") --force

    # Generate WezTerm config
    let wezterm_dir = ($configs_dir | path join "wezterm")
    mkdir $wezterm_dir
    generate_wezterm_config | save ($wezterm_dir | path join ".wezterm.lua") --force

    # Generate Kitty config
    let kitty_dir = ($configs_dir | path join "kitty")
    mkdir $kitty_dir
    generate_kitty_config | save ($kitty_dir | path join "kitty.conf") --force

    # Generate Alacritty config
    let alacritty_dir = ($configs_dir | path join "alacritty")
    mkdir $alacritty_dir
    generate_alacritty_config | save ($alacritty_dir | path join "alacritty.toml") --force

    print "✓ Generated all terminal configurations"
}