--- Zoxide jump that opens the selected directory in the managed editor
--- instead of navigating Yazi. Reuses the upstream zoxide fzf flow.

local M = {}

local state = ya.sync(function(st)
	return {
		cwd = tostring(cx.active.current.cwd),
		empty = st.empty,
	}
end)

local set_state = ya.sync(function(st, empty) st.empty = empty end)

function M:entry()
	local st = state()
	if st.empty == nil then
		st.empty = M.is_empty(st.cwd)
		set_state(st.empty)
	end

	if st.empty then
		return ya.notify {
			title = "Zoxide",
			content = "No directory history found.",
			timeout = 5,
			level = "error",
		}
	end

	local permit = ui.hide()
	local target, err = M.run_with(st.cwd)
	permit:drop()

	if not target then
		ya.notify { title = "Zoxide", content = tostring(err), timeout = 5, level = "error" }
	elseif target ~= "" then
		M.open_in_editor(target)
	end
end

function M.open_in_editor(target_dir)
	local script = os.getenv("HOME") .. "/.config/yazelix/nushell/scripts/integrations/zoxide_open_in_editor.nu"
	local child, err = Command("nu")
		:arg({ script, target_dir })
		:stdout(Command.PIPED)
		:stderr(Command.PIPED)
		:spawn()

	if not child then
		ya.notify { title = "Zoxide Editor", content = "Failed to run script: " .. tostring(err), timeout = 5, level = "error" }
		return
	end

	local output, err = child:wait_with_output()
	if not output then
		ya.notify { title = "Zoxide Editor", content = "Script error: " .. tostring(err), timeout = 5, level = "error" }
	elseif not output.status.success then
		ya.notify { title = "Zoxide Editor", content = "Script failed: " .. output.stderr, timeout = 5, level = "error" }
	end
end

--- Reuse upstream zoxide fzf options
function M.options()
	local default = {
		"--exact",
		"--no-sort",
		"--bind=ctrl-z:ignore,btab:up,tab:down",
		"--cycle",
		"--keep-right",
		"--layout=reverse",
		"--height=100%",
		"--border",
		"--scrollbar=\xe2\x96\x8c",
		"--info=inline",
		"--tabstop=1",
		"--exit-0",
		"--preview-window=down,30%,sharp",
		[[--preview='\command -p ls -Cp --color=always --group-directories-first {2..}']],
	}

	return (os.getenv("FZF_DEFAULT_OPTS") or "")
		.. " "
		.. table.concat(default, " ")
		.. " "
		.. (os.getenv("YAZI_ZOXIDE_OPTS") or "")
end

function M.is_empty(cwd)
	local child = Command("zoxide"):arg({ "query", "-l", "--exclude", cwd }):stdout(Command.PIPED):spawn()
	if not child then
		return true
	end
	local first = child:read_line()
	child:start_kill()
	return not first
end

function M.run_with(cwd)
	local child, err = Command("zoxide")
		:arg({ "query", "-i", "--exclude", cwd })
		:env("SHELL", "sh")
		:env("CLICOLOR", 1)
		:env("CLICOLOR_FORCE", 1)
		:env("_ZO_FZF_OPTS", M.options())
		:stdin(Command.INHERIT)
		:stdout(Command.PIPED)
		:stderr(Command.PIPED)
		:spawn()

	if not child then
		return nil, Err("Failed to start `zoxide`, error: %s", err)
	end

	local output, err = child:wait_with_output()
	if not output then
		return nil, Err("Cannot read `zoxide` output, error: %s", err)
	elseif not output.status.success and output.status.code ~= 130 then
		return nil, Err("`zoxide` exited with code %s: %s", output.status.code, output.stderr:gsub("^zoxide:%s*", ""))
	end
	return output.stdout:gsub("\n$", ""), nil
end

return M
