local M = {}
local PANE_ORCHESTRATOR_PLUGIN_ALIAS = "yazelix_pane_orchestrator"

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

local function current_cwd()
	if not cx or not cx.active or not cx.active.current then
		return nil
	end

	local cwd = cx.active and cx.active.current and cx.active.current.cwd
	if not cwd then
		return nil
	end

	return tostring(cwd)
end

local function json_escape(value)
	if value == nil then
		return ""
	end

	return tostring(value)
		:gsub("\\", "\\\\")
		:gsub('"', '\\"')
		:gsub("\n", "\\n")
		:gsub("\r", "\\r")
		:gsub("\t", "\\t")
end

local function register_sidebar_state_with_pane_orchestrator(yazi_id, pane_id, cwd)
	if not yazi_id or yazi_id == "" or not pane_id or pane_id == "" or not cwd or cwd == "" then
		return
	end

	local payload = string.format(
		'{"pane_id":"%s","yazi_id":"%s","cwd":"%s"}',
		json_escape(pane_id),
		json_escape(yazi_id),
		json_escape(cwd)
	)
	os.execute(string.format(
		"zellij action pipe --plugin %q --name register_sidebar_yazi_state -- %q >/dev/null 2>&1",
		PANE_ORCHESTRATOR_PLUGIN_ALIAS,
		payload
	))
end

local function write_sidebar_state()
	local home = os.getenv("HOME")
	local session_name = sanitize_component(os.getenv("ZELLIJ_SESSION_NAME"))
	local normalized_pane_id = normalize_pane_id(os.getenv("ZELLIJ_PANE_ID"))
	local pane_id = sanitize_component(normalized_pane_id)
	local yazi_id = os.getenv("YAZI_ID")
	local cwd = current_cwd()

	if not home or not session_name or not pane_id or not yazi_id or yazi_id == "" or not cwd or cwd == "" then
		return
	end

	register_sidebar_state_with_pane_orchestrator(yazi_id, normalized_pane_id, cwd)

	local state_dir = home .. "/.local/share/yazelix/state/yazi/sidebar"
	os.execute(string.format("mkdir -p %q", state_dir))

	local state_path = string.format("%s/%s__%s.txt", state_dir, session_name, pane_id)
	os.execute(string.format("find %q -maxdepth 1 -type f -name %q ! -path %q -delete", state_dir, session_name .. "__*.txt", state_path))
	local file = io.open(state_path, "w")
	if not file then
		return
	end

	file:write(yazi_id)
	file:write("\n")
	file:write(cwd)
	file:write("\n")
	file:close()
end

local function emit_sidebar_git_refresh()
	local emit = ya.emit or ya.manager_emit
	emit("plugin", { "git", "refresh-sidebar" })
end

local function emit_sidebar_starship_refresh()
	local cwd = current_cwd()
	if not cwd or cwd == "" then
		return
	end

	local emit = ya.emit or ya.manager_emit
	emit("plugin", { "starship", ya.quote(cwd, true) })
end

function M.setup()
	write_sidebar_state()
	emit_sidebar_git_refresh()
	emit_sidebar_starship_refresh()

	ps.sub("cd", function()
		write_sidebar_state()
		emit_sidebar_git_refresh()
	end)
	ps.sub("tab", function()
		write_sidebar_state()
		emit_sidebar_git_refresh()
	end)
end

return M
