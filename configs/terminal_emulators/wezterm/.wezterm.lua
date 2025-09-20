-- Pull in the wezterm API
local wezterm = require 'wezterm'

-- This will hold the configuration.
local config = wezterm.config_builder()

-- Basic settings
config.color_scheme = 'Abernathy'
config.hide_tab_bar_if_only_one_tab = true

-- Desktop branding is handled via --class command line argument in wrapper
-- WezTerm automatically sets window_class and wayland_app_id from --class argument

-- Start Yazelix via login shell to ensure Nix environment is loaded
config.default_prog = { 'bash', '-l', '-c', 'nu ~/.config/yazelix/nushell/scripts/core/start_yazelix.nu' }

-- Alternative: Test without Zellij to debug crash (uncomment to test)
-- config.default_prog = { 'bash', '-c', 'cd ~/.config/yazelix && nix develop --command nu' }

-- Remove title bar but keep resize border (recommended by WezTerm docs)
config.window_decorations = "RESIZE"
-- Ensure clean exit to reduce Wayland resource leaks
config.clean_exit_codes = { 0, 1 }

-- Lets make it more transparent
-- config.window_background_opacity = 0.9

-- Enable debug logging to diagnose crash (commented out as stable)
-- config.debug_key_events = true

-- Return the configuration to wezterm
return config
