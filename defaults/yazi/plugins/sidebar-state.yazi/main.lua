local M = {}

local ORCHESTRATOR = "yazelix_pane_orchestrator"
local RETRY_DELAYS = { 0, 0.15, 0.35, 0.75, 1.25 }
local WORKSPACE_POPUP = os.getenv("YZX_YAZI_ROLE") == "workspace-popup"
local generation = 0

local function json_escape(value)
	return tostring(value)
		:gsub("\\", "\\\\")
		:gsub('"', '\\"')
		:gsub("\n", "\\n")
		:gsub("\r", "\\r")
		:gsub("\t", "\\t")
end

local function pane_id()
	local id = os.getenv("ZELLIJ_PANE_ID")
	if not id or id == "" then
		return nil
	end
	if id:find(":", 1, true) then
		return id
	end
	return "terminal:" .. id
end

local function cwd()
	if cx and cx.active and cx.active.current and cx.active.current.cwd then
		return tostring(cx.active.current.cwd)
	end
	return nil
end

local registration_payload = ya.sync(function()
	local yazi_id = os.getenv("YAZI_ID")
	local current_pane = pane_id()
	local current_cwd = cwd()
	if not yazi_id or yazi_id == "" or not current_pane or not current_cwd then
		return nil
	end

	return string.format(
		'{"pane_id":"%s","yazi_id":"%s","cwd":"%s"}',
		json_escape(current_pane),
		json_escape(yazi_id),
		json_escape(current_cwd)
	)
end)

local function pipe_registration(payload)
	local program = os.getenv("YZX_ZELLIJ")
	local command = Command(program and program ~= "" and program or "zellij")
	local session_name = os.getenv("YAZELIX_ZELLIJ_SESSION_NAME")
	if session_name and session_name ~= "" then
		command:env("ZELLIJ_SESSION_NAME", session_name)
	end
	local output = command
		:arg({
			"action",
			"pipe",
			"--plugin",
			ORCHESTRATOR,
			"--name",
			WORKSPACE_POPUP and "register_workspace_popup_yazi_state" or "register_sidebar_yazi_state",
			"--",
			payload,
		})
		:stdin(Command.NULL)
		:output()
	return output
		and output.status
		and output.status.success
		and tostring(output.stdout):gsub("^%s+", ""):gsub("%s+$", "") == "ok"
end

local function publish()
	generation = generation + 1
	local current_generation = generation
	ya.async(function()
		for _, delay in ipairs(RETRY_DELAYS) do
			if current_generation ~= generation then
				return
			end
			if delay > 0 then
				ya.sleep(delay)
			end
			if current_generation ~= generation then
				return
			end
			local payload = registration_payload()
			if payload and pipe_registration(payload) then
				return
			end
		end
	end)
end

local function emit_sidebar_git_refresh()
	local emit = ya.emit or ya.manager_emit
	emit("plugin", { "git", "refresh-sidebar" })
end

function M.setup()
	publish()
	if WORKSPACE_POPUP then
		return
	end
	emit_sidebar_git_refresh()
	ps.sub("cd", function()
		publish()
		emit_sidebar_git_refresh()
	end)
	ps.sub("tab", function()
		publish()
		emit_sidebar_git_refresh()
	end)
end

return M
