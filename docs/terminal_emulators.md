# Terminal Emulator Comparison

This document compares the terminal emulators Yazelix currently scores:
Mars, Ghostty, Rio, WezTerm, Ratty, Kitty, and Foot. Yazelix supports capable
terminals through `yzx enter`; Mars is the packaged terminal, and Ghostty is the
most tested mature host-terminal path.

Data summarized from:

- https://ghostty.org/docs/config/reference
- https://github.com/luccahuguet/mars
- https://github.com/raphamorim/rio
- https://wezterm.org/features.html
- https://codeberg.org/dnkl/foot
- https://sw.kovidgoyal.net/kitty/protocol-extensions/
- https://sw.kovidgoyal.net/kitty/graphics-protocol/
- https://sw.kovidgoyal.net/kitty/keyboard-protocol/
- https://github.com/orhun/ratty
- Local reference clones under `/home/lucca/pjs/open_source/yazelix_related/`

Alacritty is omitted from the scored comparison because it is not part of the
terminal feature race Yazelix currently studies. It can still run the normal
Yazelix workspace through `yzx enter`. Foot is tracked as a Linux-focused
terminal entrypoint; detailed feature scoring should be refreshed after live
Yazelix validation.

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
| Kitty | 76 | 18 | 2 | 5 | Strong protocol reference and supported host-terminal entrypoint; no first-party control |
| Ghostty | 60 | 15 | 0 | 10 | Most tested mature host-terminal path; strongest shader story; strong macOS choice |
| Mars | 56 | 13 | 2 | 10 | Default because Yazelix owns the Rust fork and can optimize stack compatibility and agent-driven workflows |
| Rio | 48 | 11 | 2 | 12 | Upstream Rio path with modern image/protocol support; less Yazelix soak than Ghostty |
| WezTerm | 48 | 11 | 2 | 12 | Stable host-terminal path with broad image support; fewer modern Kitty extensions |
| Ratty | 28 | 6 | 2 | 17 | Experimental but uniquely interesting because of inline 3D graphics |

## Criteria

| ID | Criterion | Weight | What Counts |
| --- | --- | ---: | --- |
| C1 | Supported Yazelix entrypoint | 4 | Can run the normal Yazelix workspace through a maintained `yzx enter` path |
| C2 | First-party control path | 4 | Yazelix owns the fork or can directly evolve the terminal behavior |
| C3 | Config and transparency path | 4 | The terminal has a usable config/transparency path, either Yazelix-generated for Mars or native/user-owned for host terminals |
| C4 | Startup integration | 4 | The terminal can start Yazelix directly through `yzx enter` or the packaged Mars launch path |
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

| Criterion | Mars | Ghostty | Rio | WezTerm | Ratty | Kitty |
| --- | --- | --- | --- | --- | --- | --- |
| C1 Supported Yazelix entrypoint | Yes | Yes | Yes | Yes | Yes | Yes |
| C2 First-party control path | Yes | No | No | No | No | No |
| C3 Config and transparency path | Yes | Yes | Yes | Yes | Yes | Yes |
| C4 Startup integration | Yes | Yes | Yes | Yes | Yes | Yes |
| C5 GPU renderer | Yes | Yes | Yes | Yes | Yes | Yes |
| C6 Production confidence | Partial | Yes | Partial | Yes | Partial | Yes |
| C7 Yazelix stack validation | Yes | Yes | Partial | Partial | Partial | Partial |

## Rendering And Images

| Criterion | Mars | Ghostty | Rio | WezTerm | Ratty | Kitty |
| --- | --- | --- | --- | --- | --- | --- |
| C8 Ghostty-style cursor shaders | Partial | Yes | No | No | No | Partial |
| C9 Kitty graphics | Yes | Yes | Yes | Yes | Yes | Yes |
| C10 Sixel | Yes | No | Yes | Yes | No | No |
| C11 iTerm2 images | Yes | No | Yes | Yes | No | No |
| C25 Inline 3D graphics | No | No | No | No | Yes | No |

