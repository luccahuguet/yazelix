--- @since 26.1.22

local WINDOWS = ya.target_family() == "windows"

-- The code of supported git status,
-- also used to determine which status to show for directories when they contain different statuses
-- see `bubble_up`
---@enum CODES
local CODES = {
	unknown = 100, -- status cannot/not yet determined
	excluded = 99, -- ignored directory
	ignored = 6, -- ignored file
	untracked = 5,
	modified = 4,
	added = 3,
	deleted = 2,
	updated = 1,
	clean = 0,
}

---@param signs string
---@return CODES?
local function status_code(signs)
	local index = signs:sub(1, 1)
	local worktree = signs:sub(2, 2)

	if signs == "!!" then
		return CODES.ignored
	elseif signs == "??" then
		return CODES.untracked
	elseif index == "U" or worktree == "U" or signs:find("[AD][AD]") then
		return CODES.updated
	elseif index == "D" or worktree == "D" then
		return CODES.deleted
	elseif index:find("[AC]") then
		return CODES.added
	elseif index:find("[MT]") then
		return CODES.updated
	elseif worktree:find("[MT]") then
		return CODES.modified
	end
end

---@param line string
---@return CODES, string
local function match(line)
	local signs = line:sub(1, 2)
	local code = status_code(signs)
	if not code then
		return
	end

	local path = line:sub(4, 4) == '"' and line:sub(5, -2) or line:sub(4)
	path = WINDOWS and path:gsub("/", "\\") or path
	if path:find("[/\\]$") then
		-- Mark the ignored directory as `excluded`, so we can process it further within `propagate_down`
		return code == CODES.ignored and CODES.excluded or code, path:sub(1, -2)
	else
		return code, path
	end
end

---@param cwd Url
---@return string?
local function root(cwd)
	local is_worktree = function(url)
		local file, head = io.open(tostring(url)), nil
		if file then
			head = file:read(8)
			file:close()
		end
		return head == "gitdir: "
	end

	repeat
		local next = cwd:join(".git")
		local cha = fs.cha(next)
		if cha and (cha.is_dir or is_worktree(next)) then
			return tostring(cwd)
		end
		cwd = cwd.parent
	until not cwd
end

---@param changed Changes
---@return Changes
local function bubble_up(changed)
	local new, empty = {}, Url("")
	for path, code in pairs(changed) do
		if code ~= CODES.ignored then
			local url = Url(path).parent
			while url and url ~= empty do
				local s = tostring(url)
				new[s] = (new[s] or CODES.clean) > code and new[s] or code
				url = url.parent
			end
		end
	end
	return new
end

---@param excluded string[]
---@param cwd Url
---@param repo Url
---@return Changes
local function propagate_down(excluded, cwd, repo)
	local new, rel = {}, cwd:strip_prefix(repo)
	for _, path in ipairs(excluded) do
		if rel:starts_with(path) then
			-- If `cwd` is a subdirectory of an excluded directory, also mark it as `excluded`
			new[tostring(cwd)] = CODES.excluded
		elseif cwd == repo:join(path).parent then
			-- If `path` is a direct subdirectory of `cwd`, mark it as `ignored`
			new[path] = CODES.ignored
		else
			-- Skipping, we only care about `cwd` itself and its direct subdirectories for maximum performance
		end
	end
	return new
end

---@param cwd string
---@param repo string
---@param changed Changes
local add = ya.sync(function(st, cwd, repo, changed)
	---@cast st State

	st.dirs[cwd] = repo
	st.repos[repo] = st.repos[repo] or {}
	for path, code in pairs(changed) do
		if code == CODES.clean then
			st.repos[repo][path] = nil
		elseif code == CODES.excluded then
			-- Mark the directory with a special value `excluded` so that it can be distinguished during UI rendering
			st.dirs[path] = CODES.excluded
		else
			st.repos[repo][path] = code
		end
	end
	ui.render()
end)

---@param cwd string
local remove = ya.sync(function(st, cwd)
	---@cast st State

	local repo = st.dirs[cwd]
	if not repo then
		return
	end

	ui.render()
	st.dirs[cwd] = nil
	if not st.repos[repo] then
		return
	end

	for _, r in pairs(st.dirs) do
		if r == repo then
			return
		end
	end
	st.repos[repo] = nil
end)

