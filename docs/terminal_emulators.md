# Terminal Emulator Comparison

Yazelix ships with multiple terminals so users can match platform needs and personal preferences. The table below summarizes how each option fits into the current stack and highlights what still needs work.

| Category | **WezTerm** | **Ghostty** | **Kitty** | **Alacritty** | **foot** |
| --- | --- | --- | --- | --- | --- |
| Platforms | Linux, macOS, Windows 🏆 | macOS, Linux (Wayland & X11) **(default)** | Linux, macOS | Linux, macOS, Windows 🏆 | Linux (Wayland) |
| yazi-image-preview | **Works properly** (current best inside Zellij) 🏆 | Blurry/unsupported in Zellij (no Sixel) | Blurry/unsupported in Zellij (needs Sixel) | N/A (no image protocol) | **Untested** (Sixel present; not validated) |
| Graphics protocols | Kitty Graphics **and** Sixel 🏆 | Kitty Graphics only | Kitty Graphics | None (no Kitty Graphics/Sixel) | Sixel |
| Ligature support | Full OpenType ligatures with fallback 🏆 | Full ligature shaping (Harfbuzz) | Full ligature shaping | No ligatures | Full ligature shaping (Harfbuzz) |
| Cursor shaders (cursor trail) | No | **Yes** (shader-based trails, all 8 colors) 🏆 | **Yes** (`cursor_trail` presets, snow only) 🏆 | No | No |
| Startup speed | Fast | Very fast 🏆 | Fast | Very fast 🏆 | Very fast 🏆 |
| Render speed | very fast  | blazing 🏆 | very fast  | okay | very fast  |
| 🏆 Score | 4 | 3 | 1 | 2 | 1 |

## Qualitative deep dive

| Category | **WezTerm** | **Ghostty** | **Kitty** | **Alacritty** | **foot** |
| --- | --- | --- | --- | --- | --- |
| SSH (latency/remote UX) | Excellent over SSH; smooth scrollback; resize stable | Very good; fast input echo; stable resize | Very good; solid remote feel | Good; minimal features but stable | Good; lightweight, snappy on weak links |
| Nix size (bundled) | Medium-large | Medium | Medium | Small | **Tiny** |
| nixGL / GPU | Reliable with nixGL (GPU accel) | Works with nixGL; shaders OK (Wayland/X11) | Works with nixGL (OpenGL) | Works with nixGL (OpenGL) | **No nixGL needed** (Wayland, very light deps) |
| Unicode support (emoji/CJK/ligatures) | **Excellent** fallback & shaping | Very good | Very good | Good (fallback depends on fonts) | Good |
| Extras | Lua-config automation; per-domain profiles | Quick Terminal (Wayland layer-shell); server-side decorations toggle | “Kitten” tools & remote-control API | Plain TOML, low deps, vi-mode selection | `foot`/`footclient` server-client model; fast built-in search |

## Foot evaluation notes

- Packaging: yet another maintenance burden, but could bring more options to the table. What if it's the fastest terminal? We will know after running benchmarks
- Image previews: should be verified once packaged to confirm Sixel works with Zellij + Yazi. As currently only Wezterm shows image previews on yazi under zellij correctly, that would be a nice thing to have.
- Platform coverage: No macos is unfortunate
- Conclusion: merge first, on main or a separate branch, and test for a while

