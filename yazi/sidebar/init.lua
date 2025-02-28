require("sidebar-status"):setup()

local sidebar_size = os.getenv("YAZI_SIDEBAR_SIZE") or "medium"
if sidebar_size == "big" then
    require("toggle-pane"):entry("reset")       -- Wide terminals
elseif sidebar_size == "medium" then
    require("toggle-pane"):entry("reset")       -- Medium terminals
elseif sidebar_size == "small" then
    require("toggle-pane"):entry("max-current") -- Narrow terminals
else
    require("toggle-pane"):entry("reset")       -- Fallback
end
