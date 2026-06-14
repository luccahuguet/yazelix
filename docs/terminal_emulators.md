# Terminal Emulator Comparison

This document compares the terminal emulators Yazelix currently scores:
Ghostty, Yazelix Terminal, Rio, WezTerm, Ratty, and Kitty. Foot is restored as
a Linux-only packaged terminal variant, but its detailed feature score needs a
fresh validation pass.

Data summarized from:

- https://github.com/luccahuguet/yazelix-terminal
- https://github.com/luccahuguet/yazelix-terminal/blob/main/docs/yazelix/fork_feature_verification.md
- https://github.com/luccahuguet/yazelix-terminal/blob/main/docs/yazelix/frontier_kitty_protocols.md
- https://ghostty.org/docs/config/reference
- https://github.com/raphamorim/rio
- https://wezterm.org/features.html
- https://codeberg.org/dnkl/foot
- https://sw.kovidgoyal.net/kitty/protocol-extensions/
- https://sw.kovidgoyal.net/kitty/graphics-protocol/
- https://sw.kovidgoyal.net/kitty/keyboard-protocol/
- https://github.com/orhun/ratty
- Local reference clones under `/home/lucca/pjs/open_source/yazelix_related/`

Alacritty is intentionally omitted from the current comparison because it is no
longer part of the maintained Yazelix terminal set. Foot is restored as a
Linux-only packaged terminal variant; detailed feature scoring should be
refreshed after live Yazelix validation.

## Scoring

The comparison score is 100 points total.

- 25 criteria
- 4 points per criterion (`100 / 25 = 4`)
- `Yes` = 4 points
- `Partial` = 2 points
- `No` or unknown = 0 points

Implementation language and terminal-native multiplexer overlap are deliberately
out of the score. They can matter when maintaining or packaging a terminal, but
they are not feature/protocol capabilities by themselves.

| Terminal | Score | Full | Partial | No | Read |
| --- | ---: | ---: | ---: | ---: | --- |
| Yazelix Terminal | 94 | 23 | 1 | 1 | Best feature/protocol coverage and first-party control; still experimental |
| Kitty | 76 | 18 | 2 | 5 | Strong protocol reference and packaged alternate; no first-party control |
| Ghostty | 60 | 15 | 0 | 10 | Best mature default today; strongest shader story; fewer Kitty frontier protocols |
| Rio | 48 | 11 | 2 | 12 | Upstream Rio path with modern image/protocol support; less Yazelix soak than Ghostty |
| WezTerm | 48 | 11 | 2 | 12 | Stable alternate with broad image support; fewer modern Kitty extensions |
| Ratty | 28 | 6 | 2 | 17 | Experimental but uniquely interesting because of inline 3D graphics |

## Criteria

| ID | Criterion | Weight | What Counts |
| --- | --- | ---: | --- |
| C1 | Packaged Yazelix runtime | 4 | Available as a first-class Yazelix package/runtime variant |
| C2 | First-party control path | 4 | Yazelix owns the fork or can directly evolve the terminal behavior |
| C3 | Generated config and transparency | 4 | Yazelix can materialize config and map `terminal.transparency` into it |
| C4 | Runtime launcher integration | 4 | Launch path is modeled by Yazelix rather than only PATH discovery |
| C5 | GPU renderer | 4 | Terminal renders through a GPU-accelerated stack |
| C6 | Production confidence | 4 | Mature enough to recommend broadly as a daily driver |
| C7 | Yazelix stack validation | 4 | Evidence exists for Yazelix/Zellij/Yazi/Helix behavior, especially graphics |
| C8 | Ghostty-style cursor shaders | 4 | Supports Ghostty-compatible shader/trail behavior, not just cursor color |
| C9 | Kitty graphics | 4 | Supports Kitty Graphics Protocol image placement |
| C10 | Sixel | 4 | Supports Sixel image rendering |
| C11 | iTerm2 images | 4 | Supports OSC 1337 iTerm2-style inline images |
| C12 | Kitty keyboard | 4 | Supports Kitty keyboard protocol |
| C13 | OSC 8 hyperlinks | 4 | Supports terminal hyperlinks |
| C14 | OSC 52 clipboard | 4 | Supports clipboard read/write policy through OSC 52 |
| C15 | OSC 133 semantic prompts | 4 | Supports prompt/input/output region markers |
| C16 | OSC 21 color control | 4 | Supports Kitty color control |
| C17 | OSC 22 pointer shapes | 4 | Supports Kitty pointer shape control |
| C18 | OSC 66 text sizing | 4 | Supports Kitty text sizing |
| C19 | OSC 99 notifications | 4 | Supports Kitty desktop notification protocol, not only OSC 9/777 |
| C20 | Kitty multiple cursors | 4 | Supports the Kitty multiple-cursor protocol |
| C21 | Kitty file transfer | 4 | Supports a safe OSC 5113 file-transfer runtime path |
| C22 | OSC 5522 text clipboard | 4 | Supports the text/plain slice of Kitty rich clipboard |
| C23 | Kitty DECCARA | 4 | Supports Kitty's all-SGR rectangular styling extension |
| C24 | Kitty unscrolling | 4 | Supports Kitty unscrolling |
| C25 | Inline 3D graphics | 4 | Supports terminal-native inline 3D, such as Ratty Graphics Protocol |

