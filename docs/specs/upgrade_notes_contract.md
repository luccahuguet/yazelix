# Upgrade Notes Contract

## Summary

Yazelix should keep one short human-facing upgrade surface and one canonical structured upgrade-notes source, then enforce both with cheap local validation and stricter CI checks so config-affecting changes cannot quietly ship without release-note coverage.

## Why

The upgrade UX beads depend on durable, machine-readable release notes rather than ad hoc markdown edits or commit archaeology. Without an explicit contract and validator lane, it is too easy to change config surfaces or bump versions without recording what users need to know.

## Scope

This spec covers:

- the root `CHANGELOG.md`
- `docs/upgrade_notes.toml` as the canonical structured source
- optional major-series summaries in `docs/upgrade_notes.toml` for README projections
- required release-note fields for released and unreleased work
- validator wiring in `prek` and CI
- guarded-file enforcement for upgrade-sensitive surfaces

## Behavior

Yazelix should keep:

- a root `CHANGELOG.md` for concise, user-facing upgrade notes
- `docs/upgrade_notes.toml` as the canonical structured source

The structured notes must include at least the current `YAZELIX_VERSION` and an `unreleased` bucket so the live branch can describe post-release work honestly without inventing fake versions. Each entry must declare its upgrade impact, any historical migration ids it still references, and manual actions when user action is required.

`docs/upgrade_notes.toml` may also include optional major-series summaries, such as `series.v14`, when Yazelix wants a curated README projection of the latest tagged line. Those summaries are not exact-version release notes and should stay high-level.

Cheap validation should run locally and in CI to ensure the files exist, the current version is represented, and the changelog headings and headlines line up with the structured data. Because v15 no longer ships a live config-migration registry, `unreleased` must not use `migration_available`, and historical migration ids are preserved as release-note history rather than validated against live migration rules.

CI should additionally inspect the diff. Version bumps must update both release-note surfaces in the same change. Changes to guarded upgrade-sensitive files must be acknowledged in the relevant structured entry, and the root changelog plus structured notes should change together so the two surfaces do not drift.

## Non-goals

- generating polished prose automatically from commit history
- mutating Beads or GitHub state from CI
- requiring every documentation typo fix elsewhere in the repo to touch the upgrade contract

## Acceptance Cases

1. When `YAZELIX_VERSION` changes, CI fails unless both `CHANGELOG.md` and `docs/upgrade_notes.toml` are updated in the same diff.
2. When guarded config-contract files change without a version bump, CI fails unless the `unreleased` entry acknowledges the changed paths.
3. When `unreleased` declares `upgrade_impact = "migration_available"`, validation fails because v15 no longer ships a live config-migration engine.
4. When the changelog or structured notes drift out of lockstep, validation fails clearly with the exact missing requirement.
5. When maintainers run local hooks, the cheap validator catches missing files, missing entries, or malformed note metadata quickly.

## Verification

- unit tests: validator coverage for required fields, version alignment, no-live-migration enforcement, and diff-aware guarded-file enforcement
- integration tests: `nu nushell/scripts/dev/validate_upgrade_contract.nu`
- e2e scripts: `nu nushell/scripts/dev/test_upgrade_contract_e2e.nu`
- CI checks: `nu nushell/scripts/dev/validate_upgrade_contract.nu --ci`
- manual verification: edit guarded files or note files in a temp repo copy and confirm the validator output

## Traceability

- Bead: `yazelix-27q.2`
- Defended by: `nu nushell/scripts/dev/validate_upgrade_contract.nu`
- Defended by: `nu nushell/scripts/dev/test_upgrade_contract_e2e.nu`

## Open Questions

- A later pass may decide whether some parser-only surfaces deserve promotion into the guarded-file set, but the initial set should stay intentionally narrow to avoid noisy false positives.
