# Development

## CI

Normal CI runs Linux checks and the Darwin no-Helix evaluation guard on push,
pull request, and manual dispatch
`Publish Nix Cache` publishes all three Linux packages and representative Home
Manager closures from `main` and manual dispatch. `Version Gate` is manual and
includes all three Linux profile shapes, all three `aarch64-darwin` packages,
the Darwin Home Manager closure, and the Darwin no-Helix contract. `Darwin
Package Smoke` runs the same Darwin verification weekly on Monday when `main`
has commits in the last 7 days, and on manual dispatch always, while idle weeks
skip the macOS build. Both macOS jobs assert that Darwin packages contain no
Linux desktop entry. The flake advertises the optional Yazelix Cachix cache,
while source builds remain valid without it. Use Version Gate before publishing
a release

## Local development

Use local sibling repositories while hacking runtime inputs:

```sh
nix run --override-input mars ../mars
nix run --override-input yazelixZellij ../yazelix-zellij
nix run --override-input yazelixHelix ../yazelix-helix
nix run --override-input yazelixZellijPopup ../yazelix-zellij-popup
nix run --override-input yazelixZellijBar ../yazelix-zellij-bar
nix run --override-input yazelixZellijPaneOrchestrator ../yazelix-zellij-pane-orchestrator
```

Useful local checks:

```sh
nix flake check
nix flake show --all-systems
nix build .#yazelix --no-link --print-build-logs
nix build .#yazelix-no-helix --no-link --print-build-logs
nix build .#runtime --no-link --print-build-logs
nix build .#checks.x86_64-linux.no_helix_contracts --no-link
nix build .#checks.x86_64-linux.yzx_yazi_materialization --no-link
```

Runtime package changes should also pass a temporary profile install:

```sh
nix profile add --refresh /absolute/path/to/yazelix --profile /tmp/yzx-profile
```

Detailed launch, config, editor, and shell contracts live in
[Runtime Notes](runtime-notes.md)

## LOC scorecard

Counts **tracked text** project files. Excludes Beads state (`.beads/`),
lockfiles (`*.lock`), and binary assets. New owned sources count automatically
once committed

```sh
git ls-files | grep -Ev '^\.beads/|\.lock$|^assets/' | xargs wc -l
```

| Language | Lines |
| --- | ---: |
| Ignore (`.gitignore`) | 19 |
| License | 201 |
| Markdown | 2098 |
| Nix | 1264 |
| Shell | 84 |
| YAML | 413 |
| TOML | 245 |
| KDL | 218 |
| Nu | 11 |
| Lua | 251 |
| Rust | 14739 |
| Text | 41 |
| Total | 19584 |
