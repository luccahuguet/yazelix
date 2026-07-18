local plugin_path = assert(arg[1], "missing sidebar-state plugin path")
local commands = {}
local sleeps = 0

local environment = {
	YAZELIX_ZELLIJ_SESSION_NAME = "test-session",
	YAZI_ID = "12345",
	YZX_YAZI_ROLE = "workspace-popup",
	YZX_ZELLIJ = "/bin/zellij",
	ZELLIJ_PANE_ID = "7",
}

os.getenv = function(name)
	return environment[name]
end

ya = {
	async = function(callback)
		callback()
	end,
	sleep = function(_)
		sleeps = sleeps + 1
		cx = { active = { current = { cwd = "/workspace" } } }
	end,
	sync = function(callback)
		return function(...)
			return callback({}, ...)
		end
	end,
}

Command = setmetatable({ NULL = {} }, {
	__call = function(_, program)
		local command = { program = program }

		function command:arg(args)
			self.args = args
			return self
		end

		function command:env()
			return self
		end

		function command:stdin()
			return self
		end

		function command:output()
			commands[#commands + 1] = self
			return { status = { success = true }, stdout = "ok\n" }
		end

		return command
	end,
})

local plugin = dofile(plugin_path)
plugin.setup()

assert(sleeps == 1, "registration must retry after startup state becomes ready")
assert(#commands == 1, "workspace popup must publish exactly one successful registration")
assert(commands[1].program == "/bin/zellij")
assert(commands[1].args[6] == "register_workspace_popup_yazi_state")
assert(commands[1].args[8] == [[{"pane_id":"terminal:7","yazi_id":"12345","cwd":"/workspace"}]])
