-- Auto-Layout Yazi Plugin
-- Automatically adjusts the number of Yazi panes based on terminal width.
-- Attempts to use Yazi's configured ratios via `rt.mgr.ratio`.
-- Falls back to fixed internal ratios if `rt.mgr.ratio` is unavailable.
--
-- Usage in init.lua:
--   require("auto-layout").setup({
--     breakpoint_large = 110, -- Width threshold for 3 panes (default: 100)
--     breakpoint_medium = 60, -- Width threshold for 2 panes (default: 50)
--   })
-- If setup is not called, default breakpoints are used.

-- --- Debug Flag ---
-- Set to true to enable logging to the file specified below.
local DEBUG_MODE = false

-- --- Logging Setup ---
local log_file_path = "/tmp/yazi-autolayout-debug.log"
local log_enabled = DEBUG_MODE
local function write_log(level, message) if not log_enabled then return end local success, file_or_err = pcall(io.open, log_file_path, "a") if success and file_or_err then local file = file_or_err local timestamp = os.date("%Y-%m-%d %H:%M:%S") local write_success, write_err = pcall(function() file:write(string.format("[%s] [%s] %s\n", timestamp, level, message)) file:flush() file:close() end) if not write_success then io.stderr:write(string.format("[auto-layout] LOG WRITE ERROR: %s\n", tostring(write_err))) end else io.stderr:write(string.format("[auto-layout] LOG FILE ERROR: Could not open %s - %s\n", log_file_path, tostring(file_or_err))) end end
local function log_info(message) write_log("INFO", message) end
local function log_warn(message) write_log("WARN", message) end
local function log_error(message) write_log("ERROR", message) end
if DEBUG_MODE then local success, file_or_err = pcall(io.open, log_file_path, "w"); if success and file_or_err then file_or_err:close() end end
-- --- End Logging Setup ---


-- --- Configuration ---
local plugin_config = {
  breakpoint_large = 100,
  breakpoint_medium = 50,
}
-- Define fallback ratios in case Yazi's runtime ratios are unavailable.
local fallback_ratios = { parent = 2, current = 3, preview = 4, all = 9 }
-- --- End Configuration ---


-- --- Plugin Module ---
local M = {}

local original_layout = nil
local layout_overridden = false

-- Get layout ratios, trying the modern 'rt.mgr.ratio' first.
local function get_layout_ratios()
  local using_fallback = true -- Assume fallback needed initially
  local ratios = fallback_ratios -- Default to fallback

  log_info("Attempting to get ratios from rt.mgr.ratio...")
  -- Check the modern Yazi runtime object path
  if rt and rt.mgr and rt.mgr.ratio and
     rt.mgr.ratio.parent and rt.mgr.ratio.current and
     rt.mgr.ratio.preview and rt.mgr.ratio.all then
      log_info(" -> Success: Found ratios in rt.mgr.ratio.")
      ratios = rt.mgr.ratio
      using_fallback = false
  else
      -- Log the specific reason for failure if possible
      if not rt then log_warn(" -> Failed: 'rt' object not available.")
      elseif not rt.mgr then log_warn(" -> Failed: 'rt.mgr' not available.")
      elseif not rt.mgr.ratio then log_warn(" -> Failed: 'rt.mgr.ratio' not available.")
      else log_warn(" -> Failed: One or more required ratios (parent, current, preview, all) missing in rt.mgr.ratio.") end
      log_warn(" -> Using fallback ratios.")
  end
  return ratios, using_fallback
end

-- The layout override function
local function auto_layout_override(self)
  log_info("--- Tab:layout Start ---")

  if not self._area or not self._area.w then
    log_error("ERROR - self._area or self._area.w is nil!")
    if original_layout then original_layout(self) end
    return
  end
  if not ui or not ui.Layout or not ui.Constraint then
     log_error("ERROR - 'ui' object or components not available!")
    if original_layout then original_layout(self) end
    return
  end

  local w = self._area.w
  -- Get ratios using the function that tries 'rt.mgr.ratio' first
  local ratios, using_fallback = get_layout_ratios()

  log_info(string.format("Width=%d, Ratios: p=%d, c=%d, pv=%d, all=%d%s",
             w, ratios.parent, ratios.current, ratios.preview, ratios.all,
             using_fallback and " (FALLBACK)" or " (FROM rt.mgr.ratio)"))

  local success, result = pcall(function()
    local constraints = nil
    local layout_type = ""
    if w > plugin_config.breakpoint_large then
      layout_type = "3-column"
      constraints = {
        ui.Constraint.Ratio(ratios.parent, ratios.all),
        ui.Constraint.Ratio(ratios.current, ratios.all),
        ui.Constraint.Ratio(ratios.preview, ratios.all),
      }
    elseif w > plugin_config.breakpoint_medium then
      layout_type = "2-column"
      constraints = {
        ui.Constraint.Ratio(0, ratios.all),
        ui.Constraint.Ratio(ratios.current + ratios.parent, ratios.all),
        ui.Constraint.Ratio(ratios.preview + ratios.parent, ratios.all),
      }
    else
      layout_type = "1-column"
      constraints = {
        ui.Constraint.Ratio(0, ratios.all),
        ui.Constraint.Ratio(ratios.all, ratios.all),
        ui.Constraint.Ratio(0, ratios.all),
      }
    end

    log_info(string.format("Applying %s layout.", layout_type))
    self._chunks = ui.Layout() :direction(ui.Layout.HORIZONTAL) :constraints(constraints) :split(self._area)
    if not self._chunks then error("Layout split returned nil chunks") end
  end)

  if not success or not self._chunks then
    log_error(string.format("ERROR - Layout failed (success=%s): %s. Falling back.", tostring(success), tostring(result or "chunks nil")))
    if original_layout then original_layout(self) end
  else
     log_info("Layout split successful.")
  end
   log_info("--- Tab:layout End ---")
end

-- Setup function
function M.setup(user_config)
  if DEBUG_MODE and not layout_overridden then
      print(string.format("[auto-layout] DEBUG MODE ENABLED. Logging to: %s", log_file_path))
  end
  log_info("--- Plugin Setup Start ---")

  if type(user_config) == "table" then
     log_info("Applying user config:")
    for k, v in pairs(user_config) do
      if plugin_config[k] ~= nil then
         log_info(string.format("  Setting plugin_config[%s] = %s", k, tostring(v)))
         plugin_config[k] = v
      else
         log_warn(string.format("WARN - Ignoring unknown config key: %s", k))
      end
    end
  else
     log_info("No user config provided or invalid type.")
  end
   log_info(string.format("Final plugin config - large: %d, medium: %d", plugin_config.breakpoint_large, plugin_config.breakpoint_medium))

  if layout_overridden then
     log_warn("WARN - Tab:layout already overridden. Skipping.")
     return
  end

  log_info("Attempting to override Tab:layout...")
  if not Tab then
    log_error("ERROR - Cannot override layout: Global 'Tab' is not available!")
    return
  end

  if not original_layout then
     log_info("Storing original Tab.layout function.")
    original_layout = Tab.layout
    if not original_layout then
      log_warn("WARN - Original Tab.layout is nil! Fallback may not work.")
      original_layout = function() log_error("ERROR - Executing dummy original_layout (original was nil).") end
    end
  end

  log_info("Assigning auto_layout_override to Tab.layout.")
  Tab.layout = auto_layout_override
  layout_overridden = true
  log_info("Tab:layout override complete.")
  log_info("--- Plugin Setup End ---")
end

return M
