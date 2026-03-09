local M = {}

local function sanitize_component(value)
	if not value or value == "" then
		return nil
	end

	return value:gsub("[^%w%-_.]", "_")
end

local function normalize_pane_id(value)
	if not value or value == "" then
		return nil
	end

	if value:find(":", 1, true) then
		return value
	end

	return "terminal:" .. value
end

local function write_sidebar_state()
	local home = os.getenv("HOME")
	local session_name = sanitize_component(os.getenv("ZELLIJ_SESSION_NAME"))
	local pane_id = sanitize_component(normalize_pane_id(os.getenv("ZELLIJ_PANE_ID")))
	local yazi_id = os.getenv("YAZI_ID")

	if not home or not session_name or not pane_id or not yazi_id or yazi_id == "" then
		return
	end

	local state_dir = home .. "/.local/share/yazelix/state/yazi/sidebar"
	os.execute(string.format("mkdir -p %q", state_dir))

	local state_path = string.format("%s/%s__%s.txt", state_dir, session_name, pane_id)
	local file = io.open(state_path, "w")
	if not file then
		return
	end

	file:write(yazi_id)
	file:write("\n")
	file:close()
end

function M.setup()
	write_sidebar_state()
end

return M
