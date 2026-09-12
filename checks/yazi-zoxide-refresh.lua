local state = {}
local notifications = 0

cx = { active = { current = { cwd = "/current" } } }
ya = {
	sync = function(fn) return function(...) return fn(state, ...) end end,
	notify = function() notifications = notifications + 1 end,
}
ui = { hide = function() return { drop = function() end } end }

local plugin = assert(dofile(assert(arg[1])))
local empty = true
local selection = ""
local checks = 0
local runs = 0
local changes = {}
local opens = {}

plugin.is_empty = function(cwd)
	assert(cwd == "/current")
	checks = checks + 1
	return empty
end
plugin.run_with = function(cwd)
	assert(cwd == "/current")
	runs = runs + 1
	return selection
end
plugin.change_dir = function(target) changes[#changes + 1] = target end
plugin.open_in_editor = function(target) opens[#opens + 1] = target end

plugin:entry()
assert(checks == 1 and runs == 0 and notifications == 1)

empty = false
plugin:entry()
assert(checks == 2 and runs == 1 and notifications == 1)
assert(#changes == 0 and #opens == 0)

selection = "/target"
plugin:entry()
assert(checks == 3 and runs == 2 and notifications == 1)
assert(changes[1] == "/target" and opens[1] == "/target")
