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

## Hack On Mars

```sh
nix run --override-input mars ../mars
nix run --override-input yazelixZellij ../yazelix-zellij
```

## LOC Scorecard

Counts project files by language with `wc -l`. `flake.lock` is generated and
kept separate.

```sh
wc -l AGENTS.md README.md flake.nix mars.toml config.kdl layout.kdl nu/config.nu nu/env.nu flake.lock
```

| Language | Files | Lines | Kind |
| --- | --- | ---: | --- |
| Markdown | `AGENTS.md`, `README.md` | 123 | Handwritten |
| Nix | `flake.nix` | 154 | Handwritten |
| TOML | `mars.toml` | 74 | Handwritten |
| KDL | `config.kdl`, `layout.kdl` | 21 | Handwritten |
| Nu | `nu/config.nu`, `nu/env.nu` | 15 | Handwritten |
| JSON | `flake.lock` | 138 | Generated |
| Total | project files | 525 | Mixed |
