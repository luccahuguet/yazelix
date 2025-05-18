-- Pull in the wezterm API
local wezterm = require 'wezterm'

-- This will hold the configuration.
local config = wezterm.config_builder()

-- This is where you actually apply your config choices
-- For example, changing the color scheme:
config.color_scheme = 'Abernathy'

-- to start with yazelix, creating a new session or attaching if a session called yazelix already exists
config.default_prog = { 'nu', '-c', "zellij --config-dir ~/.config/yazelix/zellij attach --create yazelix_wez options --default-layout yazelix" }
 
-- Alternative: Pick a layout every time
-- config.default_prog = { 'nu', '-c', "zellij -l welcome --config-dir ~/.config/yazelix/zellij options --layout-dir ~/.config/yazelix/zellij/layouts" }
 
config.hide_tab_bar_if_only_one_tab = true
config.window_decorations = "NONE"

-- and finally, return the configuration to wezterm
return config
