---
id: 019f2231-0581-7ed1-bd36-f92ed6b36327
slug: tasks/envctl-pr409-loop-lib-api-drift
title: "Fix envctl PR 409 loop_lib API drift"
type: task
status: completed
priority: medium
tags: [envctl, ci, loop_lib, github_actions]
---

## Overview

Envctl PR #409 failed CI because `crates/engine/src/runner.rs` imports `loop_lib::build_command` and `loop_lib::SpawnSpec`, but the CI materialization script fetched `FlexNetOS/loop_lib` `HEAD`, whose current `main` did not expose those APIs.

This is a dependency/API drift issue surfaced during the CodeDB/envctl ingest completion audit. The fix must upgrade the shared `loop_lib` substrate and keep envctl delegated to that substrate, rather than copying command construction into envctl.

## Acceptance Criteria

- [x] The loop_lib API needed by envctl exists on a pushed FlexNetOS/loop_lib branch.
- [x] Envctl CI materializes a loop_lib revision/branch containing `build_command` and `SpawnSpec`.
- [x] Envctl still imports and delegates to `loop_lib`; it does not take local ownership of command construction.
- [x] Local verification covers the loop_lib API and envctl runner compile/gates.
- [x] Envctl PR #409 is repushed after the fix.

## Evidence

Failing GitHub Actions log on 2026-07-02:

```text
error[E0432]: unresolved imports `loop_lib::build_command`, `loop_lib::SpawnSpec`
```

Fixes pushed:

- loop_lib branch: `codex/envctl-runner-command-builder`
- loop_lib commit: `0b46ee6 feat: expose command builder substrate`
- loop_lib draft PR: https://github.com/FlexNetOS/loop_lib/pull/9
- envctl commit: `4e76131 ci: materialize upgraded loop_lib substrate`
- envctl draft PR: https://github.com/FlexNetOS/envctl/pull/409

Verification passed:

- loop_lib temporary workspace: `cargo fmt --check`
- loop_lib temporary workspace: `cargo test test_build_command`
- envctl: `bash ci/setup-meta-deps.sh`
- envctl: `cargo check -p envctl-engine`
- envctl: `bash ci/gates/meta-substrates.sh`
- envctl: `bash ci/gates/agent-env.sh`

## Progress Log

### 2026-07-02

- Created after envctl PR #409 CI failed on the stale loop_lib API.
- Added the missing API in loop_lib rather than bypassing the shared substrate.
- Updated envctl CI setup to fetch `LOOP_LIB_REF`, defaulting to `refs/heads/codex/envctl-runner-command-builder`.
