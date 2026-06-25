# Yazelix Next

Small start: a Nix flake that installs `yzn`, which opens Mars with a Yazi-first
layout that becomes a sidebar plus stacked work panes, a bridge-enabled Yazelix
Helix editor, reef cursor colors, and the Yazelix Zellij fork.

## Run

```sh
nix run
nix run .#yzn
```

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

## Mars Config

`yzn` uses the packaged Mars config unless this native Mars config exists:

```text
~/.config/yazelix-next/mars/config.toml
```

Set `YAZELIX_NEXT_CONFIG_HOME` to use a different Yazelix Next config root.
The Mars config controls terminal preferences; `yzn` still owns the Mars launch
command and the managed Zellij runtime.

## Nushell Config

`yzn` does not read normal Nushell config. It loads packaged Yazelix Next
`nu/env.nu` and `nu/config.nu` first, then optional user files:

```text
~/.config/yazelix-next/nu/env.nu
~/.config/yazelix-next/nu/config.nu
```

The same `YAZELIX_NEXT_CONFIG_HOME` root applies here.

## Starship Config

`yzn` sets `STARSHIP_CONFIG` to this native Starship config when it exists:

```text
~/.config/yazelix-next/starship.toml
```

Otherwise it uses an empty config, so normal `~/.config/starship.toml` does not
affect managed shells. The file uses Starship TOML; user `nu/config.nu` can
still override prompt variables for advanced cases.

## Editor Opens

Yazi opens files through the packaged `yzn-open` Rust helper. If no Helix bridge
is live, `yzn-open` opens `yzn-hx` in a Zellij pane. If the Helix bridge is
live, `yzn-open` sends the file or directory open request to that editor.

`yzn-open` writes bounded diagnostics to
`${YAZELIX_STATE_DIR}/logs/yzn-open.log` and keeps one rotated
`yzn-open.log.1` file. Set `YZN_OPEN_LOG` to `off`, `error`, `info`, or
`debug`; the default is `info`.

## Keybindings

`Ctrl p/t/n/q` are the high-frequency Zellij controls. The rest of the native
Zellij layer uses `Ctrl Alt`, leaving most plain `Ctrl` keys available to
Helix, Nushell, Yazi, and terminal programs.

| Key | Action |
| --- | --- |
| `Ctrl Alt g/s/o` | lock, search, session |
| `Ctrl p/t/n/q` | pane, tab, resize, quit |
| `Ctrl Alt h/j/k/l` | move tab left, move pane down/up, move tab right |
| `Alt m` | new pane in the stacked layout |
| `Alt Shift h` | show or hide the Yazi sidebar |

Move mode is intentionally unbound.

## Hack On Mars

```sh
nix run --override-input mars ../mars
nix run --override-input yazelixZellij ../yazelix-zellij
nix run --override-input yazelixHelix ../yazelix-helix
```

## LOC Scorecard

Counts owned project files by language with `wc -l`.

```sh
wc -l .gitignore AGENTS.md README.md CHANGELOG.md ARCHITECTURE.md flake.nix mars.toml config.kdl layout.kdl layout.swap.kdl nu/config.nu nu/env.nu helix/config.toml yazi/init.lua yazi/plugins/sidebar-status.yazi/main.lua yazi/yazi.toml crates/yzn-open/Cargo.toml crates/yzn-open/src/main.rs checks/zellij-layout.rs checks/yzn-contracts.rs runtime/yzn-nu.rs
```

| Language | Files | Lines |
| --- | --- | ---: |
| Ignore | `.gitignore` | 1 |
| Markdown | `AGENTS.md`, `README.md`, `CHANGELOG.md`, `ARCHITECTURE.md` | 375 |
| Nix | `flake.nix` | 258 |
| TOML | `mars.toml`, `helix/config.toml`, `yazi/yazi.toml`, `crates/yzn-open/Cargo.toml` | 106 |
| KDL | `config.kdl`, `layout.kdl`, `layout.swap.kdl` | 119 |
| Nu | `nu/config.nu`, `nu/env.nu` | 11 |
| Lua | `yazi/init.lua`, `yazi/plugins/sidebar-status.yazi/main.lua` | 16 |
| Rust | `crates/yzn-open/src/main.rs`, `checks/zellij-layout.rs`, `checks/yzn-contracts.rs`, `runtime/yzn-nu.rs` | 1323 |
| Total | owned project files | 2209 |
