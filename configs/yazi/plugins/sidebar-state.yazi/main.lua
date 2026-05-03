local M = {}
local PANE_ORCHESTRATOR_PLUGIN_ALIAS = "yazelix_pane_orchestrator"
local STARTUP_REGISTER_DELAY_SECONDS = 1.25

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

local function register_sidebar_state_with_pane_orchestrator(yazi_id, pane_id, cwd, delay_seconds)
	if not yazi_id or yazi_id == "" or not pane_id or pane_id == "" or not cwd or cwd == "" then
		return
	end

	local payload = string.format(
		'{"pane_id":"%s","yazi_id":"%s","cwd":"%s"}',
		json_escape(pane_id),
		json_escape(yazi_id),
		json_escape(cwd)
	)
	ya.async(function()
		if delay_seconds and delay_seconds > 0 then
			ya.sleep(delay_seconds)
		end

		Command("zellij")
			:arg({
				"action",
				"pipe",
				"--plugin",
				PANE_ORCHESTRATOR_PLUGIN_ALIAS,
				"--name",
				"register_sidebar_yazi_state",
				"--",
				payload,
			})
			:stdin(Command.NULL)
			:stdout(Command.NULL)
			:stderr(Command.NULL)
			:status()
	end)
end

local function publish_sidebar_state(delay_seconds)
	local normalized_pane_id = normalize_pane_id(os.getenv("ZELLIJ_PANE_ID"))
	local yazi_id = os.getenv("YAZI_ID")
	local cwd = current_cwd()

	if not normalized_pane_id or not yazi_id or yazi_id == "" or not cwd or cwd == "" then
		return
	end

	register_sidebar_state_with_pane_orchestrator(yazi_id, normalized_pane_id, cwd, delay_seconds)
end

local function publish_sidebar_state_after_startup_delay()
	publish_sidebar_state(STARTUP_REGISTER_DELAY_SECONDS)
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
	publish_sidebar_state_after_startup_delay()
	emit_sidebar_git_refresh()
	emit_sidebar_starship_refresh()

	ps.sub("cd", function()
		publish_sidebar_state()
		emit_sidebar_git_refresh()
	end)
	ps.sub("tab", function()
		publish_sidebar_state()
		emit_sidebar_git_refresh()
	end)
end

return M
