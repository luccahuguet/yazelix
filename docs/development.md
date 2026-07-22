# Development

## CI

Normal CI runs Linux checks and the Darwin no-Helix evaluation guard on push,
pull request, and manual dispatch
`Publish Nix Cache` publishes all eight Linux packages and representative Home
Manager closures from `main` and manual dispatch. `Version Gate` is manual and
includes all eight Linux profile shapes, all eight `aarch64-darwin` packages,
the Darwin Home Manager closure, and the Darwin no-Mars, no-Helix, and host-Yazi
contracts.
`Darwin Package Smoke` runs the same Darwin verification weekly on Monday when
`main` has commits in the last 7 days, and on manual dispatch always, while
idle weeks skip the macOS build. Both macOS jobs assert that Darwin packages
contain no Linux desktop entry. The flake advertises the optional Yazelix
Cachix cache, while source builds remain valid without it. Use Version Gate
before publishing a release

## Main and stable

Development commits land on `main`. CI and cache publishing run there, and
users who select `main` accept development-channel changes.

The protected `stable` branch accepts fast-forward promotions from `main`. Its
required checks are `linux`, `publish_x86_64_linux`, and
`publish_aarch64_darwin`, including for maintainers. GitHub rejects force-pushes
and branch deletion.

Before promotion, verify that the candidate descends from the current `stable`,
belongs to `main`, passes the release checks for its changed surface, and has no
known P0 or P1 regression. User-visible runtime interaction changes also need a
fresh-session dogfood pass. Promote at most once per week unless an urgent fix
needs an earlier release:

```sh
git fetch origin main stable
git merge-base --is-ancestor origin/stable <sha>
git merge-base --is-ancestor <sha> origin/main
git push origin <sha>:stable
```

Skip promotion when no candidate meets the contract. To roll back, commit the
revert on `main`, verify the new commit, and promote it through the same path.
Do not move `stable` backward.

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
nix build .#yazelix-no-mars --no-link --print-build-logs
nix build .#yazelix-no-mars-no-helix-no-yazi --no-link --print-build-logs
nix build .#checks.x86_64-linux.no_mars_contracts --no-link
nix build .#checks.x86_64-linux.host_yazi_contracts --no-link
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
| Markdown | 2425 |
| Nix | 1541 |
| Shell | 84 |
| YAML | 450 |
| TOML | 245 |
| KDL | 233 |
| Nu | 11 |
| Lua | 253 |
| Rust | 15355 |
| Text | 41 |
| Total | 20858 |
