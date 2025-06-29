# Styling and Themes

## Terminal Transparency

WezTerm includes transparency by default:
```lua
config.window_background_opacity = 0.9
```

Edit `~/.wezterm.lua` to customize (1.0 = opaque, 0.5 = very transparent, etc).

## Helix Themes

Recommended transparent theme:
```toml
# ~/.config/helix/config.toml
theme = "term16_dark"
```

Alternative: `base16_transparent`

Popular non-transparent themes: `ao`, `dark_plus`, `onedark`, `gruvbox`, `catppuccin_mocha`

## WezTerm Color Schemes

Default: `Abernathy`

Change in `~/.wezterm.lua`:
```lua
-- example:
config.color_scheme = 'Tokyo Night'  -- or Nord, Dracula, etc.
```

## Tips

- Disable transparency if performance issues arise: `window_background_opacity = 1.0`