## Runtime And Integration

| Criterion | Ghostty | Yazelix Terminal | Rio | WezTerm | Ratty | Kitty |
| --- | --- | --- | --- | --- | --- | --- |
| C1 Packaged Yazelix runtime | Yes | Yes | Yes | Yes | Yes | Yes |
| C2 First-party control path | No | Yes | No | No | No | No |
| C3 Generated config and transparency | Yes | Yes | Yes | Yes | Yes | Yes |
| C4 Runtime launcher integration | Yes | Yes | Yes | Yes | Yes | Yes |
| C5 GPU renderer | Yes | Yes | Yes | Yes | Yes | Yes |
| C6 Production confidence | Yes | Partial | Partial | Yes | Partial | Yes |
| C7 Yazelix stack validation | Yes | Yes | Partial | Partial | Partial | Partial |

## Rendering And Images

| Criterion | Ghostty | Yazelix Terminal | Rio | WezTerm | Ratty | Kitty |
| --- | --- | --- | --- | --- | --- | --- |
| C8 Ghostty-style cursor shaders | Yes | Yes | No | No | No | Partial |
| C9 Kitty graphics | Yes | Yes | Yes | Yes | Yes | Yes |
| C10 Sixel | No | Yes | Yes | Yes | No | No |
| C11 iTerm2 images | No | Yes | Yes | Yes | No | No |
| C25 Inline 3D graphics | No | No | No | No | Yes | No |

Kitty receives partial credit for C8 because it has cursor trails, but not the
Ghostty-compatible shader ABI Yazelix wants. Ratty receives full credit for C25
because Ratty Graphics Protocol supports inline `.obj` and `.glb` objects.

## Core Protocols

| Criterion | Ghostty | Yazelix Terminal | Rio | WezTerm | Ratty | Kitty |
| --- | --- | --- | --- | --- | --- | --- |
| C12 Kitty keyboard | Yes | Yes | Yes | Partial | No | Yes |
| C13 OSC 8 hyperlinks | Yes | Yes | Yes | Yes | No | Yes |
| C14 OSC 52 clipboard | Yes | Yes | Yes | Yes | No | Yes |
| C15 OSC 133 semantic prompts | Yes | Yes | No | Yes | No | No |
| C16 OSC 21 color control | Yes | Yes | No | No | No | Yes |
| C17 OSC 22 pointer shapes | Yes | Yes | Yes | No | No | Yes |
| C18 OSC 66 text sizing | Yes | Yes | No | No | No | Yes |
| C19 OSC 99 notifications | No | Yes | No | No | No | Yes |

WezTerm receives partial credit for C12 because Kitty keyboard support exists but
is opt-in through `enable_kitty_keyboard`. Ghostty supports notification escape
paths, but not the Kitty OSC 99 protocol surface scored here.

## Frontier Kitty Protocols

