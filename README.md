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

## Hack On Mars

```sh
nix run --override-input mars ../mars
nix run --override-input yazelixZellij ../yazelix-zellij
nix run --override-input yazelixHelix ../yazelix-helix
```

## LOC Scorecard

Counts owned project files by language with `wc -l`.

```sh
wc -l .gitignore AGENTS.md README.md flake.nix mars.toml config.kdl layout.kdl layout.swap.kdl nu/config.nu nu/env.nu scripts/yzn-nu.sh helix/config.toml yazi/init.lua yazi/plugins/sidebar-status.yazi/main.lua yazi/yazi.toml crates/yzn-open/Cargo.toml crates/yzn-open/src/main.rs checks/zellij-layout.rs
```

| Language | Files | Lines |
| --- | --- | ---: |
| Ignore | `.gitignore` | 1 |
| Markdown | `AGENTS.md`, `README.md` | 144 |
| Nix | `flake.nix` | 230 |
| TOML | `mars.toml`, `helix/config.toml`, `yazi/yazi.toml`, `crates/yzn-open/Cargo.toml` | 106 |
| KDL | `config.kdl`, `layout.kdl`, `layout.swap.kdl` | 58 |
| Nu | `nu/config.nu`, `nu/env.nu` | 11 |
| Shell | `scripts/yzn-nu.sh` | 36 |
| Lua | `yazi/init.lua`, `yazi/plugins/sidebar-status.yazi/main.lua` | 16 |
| Rust | `crates/yzn-open/src/main.rs`, `checks/zellij-layout.rs` | 676 |
| Total | owned project files | 1278 |
