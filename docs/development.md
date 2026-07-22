# Development

## CI

Normal CI runs Linux checks and the Darwin no-Helix evaluation guard on push,
pull request, and manual dispatch with Nushell as the run-step shell. No
workflow restores or publishes a remote build cache; Kache is the only
persistent build cache. `Version Gate` is manual and includes all eight Linux
profile shapes, all eight `aarch64-darwin` packages, the Darwin Home Manager
closure, and the Darwin no-Mars, no-Helix, and host-Yazi contracts.
`Darwin Package Smoke` runs the same Darwin verification weekly on Monday when
`main` has commits in the last 7 days, and on manual dispatch always, while
idle weeks skip the macOS build. Both macOS jobs assert that Darwin packages
contain no Linux desktop entry.
Use Version Gate before publishing a release

## Main and stable

Development commits land on `main`. CI runs there, and users who select `main`
accept development-channel changes.

The protected `stable` branch accepts fast-forward promotions from `main`.
GitHub rejects force-pushes and branch deletion, including for maintainers.

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

```nu
nix run --override-input mars ../mars
nix run --override-input yazelixZellij ../yazelix-zellij
nix run --override-input yazelixHelix ../yazelix-helix
nix run --override-input yazelixZellijPopup ../yazelix-zellij-popup
nix run --override-input yazelixZellijBar ../yazelix-zellij-bar
nix run --override-input yazelixZellijPaneOrchestrator ../yazelix-zellij-pane-orchestrator
nix build --override-input flexnetos_runner_source ../flexnetos_runner .#lifeos_foundation_yzx
```

Useful local checks:

```nu
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
nix build .#checks.x86_64-linux.cache_shell_policy --no-link
```

Runtime package changes should also pass a temporary profile install:

```nu
nix profile add --refresh /absolute/path/to/yazelix --profile /tmp/yzx-profile
```

Detailed launch, config, editor, and shell contracts live in
[Runtime Notes](runtime-notes.md)

## LOC scorecard

Counts **tracked text** project files. Excludes Beads state (`.beads/`),
lockfiles (`*.lock`), and binary assets. New owned sources count automatically
once committed

```nu
git ls-files
| lines
| where {|path|
    not ($path | str starts-with ".beads/")
    and not ($path | str starts-with "assets/")
    and not ($path | str ends-with ".lock")
}
| each {|path| {path: $path, lines: (open --raw $path | str stats | get lines)}}
```

| Language | Lines |
| --- | ---: |
| Ignore (`.gitignore`) | 18 |
| License | 201 |
| Markdown | 2237 |
| Nix | 2611 |
| Shell | 0 |
| YAML | 235 |
| TOML | 246 |
| KDL | 251 |
| Nu | 2896 |
| Lua | 247 |
| Rust | 14039 |
| Host policy (conf/JSON/shells) | 408 |
| Systemd units | 87 |
| Total | 23476 |
