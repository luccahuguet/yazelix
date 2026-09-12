require("auto-layout"):setup()
require("git"):setup()
require("starship"):setup({
	config_file = os.getenv("YZX_YAZI_STARSHIP_CONFIG"),
})
require("zoxide"):setup({
	update_db = true,
})

if os.getenv("YZX_YAZI_ROLE") == "startup-picker" then
	for id = 1, 6 do
		Status:children_remove(id, id <= 3 and Status.LEFT or Status.RIGHT)
	end
	Status:children_add(function()
		return tostring(cx.layer) == "mgr"
			and " Enter Open · Tab Quick search · q/Esc/Ctrl+C Cancel · F1 Help"
			or ""
	end, 1000, Status.LEFT)
end
