# First-Run Upgrade Summary

## Summary

Yazelix should show a concise once-per-version summary on the first relevant interactive run after an upgrade, persist that the version was seen in Yazelix-managed state, and let users reopen the same summary later with `yzx whats_new`.

## Why

The changelog and structured upgrade notes are passive surfaces. Users also need a proactive summary at the moment an upgrade becomes relevant, without turning normal startup into noisy release marketing or hiding the recovery commands when config changes matter.

## Scope

This contract covers:

- persisted last-seen version state outside `settings.jsonc`
- current-version note selection from `docs/upgrade_notes.toml`
- first-run suppression on repeated launches
- manual reopen and installed-runtime comparison via `yzx whats_new`
- historical config-shape guidance when older release notes mention migration-era changes

## Behavior

Yazelix should read the current installed `YAZELIX_VERSION`, look up that exact release in `docs/upgrade_notes.toml`, and render a short summary from that record. The summary should be eligible to appear only once automatically per version, on the interactive startup path.

The last-seen version must be stored in Yazelix-managed state, not in the user config file. When the stored version already matches the current installed version, automatic startup display should stay quiet. When the stored version is missing or older, the startup path should show the summary and then record the current version as seen.

`yzx whats_new` should render the same current-version summary on demand when the active runtime has no newer structured notes. When `docs/upgrade_notes.toml` bundled with the active runtime includes release entries newer than the installed `YAZELIX_VERSION`, or a populated `unreleased` entry, the command should instead render those known changes as "changes since installed runtime". The comparison is offline and uses only the active runtime's `YAZELIX_VERSION`, `runtime_identity.json`, and bundled `docs/upgrade_notes.toml`.

Dirty, dev, or unknown source snapshots must be named explicitly in the output instead of pretending the source revision maps exactly to a tagged release. Non-release version strings such as `dev` must fail clearly when the command is asked to select a release-note range.

The command should also mark the current version as seen so intentional manual review does not force a duplicate automatic prompt later.

When historical release notes declare `upgrade_impact = "migration_available"`, the rendered summary should explain that v15 no longer ships an automatic config-migration engine. It should point users toward manual comparison with the current template or `yzx reset config` as a blunt fresh-start path. It should not probe the current config through a migration registry, because that registry is no longer part of the live product.

If the current version has no release-note entry, startup should stay quiet instead of inventing notes. `yzx whats_new` should fail clearly in that case.

## Non-goals

- showing upgrade notes on every launch
- scraping `CHANGELOG.md` directly at runtime
- mutating `settings.jsonc` automatically from the summary path
- showing the full historical changelog during startup
- contacting the network to discover newer release notes
- restoring migration-registry probing to the v15 upgrade-summary path

## Acceptance Cases

1. When the stored last-seen version is absent or older than the installed version, the first startup summary appears and then records the current version as seen.
2. When the same version starts a second time, the automatic summary stays quiet.
3. When the user runs `yzx whats_new` and there are no newer structured notes, the current-version summary renders even if the version was already seen.
4. When the active runtime bundles newer release notes or a populated `unreleased` entry, `yzx whats_new` renders those entries as changes since the installed runtime.
5. Dirty, dev, or unknown runtime snapshots are reported explicitly in `yzx whats_new` output or errors.
6. When historical release notes declare migration ids, the summary renders no-migration-engine guidance instead of pointing to `yzx doctor --fix` as a config rewrite path.
7. When the current version is missing from `docs/upgrade_notes.toml` and no valid release-note range can be selected, `yzx whats_new` fails clearly and startup does not invent a summary.

## Verification

- unit tests: state persistence, suppression logic, and historical config-shape summary rendering in `nushell/scripts/dev/test_yzx_core_commands.nu`
- e2e scripts: `nu nushell/scripts/dev/test_upgrade_summary_e2e.nu`
- integration checks: `nu nushell/scripts/dev/test_yzx_commands.nu`

## Traceability
- Defended by: `nu nushell/scripts/dev/test_yzx_commands.nu`
- Defended by: `nu nushell/scripts/dev/test_upgrade_summary_e2e.nu`
