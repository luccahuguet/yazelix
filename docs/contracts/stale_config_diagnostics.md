# Stale Config Diagnostics

## Summary

Yazelix should surface stale or unsupported `config.toml` problems through one shared diagnostic contract so startup and `yzx doctor` both explain the same issue in the same terms and point to the same safest next action.

## Why

When a config change breaks startup, generic wrapper failures force users to guess whether Yazelix itself is broken or whether their config is stale. The live contract is narrow: inherit omitted values from the packaged defaults, detect unsupported explicit config precisely, fail fast, and point users to manual cleanup or `yzx reset config` instead of rewriting user input automatically.

## Scope

This contract covers:

- startup-blocking config diagnostics
- `yzx doctor` reporting for unsupported config issues
- explicit manual recovery guidance for removed or stale config fields
- sparse explicit-value ownership for user-owned `config.toml`
- message consistency between startup and doctor

## Behavior

`config_default.toml` owns the current packaged semantic defaults. User-owned `config.toml` contains only explicit values. An absent file or field inherits the packaged value, while a present field remains explicit even when its value happens to equal the current default. Yazelix must not write omitted defaults into the user file or attach hidden contract state to it.

`yzx config set`, `yzx config ui`, and onboarding write only explicit values. `yzx config unset` removes the selected value and removes the file when no semantic values remain. `yzx reset config` backs up and removes the file, returning the whole root surface to inherited defaults. Home Manager renders only declared semantic options when it owns the file.

Yazelix builds a shared config-diagnostic report before continuing. Startup blocks on malformed TOML, unknown fields, type mismatches, unsupported enum values, removed fields, ambiguous stale fields, or unsupported old TOML inputs next to the canonical file. It does not report omitted fields as drift because omission is the supported inheritance mechanism.

For unsupported config, Yazelix should fail clearly without pretending an unsafe migration exists. Messages should identify the exact field or stale input and next step, usually manual cleanup followed by a retry. `yzx reset config` is a blunt fallback, not a silent rewrite path.

`yzx doctor` consumes the same structured report. An absent `config.toml` is healthy and does not offer a fix action. `yzx doctor --fix` must not materialize inherited defaults or silently rewrite user-authored settings.

The retired `settings.jsonc` contract chain may still use Ratconfig contract state internally while performing the one-time, backup-first Classic migration. That state is migration input only and must not survive in canonical `config.toml`.

## Non-goals

- silent ambiguous startup rewrites
- materializing packaged defaults into user config
- mutating Home Manager-owned or read-only settings
- hiding unknown config problems behind generic launch or refresh wrappers
- treating missing files or fields as drift
- inventing migration guidance for unsupported config
- restoring broad historical config-migration registries or old-TOML-to-JSONC auto-rewrites
- restoring retired `settings.jsonc` as a second live owner

## Acceptance Cases

1. When config contains an unsupported value such as `"welcome_style": "loud"`, Yazelix fails clearly as a config problem but does not pretend a migration exists.
2. When `yzx doctor --verbose` sees the same stale config, it reports the same path and manual next steps without advertising `yzx doctor --fix` as a config migration path.
3. When config contains a removed legacy field such as `shell.enable_atuin`, startup fails clearly and leaves the file untouched.
4. When config omits fields, startup inherits them from `config_default.toml`, doctor reports the surface as healthy, and the user file remains untouched.
5. When a packaged default changes, an omitted field follows the new value while a present field remains explicit even when it matched the old default.
6. When the last explicit field is unset, Yazelix removes the empty config file; `yzx reset config` does the same for the whole root surface after its backup-first flow.
7. When legacy `settings.jsonc` migrates, every surviving legacy value becomes explicit in `config.toml`; migration does not guess intent by comparing values with current defaults and does not copy hidden Ratconfig state.

## Verification

- unit tests: sparse normalization, packaged-default inheritance, explicit-equals-default persistence, unset/reset removal, startup-blocking classification, doctor rendering, owner/read-only no-write behavior, removed-field rejection, and legacy migration
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
