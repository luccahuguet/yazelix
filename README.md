# Yazelix Next

Small start: a Nix flake that installs `yzn`, a conflict-free dev command that
opens Mars with a Yazi-first layout that becomes a sidebar plus stacked work
panes, a bridge-enabled Yazelix Helix editor, reef cursor colors, and the
Yazelix Zellij fork. The top bar uses a rendered Yazelix Zellij Bar tray with
configurable widgets, a `YZN` runtime marker, and bundled `tu` for usage
widgets. `Alt Shift J/K/M` toggle managed LazyGit, config, and menu popups;
`Alt Shift L` hides or shows a persistent Codex resume popup.

## Run

```sh
nix run
nix run .#yzn
nix run .#yzn -- help
nix run .#yzn -- config
nix run .#yzn -- doctor
nix run .#yzn -- enter
nix run .#yzn -- launch
nix run .#yzn -- menu
nix run .#yzn -- status
nix run .#yzn -- sponsor
```

`yzn help` prints help, `yzn config` opens the Ratconfig UI, `yzn
doctor` checks owned runtime setup without launching Mars or Zellij, `yzn
enter` starts the managed Zellij runtime inside the current terminal, `yzn
launch` opens Mars first, and `yzn menu` prints the compact command/key
reference. `yzn status` prints a compact runtime/config summary, including
popup size and selected bar widgets, without launching Mars or Zellij. `yzn
sponsor` opens the GitHub Sponsors page when a host opener is available,
otherwise it prints the URL. Bare `yzn` defaults to `yzn launch`. If `doctor`,
`enter`, `launch`, or `status` fails before handing control to Zellij or Mars,
`yzn` prints a concise startup diagnostic with the reason and, when applicable,
the config path to check.

## Install

```sh
nix profile add --refresh /absolute/path/to/yazelix-next
yzn
```

Profile installs include `bin/yzn` and a `Yazelix Next` desktop entry.

## Update

```sh
nix profile upgrade --refresh yazelix-next
```

## Config

`yzn config` opens the Ratconfig UI in the current terminal and creates the
owned config sources when they are missing:

```text
~/.config/yazelix-next/config.toml
~/.config/yazelix-next/mars/config.toml
~/.config/yazelix-next/zellij/config.kdl
```

The `config` tab controls `open.log_level`, which sets the managed
`YZN_OPEN_LOG` level used by Yazi-to-Helix opens. Values are `off`, `error`,
`info`, and `debug`. It also controls `shell.program`, which selects the
packaged shell for new Zellij panes. Values are `nu`, `bash`, `zsh`, and
`fish`. The same tab edits `[popup].size`, the shared width and height
percentage for managed popups, and `[bar].widgets`, whose default tray is
`editor`, `shell`, `term`, `codex_usage`, `cpu`, and `ram`; allowed opt-ins are
`session`, `claude_usage`, and `opencode_go_usage`. The `mars` and `zellij`
tabs edit native sidecars that apply to new launches. The `keys` tab lists
current packaged bindings in read-only group, key, action, and owner columns,
with source paths in details. The `advanced` tab opens `nu/env.nu`,
`nu/config.nu`, and `starship.toml` in the managed editor. Native files are
created only when their row is activated.

## Shell Config

`config.toml` defaults to `shell.program = "nu"`. New Zellij panes start a
packaged shell dispatcher that reads this value and execs the matching packaged
`nu`, `bash`, `zsh`, or `fish`. The selection applies to new panes and
sessions. Bash, Zsh, and Fish are packaged binaries with their normal
interactive startup behavior; Yazelix Next only manages extra shell config for
Nu.

## Mars Config

`yzn` uses the packaged Mars config unless this managed native Mars config
exists:

```text
~/.config/yazelix-next/mars/config.toml
```

`yzn config` creates it from the packaged generated Mars config and exposes
basic terminal preferences such as window size, opacity, font size, line
height, scrollbar, bell, and cursor trail. Set `YAZELIX_NEXT_CONFIG_HOME` to
use a different Yazelix Next config root. `yzn` still owns the Mars launch
command and the managed Zellij runtime.

## Zellij Config

`yzn` owns Zellij keybindings, layout, plugin/runtime spine, and the managed
default shell dispatcher. Safe native Zellij preferences live in this managed
sidecar:

```text
~/.config/yazelix-next/zellij/config.kdl
```

`yzn config` edits scalar preferences such as pane frames, mouse mode,
scrollback size, copy behavior, styled underlines, startup tips, and
`ui.pane_frames.rounded_corners`. The sidecar is a simple guardrail, not a KDL
merge engine. It is rejected before launch and blocked inside the config UI
when an uncommented line starts with integration-critical ownership such as
`keybinds`, `default_shell`, `default_layout`, `layout`, `plugins`,
`load_plugins`, `support_kitty_keyboard_protocol`, `env`, `session_name`, or
`attach_to_session`.

The packaged config enables Zellij's Kitty keyboard protocol for modified key
chords such as `Alt Shift J/K/L/M`.

## Nushell Config

When `shell.program` is `nu`, `yzn` does not read normal Nushell config. It
loads packaged Yazelix Next `nu/env.nu` and `nu/config.nu` first, then optional
user files:

