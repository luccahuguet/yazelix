# Config Migration Engine

## Summary

Yazelix should own a shared config-migration engine that can detect known stale `yazelix.toml` shapes, preview safe rewrites, apply only deterministic fixes with backup, and clearly separate manual-only migrations from blunt reset flows.

## Why

Users should not need to diff old commits or reset their entire config just because a small schema change landed. The migration engine is the foundation for startup diagnostics, `yzx doctor`, `yzx config migrate`, and later upgrade-summary UX, so it needs one canonical rule registry and one predictable preview/apply contract.

## Scope

This spec covers:

- the shared migration-rule registry and its required metadata
- preview-first behavior for `yzx config migrate`
- explicit apply behavior with backup and normalized TOML rewrite
- rule matching and ordering for safe and manual-only migrations
- migration retention and review policy
- metadata validation and high-signal automated verification

## Behavior

`yzx config migrate` should inspect the active `yazelix.toml` and build a migration plan from a shared registry of known rules. Each rule must carry stable metadata, including an id, release/date context, guarded config paths, rationale, manual-fix guidance, and whether Yazelix may rewrite the config automatically.

The command must default to a read-only preview. The preview should enumerate safe rewrites in rule order, explain manual-only findings without touching them, and state clearly when no known migrations were detected.

When the user reruns with `--apply`, Yazelix should write only the deterministic rewrites from the plan. Before writing, it must back up the original `yazelix.toml`. Because the file is rewritten from parsed TOML, comments and key ordering may be normalized; the command should say so explicitly.

When a rule is ambiguous or lossy, the migration engine must not guess. It should leave the config unchanged for that rule and explain the manual follow-up needed.

## Migration Retention Policy

Migration rules should not accumulate forever without review. Every rule in the shared registry must declare a `review_after_days` horizon so maintainers can revisit whether the rule is still worth carrying.

The policy is review-based, not time-based auto-deletion:

- auto-apply deterministic rewrites should usually be reviewed after about 180 days
- manual-only migration guards should usually be reviewed after about 365 days
- especially dangerous legacy shapes may remain longer, but only by explicit maintainer choice after review

The review question is whether the rule still pays for its complexity. Old low-value rewrites should be removed first. Manual-only guards may stay longer when they keep startup and doctor guidance humane for users who update infrequently.

## Non-goals

- silent config mutation during startup
- treating `yzx config migrate` as a replacement for `yzx config reset`
- inventing fake release metadata for unreleased config changes
- auto-fixing unknown config drift that is not captured by an explicit rule

## Acceptance Cases

1. When a config still contains `zellij.widget_tray = ["layout", ...]`, preview shows the broken value removal and apply rewrites the list without `layout`.
2. When a config still uses `terminal.preferred_terminal` and `terminal.extra_terminals`, preview shows the ordered `terminal.terminals` replacement and apply preserves the same terminal preference order without duplicates.
3. When a config contains legacy cursor-trail fields whose meaning is no longer deterministic, preview marks them manual-only and apply leaves them untouched.
4. When a config is already current, preview says there are no known migrations and apply does not create a backup or rewrite the file.
5. When the migration registry is malformed, validation fails loudly before the engine is trusted by higher-level UX.
6. When a migration rule is added without a positive `review_after_days` value, validation fails loudly before the engine is trusted by higher-level UX.

## Verification

- unit tests: direct migration-plan tests for rule metadata completeness, ordering, deterministic rewrite behavior, manual-only classification, preview rendering, apply behavior, and backup creation
- integration tests: `nu nushell/scripts/dev/test_yzx_commands.nu`
- e2e scripts: `nu nushell/scripts/dev/test_config_migrate_e2e.nu`
- CI checks: `nu nushell/scripts/dev/validate_specs.nu` and `nu nushell/scripts/dev/validate_config_migration_rules.nu`
- manual verification: run `yzx config migrate` and `yzx config migrate --apply` against temp config roots with known stale configs
- metadata validation must also reject rules that do not declare a positive `review_after_days`

## Traceability

- Bead: `yazelix-cr3`
- Defended by: `nu nushell/scripts/dev/test_yzx_commands.nu`
- Defended by: `nu nushell/scripts/dev/test_config_migrate_e2e.nu`
- Defended by: `nu nushell/scripts/dev/validate_config_migration_rules.nu`

## Open Questions

- The `layout` widget removal is newer than the latest tag, so the rule metadata should keep its date plus `introduced_after_version` until the next real release is tagged.
- Later upgrade-note validation should consume this same registry instead of introducing a second migration taxonomy.
