-- Pull in the wezterm API
local wezterm = require 'wezterm'

-- This will hold the configuration.
local config = wezterm.config_builder()

-- This is where you actually apply your config choices
-- For example, changing the color scheme:
config.color_scheme = 'Abernathy'

-- Run Zellij directly with the yazelix session
config.default_prog = { 'zellij', '--config-dir', '~/.config/yazelix/zellij', 'attach', '--create', 'yazelix', 'options', '--default-layout', 'yazelix' }

-- Alternative: Use the welcome layout directly
-- config.default_prog = { 'zellij', '-l', 'welcome', '--config-dir', '~/.config/yazelix/zellij', 'options', '--layout-dir', '~/.config/yazelix/zellij/layouts' }

-- Fallback: Yazelix session with nu -c (if direct fails)
-- config.default_prog = { 'nu', '-c', "zellij --config-dir ~/.config/yazelix/zellij attach --create yazelix options --default-layout yazelix" }
 
-- Fallback: Welcome layout with nu -c (if direct fails)
-- config.default_prog = { 'nu', '-c', "zellij -l welcome --config-dir ~/.config/yazelix/zellij options --layout-dir ~/.config/yazelix/zellij/layouts" }
 
config.hide_tab_bar_if_only_one_tab = true
config.window_decorations = "NONE"

-- and finally, return the configuration to wezterm
return config
