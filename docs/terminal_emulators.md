# Terminal Emulator Comparison

Yazelix ships with multiple terminals so users can match platform needs and personal preferences. The table below summarizes how each option fits into the current stack and highlights what still needs work.

| Category | **WezTerm** | **Ghostty** | **Kitty** | **Alacritty** | **foot** |
| --- | --- | --- | --- | --- | --- |
| Platforms | Linux, macOS, Windows 🏆 | macOS, Linux (Wayland & X11) **(default)** | Linux, macOS | Linux, macOS, Windows 🏆 | Linux (Wayland) |
| yazi-image-preview | **Works properly** (current best inside Zellij) 🏆 | Blurry/unsupported in Zellij (no Sixel) | Blurry/unsupported in Zellij (needs Sixel) | N/A (no image protocol) | **Untested** (Sixel present; not validated) |
| Graphics protocols | Kitty Graphics **and** Sixel 🏆 | Kitty Graphics only | Kitty Graphics | None (no Kitty Graphics/Sixel) | Sixel |
| Cursor shaders (cursor trail) | No | **Yes** (shader-based trails, all 8 colors) 🏆 | **Yes** (`cursor_trail` presets, snow only) 🏆 | No | No |
| Startup speed | Fast | Very fast 🏆 | Fast | Very fast 🏆 | Very fast 🏆 |
| Render speed (5-tier) | **very fast** 🏆 | **blazing** 🏆 | **very fast** 🏆 | **okay** | **very fast** 🏆 |
| 🏆 Score | 4 | 4 | 2 | 2 | 2 |

## Qualitative deep dive

| Category | **WezTerm** | **Ghostty** | **Kitty** | **Alacritty** | **foot** |
| --- | --- | --- | --- | --- | --- |
| SSH (latency/remote UX) | Excellent over SSH; smooth scrollback; resize stable | Very good; fast input echo; stable resize | Very good; solid remote feel | Good; minimal features but stable | Good; lightweight, snappy on weak links |
| Nix size (bundled) | Medium-large | Medium | Medium | Small | **Tiny** |
| nixGL / GPU | Reliable with nixGL (GPU accel) | Works with nixGL; shaders OK (Wayland/X11) | Works with nixGL (OpenGL) | Works with nixGL (OpenGL) | **No nixGL needed** (Wayland, very light deps) |
| Unicode support (emoji/CJK/ligatures) | **Excellent** fallback & shaping | Very good | Very good | Good (fallback depends on fonts) | Good |
| Extras | Lua-config automation; per-domain profiles | Quick Terminal (Wayland layer-shell); server-side decorations toggle | “Kitten” tools & remote-control API | Plain TOML, low deps, vi-mode selection | `foot`/`footclient` server-client model; fast built-in search |

## Foot evaluation notes

- Packaging: we would need to add `foot` plus its shader/fonts story to the devshell; currently nothing in `yazelix_default.nix` or Home Manager module handles it.
- Configuration: Yazelix relies on generated configs per terminal. foot would require a new template plus logic for `terminal_config_mode` handling.
- Multiplexing: lacks native tabbing, so Yazelix would lean on Zellij even more; acceptable, but we should document it explicitly when shipping support.
- Image previews: should be verified once packaged to confirm Sixel works with Zellij + Yazi.
- Platform coverage: pure Wayland means we must gate enablement where Ghostty/WezTerm cover X11 or macOS; documentation should steer users accordingly.

These gaps are the blockers to making foot a first-class option. Once the packaging and config automation land, revisit this table to update its status.
