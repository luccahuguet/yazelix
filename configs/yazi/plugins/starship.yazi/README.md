# starship.yazi

Starship prompt plugin for [Yazi](https://github.com/sxyazi/yazi)

<https://github.com/Rolv-Apneseth/starship.yazi/assets/69486699/f7314687-5cb1-4d66-8d9d-cca960ba6716>

## Requirements

- [Yazi](https://github.com/sxyazi/yazi) (v25.4.8+)
- [starship](https://github.com/starship/starship)

## Installation

```bash
ya pkg add Rolv-Apneseth/starship
```

### Manual

```sh
# Linux / MacOS
git clone https://github.com/Rolv-Apneseth/starship.yazi.git ~/.config/yazi/plugins/starship.yazi
# Windows
git clone https://github.com/Rolv-Apneseth/starship.yazi.git %AppData%\yazi\config\plugins\starship.yazi
```

## Usage

Add this to `~/.config/yazi/init.lua`:

```lua
require("starship"):setup()
```

Make sure you have [starship](https://github.com/starship/starship) installed and in your `PATH`.

## Config

Here is an example with all available config options:

```lua
require("starship"):setup({
    -- Hide flags (such as filter, find and search). This can be beneficial for starship themes
    -- which are intended to go across the entire width of the terminal.
    hide_flags = false,
    -- Whether to place flags after the starship prompt. False means the flags will be placed before the prompt.
    flags_after_prompt = true,
    -- Custom starship configuration file to use
    config_file = "~/.config/starship_full.toml", -- Default: nil
    -- Whether to enable support for starship's right prompt (i.e. `starship prompt --right`).
    show_right_prompt = false,
    -- Whether to hide the count widget, in case you want only your right prompt to show up. Only has
    -- an effect when `show_right_prompt = true`
    hide_count = false,
    -- Separator to place between the right prompt and the count widget. Use `count_separator = ""`
    -- to have no space between the widgets.
    count_separator = " ",
})
```

## Extra

If you use a `starship` theme with a background colour, it might look a bit to cramped on just the one line `Yazi` gives the header by default. To fix this, you can add this to your `init.lua`:

<details>
<summary>Click to expand</summary>

```lua
local old_build = Tab.build

Tab.build = function(self, ...)
    local bar = function(c, x, y)
        if x <= 0 or x == self._area.w - 1 then
            return ui.Bar(ui.Bar.TOP):area(ui.Rect.default)
        end

        return ui.Bar(ui.Bar.TOP)
            :area(ui.Rect({
                x = x,
                y = math.max(0, y),
                w = ya.clamp(0, self._area.w - x, 1),
                h = math.min(1, self._area.h),
            }))
            :symbol(c)
    end

    local c = self._chunks
    self._chunks = {
        c[1]:pad(ui.Pad.y(1)),
        c[2]:pad(ui.Pad(1, c[3].w > 0 and 0 or 1, 1, c[1].w > 0 and 0 or 1)),
        c[3]:pad(ui.Pad.y(1)),
    }

    local style = th.mgr.border_style
    self._base = ya.list_merge(self._base or {}, {
        ui.Bar(ui.Bar.RIGHT):area(self._chunks[1]):style(style),
        ui.Bar(ui.Bar.LEFT):area(self._chunks[1]):style(style),

        bar("┬", c[1].right - 1, c[1].y),
        bar("┴", c[1].right - 1, c[1].bottom - 1),
        bar("┬", c[2].right, c[2].y),
        bar("┴", c[2].right, c[2].bottom - 1),
    })

    old_build(self, ...)
end
```

</details>

> [!NOTE]
> This works by overriding your `Tab.build` function so make sure this is the only place you're doing that in your config. For example, this would be incompatible with the [full-border plugin](https://github.com/yazi-rs/plugins/tree/main/full-border.yazi)

## Thanks

- [sxyazi](https://github.com/sxyazi) for providing the code for this plugin and the demo video [in this comment](https://github.com/sxyazi/yazi/issues/767#issuecomment-1977082834)
