# Development

## CI

Normal CI runs Linux checks and the Darwin no-Helix evaluation guard on push,
pull request, and manual dispatch
The Home Manager evaluation guard also overrides nixpkgs with a reviewed
revision containing Zellij 0.45.1 on Linux and Darwin, exercising the consumer
`follows` path with the existing configuration and package-override cases.
This guards evaluation; package builds still use the repository lockfile.
`Publish Nix Cache` publishes all four Linux capability variants, the Main and
Edge full-package launcher outputs, and representative Home Manager closures
from `main` and manual dispatch. `Version Gate` is manual and
includes all four Linux profile shapes, all four `aarch64-darwin` packages,
the Darwin Home Manager closure, and the Darwin Rio, no-Helix, and host-Yazi
contracts.
`Darwin Package Smoke` runs the same Darwin verification weekly on Monday when
`main` has commits in the last 7 days, and on manual dispatch always, while
idle weeks skip the macOS build. Both macOS jobs assert that Darwin packages
contain no Linux desktop entry. The flake advertises the optional Yazelix
Cachix cache, while source builds remain valid without it. Use Version Gate
before publishing a release

Linux CI and Darwin Package Smoke build `helix_grammar_sources`, Nova Helix's
`HELIX-GRAMMAR-SOURCES-001` check. It rejects Codeberg network fetches during Nix
evaluation and compiles the pinned Codeberg grammars from bundled, hash-verified
sources. Grammar revisions and snapshot maintenance belong to Nova Helix.

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

### Release notes

End each GitHub release body with this sponsorship footer, once:

```markdown
If Yazelix saves you time, you can support continued development on [GitHub Sponsors](https://github.com/sponsors/luccahuguet).
```

For existing releases, save the original body and metadata, append only the footer,
and verify the published body, tags, titles, flags and assets through the GitHub API.

## Local development

Use local sibling repositories while hacking runtime inputs:

```sh
nix run --override-input rio ../nova-rio
nix run --override-input yazelixZellij ../nova-zellij
nix run --override-input yazelixHelix ../nova-helix
nix run --override-input yazelixForest ../yazelix-forest
nix run --override-input zjRadar ../zj-radar
nix run --override-input yazelixZellijPopup ../zellij-popup
nix run --override-input novaBar ../nova-bar
nix run --override-input yazelixZellijPaneOrchestrator ../zellij-pane-orchestrator
nix run --override-input yaziBistro ../yazi-bistro
```

For coupled child changes, use the same complete override set for builds,
checks, and every dogfood profile refresh. The profile does not retain overrides.
Publish matching child revisions and lock them before verifying without overrides.
Check popup open/hide and sidebar toggle together in a fresh installed session.

Useful local checks:

```sh
nix flake check
nix flake show --all-systems
nix build .#yazelix --no-link --print-build-logs
nix build .#yazelix-no-helix --no-link --print-build-logs
nix build .#yazelix-no-helix-no-yazi --no-link --print-build-logs
nix build .#checks.x86_64-linux.rio_contracts --no-link
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

The Rio configuration contract can be exercised against a candidate native binary
with `YZX_TEST_RIO` set to its absolute path: run the `yzx-config` crate test
`rio_native_controls_preserve_validate_reset_and_respect_ownership` with
`--include-ignored`. It covers real native validation, preserved comments, reset,
invalid-document fallback, Home Manager protection, and Rio-free models. Ordinary
crate checks omit this external-tool test; the packaged `rio_contracts` check also
validates the seeded config through the exact pinned Rio executable.

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
| Markdown | 4455 |
| JSON | 117 |
| Nix | 1898 |
| Shell | 126 |
| YAML | 457 |
| TOML | 523 |
| KDL | 257 |
| Nu | 14 |
| Lua | 133 |
| Rust | 20485 |
| Text | 84 |
| Total | 28769 |
