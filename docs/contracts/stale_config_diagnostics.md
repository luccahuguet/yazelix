# Stale Config Diagnostics

## Summary

Yazelix should surface stale or unsupported `settings.jsonc` problems through one shared diagnostic contract so startup and `yzx doctor` both explain the same issue in the same terms and point to the same safest next action.

## Why

When a config change breaks startup, generic wrapper failures force users to guess whether Yazelix itself is broken or whether their config is stale. The live contract is narrow: auto-migrate old TOML inputs only when conversion is unambiguous and safe, otherwise detect unsupported config precisely, fail fast, and point users to manual cleanup or `yzx reset config`.

## Scope

This contract covers:

- startup-blocking config diagnostics
- `yzx doctor` reporting for unsupported config issues
- explicit manual recovery guidance for removed or stale config fields
- message consistency between startup and doctor

## Behavior

When Yazelix reads `settings.jsonc`, it should build a shared config-diagnostic report before continuing. Startup should block only on genuinely unsupported config problems such as malformed JSONC, unknown fields, type mismatches, unsupported enum values, or stale old TOML inputs next to the canonical file. It should not block on merely omitted fields that Yazelix can still default safely.

For unsupported config, Yazelix should fail clearly without pretending an unsafe migration exists. Messages should identify the exact field or stale input and next step, usually manual cleanup followed by a retry. `yzx reset config` is a blunt fallback, not a silent rewrite path.

`yzx doctor` should consume the same structured report, but it may additionally show missing-field hygiene findings that startup intentionally tolerates. `yzx doctor --fix` may create missing defaults, but it should not perform ambiguous config migration or silently rewrite user-authored settings.

## Non-goals

- silent startup rewrites
- hiding unknown config problems behind generic launch or refresh wrappers
- treating missing fields as startup blockers when Yazelix can still supply safe defaults
- inventing migration guidance for unsupported config
- restoring broad historical config-migration registries outside the safe old-TOML-to-JSONC gate

## Acceptance Cases

1. When config contains an unsupported value such as `"welcome_style": "loud"`, Yazelix fails clearly as a config problem but does not pretend a migration exists.
2. When `yzx doctor --verbose` sees the same stale config, it reports the same path and manual next steps without advertising `yzx doctor --fix` as a config migration path.
3. When config contains a removed legacy field such as `shell.enable_atuin`, startup fails clearly and leaves the file untouched.
4. When config merely omits fields that Yazelix can default, startup does not fail solely because of the omission, though doctor may still report the drift.

## Verification

- unit tests: startup-blocking classification, startup rendering, doctor rendering, removed-field rejection, and no-migration guidance
- integration tests: `nu nushell/scripts/dev/test_yzx_core_commands.nu`, `nu nushell/scripts/dev/test_yzx_generated_configs.nu`, and `nu nushell/scripts/dev/test_yzx_doctor_commands.nu`
- e2e scripts: `nu nushell/scripts/dev/test_stale_config_diagnostics_e2e.nu`
- CI checks: `yzx_repo_validator validate-contracts`
- manual verification: run startup, doctor, and doctor-fix flows against temp homes with known stale configs

## Traceability
- Defended by: `nu nushell/scripts/dev/test_yzx_core_commands.nu`
- Defended by: `nu nushell/scripts/dev/test_yzx_generated_configs.nu`
- Defended by: `nu nushell/scripts/dev/test_yzx_doctor_commands.nu`
- Defended by: `nu nushell/scripts/dev/test_stale_config_diagnostics_e2e.nu`

## Open Questions

- Historical upgrade notes can still say that older releases had migration-aware UX, but the live diagnostic contract should not depend on a broad migration registry.
