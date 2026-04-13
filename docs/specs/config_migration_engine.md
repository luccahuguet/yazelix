# Config Migration Engine

> Status: Historical v13/v14 migration-era contract.
> v15 no longer ships the automatic config-migration engine described here. Keep this file as design history only.
> Current v15 behavior is fail-fast unsupported config diagnostics plus manual cleanup or `yzx config reset`; see [v15_trimmed_runtime_contract.md](./v15_trimmed_runtime_contract.md) and [stale_config_diagnostics.md](./stale_config_diagnostics.md).

## Summary

Yazelix should own a shared config-migration engine that can detect known stale `yazelix.toml` shapes, render safe-rewrite guidance, apply only deterministic fixes through the managed-config transaction contract, and clearly separate manual-only migrations from blunt reset flows.

## Why

Users should not need to diff old commits or reset their entire config just because a small schema change landed. The migration engine is the foundation for startup diagnostics, `yzx doctor`, and later upgrade-summary UX, so it needs one canonical rule registry and one predictable preview/apply contract.

## Scope

This spec covers:

- the shared migration-rule registry and its required metadata
- preview/report behavior for startup diagnostics and `yzx doctor`
- explicit apply behavior with backup and normalized TOML rewrite
- rule matching and ordering for safe and manual-only migrations
- migration retention and review policy
- metadata validation and high-signal automated verification

## Behavior

The migration engine should inspect the active `yazelix.toml` and build a migration plan from a shared registry of known rules. Each rule must carry stable metadata, including an id, release/date context, guarded config paths, rationale, manual-fix guidance, and whether Yazelix may rewrite the config automatically.

Startup and `yzx doctor --verbose` must default to a read-only preview/report. That reporting should enumerate safe rewrites in rule order, explain manual-only findings without touching them, and state clearly when no known migrations were detected.

When the user reruns `yzx doctor --fix`, Yazelix should stage only the deterministic rewrites from the plan and commit them through the managed-config migration transaction contract. That means the final managed config set is validated before commit, rollback artifacts are prepared before any canonical target is replaced, and partial writes must not leave the managed config surfaces in a half-applied state. Because the file set is rewritten from parsed TOML, comments and key ordering may be normalized; the fix flow should say so explicitly.

When a rule is ambiguous or lossy, the migration engine must not guess. It should leave the config unchanged for that rule and explain the manual follow-up needed.

## Migration Retention Policy

Migration rules should not accumulate forever without review. Every rule in the shared registry must declare:

- a `review_after_days` horizon
- a `retirement_policy`
- an optional `last_reviewed_on` anchor

The validator should treat that metadata as a maintained contract, not as advisory comments.

The policy is review-based, not time-based auto-deletion:

- auto-apply deterministic rewrites should usually be reviewed after about 180 days
- manual-only migration guards should usually be reviewed after about 365 days
- especially dangerous legacy shapes may remain longer, but only by explicit maintainer choice after review

The review question is whether the rule still pays for its complexity. Old low-value rewrites should be removed first. Manual-only guards may stay longer when they keep startup and doctor guidance humane for users who update infrequently.

### Retirement Workflow

The repo should follow a demote-before-delete policy for deterministic auto-apply rewrites:

1. while a rule is current, it may participate fully in startup preflight and `yzx doctor --fix`
2. once the review window is reached, maintainers must explicitly review the rule
3. if the rule is still worth carrying but no longer worth automatic entrypoint application, it should first be demoted to explicit diagnostic-only reporting
4. only after that demotion phase should the rule be deleted entirely, unless maintainers decide it still pays for itself and record that review

Manual-only guards follow a slightly different policy:

- they do not need a demotion phase because they are already explicit and non-mutating
- at review time, maintainers should either keep them, rewrite them, or delete them

This workflow is encoded in rule metadata:

- `retirement_policy = "demote_to_explicit_then_delete"`
  - required for auto-apply rules
- `retirement_policy = "review_then_delete_or_keep"`
  - required for manual-only rules

`last_reviewed_on` should stay `null` until a human review actually happens. After review, maintainers should either:

- delete the rule
- demote or rewrite it and set a new `last_reviewed_on`
- or explicitly keep it and set a new `last_reviewed_on`

Validation should fail when a rule is overdue for review based on:

- `last_reviewed_on`, when present
- otherwise `introduced_on`

See [Managed Config Migration Transaction Contract](./managed_config_migration_transaction_contract.md) for the narrower write/rollback model that defines how the managed config surfaces are staged and committed safely.

## Non-goals

- silent config mutation during startup
- treating the migration engine as a replacement for `yzx config reset`
- inventing fake release metadata for unreleased config changes
- auto-fixing unknown config drift that is not captured by an explicit rule

## Acceptance Cases

1. When a config still contains `zellij.widget_tray = ["layout", ...]`, preview shows the broken value removal and apply rewrites the list without `layout`.
2. When a config still uses `terminal.preferred_terminal` and `terminal.extra_terminals`, preview shows the ordered `terminal.terminals` replacement and apply preserves the same terminal preference order without duplicates.
3. When a config contains legacy cursor-trail fields whose meaning is no longer deterministic, preview marks them manual-only and apply leaves them untouched.
4. When a config is already current, preview says there are no known migrations and apply does not create a backup or rewrite the file.
5. When the migration registry is malformed, validation fails loudly before the engine is trusted by higher-level UX.
6. When a migration rule is added without a positive `review_after_days` value, validation fails loudly before the engine is trusted by higher-level UX.
7. When a migration rule is missing `retirement_policy`, has an invalid `last_reviewed_on`, or is overdue for retirement review, validation fails loudly before the engine is trusted by higher-level UX.

## Verification

This is now a historical contract. It is no longer defended by live migration-engine tests or the deleted migration-rule validator. Current v15 config behavior is defended by [stale_config_diagnostics.md](./stale_config_diagnostics.md) and [v15_trimmed_runtime_contract.md](./v15_trimmed_runtime_contract.md).

- historical spec validation: `nu nushell/scripts/dev/validate_specs.nu`

## Traceability

- Bead: `yazelix-cr3`
- Defended by: `nu nushell/scripts/dev/validate_specs.nu`

## Open Questions

- The `layout` widget removal is newer than the latest tag, so the rule metadata should keep its date plus `introduced_after_version` until the next real release is tagged.
- Later upgrade-note validation should consume this same registry instead of introducing a second migration taxonomy.
