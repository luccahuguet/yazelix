require("sidebar-status"):setup()

-- Function to get terminal width
local function get_terminal_width()
    local handle = io.popen("tput cols")
    local width = handle:read("*n") -- Read the number of columns
    handle:close()
    return width or 80 -- Fallback to 80 if it fails
end

local width = get_terminal_width()

-- Define your logic for width adjustment
if width > 120 then
    -- For wider terminals, set a larger sidebar width
    require("toggle-pane"):entry("reset") -- Or a custom setting if available
elseif width > 80 then
    -- For medium-sized terminals, adjust accordingly
    require("toggle-pane"):entry("max-current")
else
    -- For narrow terminals, maybe skip or minimize the sidebar
    require("toggle-pane"):entry("max-current")
    -- You could define a custom entry or skip loading
end
