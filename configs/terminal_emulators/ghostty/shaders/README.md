# Cursor Trail Shaders

This directory contains cursor trail shaders for Ghostty terminal.

## Structure

The cursor trail shaders are built from modular source files:

```
shaders/
├── cursor_trail_common.glsl     # Shared functions
├── variants/                     # Variant-specific code (3-60 lines each)
│   ├── blaze.glsl
│   ├── white.glsl
│   ├── sunset.glsl
│   ├── ocean.glsl
│   ├── forest.glsl
│   ├── cosmic.glsl
│   ├── neon.glsl
│   ├── eclipse.glsl
│   ├── dusk.glsl
│   ├── orchid.glsl
│   ├── reef.glsl
│   └── magma.glsl
├── build_shaders.nu             # Build script (nushell, runs automatically)
└── cursor_trail_*.glsl          # Generated locally/runtime only (gitignored)
```

## How It Works

Yazelix copies the shader sources into the runtime Ghostty shader directory, runs
`build_shaders.nu`, and then writes data-driven cursor variants from
`yazelix_cursors.toml`. Hand-tuned variants remain in `variants/`, while `mono`
and `split` presets are rendered from cursor registry data.

## Making Changes

### To modify shared functions:

1. Edit `cursor_trail_common.glsl`
2. Shaders will be **automatically rebuilt** next time Yazelix starts or configs are regenerated

### To modify a specific shader variant:

1. Edit the variant file in `variants/` directory (e.g., `variants/white.glsl`)
2. Shaders will be **automatically rebuilt** next time Yazelix starts or configs are regenerated

### To create a new variant:

1. Create a new file in `variants/` directory (e.g., `variants/new_variant.glsl`)
2. Add your variant-specific code (constants, helper functions, mainImage)
3. Add the cursor to `yazelix_cursors_default.toml` or your local `user_configs/yazelix_cursors.toml`
4. Shaders will be **automatically rebuilt** next time Yazelix starts or configs are regenerated

### Manual build (for testing or local preview):

```bash
nu build_shaders.nu medium
```

By default, that writes generated shaders into the normal runtime output tree:

```text
~/.local/share/yazelix/configs/terminal_emulators/ghostty/shaders
```

If you really want to write to a different destination, pass an explicit output directory:

```bash
nu build_shaders.nu medium /tmp/ghostty_shader_preview
```

## Build Process

The build is **fully automatic**:
- Runs during Yazelix startup when terminal configs are generated
- Combines `cursor_trail_common.glsl` with each variant in `variants/`
- Outputs complete shaders ready for Ghostty to use
- No manual intervention needed!
- Honors `settings.glow = none | low | medium | high` from `yazelix_cursors.toml`
- Honors `settings.duration = 0.25..4.0` from `yazelix_cursors.toml` as a multiplier for movement-trail timing

## Important Notes

- **DO NOT directly edit** the generated `cursor_trail_*.glsl` files - your changes will be overwritten
- **ALWAYS edit** either `cursor_trail_common.glsl` or files in `variants/`
- Shaders are **automatically built** during Yazelix startup - no manual steps needed!
- The generated shaders are **not** git-tracked; the maintained source is the common library, variants, and build script
- The manual build command defaults to the runtime output directory so it does not dirty the source tree

## Variant Categories

### Mono (6 data-driven presets)
- `blaze`, `snow`, `sunset`, `ocean`, `forest`, `cosmic`
- Each preset defines one base color in `yazelix_cursors_default.toml`; Yazelix derives the accent unless `accent_color` overrides it

### Split (5 data-driven presets)
- `eclipse`, `dusk`, `orchid`, `reef`, `magma`
- Each preset defines two colors plus `direction = "vertical" | "horizontal"` and `blend = true | false`

### Curated Template (1 variant)
- `neon`
- Keeps hand-tuned shader logic selected by `template = "neon"`