| Criterion | Ghostty | Yazelix Terminal | Rio | WezTerm | Ratty | Kitty |
| --- | --- | --- | --- | --- | --- | --- |
| C20 Kitty multiple cursors | No | Yes | No | No | No | Yes |
| C21 Kitty file transfer | No | Yes | No | No | No | Yes |
| C22 OSC 5522 text clipboard | No | Yes | No | No | No | Yes |
| C23 Kitty DECCARA | No | Yes | No | No | No | Yes |
| C24 Kitty unscrolling | No | Yes | No | No | No | Yes |

Yazelix Terminal intentionally scores only the text/plain OSC 5522 clipboard
slice. Full arbitrary-MIME OSC 5522 remains frontier work because it needs a
real platform clipboard provider that preserves MIME types. The current Ghostty
source parses OSC 5522, but Yazelix Terminal's verification ledger treats
parser-only behavior as no runtime support.

## Terminal Notes

### Ghostty

Ghostty remains the best mature default for Yazelix today. It has excellent
shader support, strong Kitty graphics support, OSC 133 shell integration, Kitty
keyboard, and a stable daily-driver posture. Its lower score comes from this
comparison weighting Sixel, iTerm2 images, OSC 99, and newer Kitty frontier
protocols heavily.

### Yazelix Terminal

Yazelix Terminal is the protocol-forward path. It starts from Rio and adds or
validates Ghostty-compatible cursor shaders, Yazelix host mode, event-mode
cursor animation, Kitty graphics, Sixel, iTerm2 images, OSC 133, OSC 66, OSC 99,
OSC 52, OSC 21, OSC 22, Kitty keyboard, Kitty multiple cursors, safe Kitty file
transfer, OSC 5522 text clipboard, DECCARA, and unscrolling.
For Yazelix workflows, that already puts it ahead of vanilla Rio on protocol
coverage, cursor/shader integration, package metadata, BELL/notification
behavior, and stack image-preview fixes.

Its main weakness is not protocol ambition. The risk is release maturity:
Wayland/windowing/GPU behavior, packaged desktop launch, input responsiveness,
graphics previews, and cursor shader alignment need continued soak time.
The experimental release closeout is recorded in
`yazelix-terminal` at `docs/yazelix/release_closeout_2026_06.md`.

### Rio

Rio is the upstream path behind Yazelix Terminal. Yazelix packages it as
`#yazelix_rio` for users who want upstream Rio with Yazelix-owned generated
config, launch integration, and the Zellij Kitty graphics bridge. The
packaged config enables Rio's native trail cursor, leaves renderer backend
selection to Rio's platform default, maps `terminal.transparency` to Rio's
supported window opacity setting, and points Rio at packaged Nerd Font and
emoji fallback directories.
It supports modern image paths and several useful OSC protocols, but Yazelix
does not control its roadmap and does not apply Yazelix Terminal's cursor shader
profile behavior to the vanilla package.

### WezTerm

WezTerm is the conservative stable alternate. It is packaged, cross-platform,
has strong image support across Kitty graphics, Sixel, and iTerm2 images, and is
a proven daily terminal. Its weaker score reflects the newer Kitty protocol
extensions it does not expose today and the lack of Ghostty-style cursor shader
parity.

### Ratty

Ratty is not trying to be the safest all-purpose terminal in this comparison.
Its value is that it proves a different frontier: GPU-rendered terminal UI plus
inline 3D objects through Ratty Graphics Protocol. Yazelix packages it on Linux
as an experimental runtime and can use the Yazelix Zellij Kitty graphics
bridge, but Yazelix does not claim RGP passthrough inside Zellij.

### Kitty

Kitty is the protocol reference. It leads on Kitty graphics, keyboard handling,
desktop notifications, multiple cursors, file transfer, text sizing, pointer
shape, unscrolling, rich clipboard, and related protocol extensions. Its lower
score than Yazelix Terminal comes from first-party control and Yazelix-owned
cursor shader behavior, not from terminal capability. Yazelix can package Kitty
as the `#yazelix_kitty` runtime variant while still supporting host
PATH-provided Kitty.
