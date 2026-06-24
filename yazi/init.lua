require("auto-layout"):setup()
require("git"):setup()
require("starship"):setup({
	config_file = os.getenv("YZN_YAZI_STARSHIP_CONFIG"),
})
require("zoxide"):setup({
	update_db = true,
})
