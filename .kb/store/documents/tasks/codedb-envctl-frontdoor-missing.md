---
id: 019f2213-38f0-7fe3-acae-4068db180490
slug: tasks/codedb-envctl-frontdoor-missing
title: "Restore envctl frontdoor for CodeDB table visibility"
type: task
status: completed
priority: high
tags: [envctl, codedb, catalog, yazelix, frontdoor]
---

## Overview

The live CodeDB-to-envctl table load surfaced that `/home/flexnetos/FlexNetOS/usr/bin/envctl` is not currently installed, even though the envctl source checkout exists at `/home/flexnetos/FlexNetOS/src/envctl`. This blocks the preferred workspace frontdoor for app-visible envctl catalog/table commands.

The deeper issue is that the current envctl catalog command requires an envctl `manifest/` in the target working directory and only imports envctl control-plane file families. Yazelix has no envctl manifest at its repo root, so `envctl catalog import` cannot load Yazelix config/settings files into envctl tables without an upgrade.

## Evidence

- `/home/flexnetos/FlexNetOS/usr/bin/envctl --help` failed with `No such file or directory`.
- `/home/flexnetos/FlexNetOS/src/envctl` exists and builds/runs from source.
- `cargo run -p envctl -- --help` from the envctl checkout succeeds.
- Running envctl from the Yazelix repo with the envctl manifest path fails because Yazelix has no local `manifest/`: `Error: manifest dir not found or unreadable: manifest`.
- Existing envctl catalog tables include `config_files`, `settings`, `paths`, and `observed_facts`, but the default source discovery is envctl-control-plane oriented.

## Acceptance Criteria

- [x] Envctl catalog commands can inspect an explicit Yazelix repo root without requiring Yazelix to contain an envctl `manifest/`.
- [x] Yazelix config/settings file families populate envctl `config_files` rows with `owner_component = yazelix`.
- [x] The app/dashboard render path can display the imported rows from generated catalog projections.
- [x] The missing installed frontdoor is either restored or documented with a verified source-run fallback until install ownership is repaired.
- [x] Verification evidence is recorded in [[tasks/codedb-envctl-yazelix-config-ingest]].

## Progress Log

### 2026-07-02

- Began fix-forward upgrade in `/home/flexnetos/FlexNetOS/src/envctl`.
- Added explicit catalog root options and Yazelix config-family discovery to envctl source.
- Source-run fallback is verified with `cargo run -q -p envctl -- --json catalog --repo-root /home/flexnetos/FlexNetOS/src/yazelix import`.
- Remaining open part: the installed `/home/flexnetos/FlexNetOS/usr/bin/envctl` frontdoor still needs ownership repair, but it no longer blocks source-run live table validation.

## Completion Evidence

- Fix-forward source changes landed in `/home/flexnetos/FlexNetOS/src/envctl`.
- Envctl source-run fallback imported Yazelix rows without a Yazelix-local manifest.
- Envctl render produced dashboard-visible rows under `/tmp/yazelix-envctl-catalog-render`.
- Follow-up frontdoor ownership repair can be handled separately from this table-visibility blocker.