---@param cwd Url
---@param repo string
---@param paths string[]
---@param has_directories boolean
---@return boolean, string?
local function update_changed_state(cwd, repo, paths, has_directories)
	-- stylua: ignore
	local output, err = Command("git")
		:cwd(tostring(cwd))
		:arg({ "--no-optional-locks", "-c", "core.quotePath=", "status", "--porcelain", "-unormal", "--no-renames", "--ignored=matching" })
		:arg(paths)
		:output()
	if not output then
		return false, Err("Cannot spawn `git` command, error: %s", err)
	end

	local changed, excluded = {}, {}
	for line in output.stdout:gmatch("[^\r\n]+") do
		local code, path = match(line)
		if code == CODES.excluded then
			excluded[#excluded + 1] = path
		else
			changed[path] = code
		end
	end

	if has_directories then
		ya.dict_merge(changed, bubble_up(changed))
	end
	ya.dict_merge(changed, propagate_down(excluded, cwd, Url(repo)))

	-- Reset the status of any files that don't appear in the output of `git status` to `clean`,
	-- so that cleaning up outdated statuses from `st.repos`
	for _, path in ipairs(paths) do
		local s = path:sub(#repo + 2)
		changed[s] = changed[s] or CODES.clean
	end

	add(tostring(cwd), repo, changed)
	return true
end

---@return { cwd: string, paths: string[], has_directories: boolean }?
local snapshot_active_folder = ya.sync(function()
	local current = cx and cx.active and cx.active.current
	if not current or not current.files or #current.files == 0 then
		return nil
	end

	local paths, has_directories = {}, false
	local first = current.files[1]
	if not first or not first.url then
		return nil
	end

	local cwd = first.url.base or first.url.parent
	if not cwd then
		return nil
	end

	for i = 1, #current.files do
		local file = current.files[i]
		if file then
			paths[#paths + 1] = tostring(file.url)
			has_directories = has_directories or file.cha.is_dir
		end
	end

	return {
		cwd = tostring(cwd),
		paths = paths,
		has_directories = has_directories,
	}
end)

---@param st State
---@param file File
---@return CODES
local function code_for_file(st, file)
	if not file.in_current then
		return CODES.clean
	end

	local url = file.url
	local cwd = url.base or url.parent
	if not cwd then
		return CODES.unknown
	end

	local repo = st.dirs[tostring(cwd)]
	if not repo then
		return CODES.unknown
	elseif repo == CODES.excluded then
		return CODES.ignored
	end

	local repo_state = st.repos[repo]
	if not repo_state then
		return CODES.clean
	end

	return repo_state[tostring(url):sub(#repo + 2)] or CODES.clean
end

---@param content string
local function fail(content)
	ya.notify { title = "Git", content = content, timeout = 5, level = "error" }
end

---@param st State
---@param opts Options
local function setup(st, opts)
	st.dirs = {}
	st.repos = {}

	opts = opts or {}
	opts.order = opts.order or 1500

	local t = th.git or {}
	local styles = {
		[CODES.unknown] = t.unknown or ui.Style(),
		[CODES.ignored] = t.ignored or ui.Style():fg("darkgray"),
		[CODES.untracked] = t.untracked or ui.Style():fg("magenta"),
		[CODES.modified] = t.modified or ui.Style():fg("yellow"),
		[CODES.added] = t.added or ui.Style():fg("green"),
		[CODES.deleted] = t.deleted or ui.Style():fg("red"),
		[CODES.updated] = t.updated or ui.Style():fg("cyan"),
		[CODES.clean] = t.clean or ui.Style(),
	}
	local signs = {
		[CODES.unknown] = t.unknown_sign or "",
		[CODES.ignored] = t.ignored_sign or " ",
		[CODES.untracked] = t.untracked_sign or "? ",
		[CODES.modified] = t.modified_sign or " ",
		[CODES.added] = t.added_sign or " ",
		[CODES.deleted] = t.deleted_sign or " ",
		[CODES.updated] = t.updated_sign or " ",
		[CODES.clean] = t.clean_sign or "",
	}

	Entity._yazelix_git_text_styles = {
		[CODES.untracked] = styles[CODES.untracked],
		[CODES.modified] = styles[CODES.modified],
		[CODES.added] = styles[CODES.added],
		[CODES.deleted] = styles[CODES.deleted],
		[CODES.updated] = styles[CODES.updated],
	}
	Entity._yazelix_git_code_for_file = function(file)
		return code_for_file(st, file)
	end

	if not Entity._yazelix_git_style_patched then
		local entity_style = Entity.style
		Entity.style = function(self)
			local style = entity_style(self)
			local resolve_code = Entity._yazelix_git_code_for_file
			local text_styles = Entity._yazelix_git_text_styles
			if not resolve_code or not text_styles then
				return style
			end

			local text_style = text_styles[resolve_code(self._file)]
			return text_style and style:patch(text_style) or style
		end
		Entity._yazelix_git_style_patched = true
	end

	Linemode:children_add(function(self)
		if not self._file.in_current then
			return ""
		end

		local code = code_for_file(st, self._file)

		if signs[code] == "" then
			return ""
		elseif self._file.is_hovered then
			return ui.Line { " ", signs[code] }
		else
			return ui.Line { " ", ui.Span(signs[code]):style(styles[code]) }
		end
	end, opts.order)
end

---@type UnstableFetcher
local function fetch(_, job)
	local cwd = job.files[1].url.base or job.files[1].url.parent
	local repo = root(cwd)
	if not repo then
		remove(tostring(cwd))
		return true
	end

	local paths = {}
	for _, file in ipairs(job.files) do
		paths[#paths + 1] = tostring(file.url)
	end

	local ok, err = update_changed_state(cwd, repo, paths, job.files[1].cha.is_dir)
	if not ok then
		return true, err
	end

	return false
end

local function entry()
	local snapshot = snapshot_active_folder()
	if not snapshot or #snapshot.paths == 0 then
		return
	end

	local cwd = Url(snapshot.cwd)
	local repo = root(cwd)
	if not repo then
		remove(snapshot.cwd)
		return
	end

	local ok, err = update_changed_state(cwd, repo, snapshot.paths, snapshot.has_directories)
	if not ok then
		fail(tostring(err))
	end
end

return { setup = setup, fetch = fetch, entry = entry }
