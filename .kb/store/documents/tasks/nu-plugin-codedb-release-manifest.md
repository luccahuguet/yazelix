---
id: 019f21b0-dc77-7542-9ff2-ad6d9067f8c0
slug: tasks/nu-plugin-codedb-release-manifest
title: "Generate CodeDB release manifest"
type: task
status: completed
priority: high
tags: [codedb, nu_plugin, release, manifest, CDB047]
---

## Source Task

`/home/flexnetos/Downloads/nu_plugin/execution/TASK_GRAPH.csv` row CDB047.

- Phase: release
- Depends on: CDB046
- Blocks: CDB048
- Target surface: runner
- Allowed files: `manifests/**`
- Forbidden: untracked raw secrets
- Primary artifact: release manifest
- Execution gate: manifest checksums match
- Raw log: `logs/CDB047-manifest.log`
- PRD sections: 19, 21

## Acceptance Criteria

- [x] Release manifest exists under `manifests/**`.
- [x] Checksum file exists under `manifests/**`.
- [x] Package validation summary exists under `manifests/**`.
- [x] Manifest checksums match actual files.
- [x] Manifest does not include raw secret-looking fixture values.
- [x] Validation records evidence in `logs/CDB047-manifest.log`.

## Notes

- Regenerated:
  - `manifests/PACK_MANIFEST.json`
  - `manifests/CHECKSUMS.sha256`
  - `manifests/PACKAGE_VALIDATION.json`
- Checksum scope excludes self-referential generated files:
  `manifests/CHECKSUMS.sha256`, `manifests/PACKAGE_VALIDATION.json`,
  `manifests/PACK_MANIFEST.json`, and the current raw validation log
  `logs/CDB047-manifest.log`.
- The release manifest now reflects the implemented package rather than the
  original execution-pack scaffold.

## Completion Evidence

- Validation log: `/home/flexnetos/Downloads/nu_plugin/logs/CDB047-manifest.log`.
- Evidence:
  - `manifests/PACK_MANIFEST.json` reports `task_id = CDB047`.
  - `file_count_in_checksum_scope = 166`.
  - `wc -l manifests/CHECKSUMS.sha256` reports `166`.
  - `sha256sum -c manifests/CHECKSUMS.sha256` passes for every checksum-scope file.
  - Manifest/checksum parser count parity returned `166`.
  - Raw placeholder secret search over `PACK_MANIFEST.json`, `CHECKSUMS.sha256`, and `PACKAGE_VALIDATION.json` returned none.
  - `manifests/PACKAGE_VALIDATION.json` reports `status = passed` and `checksum_issues = []`.
