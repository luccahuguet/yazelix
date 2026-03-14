# Vendored Ghostty Cursor Effect Templates

These shader templates are vendored from:

- https://github.com/sahaj-b/ghostty-cursor-shaders

Yazelix owns the integration layer on top of them:

- `cursor_trail` selects the Yazelix color palette
- `ghostty_cursor_effects_random` / `ghostty_cursor_effects` select the Ghostty effect stack
- `build_shaders.nu` injects the selected palette color into these templates and generates the final shader files under `generated_effects/`

The upstream README states the source repository is MIT licensed.
