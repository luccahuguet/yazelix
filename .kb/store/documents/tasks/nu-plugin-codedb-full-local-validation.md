---
id: 019f21ac-6bf8-7322-b113-f5620e923632
slug: tasks/nu-plugin-codedb-full-local-validation
title: "Run CodeDB full local validation"
type: task
status: completed
priority: high
tags: [codedb, nu_plugin, validation, release, CDB046]
---

## Source Task

`/home/flexnetos/Downloads/nu_plugin/execution/TASK_GRAPH.csv` row CDB046.

- Phase: release
- Depends on: CDB042; CDB043; CDB044; CDB045
- Blocks: CDB047
- Target surface: runner
- Allowed files: all project files
- Forbidden: release without logs
- Primary artifact: validation logs
- Execution gate: fmt/clippy/test/doctor pass
- Raw log: `logs/CDB046-validation.log`
- PRD sections: 19

## Acceptance Criteria

- [x] Full local validation log exists at `logs/CDB046-validation.log`.
- [x] `cargo fmt --check` passes.
- [x] `cargo clippy --all-targets --all-features` exits successfully, with warnings documented.
- [x] `cargo test` passes.
- [x] `codedb doctor --nu`, `codedb doctor --yazelix`, and `codedb doctor --codex` return machine-readable status rows.
- [x] Envctl export JSON, NUON, and CSV parse successfully.
- [x] Task graph duplicate-ID check passes.
- [x] CDB042 deterministic scan test passes.
- [x] CDB043 security/no-leak test passes.
- [x] CDB044 no-mutation test passes.
- [x] CDB045 unsafe-capture test passes.
- [x] Source fixtures remain free of generated `Cargo.lock` files.

## Notes

- Mechanical formatting was applied with `cargo fmt` before the successful run.
- Validation used `/nix/store/amnp0j1awapnf8vqs3bwlyvffpkjl5cl-rust-mixed/bin`
  because the earlier cargo-only path did not provide `cargo fmt`.
- `cargo clippy --all-targets --all-features` exited successfully but emitted
  warnings. Current documented exceptions:
  - `codedb_core::IdentityKey::new` has 9 arguments.
  - `codedb_core` newline-style boolean can be simplified.
  - `codedb_rust_static` has `sort_by`/collapsible-if cleanup warnings.
  - `codedb_store_redb::StoreError` has large error variants.
- `codedb doctor --yazelix` returned a clear degraded row because explicit Yazelix
  Nu environment variables were not set in this shell. This satisfies the CDB031
  behavior of reporting usable or degraded status clearly.

## Completion Evidence

- Validation log: `/home/flexnetos/Downloads/nu_plugin/logs/CDB046-validation.log`.
- Toolchain:
  `/nix/store/amnp0j1awapnf8vqs3bwlyvffpkjl5cl-rust-mixed/bin`.
- Evidence from log:
  - `cargo fmt --check`: passed.
  - `cargo clippy --all-targets --all-features`: exited successfully with documented warnings.
  - `cargo test`: all crate unit/doc tests passed.
  - doctor rows:
    - `--nu`: host Nu available, protocol compatibility available, plugin registration status degraded/unknown without mutation.
    - `--yazelix`: degraded with explicit action to set Yazelix Nu env vars.
    - `--codex`: Codex tool path and integration boundary available.
  - task graph duplicate-ID check counted `69` unique task rows with no duplicates.
  - envctl export formats parsed as `24` rows for JSON, NUON, and CSV.
  - CDB042/CDB043/CDB044/CDB045 Nu tests passed.
  - fixture source lockfile check reported `none`.
