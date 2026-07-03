---
id: 019f2219-0dc5-70b0-b0af-0bb2770554cf
slug: tasks/codedb-envctl-jsonc-settings-parser
title: "Parse Yazelix JSONC settings into envctl catalog rows"
type: task
status: completed
priority: high
tags: [envctl, yazelix, jsonc, catalog, settings]
---

## Overview

The live Yazelix config ingest exposed that canonical Yazelix settings use JSONC (`settings_default.jsonc` and user `settings.jsonc`), while envctl catalog only parsed TOML/YAML/JSON into `settings` rows. Without JSONC parsing, Yazelix settings files can appear in `config_files` but do not become queryable envctl settings rows, which is not enough for later reproduction/materialization.

## Evidence

- `/home/flexnetos/FlexNetOS/src/yazelix/settings_default.jsonc` starts with `//` comments.
- The same file contains JSONC-style trailing commas in nested arrays.
- The new engine regression initially failed because `settings_default.jsonc` produced no `settings` row with `scope = yazelix`.

## Acceptance Criteria

- [x] Envctl catalog infers `.jsonc` files as JSONC, not unknown.
- [x] JSONC comments and trailing commas parse into settings rows without modifying source files.
- [x] Comment-looking strings such as URLs or literal comment markers are preserved.
- [x] Yazelix `settings_default.jsonc` loads into `settings` rows when scanning an explicit Yazelix repo root.

## Progress Log

### 2026-07-02

- Added a small JSONC normalizer to envctl catalog parsing.
- Added parser coverage for comments, trailing commas, URLs, and literal comment marker strings.

## Completion Evidence

- `cargo test -p envctl-engine catalog::tests` passed with `jsonc_parser_keeps_strings_and_removes_comments_and_trailing_commas`.
- Live envctl import reported `settings_default.jsonc` as `format = jsonc`, `parse_status = ok`.
- Live envctl import produced 1,230 settings rows, including `core.debug_mode`, `appearance.mode`, and other rows sourced from `settings_default.jsonc`.