```text
~/.config/yazelix-next/nu/env.nu
~/.config/yazelix-next/nu/config.nu
```

The same `YAZELIX_NEXT_CONFIG_HOME` root applies here.

## Starship Config

When `shell.program` is `nu`, `yzn-nu` sets `STARSHIP_CONFIG` to this native
Starship config when it exists:

```text
~/.config/yazelix-next/starship.toml
```

Otherwise it uses an empty config, so normal `~/.config/starship.toml` does not
affect the managed Nu prompt. The file uses Starship TOML; user `nu/config.nu`
can still override prompt variables for advanced cases. `format` controls the
left prompt, and `right_format` controls the right prompt.

## Editor Opens

Yazi opens files through the packaged `yzn-open` Rust helper. If no Helix bridge
is live, `yzn-open` opens `yzn-hx` in a Zellij pane. If a Helix bridge pane is
live in the same Zellij tab as the invoking Yazi pane, `yzn-open` sends the file
or directory open request to that editor. Managed Yazi uses scoped Kitty
graphics environment for image previews while preserving the real Zellij
session for editor routing.

Inside the packaged Yazi sidebar, `Alt z` opens a zoxide picker, moves Yazi to
the selected directory, sends it through `yzn-open`, and renames the tab to the
workspace root. In Git repositories, Helix keeps the selected picker directory
while editor cwd and tab name use the repo root.

`yzn-open` writes bounded diagnostics to
`${YAZELIX_STATE_DIR}/logs/yzn-open.log` and keeps one rotated
`yzn-open.log.1` file. Managed `yzn` sessions set `YZN_OPEN_LOG` from
`open.log_level` in `config.toml`; the default is `info`.

## Keybindings

`Ctrl p/t/n/q` are the high-frequency Zellij controls. The rest of the native
Zellij layer uses `Ctrl Alt`, leaving most plain `Ctrl` keys available to
Helix, Nushell, Yazi, and terminal programs.

| Key | Action |
| --- | --- |
| `Ctrl Alt g/s/o` | lock, search, session |
| `Ctrl p/t/n/q` | pane, tab, resize, quit |
| `Ctrl Alt h/j/k/l` | move tab left, move pane down/up, move tab right |
| `Alt h/l` | move focus left/right across visible panes or previous/next tab |
| `Alt m` | new pane in the stacked layout |
| `Alt z` | Yazi zoxide jump into the managed editor |
| `Alt Shift J` | toggle the LazyGit popup |
| `Alt Shift K` | toggle the config popup |
| `Alt Shift L` | hide/show the Codex resume popup |
| `Alt Shift M` | toggle the menu popup |
| `Alt Shift h` | toggle the Yazi sidebar layout |

Move mode is intentionally unbound.

## Hack On Mars

```sh
nix run --override-input mars ../mars
nix run --override-input yazelixZellij ../yazelix-zellij
nix run --override-input yazelixHelix ../yazelix-helix
nix run --override-input yazelixZellijPopup ../yazelix-zellij-popup
nix run --override-input yazelixZellijBar ../yazelix-zellij-bar
nix run --override-input yazelixZellijPaneOrchestrator ../yazelix-zellij-pane-orchestrator
```

## LOC Scorecard

Counts owned project files by language with `wc -l`.

```sh
wc -l .gitignore AGENTS.md README.md CHANGELOG.md ARCHITECTURE.md flake.nix packaging/tokenusage.nix config.toml mars.toml config.kdl layout.kdl layout.swap.kdl nu/config.nu nu/env.nu helix/config.toml yazi/init.lua yazi/keymap.toml yazi/plugins/sidebar-status.yazi/main.lua yazi/plugins/zoxide-editor.yazi/main.lua yazi/yazi.toml crates/yzn-config/Cargo.toml crates/yzn-config/src/main.rs crates/yzn-open/Cargo.toml crates/yzn-open/src/main.rs checks/zellij-layout.rs checks/yzn-contracts.rs runtime/yzn-nu.rs runtime/yzn.rs runtime/yzn-zellij-config.rs
```

| Language | Files | Lines |
| --- | --- | ---: |
| Ignore | `.gitignore` | 4 |
| Markdown | `AGENTS.md`, `README.md`, `CHANGELOG.md`, `ARCHITECTURE.md` | 746 |
| Nix | `flake.nix`, `packaging/tokenusage.nix` | 485 |
| TOML | `config.toml`, `mars.toml`, `helix/config.toml`, `yazi/yazi.toml`, `yazi/keymap.toml`, `crates/yzn-config/Cargo.toml`, `crates/yzn-open/Cargo.toml` | 139 |
| KDL | `config.kdl`, `layout.kdl`, `layout.swap.kdl` | 189 |
| Nu | `nu/config.nu`, `nu/env.nu` | 11 |
| Lua | `yazi/init.lua`, `yazi/plugins/sidebar-status.yazi/main.lua`, `yazi/plugins/zoxide-editor.yazi/main.lua` | 137 |
| Rust | `crates/yzn-config/src/main.rs`, `crates/yzn-open/src/main.rs`, `checks/zellij-layout.rs`, `checks/yzn-contracts.rs`, `runtime/yzn-nu.rs`, `runtime/yzn.rs`, `runtime/yzn-zellij-config.rs` | 5299 |
| Total | owned project files | 7010 |
