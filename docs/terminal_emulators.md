# Terminal Emulator Compatibility

Data summarized from:
- https://tmuxai.dev/terminal-compatibility/
- https://terminaltrove.com/terminals/
- https://github.com/luccahuguet/yazelix-terminal
- https://github.com/orhun/ratty

## Summary Table

Score rubric (1–10):
- Platforms: +0 to +3 (one point each for Linux/macOS/Windows)
- GPU acceleration: +1 if Yes
- Image protocol support: +1 if Yes
- Sixel support: +1 if Yes
- Open source: +1 if Yes
- Graphics protocol coverage: +1 if tmuxai lists Kitty graphics or “all image protocols”
- Implementation: +1 if written in Rust or Zig
- Ligature support: +1 if tmuxai strengths mention ligatures

| Terminal | Platforms (TerminalTrove) | Language (TerminalTrove) | GPU accel | Image protocol | Sixel | Source | Score |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Ghostty | macOS, Linux | Zig | Yes | Yes | No | Open Source (MIT) | 7 |
| Yazelix Terminal | Linux, macOS | Rust | Yes | Yes | Yes | Open Source | 8 |
| WezTerm | Linux, macOS, Windows | Rust | Yes | Yes | Yes | Open Source (MIT) | 9 |
| Ratty | Linux | Rust | Yes | Yes | No | Open Source | 5 |
| Kitty | Linux, macOS | Python | Yes | Yes | No | Open Source (GPL-3) | 7 |

## Ghostty
- Platforms: macOS, Linux (TerminalTrove)
- Language: Zig (TerminalTrove)
- Hardware acceleration: Yes (TerminalTrove)
- Image protocol support: Yes; Sixel: No (TerminalTrove)
- Source: Open Source (MIT) (TerminalTrove)
- Summary: Fast, feature-rich GPU-accelerated terminal written in Zig with native platform integration. (tmuxai)
- Strengths: Exceptional performance; native tabs/splits; Kitty graphics protocol; low latency. (tmuxai)
- Gaps: No Windows support; no Sixel; newer project (less ecosystem). (tmuxai)

Yazelix uses Ghostty as the default packaged terminal and pins temporary Zellij/Yazi forks so Yazi image previews can use Kitty graphics through Zellij. Those forks should be dropped and archived once upstream Zellij supports the required Kitty graphics path directly enough for the normal upstream packages to replace them.

## Yazelix Terminal
- Platforms: Linux and macOS through the Nix flake systems; Windows remains outside the Yazelix package path
- Language: Rust
- Hardware acceleration: Yes; Rio-derived renderer stack
- Image protocol support: Yes, including Kitty graphics and other modern protocol work carried by the fork
- Source: Open Source
- Summary: Experimental first-party Yazelix terminal path based on Rio, packaged through the `yazelix-terminal` child repository.
- Strengths: Ghostty-style cursor shader presets; generated Yazelix config; `terminal.transparency` support; child-owned launcher wrapper; intended as the long-term terminal Yazelix can evolve directly.
- Gaps: Experimental; smaller real-world validation than Ghostty and WezTerm; Yazelix still treats it as opt-in until event-mode responsiveness, graphics previews, and cursor shader behavior have more soak time.

Yazelix exposes Yazelix Terminal as `#yazelix_terminal`, `#runtime_yazelix_terminal`, and `programs.yazelix.runtime_variant = "yazelix_terminal"`. Launch goes through the child-owned `yazelix-terminal-desktop` wrapper. Yazelix materializes a generated `config.toml` from the packaged child config and injects the selected `terminal.transparency` value before launch.

## WezTerm
- Platforms: Linux, macOS, Windows (TerminalTrove)
- Language: Rust (TerminalTrove)
- Hardware acceleration: Yes (TerminalTrove)
- Image protocol support: Yes; Sixel: Yes (TerminalTrove)
- Source: Open Source (MIT) (TerminalTrove)
- Summary: GPU-accelerated terminal with built-in multiplexer and powerful Lua scripting configuration. (tmuxai)
- Strengths: Supports all image protocols; Lua scripting config; built-in multiplexer; cross-platform. (tmuxai)
- Gaps: Higher memory usage; steep config learning curve; larger binary size. (tmuxai)

## Ratty
- Platforms: Linux in nixpkgs
- Language: Rust
- Hardware acceleration: Yes; built on Bevy and wgpu
- Image protocol support: Yes, via Kitty Graphics Protocol
- Source: Open Source
- Summary: Experimental GPU-rendered terminal emulator with inline 3D graphics and a 2D/3D mode toggle
- Strengths: Kitty graphics support; generated TOML config; distinctive inline 3D workflow; available as `#yazelix_ratty` on Linux
- Gaps: Linux-only in nixpkgs; young terminal with a smaller production track record; Yazelix does not claim Ratty Graphics Protocol passthrough inside Zellij

Yazelix exposes Ratty as an experimental Linux packaged runtime. Ratty can use the same Yazelix Zellij/Yazi Kitty graphics bridge as Ghostty for image previews, but Ratty's own 3D object protocol remains terminal-native behavior outside the current Zellij integration contract.

## Kitty
- Platforms: Linux, macOS (TerminalTrove)
- Language: Python (TerminalTrove)
- Hardware acceleration: Yes (TerminalTrove)
- Image protocol support: Yes; Sixel: No (TerminalTrove)
- Source: Open Source (GPL-3) (TerminalTrove)
- Summary: Fast, feature-rich GPU-based terminal with its own superior graphics protocol. (tmuxai)
- Strengths: Kitty graphics protocol (best for images); GPU-accelerated; extensible via kittens; full ligatures. (tmuxai)
- Gaps: No Windows support; no Sixel (uses own protocol); learning curve for config. (tmuxai)
