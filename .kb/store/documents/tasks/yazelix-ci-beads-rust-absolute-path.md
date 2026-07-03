---
id: 019f222a-fc46-7402-ba2d-7ad1fd5a284c
slug: tasks/yazelix-ci-beads-rust-absolute-path
title: "Fix Yazelix CI beads_rust absolute path input"
type: task
status: completed
priority: medium
tags: [ci, nix, beads_rust, github_actions]
---

## Overview

Yazelix PR #8 currently fails the GitHub Actions `Build validation helpers` job because the branch-level flake input `beads_rust_source` points at a host-local absolute path:

```text
path:/home/flexnetos/FlexNetOS/src/meta/beads_rust
```

GitHub runners do not have that path, so `nix develop .#ci` fails before building validation helpers. This was surfaced during the CodeDB/envctl ingest completion audit and must be fixed before the PR can prove the branch.

## Acceptance Criteria

- [x] `flake.nix` does not default to a host-local absolute `path:` input for `beads_rust_source`.
- [x] `flake.lock` no longer locks `beads_rust_source` to `/home/flexnetos/FlexNetOS/src/meta/beads_rust`.
- [x] Local maintainer override remains possible with `--override-input beads_rust_source path:<checkout>` when intentionally testing unpublished Beads Rust changes.
- [x] `nix develop .#ci -c cargo build --quiet --manifest-path rust_core/Cargo.toml -p yazelix_maintainer --bin yzx_repo_validator -p yazelix_core --bin yzx_core` no longer fails on the missing absolute path.
- [x] PR #8 CI is repushed after the fix.

## Evidence

Failing GitHub Actions log on 2026-07-02:

```text
error: path '//home/flexnetos/FlexNetOS/src/meta/beads_rust' does not exist
```

The failing job is `Build validation helpers` in workflow `CI` for PR #8.

## Progress Log

### 2026-07-02

- Created during completion audit before changing flake inputs.
- Changed `beads_rust_source` from `path:/home/flexnetos/FlexNetOS/src/meta/beads_rust` to `github:FlexNetOS/beads_rust`.
- Updated `flake.lock` to `FlexNetOS/beads_rust` revision `2498339168b8e88d641e8ae1664843fc69740012`.
- Local maintainer override remains available with `--override-input beads_rust_source path:<checkout>`, but the default pushed flake is now CI-portable.
- Verification passed:
  - `nix develop .#ci -c cargo build --quiet --manifest-path rust_core/Cargo.toml -p yazelix_maintainer --bin yzx_repo_validator -p yazelix_core --bin yzx_core`
