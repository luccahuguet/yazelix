-- Pull in the wezterm API
local wezterm = require 'wezterm'
-- This will hold the configuration.
local config = wezterm.config_builder()
-- This is where you actually apply your config choices
-- For example, changing the color scheme:
config.color_scheme = 'Abernathy'
-- Spawn a nushell shell in login mode
-- config.default_prog = { '/home/lucca/.cargo/bin/nu', '-c', "zellij -l welcome --config-dir ~/.config/yazelix/zellij options --layout-dir ~/.config/yazelix/zellij/layouts" }
config.default_prog = { 'nu', '-c', "zellij -l welcome --config-dir ~/.config/yazelix/zellij options --layout-dir ~/.config/yazelix/zellij/layouts" }
-- Others
config.hide_tab_bar_if_only_one_tab = true
config.window_decorations = "NONE"
-- and finally, return the configuration to wezterm
return config
