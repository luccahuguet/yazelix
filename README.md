# Yazelix Next

Small start: a Nix flake that installs `yzn`, which opens Mars with a small
Yazelix config, reef cursor colors, and the Yazelix Zellij fork.

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
wc -l AGENTS.md README.md flake.nix mars.toml flake.lock
```

| Language | Files | Lines | Kind |
| --- | --- | ---: | --- |
| Markdown | `AGENTS.md`, `README.md` | 120 | Handwritten |
| Nix | `flake.nix` | 116 | Handwritten |
| TOML | `mars.toml` | 74 | Handwritten |
| JSON | `flake.lock` | 138 | Generated |
| Total | project files | 448 | Mixed |
