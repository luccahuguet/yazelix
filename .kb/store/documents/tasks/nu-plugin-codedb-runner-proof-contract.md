---
id: 019f2193-c418-7d11-97a1-abd137f2f6fb
slug: tasks/nu-plugin-codedb-runner-proof-contract
title: "Implement CodeDB runner proof contract"
type: task
status: completed
priority: high
tags: [codedb, nu_plugin, runner, release_gate, CDB039]
---

## Source Task

`/home/flexnetos/Downloads/nu_plugin/execution/TASK_GRAPH.csv` row CDB039.

- Phase: integration
- Depends on: CDB028; CDB029; CDB032
- Target surface: docs; code
- Allowed files: `docs/RELEASE_GATE.md`; `crates/codedb/**`
- Forbidden: release without provenance
- Primary artifact: proof export
- Execution gate: runner-readable proof manifest exists
- Raw log: `logs/CDB039-runner.log`
- PRD sections: 16.6, 19

## Acceptance Criteria

- [x] `codedb export runner_proof_manifest` emits a runner-readable proof manifest.
- [x] The manifest names required release proof gates and records whether each gate is satisfied, degraded, or pending.
- [x] The manifest includes provenance inputs such as schema/table checksums, validation errors, capture gaps, no-mutation proof, and bounded MCP status.
- [x] `docs/RELEASE_GATE.md` documents that runner/fxrun owns release readiness and must consume CodeDB proof exports rather than trusting ad-hoc claims.
- [x] The implementation does not claim release readiness without provenance.
- [x] `logs/CDB039-runner.log` records validation commands and results.

## Notes

- This task prepares the proof contract; it does not ship a release or mark the package complete.
- Keep the export machine-readable and conservative. Unknown or future gates should be explicit rows, not hidden assumptions.

## Completion Evidence

Implemented in the source package under `/home/flexnetos/Downloads/nu_plugin`:

- `crates/codedb/src/main.rs` now supports `codedb export runner_proof_manifest`.
- The export emits `runner_proof_manifest` rows for scan, schema introspection, export checksums, capture gaps, validation errors, no-mutation proof, bounded MCP status, unsafe capture default, redb backup/restore, fixture matrix, generated artifact reproduction, and release readiness.
- Pending and degraded gates are explicit; release readiness remains `pending` when future gates are not complete.
- Every proof row includes `release_without_provenance = forbidden`.
- `docs/RELEASE_GATE.md` now states runner/fxrun owns release readiness and must consume the proof export.

Validation log: `/home/flexnetos/Downloads/nu_plugin/logs/CDB039-runner.log`.

Passing gates recorded there:

- `cargo fmt -p codedb --check`
- `cargo build -p codedb`
- `codedb export runner_proof_manifest --format json` parsed by Nushell; 13 rows
- `codedb export runner_proof_manifest --format nuon` parsed by Nushell; 13 rows
- `codedb export runner_proof_manifest --format csv` parsed by Nushell; 13 rows
- runner manifest, checksum, no-mutation, and pending release-readiness rows verified
- release gate docs contain runner/fxrun ownership and provenance requirements
