# Terminal Emulator Comparison

Yazelix ships with multiple terminals so users can match platform needs and personal preferences. The table below summarizes how each option fits into the current stack and highlights what still needs work.

| Category | **WezTerm** | **Ghostty** | **Kitty** | **Alacritty** | **foot** |
| --- | --- | --- | --- | --- | --- |
| Platforms | Linux, macOS, Windows 🏆 | macOS, Linux (Wayland & X11) | Linux, macOS | Linux, macOS, Windows 🏆 | Linux (Wayland) |
| Yazelix status | Optional (`preferred_terminal`, `extra_terminals`) | **Default**; bundled with nixGL 🏆 | Optional (`preferred_terminal`, `extra_terminals`) | Optional (`preferred_terminal`, `extra_terminals`) | Under evaluation (not yet packaged) |
| yazi-image-preview | **Works properly** (current best inside Zellij) 🏆 | Blurry/unsupported in Zellij (no Sixel) | Blurry/unsupported in Zellij (needs Sixel) | N/A (no image protocol) | **Untested** (Sixel present; not validated) |
| Graphics protocols | Kitty Graphics **and** Sixel 🏆 | Kitty Graphics only | Kitty Graphics | None (no Kitty Graphics/Sixel) | Sixel |
| Cursor shaders (cursor trail) | No | **Yes** (shader-based trails, all 8 colors) 🏆 | **Yes** (`cursor_trail` presets, snow only) 🏆 | No | No |
| Startup - bundled | Fast | Very fast 🏆 | Fast | Very fast 🏆 | Very fast 🏆 |
| Render - bundled | Fast 🏆 | Fast 🏆 | Fast 🏆 | Average | Fast 🏆 |
| 🏆 Score | 4 | 4 | 2 | 2 | 2 |

## Qualitative deep dive

| Category | **WezTerm** | **Ghostty** | **Kitty** | **Alacritty** | **foot** |
| --- | --- | --- | --- | --- | --- |
| SSH (latency/remote UX) | Excellent over SSH; smooth scrollback; resize stable | Very good; fast input echo; stable resize | Very good; solid remote feel | Good; minimal features but stable | Good; lightweight, snappy on weak links |
| Nix size (bundled) | Medium-large | Medium | Medium | Small | **Tiny** |
| nixGL / GPU | Works reliably with nixGL (GPU accel) | Works with nixGL; shaders OK (Wayland/X11) | Works with nixGL (OpenGL) | Works with nixGL (OpenGL) | **No nixGL needed** (software render; Wayland) |
| Unicode support (emoji/CJK/ligatures) | **Excellent** font fallback & shaping | Very good | Very good | Good (fallback depends on fonts) | Good |
| Extras | Built-in mux & SSH “domains”; great font fallback | Quick Terminal (Wayland); cursor shaders (cursor trail) | `icat`/Kitty Graphics; remote control; cursor trail | Minimalist; easy to script; very small footprint | Sixel built-in; ultra-light; Wayland-native |

## Foot evaluation notes

- Packaging: we would need to add `foot` plus its shader/fonts story to the devshell; currently nothing in `yazelix_default.nix` or Home Manager module handles it.
- Configuration: Yazelix relies on generated configs per terminal. foot would require a new template plus logic for `terminal_config_mode` handling.
- Multiplexing: lacks native tabbing, so Yazelix would lean on Zellij even more; acceptable, but we should document it explicitly when shipping support.
- Image previews: should be verified once packaged to confirm Sixel works with Zellij + Yazi.
- Platform coverage: pure Wayland means we must gate enablement where Ghostty/WezTerm cover X11 or macOS; documentation should steer users accordingly.

These gaps are the blockers to making foot a first-class option. Once the packaging and config automation land, revisit this table to update its status.
