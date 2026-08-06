# Development

## CI

Normal CI runs Linux checks and the Darwin no-Helix evaluation guard on push,
pull request, and manual dispatch
`Publish Nix Cache` publishes all eight Linux capability variants, the Main and
Edge full-package launcher outputs, and representative Home Manager closures
from `main` and manual dispatch. `Version Gate` is manual and
includes all eight Linux profile shapes, all eight `aarch64-darwin` packages,
the Darwin Home Manager closure, and the Darwin no-Mars, no-Helix, and host-Yazi
contracts.
`Darwin Package Smoke` runs the same Darwin verification weekly on Monday when
`main` has commits in the last 7 days, and on manual dispatch always, while
idle weeks skip the macOS build. Both macOS jobs assert that Darwin packages
contain no Linux desktop entry. The flake advertises the optional Yazelix
Cachix cache, while source builds remain valid without it. Use Version Gate
before publishing a release

## Edge, main, and stable

All development commits land on `edge`, including fixes, reverts,
documentation, and Beads updates. CI runs there, and users who select `edge`
accept the active experimental dogfood channel.

`main` is promotion-only accepted development. After an `edge` revision is
accepted and verified, advance `main` to that exact revision without merging or
cherry-picking. CI and cache publishing run on `main`, and users who select it
accept more frequent updates than `stable`:

```sh
git fetch origin edge main
git merge-base --is-ancestor origin/main <sha>
git merge-base --is-ancestor <sha> origin/edge
git push origin <sha>:main
```

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
revert on `edge`, verify it, promote it to `main`, and then promote it through
the same stable path. Do not move `stable` backward.

## Local development

Use local sibling repositories while hacking runtime inputs:

```sh
nix run --override-input mars ../mars
nix run --override-input yazelixZellij ../yazelix-zellij
nix run --override-input yazelixHelix ../yazelix-helix
nix run --override-input yazelixZellijPopup ../yazelix-zellij-popup
nix run --override-input yazelixZellijBar ../yazelix-zellij-bar
nix run --override-input yazelixZellijPaneOrchestrator ../yazelix-zellij-pane-orchestrator
nix run --override-input yaziBistro ../yazi-bistro
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
| Markdown | 3441 |
| JSON | 117 |
| Nix | 1692 |
| Shell | 87 |
| YAML | 456 |
| TOML | 468 |
| KDL | 248 |
| Nu | 11 |
| Lua | 245 |
| Rust | 19791 |
| Text | 71 |
| Total | 26847 |
