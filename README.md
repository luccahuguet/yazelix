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

## Nushell Config

`yzn` does not read normal Nushell config. It loads packaged Yazelix Next
`nu/env.nu` and `nu/config.nu` first, then optional user files:

```text
~/.config/yazelix-next/nu/env.nu
~/.config/yazelix-next/nu/config.nu
```

Set `YAZELIX_NEXT_CONFIG_HOME` to use a different config root.

## Editor Opens

Yazi opens files through the packaged `yzn-open` Rust helper. If no Helix bridge
is live, `yzn-open` opens `yzn-hx` in a Zellij pane. If the Helix bridge is
live, `yzn-open` sends the file or directory open request to that editor.

`yzn-open` writes bounded diagnostics to
`${YAZELIX_STATE_DIR}/logs/yzn-open.log` and keeps one rotated
`yzn-open.log.1` file. Set `YZN_OPEN_LOG` to `off`, `error`, `info`, or
`debug`; the default is `info`.

## Keybindings

`Ctrl Alt` keys are Zellij-native control. Plain `Ctrl` keys stay available to
Helix, Nushell, Yazi, and terminal programs.

| Key | Action |
| --- | --- |
| `Ctrl Alt g/p/t/n/s/o/q` | lock, pane, tab, resize, search, session, quit |
| `Ctrl Alt h/j/k/l` | move tab left, move pane down/up, move tab right |
| `Alt m` | new stacked pane |
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
wc -l .gitignore AGENTS.md README.md ARCHITECTURE.md flake.nix mars.toml config.kdl layout.kdl layout.swap.kdl nu/config.nu nu/env.nu helix/config.toml yazi/init.lua yazi/plugins/sidebar-status.yazi/main.lua yazi/yazi.toml crates/yzn-open/Cargo.toml crates/yzn-open/src/main.rs checks/zellij-layout.rs checks/yzn-contracts.rs runtime/yzn-nu.rs
```

| Language | Files | Lines |
| --- | --- | ---: |
| Ignore | `.gitignore` | 1 |
| Markdown | `AGENTS.md`, `README.md`, `ARCHITECTURE.md` | 304 |
| Nix | `flake.nix` | 246 |
| TOML | `mars.toml`, `helix/config.toml`, `yazi/yazi.toml`, `crates/yzn-open/Cargo.toml` | 106 |
| KDL | `config.kdl`, `layout.kdl`, `layout.swap.kdl` | 125 |
| Nu | `nu/config.nu`, `nu/env.nu` | 11 |
| Lua | `yazi/init.lua`, `yazi/plugins/sidebar-status.yazi/main.lua` | 16 |
| Rust | `crates/yzn-open/src/main.rs`, `checks/zellij-layout.rs`, `checks/yzn-contracts.rs`, `runtime/yzn-nu.rs` | 1173 |
| Total | owned project files | 1982 |
