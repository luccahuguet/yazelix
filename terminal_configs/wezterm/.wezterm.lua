-- Pull in the wezterm API
local wezterm = require 'wezterm'

-- This will hold the configuration.
local config = wezterm.config_builder()

-- Basic settings
config.color_scheme = 'Abernathy'
config.hide_tab_bar_if_only_one_tab = true

-- Start Yazelix via script
config.default_prog = { 'bash', '-c', '~/.config/yazelix/bash/start-yazelix.sh' }

-- Alternative: Test without Zellij to debug crash (uncomment to test)
-- config.default_prog = { 'bash', '-c', 'cd ~/.config/yazelix && nix develop --command nu' }

-- Use server-side decorations to avoid Wayland decoration manager issues
config.window_decorations = "NONE"
-- Ensure clean exit to reduce Wayland resource leaks
config.clean_exit_codes = { 0, 1 }

-- Lets make it more transparent
config.window_background_opacity = 0.9

-- Enable debug logging to diagnose crash (commented out as stable)
-- config.debug_key_events = true

-- Return the configuration to wezterm
return config
