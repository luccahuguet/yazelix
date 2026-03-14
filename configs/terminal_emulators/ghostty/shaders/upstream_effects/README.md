# Vendored Ghostty Cursor Effect Templates

These shader templates are vendored from:

- https://github.com/sahaj-b/ghostty-cursor-shaders

Yazelix owns the integration layer on top of them:

- `ghostty_trail_color` selects the Yazelix color palette
- `ghostty_trail_effect` selects the Ghostty cursor-movement effect
- `ghostty_mode_effect` selects the Ghostty mode-change effect
- `build_shaders.nu` keeps these templates color-agnostic and generates the final shader files under `generated_effects/`

The upstream README states the source repository is MIT licensed.
