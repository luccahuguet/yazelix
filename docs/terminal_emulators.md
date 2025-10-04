# Terminal Emulator Comparison

Yazelix ships with multiple terminals so users can match platform needs and personal preferences. The table below summarizes how each option fits into the current stack and highlights what still needs work.

| Terminal | Platforms | Yazelix status | yazi-image-preview | Graphics protocols | Cursor shaders (cursor trail) | Multiplexing | Performance (qualitative) | Windows support | Current gaps |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| **WezTerm** | Linux, macOS, Windows | Optional (`preferred_terminal`, `extra_terminals`) | **Works properly** (current best inside Zellij) | Kitty Graphics **and** Sixel | No | Irrelevant (Zellij handles) | Fast start; smooth scroll; stable previews | Full native | Heavier binary than Kitty/Alacritty |
| **Ghostty** | macOS, Linux (Wayland & X11) | **Default**; bundled with nixGL | Blurry/unsupported in Zellij (no Sixel) | Kitty Graphics only | **Yes** (shader-based trails) | Irrelevant (Zellij handles) | Very fast start; smooth render | Native on macOS/Linux | yazi previews unreliable under Zellij; Quick Terminal is Wayland-only |
| **Kitty** | Linux, macOS | Optional (`preferred_terminal`, `extra_terminals`) | Blurry/unsupported in Zellij (needs Sixel) | Kitty Graphics | **Yes** (`cursor_trail` presets) | Irrelevant (Zellij handles) | Fast GPU render | **No native Windows** (WSL only) | Tabs not embeddable in Yazelix layouts |
| **Alacritty** | Linux, macOS, Windows | Optional (`preferred_terminal`, `extra_terminals`) | N/A (no image protocol) | None (no Kitty Graphics/Sixel) | No | Irrelevant (Zellij handles) | Very small & quick | Full native | Fewer UX niceties; use Zellij for mux |
| **foot** | Linux (Wayland) | Under evaluation (not yet packaged) | **Untested** (Sixel present; not validated) | Sixel | No | Irrelevant (Zellij handles) | Extremely light | N/A | Wayland-only; needs packaging + config template before bundling |

## Foot evaluation notes

- Packaging: we would need to add `foot` plus its shader/fonts story to the devshell; currently nothing in `yazelix_default.nix` or Home Manager module handles it.
- Configuration: Yazelix relies on generated configs per terminal. foot would require a new template plus logic for `terminal_config_mode` handling.
- Multiplexing: lacks native tabbing, so Yazelix would lean on Zellij even more; acceptable, but we should document it explicitly when shipping support.
- Image previews: should be verified once packaged to confirm Sixel works with Zellij + Yazi.
- Platform coverage: pure Wayland means we must gate enablement where Ghostty/WezTerm cover X11 or macOS; documentation should steer users accordingly.

These gaps are the blockers to making foot a first-class option. Once the packaging and config automation land, revisit this table to update its status.
