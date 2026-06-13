# Styling and Themes

## Yazelix Appearance Mode

Set the global generated appearance in `~/.config/yazelix/settings.jsonc`:

```jsonc
{
  "appearance": {
    "mode": "dark" // "dark", "light", or "auto"
  }
}
```

`dark` is the default. `light` switches generated terminal colors and the default Zellij/Yazi themes to light palettes where that terminal supports them, while preserving explicit `zellij.theme` and `yazi.theme` choices. `auto` uses native automatic system appearance in Ghostty, WezTerm, and yzxterm; yzxterm owns its packaged dark/light themes in the terminal child package, and main Yazelix copies those themes into generated yzxterm config roots.

## Terminal Transparency

WezTerm includes transparency by default:
```lua
config.window_background_opacity = 0.9
```

Edit `~/.wezterm.lua` to customize (1.0 = opaque, 0.5 = very transparent, etc).

## Helix Themes

Recommended transparent theme:
```toml
# ~/.config/yazelix/helix/config.toml
theme = "term16_dark"
```

Alternative: `base16_transparent`

Popular non-transparent themes: `ao`, `dark_plus`, `onedark`, `gruvbox`, `catppuccin_mocha`

Custom theme TOML files for Yazelix-managed Helix sessions live under `~/.config/yazelix/helix/themes/`, and the selected theme name belongs in `~/.config/yazelix/helix/config.toml`. Native `~/.config/helix/themes/` belongs to plain Helix outside Yazelix, and the old `~/.config/yazelix/user_conf/helix/themes/` path is unsupported legacy state.

## WezTerm Color Schemes

Default: `Abernathy`

Change in `~/.wezterm.lua`:
```lua
-- example:
config.color_scheme = 'Tokyo Night'  -- or Nord, Dracula, etc.
```

## Tips

- Disable transparency if performance issues arise: `window_background_opacity = 1.0`
