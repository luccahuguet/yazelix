---
id: 019f223d-40ea-7092-82f6-8c979bf8e639
slug: tasks/envctl-pr409-cargo-audit-advisories
title: "Fix envctl PR 409 cargo audit advisories"
type: task
status: completed
priority: medium
---

# Overview

Envctl PR 409 is functionally validating the CodeDB/Yazelix config catalog path, but the GitHub `cargo audit` gate surfaced new dependency advisories after the initial proof. The fix should preserve the current CodeDB/envctl table behavior while moving vulnerable or unmaintained dependencies forward where upstream releases exist.

This task exists because the user requirement is upgrade-first: newly surfaced issues from the live PR must be tracked in GitKB and resolved with upgrades or explicit, evidence-backed follow-up when no fixed release exists.

## Goals

- Keep the CodeDB/envctl Yazelix config catalog import and render behavior intact.
- Upgrade direct and transitive dependencies that have non-vulnerable releases.
- Avoid downgrades and avoid broad ignore rules that would hide unrelated future advisories.
- Document any remaining advisory that has no available fixed release with precise scope and next action.

## Acceptance Criteria

- [x] `anyhow` advisory is resolved by lockfile/dependency upgrade.
- [x] `number_prefix` advisory from `loop_lib -> indicatif` is resolved by upgrading loop_lib's progress dependency.
- [x] GUI-stack `paste` / `ttf-parser` advisories are resolved by upgrading the egui/eframe stack or are documented as a separately scoped blocked upgrade if upstream still carries them.
- [x] `rustls-pemfile` advisory is resolved by dependency removal/replacement or documented with evidence that no fixed release exists.
- [x] `quick-xml` advisories surfaced by local audit are removed or documented with evidence that the fixed version is blocked by upstream `wayland-scanner`.
- [x] Envctl cargo audit gate is rerun locally or the closest executable equivalent is recorded.
- [x] CodeDB/Yazelix catalog import/render proof still passes after dependency changes.
- [x] Relevant envctl, loop_lib, and Yazelix KB changes are committed and pushed.

## Context

- Parent integration task: [[tasks/codedb-envctl-yazelix-config-ingest]]
- Envctl PR: https://github.com/FlexNetOS/envctl/pull/409
- loop_lib PR: https://github.com/FlexNetOS/loop_lib/pull/9
- Yazelix KB PR: https://github.com/FlexNetOS/yazelix/pull/8

The failing PR audit output named:

- `RUSTSEC-2026-0190` for `anyhow 1.0.102`
- `RUSTSEC-2025-0119` for `number_prefix 0.4.0`, via `indicatif 0.17.11` in `loop_lib`
- `RUSTSEC-2024-0436` for `paste 1.0.15`, via the GUI stack on Windows target resolution
- `RUSTSEC-2026-0192` for `ttf-parser 0.25.1`, via `egui`/`eframe` font stack
- `RUSTSEC-2025-0134` for `rustls-pemfile 2.2.0`
- Local audit rerun also surfaced `RUSTSEC-2026-0194` and `RUSTSEC-2026-0195` for `quick-xml 0.39.4`, via `wayland-scanner 0.31.10`.

## Progress Log

### 2026-07-02

- Created the task after PR 409 surfaced `cargo audit` failures.
- Confirmed with `cargo search` that current visible upgrade targets are `anyhow 1.0.103`, `eframe 0.35.0`, `egui 0.35.0`, and `indicatif 0.18.6`.
- Confirmed `rustls-pemfile` currently shows `2.2.0` as the latest crates.io release in this environment.
- Confirmed dependency paths:
  - `number_prefix` comes from `loop_lib -> indicatif 0.17.11`
  - `paste` comes from `accesskit_windows -> accesskit_winit -> egui-winit -> eframe`
  - `ttf-parser` comes from `owned_ttf_parser -> ab_glyph -> epaint/sctk-adwaita -> egui/winit`
  - `rustls-pemfile` is a direct workspace dependency used by `secretctl`, `secretd`, and `secrets-engine`
- Upgraded `loop_lib` `indicatif` to `0.18`, removing `number_prefix` from the envctl dependency graph.
- Upgraded envctl `anyhow` to `1.0.103`.
- Moved envctl GUI to the MSRV-compatible `egui`/`eframe` `0.33.3` line and pruned optional default GUI features so `paste` is no longer in the lockfile.
- Kept narrow audit exceptions for no-fixed-release paths:
  - `rustls-pemfile 2.2.0`, latest visible crates.io release
  - `ttf-parser 0.25.1`, latest visible crates.io release; egui line that removes it requires Rust 1.92, above envctl MSRV 1.88
  - `quick-xml 0.39.4`, fixed upstream at `>=0.41.0` but pinned by latest visible `wayland-scanner 0.31.10`
- Verified `cargo tree -i paste --workspace --target all` and `cargo tree -i number_prefix --workspace --target all` no longer match packages.
- Installed and ran CI's `cargo-audit 0.22.1`; `bash ci/gates/cargo-audit.sh` passed.
- Reverified envctl catalog behavior against Yazelix:
  - import summary: 10 tables, 2311 rows, 58 config files, 2032 settings, 44 env vars, non-mutating
  - render summary: 30 generated files, 30 generated config rows, 3179231 bytes, non-mutating repo
  - rendered tables: `config_files.json` has 58 rows, `settings.json` has 2032 rows
- Additional checks passed:
  - envctl `cargo fmt --check`
  - envctl `cargo clippy --workspace -- -D warnings`
  - envctl `cargo check -p envctl-gui`
  - envctl `cargo test -p envctl-engine catalog::tests`
  - envctl `bash ci/gates/meta-local-policy.sh`
  - loop_lib `cargo fmt --check`
  - loop_lib `cargo test test_build_command`

## Completion Evidence

- loop_lib commit: `1dbef01ca2b80faad71650993728e09827f7c22e`
- envctl commit: `9dac17b5bb633069766d0695d75cca699c64b547`
- envctl clippy follow-up commit: `3a2ddb137331b9280b5d625d7b2fad3dfc30f0cd`
- GitHub PRs updated:
  - loop_lib PR 9 branch `codex/envctl-runner-command-builder`
  - envctl PR 409 branch `codex/codedb-yazelix-config-catalog`
