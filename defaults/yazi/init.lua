require("auto-layout"):setup()
require("sidebar-state"):setup()
require("sidebar-status"):setup()
require("git"):setup()
require("starship"):setup({
	config_file = os.getenv("YZX_YAZI_STARSHIP_CONFIG"),
})
require("zoxide"):setup({
	update_db = true,
})