Mars receives partial credit for C8 because it supports Yazelix-native split
cursor rendering and Rio trail cursor behavior, but not the Ghostty-compatible
shader ABI. Kitty receives partial credit for C8 because it has cursor trails,
but not the Ghostty-compatible shader ABI Yazelix wants. Ratty receives full
credit for C25 because Ratty Graphics Protocol supports inline `.obj` and `.glb`
objects.

## Core Protocols

| Criterion | Mars | Ghostty | Rio | WezTerm | Ratty | Kitty |
| --- | --- | --- | --- | --- | --- | --- |
| C12 Kitty keyboard | Yes | Yes | Yes | Partial | No | Yes |
| C13 OSC 8 hyperlinks | Yes | Yes | Yes | Yes | No | Yes |
| C14 OSC 52 clipboard | Yes | Yes | Yes | Yes | No | Yes |
| C15 OSC 133 semantic prompts | No | Yes | No | Yes | No | No |
| C16 OSC 21 color control | No | Yes | No | No | No | Yes |
| C17 OSC 22 pointer shapes | Yes | Yes | Yes | No | No | Yes |
| C18 OSC 66 text sizing | No | Yes | No | No | No | Yes |
| C19 OSC 99 notifications | No | No | No | No | No | Yes |

WezTerm receives partial credit for C12 because Kitty keyboard support exists but
is opt-in through `enable_kitty_keyboard`. Ghostty supports notification escape
paths, but not the Kitty OSC 99 protocol surface scored here.

## Frontier Kitty Protocols

| Criterion | Mars | Ghostty | Rio | WezTerm | Ratty | Kitty |
| --- | --- | --- | --- | --- | --- | --- |
| C20 Kitty multiple cursors | No | No | No | No | No | Yes |
| C21 Kitty file transfer | No | No | No | No | No | Yes |
| C22 OSC 5522 text clipboard | No | No | No | No | No | Yes |
| C23 Kitty DECCARA | No | No | No | No | No | Yes |
| C24 Kitty unscrolling | No | No | No | No | No | Yes |

The current Ghostty source parses OSC 5522, but this comparison treats
parser-only behavior as no runtime support.

## Terminal Notes

### Mars

Mars is the packaged Yazelix terminal because it gives the project a controlled
Rust terminal path for stack compatibility, generated config, cursor behavior,
Kitty protocol work, and agent-driven development workflows. It has the deepest
Yazelix integration. Users who prefer a more mature standalone terminal can use
Ghostty or another host terminal and launch Yazelix with `yzx enter`.

### Ghostty

Ghostty remains the best mature host-terminal path for Yazelix today, especially
for macOS users who want a proven terminal outside the packaged Mars path. It has
excellent shader support, strong Kitty graphics support, OSC 133 shell
integration, Kitty keyboard, and a stable daily-driver posture. Its lower score
comes from this comparison weighting Sixel, iTerm2 images, OSC 99, and newer
Kitty frontier protocols heavily.

### Rio

Rio remains a supported option for users who want upstream Rio. Configure Rio to
run `yzx enter`; Rio's native config remains owned by the user.

### WezTerm

WezTerm is a conservative stable host-terminal path. It is cross-platform,
has strong image support across Kitty graphics, Sixel, and iTerm2 images, and is
a proven daily terminal. Its weaker score reflects the newer Kitty protocol
extensions it does not expose today and the lack of Ghostty-style cursor shader
parity.

### Ratty

Ratty is not trying to be the safest all-purpose terminal in this comparison.
Its value is that it proves a different frontier: GPU-rendered terminal UI plus
inline 3D objects through Ratty Graphics Protocol. Yazelix no longer packages
Ratty; use it as a host terminal running `yzx enter`.

### Kitty

Kitty is the protocol reference. It leads on Kitty graphics, keyboard handling,
desktop notifications, multiple cursors, file transfer, text sizing, pointer
shape, unscrolling, rich clipboard, and related protocol extensions. Yazelix can
use Kitty as a host terminal running `yzx enter`.
