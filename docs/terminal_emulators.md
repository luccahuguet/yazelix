# Terminal Emulator Compatibility

Data summarized from:
- https://tmuxai.dev/terminal-compatibility/
- https://terminaltrove.com/terminals/

## Summary Table

Score rubric (1–10):
- Platforms: +0 to +3 (one point each for Linux/macOS/Windows)
- GPU acceleration: +1 if Yes
- Image protocol support: +1 if Yes
- Sixel support: +1 if Yes
- Open source: +1 if Yes
- Graphics protocol coverage: +1 if tmuxai lists Kitty graphics or “all image protocols”
- Built-in multiplexing: +1 if tmuxai lists native tabs/splits or built-in multiplexer
- Implementation: +1 if written in Rust or Zig

| Terminal | Platforms (TerminalTrove) | Language (TerminalTrove) | GPU accel | Image protocol | Sixel | Source | Score |
| --- | --- | --- | --- | --- | --- | --- |
| Ghostty | macOS, Linux | Zig | Yes | Yes | No | Open Source (MIT) | 8 |
| WezTerm | Linux, macOS, Windows | Rust | Yes | Yes | Yes | Open Source (MIT) | 10 |
| Kitty | Linux, macOS | Python | Yes | Yes | No | Open Source (GPL-3) | 6 |
| Alacritty | Linux, macOS, Windows | Rust | Yes | Yes | No | Open Source (Apache 2.0) | 7 |
| Foot | Linux | C | No | Yes | Yes | Open Source (MIT) | 4 |

## Ghostty
- Platforms: macOS, Linux (TerminalTrove)
- Language: Zig (TerminalTrove)
- Hardware acceleration: Yes (TerminalTrove)
- Image protocol support: Yes; Sixel: No (TerminalTrove)
- Source: Open Source (MIT) (TerminalTrove)
- Summary: Fast, feature-rich GPU-accelerated terminal written in Zig with native platform integration. (tmuxai)
- Strengths: Exceptional performance; native tabs/splits; Kitty graphics protocol; low latency. (tmuxai)
- Gaps: No Windows support; no Sixel; newer project (less ecosystem). (tmuxai)

## WezTerm
- Platforms: Linux, macOS, Windows (TerminalTrove)
- Language: Rust (TerminalTrove)
- Hardware acceleration: Yes (TerminalTrove)
- Image protocol support: Yes; Sixel: Yes (TerminalTrove)
- Source: Open Source (MIT) (TerminalTrove)
- Summary: GPU-accelerated terminal with built-in multiplexer and powerful Lua scripting configuration. (tmuxai)
- Strengths: Supports all image protocols; Lua scripting config; built-in multiplexer; cross-platform. (tmuxai)
- Gaps: Higher memory usage; steep config learning curve; larger binary size. (tmuxai)

## Kitty
- Platforms: Linux, macOS (TerminalTrove)
- Language: Python (TerminalTrove)
- Hardware acceleration: Yes (TerminalTrove)
- Image protocol support: Yes; Sixel: No (TerminalTrove)
- Source: Open Source (GPL-3) (TerminalTrove)
- Summary: Fast, feature-rich GPU-based terminal with its own superior graphics protocol. (tmuxai)
- Strengths: Kitty graphics protocol (best for images); GPU-accelerated; extensible via kittens; full ligatures. (tmuxai)
- Gaps: No Windows support; no Sixel (uses own protocol); learning curve for config. (tmuxai)

## Alacritty
- Platforms: Linux, macOS, Windows (TerminalTrove)
- Language: Rust (TerminalTrove)
- Hardware acceleration: Yes (TerminalTrove)
- Image protocol support: Yes; Sixel: No (TerminalTrove)
- Source: Open Source (Apache 2.0) (TerminalTrove)
- Summary: Minimalist, blazing fast GPU-accelerated terminal emulator focused on performance and simplicity. (tmuxai)
- Strengths: Fastest terminal emulator; minimal resource usage (~30MB); cross-platform; simple TOML config. (tmuxai)
- Gaps: No ligatures (by design); no graphics protocols; no built-in tabs/splits. (tmuxai)

## Foot
- Platforms: Linux (TerminalTrove)
- Language: C (TerminalTrove)
- Hardware acceleration: No (TerminalTrove)
- Image protocol support: Yes; Sixel: Yes (TerminalTrove)
- Source: Open Source (MIT) (TerminalTrove)
