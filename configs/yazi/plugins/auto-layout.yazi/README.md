# auto-layout.yazi

This plugin for the [yazi file explorer](https://yazi-rs.github.io) will automatically change the number of columns to show in yazi based on the available width. This is especially useful if you use a terminal layout that might have yazi run in a sidebar (where 1 column is all that's required) but then sometimes zoom into it and you want it to update to use the full 3 column layout.

## Installation

```sh
$ ya pack -a josephschmitt/auto-layout
```

## Usage

```lua
-- In your yazi config's init.lua
require("auto-layout")
```

 If you want to customize the breakpoints where the column shifts happen:
```lua
require("auto-layout").setup({
   breakpoint_large = 110,  -- new large window threshold, defaults to 100
   breakpoint_medium = 60,  -- new medium window threshold, defaults to 50
 })
```
