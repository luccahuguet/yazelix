require("auto-layout"):setup()
local workspace_popup = os.getenv("YZX_YAZI_ROLE") == "workspace-popup"
if not workspace_popup then
	require("sidebar-state"):setup()
	require("sidebar-status"):setup()
end
require("git"):setup()
require("starship"):setup({
	config_file = os.getenv("YZX_YAZI_STARSHIP_CONFIG"),
})
require("zoxide"):setup({
	update_db = true,
})
