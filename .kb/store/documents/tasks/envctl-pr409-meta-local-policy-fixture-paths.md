---
id: 019f2238-04a2-7683-94c7-65fe00d26805
slug: tasks/envctl-pr409-meta-local-policy-fixture-paths
title: "Fix envctl PR 409 meta-local-policy fixture paths"
type: task
status: completed
priority: medium
tags: [envctl, ci, policy, fixtures]
---

## Overview

Envctl PR #409 failed the `gates` job after the loop_lib compile issue was fixed because the Yazelix catalog fixture added real-home `.local/share/yazelix` paths. `ci/gates/meta-local-policy.sh` treats those as active install-source violations.

This task tracks replacing those fixture-only paths with neutral packaged-share paths while preserving the parser assertions for `yazelix_init` and extern/completion detection.

## Acceptance Criteria

- [x] Yazelix catalog fixture data no longer contains `~/.local/share/yazelix`.
- [x] The fixture still exercises `yazelix_init` and extern/completion metadata detection.
- [x] Catalog tests still pass.
- [x] `ci/gates/meta-local-policy.sh` passes.
- [x] Envctl PR #409 is repushed after the fix.

## Evidence

Failing GitHub Actions log on 2026-07-02:

```text
meta-local-policy: real-home .local/symlink-farm references remain in active install sources:
crates/engine/src/catalog.rs:3972: source ~/.local/share/yazelix/initializers/nushell/yazelix_init.nu
crates/engine/src/catalog.rs:3973: use ~/.local/share/yazelix/completions/yazelix_extern.nu *
```

Fix pushed:

- envctl commit: `770f812 test: avoid real-home paths in Yazelix fixtures`
- envctl draft PR: https://github.com/FlexNetOS/envctl/pull/409

Verification passed:

- `cargo fmt --check`
- `cargo test -p envctl-engine catalog::tests`
- `bash ci/gates/meta-local-policy.sh`

## Progress Log

### 2026-07-02

- Created after envctl PR #409 `gates` failed on fixture-only real-home paths.
- Replaced the fixture paths with `/opt/yazelix/share/...` while keeping the parser metadata assertions meaningful.
