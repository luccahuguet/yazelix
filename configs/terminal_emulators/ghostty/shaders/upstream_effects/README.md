# Vendored Ghostty Cursor Effect Templates

These shader templates are vendored from:

- https://github.com/sahaj-b/ghostty-cursor-shaders

Yazelix owns the integration layer on top of them:

- `settings.trail` selects the Yazelix color palette
- `settings.trail_effect` selects the Ghostty cursor-movement effect
- `settings.mode_effect` selects the Ghostty mode-change effect
- `settings.glow` scales the generated effect blur and spread so `none | low | medium | high` follows the same glow contract as Yazelix trail shaders
- `settings.duration` scales cursor-movement trail duration while leaving mode-change effects at their tuned timing
- `build_shaders.nu` keeps these templates color-agnostic and generates the final shader files under `generated_effects/`

The upstream README states the source repository is MIT licensed.
