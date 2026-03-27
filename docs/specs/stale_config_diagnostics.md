# Stale Config Diagnostics

## Summary

Yazelix should surface stale or unsupported `yazelix.toml` problems through one shared diagnostic contract so startup, refresh, and `yzx doctor` all explain the same issue in the same terms and point to the same safest next action.

## Why

When a config change breaks startup, generic wrapper failures force users to guess whether Yazelix itself is broken or whether their config simply needs a migration. The migration engine from `yazelix-cr3` gives Yazelix enough knowledge to be precise; this spec makes that precision visible at the failure boundary instead of leaving it buried in docs or commit history.

## Scope

This spec covers:

- startup- and refresh-blocking config diagnostics
- `yzx doctor` reporting for known migrations and unsupported config issues
- explicit doctor-driven safe repair for deterministic migrations
- message consistency between startup, refresh, and doctor

## Behavior

When Yazelix reads `yazelix.toml`, it should build a shared config-diagnostic report before continuing. Startup and refresh should block only on known migrations and genuinely unsupported config problems such as unknown fields, type mismatches, or unsupported enum values. They should not block on merely omitted fields that Yazelix can still default safely.

For known migrations, the message should name the exact config path, what changed, and when it changed when that metadata is known. Safe rewrites should point users at `yzx config migrate` and `yzx config migrate --apply`, and the doctor flow may also advertise `yzx doctor --fix` because that path is explicit and backup-first.

For unsupported config that does not map to a known migration, Yazelix should fail clearly without pretending a migration exists. Those messages should still identify the exact field and next step.

`yzx doctor` should consume the same structured report, but it may additionally show missing-field hygiene findings that startup intentionally tolerates. When `yzx doctor --fix` is run and the report contains safe migrations, it should apply the same safe rewrites as `yzx config migrate --apply`.

## Non-goals

- silent startup rewrites
- hiding unknown config problems behind generic launch or refresh wrappers
- treating missing fields as startup blockers when Yazelix can still supply safe defaults
- inventing migration guidance for unsupported config that has no deterministic mapping

## Acceptance Cases

1. When `zellij.widget_tray` still contains `layout`, startup fails before generic Zellij-generation wrapping and points to the known migration with safe next steps.
2. When `yzx refresh` sees the same stale config, it reports the same migration-aware diagnosis instead of falling through to a generic refresh failure.
3. When `yzx doctor --verbose` sees the same stale config, it reports the same path, rationale, and next steps, and `yzx doctor --fix` applies the safe rewrite with backup.
4. When config contains an unsupported value such as `core.refresh_output = "loud"`, Yazelix fails clearly as a config problem but does not pretend a migration exists.
5. When config merely omits fields that Yazelix can default, startup does not fail solely because of the omission, though doctor may still report the drift.

## Verification

- unit tests: startup-blocking classification, startup rendering, doctor rendering, migration-vs-unsupported distinction, and doctor `--fix` application semantics
- integration tests: `nu nushell/scripts/dev/test_yzx_commands.nu` and `nu nushell/scripts/dev/test_yzx_doctor_commands.nu`
- e2e scripts: `nu nushell/scripts/dev/test_stale_config_diagnostics_e2e.nu`
- CI checks: `nu nushell/scripts/dev/validate_specs.nu`
- manual verification: run startup, refresh, doctor, and doctor-fix flows against temp homes with known stale configs

## Traceability

- Bead: `yazelix-27q.1`
- Defended by: `nu nushell/scripts/dev/test_yzx_commands.nu`
- Defended by: `nu nushell/scripts/dev/test_yzx_doctor_commands.nu`
- Defended by: `nu nushell/scripts/dev/test_stale_config_diagnostics_e2e.nu`

## Open Questions

- The changelog and first-run upgrade summary beads should later reuse the same release/date wording so config diagnostics do not drift from the published upgrade notes.
