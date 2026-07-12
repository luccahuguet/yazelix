# Styling and Themes

## Native Appearance Ownership

The semantic root does not synchronize appearance across tools. Mars, Zellij, Yazi, and Helix each own their native appearance settings under `~/.config/yazelix/`.

## Mars Transparency

Set Mars opacity in `~/.config/yazelix/mars/config.toml`:

```toml
[window]
opacity = 0.9
opacity-cells = false
```

Other terminal emulators keep their own native transparency and color settings

## Helix Themes

Recommended transparent theme:
```toml
# ~/.config/yazelix/helix/config.toml
theme = "term16_dark"
```

Alternative: `base16_transparent`

Popular non-transparent themes: `ao`, `dark_plus`, `onedark`, `gruvbox`, `catppuccin_mocha`

Custom theme TOML files for Yazelix-managed Helix sessions live under `~/.config/yazelix/helix/themes/`, and the selected theme name belongs in `~/.config/yazelix/helix/config.toml`. Native `~/.config/helix/themes/` belongs to plain Helix outside Yazelix, and the old `~/.config/yazelix/user_conf/helix/themes/` path is unsupported legacy state.

## Tips

- Disable Mars transparency with `window.opacity = 1.0`; use the equivalent native option for host terminals
