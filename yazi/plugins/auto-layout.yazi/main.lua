-- Auto-Layout Yazi Plugin
--
-- This plugin will automatically change the number of columns to show in yazi based on the
-- available width. This is especially useful if you use a terminal layout that might have
-- yazi run in a sidebar (where 1 column is all that's required) but then sometimes zoom into it
-- and you want it to update to use the full 3 column layout.
--
-- Usage:
--   In your init.lua file for yazi
--
--   require("auto-layout")
--
--   If you want to customize the breakpoints where the column shifts happen:
--
--   require("auto-layout"require("auto-layout").setup({
--     breakpoint_large = 100,  -- new large window threshold, defaults to 80
--     breakpoint_medium = 50,  -- new medium window threshold, defaults to 40
--   })

local config = {
  breakpoint_large = 80,  -- default large window threshold
  breakpoint_medium = 40, -- default medium window threshold
}

local M = {}

function M.setup(user_config)
  if type(user_config) == "table" then
    for k, v in pairs(user_config) do
      config[k] = v
    end
  end
end

function Tab:layout()
  local w = self._area.w

  if w > config.breakpoint_large then
    self._chunks = ui.Layout()
      :direction(ui.Layout.HORIZONTAL)
      :constraints({
        ui.Constraint.Ratio(MANAGER.ratio.parent,  MANAGER.ratio.all),
        ui.Constraint.Ratio(MANAGER.ratio.current, MANAGER.ratio.all),
        ui.Constraint.Ratio(MANAGER.ratio.preview, MANAGER.ratio.all),
      })
      :split(self._area)
  elseif w > config.breakpoint_medium then
    self._chunks = ui.Layout()
      :direction(ui.Layout.HORIZONTAL)
      :constraints({
        ui.Constraint.Ratio(0,                              MANAGER.ratio.all),
        ui.Constraint.Ratio(MANAGER.ratio.current + MANAGER.ratio.parent, MANAGER.ratio.all),
        ui.Constraint.Ratio(MANAGER.ratio.preview   + MANAGER.ratio.parent, MANAGER.ratio.all),
      })
      :split(self._area)
  else
    self._chunks = ui.Layout()
      :direction(ui.Layout.HORIZONTAL)
      :constraints({
        ui.Constraint.Ratio(0,               MANAGER.ratio.all),
        ui.Constraint.Ratio(MANAGER.ratio.all, MANAGER.ratio.all),
        ui.Constraint.Ratio(0,               MANAGER.ratio.all),
      })
      :split(self._area)
  end
end

return M

