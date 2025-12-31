--- @since 25.4.8

-- For development
--[[ local function notify(message) ]]
--[[     ya.notify({ title = "Starship", content = message, timeout = 3 }) ]]
--[[ end ]]

local save = ya.sync(function(st, outputs)
    st.output_left = outputs.left
    st.output_right = outputs.right

    -- Support for versions later than v25.5.31 (not yet a full release as of writing this comment)
    local render = ui.render or ya.render
    render()
end)

-- Helper function for accessing the `config_file` state variable
---@return string
local get_config_file = ya.sync(function(st)
    return st.config_file
end)

--- Helper function for accessing `show_right_prompt` state variable
---@return boolean
local should_show_right_prompt = ya.sync(function(st)
    return st.show_right_prompt
end)

return {
    ---User arguments for setup method
    ---@class SetupArgs
    ---@field config_file string Absolute path to a starship config file.
    ---@field hide_flags boolean Whether to hide all flags (such as filter and search). Default value is false.
    ---@field flags_after_prompt boolean Whether to place flags (such as filter and search) after the starship prompt. Default value is true.
    ---@field show_right_prompt boolean Whether to enable starship right prompt support. Default value is false.
    ---@field hide_count boolean Whether to hide the count widget. Only has an effect when show_right_prompt is true. Default value is false.
    ---@field count_separator string Set a custom separator between the count widget and the right prompt. Default value is " ", set to "" for no space.

    --- Setup plugin
    --- @param st table State
    --- @param args SetupArgs|nil
    setup = function(st, args)
        local hide_flags = false
        local flags_after_prompt = true
        local hide_count = false
        local count_separator = " "

        -- Check setup args
        if args ~= nil then
            if args.config_file ~= nil then
                local url = Url(args.config_file)
                if url.is_regular then
                    local config_file = args.config_file

                    -- Manually replace '~' and '$HOME' at the start of the path with the OS environment variable
                    local home = os.getenv("HOME")
                    if home then
                        home = tostring(home)
                        config_file = config_file:gsub("^~", home):gsub("^$HOME", home)
                    end

                    st.config_file = config_file
                end
            end

            if args.show_right_prompt ~= nil then
                -- Save directly to the plugin state so it can be accessed
                -- read from the entry function
                st.show_right_prompt = args.show_right_prompt
            end

            if args.hide_count ~= nil then
                hide_count = args.hide_count
            end

            if args.count_separator ~= nil then
                count_separator = args.count_separator
            end

            if args.hide_flags ~= nil then
                hide_flags = args.hide_flags
            end

            if args.flags_after_prompt ~= nil then
                flags_after_prompt = args.flags_after_prompt
            end
        end

        -- Replace default left header widget
        Header:children_remove(1, Header.LEFT)
        Header:children_add(function(self)
            if hide_flags or not st.output_left then
                return ui.Line.parse(st.output_left or "")
            end

            -- Split `st.output` at the first line break (or keep as is if none was found)
            local output = st.output_left:match("([^\n]*)\n?") or st.output_left

            local flags = self:flags()
            if flags_after_prompt then
                output = output .. " " .. flags
            else
                output = flags .. " " .. output
            end

            local line = ui.Line.parse(output)
            if line:width() > self._area.w then
                return ""
            end

            return line
        end, 1000, Header.LEFT)

        -- Support for right prompt, if enabled
        if st.show_right_prompt then
            -- Remove the default count widget
            Header:children_remove(1, Header.RIGHT)
            -- Replace with a custom widget combining the right prompt and the count widget
            Header:children_add(function(self)
                if not st.output_right then
                    st.output_right = ""
                end
                local output = st.output_right:match("([^\n]*)\n?") or st.output_right
                local line = ui.Line.parse(output)

                -- Custom count widget so that we can measure the combined width
                if not hide_count then
                    local yanked = #cx.yanked
                    local count, style
                    if yanked == 0 then
                        count = #self._tab.selected
                        style = th.mgr.count_selected
                    elseif cx.yanked.is_cut then
                        count = yanked
                        style = th.mgr.count_cut
                    else
                        count = yanked
                        style = th.mgr.count_copied
                    end

                    -- Append custom count widget
                    if count ~= 0 then
                        line = ui.Line({
                            line,
                            ui.Span(count_separator),
                            ui.Span(string.format(" %d ", count)):style(style),
                        })
                    end
                end

                -- Give precedence to the left header widget(s), hiding this component entirely if
                -- there is no room for both
                local right_width = line:width()
                if self._left_width then
                    local max = self._area.w - self._left_width
                    if max < right_width then
                        return ""
                    end
                end

                return line
            end, 1000, Header.RIGHT)

            -- Override the header's redraw method, since we want the left side of the header to have
            -- precedence over the right, unlike the default behaviour of hiding the left side if
            -- there isn't room for both.
            function Header:redraw()
                local left = self:children_redraw(self.LEFT)
                self._left_width = left:width()

                local right = self:children_redraw(self.RIGHT)

                return {
                    ui.Line(left):area(self._area),
                    ui.Line(right):area(self._area):align(ui.Align.RIGHT),
                }
            end
        end

        -- Pass current working directory and custom config path (if specified) to the plugin's entry point
        ---Callback for subscribers to update the prompt
        local callback = function()
            local cwd = cx.active.current.cwd
            if st.cwd ~= cwd then
                st.cwd = cwd

                -- `ya.emit` as of 25.5.28
                local emit = ya.emit or ya.manager_emit

                emit("plugin", {
                    st._id,
                    ya.quote(tostring(cwd), true),
                })
            end
        end

        -- Subscribe to events
        ps.sub("cd", callback)
        ps.sub("tab", callback)
    end,

    entry = function(_, job)
        local args = job.args

        -- Setup commands for left and right prompts
        local function base_command()
            return Command("starship")
                :arg("prompt")
                :stdin(Command.INHERIT)
                :cwd(args[1])
                :env("STARSHIP_SHELL", "")
                :env("PWD", args[1])
        end
        local command_left = base_command()
        local command_right = base_command():arg("--right")

        -- Point to custom starship config for both commands
        local config_file = get_config_file()
        if config_file then
            command_left = command_left:env("STARSHIP_CONFIG", config_file)
            command_right = command_right:env("STARSHIP_CONFIG", config_file)
        end

        -- Execute left prompt command and save output
        local outputs = { left = "", right = "" }
        local output_left = command_left:output()
        if output_left then
            outputs.left = output_left.stdout:gsub("^%s+", "")
        end

        -- If support for right prompt is enabled, execute right prompt command and save output
        local show_right_prompt = should_show_right_prompt()
        if show_right_prompt then
            local output_right = command_right:output()
            if output_right then
                outputs.right = output_right.stdout:gsub("^%s+", "")
            end
        end

        save(outputs)
    end,
}
