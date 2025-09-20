-- WezTerm configuration for Yazelix
local wezterm = require 'wezterm'
local config = wezterm.config_builder

-- Basic Yazelix setup
config.default_prog = { 'bash', '-l', '-c', 'nu ~/.config/yazelix/nushell/scripts/core/start_yazelix.nu' }

-- Window styling to match Ghostty
config.window_decorations = "NONE"
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

-- Transparency (configurable via yazelix.nix)
config.window_background_opacity = 0.95

-- Cursor trails: Not supported in WezTerm

return config