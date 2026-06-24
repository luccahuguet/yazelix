# Yazelix Next

Small start: a Nix flake that installs `yzn`, which opens Mars running the
Yazelix Zellij fork.

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
wc -l AGENTS.md README.md flake.nix flake.lock
```

| Language | Files | Lines | Kind |
| --- | --- | ---: | --- |
| Markdown | `AGENTS.md`, `README.md` | 117 | Handwritten |
| Nix | `flake.nix` | 81 | Handwritten |
| JSON | `flake.lock` | 138 | Generated |
| Total | project files | 336 | Mixed |
