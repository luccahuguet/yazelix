# Yazelix Next

Small start: a Nix flake that installs `yzn`, which opens Mars with a small
Yazelix config, reef cursor colors, a stacked layout, and the Yazelix Zellij
fork.

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

## Hack On Mars

```sh
nix run --override-input mars ../mars
nix run --override-input yazelixZellij ../yazelix-zellij
```

## LOC Scorecard

Counts project files by language with `wc -l`. `flake.lock` is generated and
kept separate.

```sh
wc -l AGENTS.md README.md flake.nix mars.toml config.kdl layout.kdl nu/config.nu nu/env.nu scripts/yzn-nu.sh flake.lock
```

| Language | Files | Lines | Kind |
| --- | --- | ---: | --- |
| Markdown | `AGENTS.md`, `README.md` | 136 | Handwritten |
| Nix | `flake.nix` | 154 | Handwritten |
| TOML | `mars.toml` | 74 | Handwritten |
| KDL | `config.kdl`, `layout.kdl` | 21 | Handwritten |
| Nu | `nu/config.nu`, `nu/env.nu` | 15 | Handwritten |
| Shell | `scripts/yzn-nu.sh` | 36 | Handwritten |
| JSON | `flake.lock` | 138 | Generated |
| Total | project files | 574 | Mixed |
