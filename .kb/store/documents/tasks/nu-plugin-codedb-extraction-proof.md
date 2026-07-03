---
id: 019f21ef-3196-73f0-ba3e-99f366868b6f
slug: tasks/nu-plugin-codedb-extraction-proof
title: "Verify CodeDB package extraction proof"
type: task
status: completed
priority: medium
---

## Source

- CSV task: `CDB064`
- Title: Verify ZIP extraction proof before construction
- Depends on: `CDB005`
- Blocks: `CDB065`
- Allowed files: `manifests/EXTRACTION_PROOF.json`, `logs/CDB064-extraction-proof.log`
- Forbidden action: package construction before extraction proof

## Scope

Verify the execution package extraction proof before treating the package
construction as authoritative. This is a package-governance task, not a Rust
implementation task.

## Acceptance

- [x] `manifests/EXTRACTION_PROOF.json` exists and parses
- [x] `logs/CDB064-extraction-proof.log` exists
- [x] Source ZIP SHA-256 matches the manifest
- [x] Source ZIP integrity check passes
- [x] ZIP file count and byte size match the extraction proof

## Evidence

- `jq -r '.status, .source_zip_sha256, .source_zip_path, .extracted_file_count, .extracted_byte_count' manifests/EXTRACTION_PROOF.json` reported `extracted_verified`, SHA `613f4b27326adc75bda89e590cd560cd27a9a1ac8c427f7952d17ad1fa2f39fd`, 23 files, and 140322 extracted bytes
- `sha256sum /home/flexnetos/Downloads/nu_plugin_codedb_execution_pack_v1_1_final_verified.zip` reported `613f4b27326adc75bda89e590cd560cd27a9a1ac8c427f7952d17ad1fa2f39fd`
- `unzip -t /home/flexnetos/Downloads/nu_plugin_codedb_execution_pack_v1_1_final_verified.zip` reported no compressed data errors
- A `zipfile` read confirmed 23 non-directory ZIP entries and source ZIP size 49120 bytes, matching `EXTRACTION_PROOF.json`

## Result

`hard extraction gate passed`: the local `/home/flexnetos/Downloads` copy of
the final verified ZIP matches the extraction proof manifest and has valid ZIP
member integrity.
