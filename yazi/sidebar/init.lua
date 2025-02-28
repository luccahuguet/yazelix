require("sidebar-status"):setup()

-- Function to get terminal width
local function get_terminal_width()
    local handle = io.popen("tput cols")
    local width = handle:read("*n")
    handle:close()
    return width or 80 -- Default to 80 if tput fails
end

-- Function to determine and set the layout
local function set_layout()
    -- Check env var first (set by script)
    local sidebar_size = os.getenv("YAZI_SIDEBAR_SIZE")

    -- If no env var, calculate size based on terminal width
    if not sidebar_size then
        local width = get_terminal_width()
        if width > 120 then
            sidebar_size = "big"
        elseif width > 80 then
            sidebar_size = "medium"
        else
            sidebar_size = "small"
        end
    end

    -- Map size to layout
    local layout = (sidebar_size == "small") and "max-current" or "reset"
    require("toggle-pane"):entry(layout)
end

-- Apply layout on startup
set_layout()
