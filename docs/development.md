# Development

## CI

Normal CI runs Linux checks and the Darwin no-Helix evaluation guard on push,
pull request, and manual dispatch
The Home Manager evaluation guard overrides nixpkgs with the reviewed revision
in `checks/newer-nixpkgs.txt` on Linux and Darwin, exercising the consumer
`follows` path with the existing configuration and package-override cases.
Ordinary CI evaluates this override; heavy compatibility builds run in Version Gate.
`Publish Nix Cache` publishes Linux and Darwin capability variants, the Main and
Edge Linux full-package launcher outputs, and representative Home Manager closures
from `main` and manual dispatch. Both jobs push the requested output closures
synchronously, including substituted outputs. Missing credentials or failed
uploads fail the job. `Version Gate` is manual. Its Linux job installs all four
profile shapes, runs release contracts, and builds the complete Home Manager
check with the reviewed nixpkgs override. `macos-package-smoke` retains the
locked-input Darwin packages, Home Manager, Rio, no-Helix, and host-Yazi checks.
Three dependent Darwin jobs exercise the same reviewed override:

1. `darwin-core` builds `yazelix-no-rio-no-helix-no-yazi` and publishes its closure.
2. `darwin-editor` restores the core closure, builds `yazelix-no-rio`, and publishes it.
3. `darwin-integration` restores both closures and builds the complete
   `checks.aarch64-darwin.home_manager`, including Rio and all configuration and
   installed-command assertions. It does not publish another closure.

Each stage checks out the same candidate SHA and reads `checks/newer-nixpkgs.txt`.
Prerequisite paths come from successful job outputs. `nix copy --from` restores
those exact closures from Cachix without a build fallback; missing paths or a
failed copy, build, or publication fail the stage. Rerun a failed job after
addressing its failure; completed prerequisites remain in the shared cache.

Cache writes use the existing `CACHIX_AUTH_TOKEN` only in the two prerequisite
jobs. Stages reject dispatches outside the canonical repository's `edge`, `main`,
and `stable` branches before checkout or credential use. Missing publication
credentials fail explicitly; fork and tag dispatches cannot pass this gate.
There are no compatibility push/PR triggers, store archives, retention pins,
or additional cache services. Only the two requested runtime closures are
published; common store paths are deduplicated under the existing cache policy.
Logs include exact paths, closure sizes, elapsed build time and runner disk use.
Closure size is an upper bound on added cache storage, not measured cache growth.

Version Gate permits one active and one pending run. Five native jobs have
90-minute limits and `release-gate` has five minutes, bounding configured runner
time at 455 minutes per invocation. Linux keeps two Nix build jobs, Darwin one,
with two cores each. Stage limits retain the existing cap until native timings
support tighter bounds. The aggregate requires all five jobs to succeed, even
after a dependency fails; failed, cancelled, or skipped work cannot satisfy it.
Review the comparison pin deliberately and verify all native stages when
changing it. This proves the reviewed pin, not every future nixpkgs revision.
Retain compatibility failures and measured limits before expanding the CI budget.
`Darwin Package Smoke` runs the same Darwin verification weekly on Monday when
`main` has commits in the last 7 days, and on manual dispatch always, while
idle weeks skip the macOS build. Both package-smoke jobs assert that Darwin packages
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

Keep `stable ⊆ main ⊆ edge` on one linear history. A promotion includes every
accumulated commit up to the candidate, including changes unrelated to the fix
that motivated it. Review that whole range. If intervening work is unaccepted,
accept it, revert it on `edge`, or defer promotion. Do not cherry-pick fixes,
commit directly to the promotion channels, or skip a channel.

`main` is promotion-only accepted development. After an `edge` revision is
accepted and verified, advance `main` to that exact revision without merging or
cherry-picking. CI and cache publishing run on `main`, and users who select it
accept more frequent updates than `stable`:

```sh
git fetch origin edge main
git merge-base --is-ancestor origin/main <sha>
git merge-base --is-ancestor <sha> origin/edge
git log --oneline origin/main..<sha>
git diff origin/main <sha>
git push origin <sha>:main
```

The protected `stable` branch accepts fast-forward promotions from `main`. Its
required checks are `linux`, `publish_x86_64_linux`,
`publish_aarch64_darwin`, and `release-gate`, sourced from GitHub Actions.
`main` requires `linux`. Both branches enforce checks for administrators,
require linear history, and reject force-pushes and branch deletion.

Before promotion, verify that the candidate descends from the current `stable`,
belongs to `main`, passes all required checks on that exact SHA, and has no
known P0 or P1 regression. User-visible runtime interaction changes also need a
fresh-session dogfood pass. Ordinary promotions wait at least seven days after
the last Stable promotion, including an urgent one. Routine documentation and
planning changes wait for that window. Explicitly requested urgent fixes may
skip the wait, with the same verification and acceptance of the whole candidate.
There is no automatic weekly release and no release when the gate fails.

Record the candidate range, exact proof, actual promotion time, and any urgency
reason in the owning Bead. Use the actual previous promotion time, not the source
commit date. GitHub enforces status and linear-history requirements; checking
predecessor membership, cadence, acceptance, and dogfood remains the maintainer's
responsibility. Run the push only after all checks pass:

```sh
git fetch origin main stable
git merge-base --is-ancestor origin/stable <sha>
git merge-base --is-ancestor <sha> origin/main
git log --oneline origin/stable..<sha>
git diff origin/stable <sha>
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

Run the release compatibility check locally on the matching native platform
(replace `x86_64-linux` with `aarch64-darwin` on Apple Silicon):

```nu
nix build --no-link --no-write-lock-file --print-out-paths --print-build-logs --override-input nixpkgs (open --raw checks/newer-nixpkgs.txt | str trim) .#checks.x86_64-linux.home_manager
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
| Markdown | 4613 |
| JSON | 117 |
| Nix | 1900 |
| Shell | 126 |
| YAML | 568 |
| TOML | 523 |
| KDL | 257 |
| Nu | 14 |
| Lua | 133 |
| Rust | 20492 |
| Text | 85 |
| Total | 29048 |
