# Streamlining Audit (April 2026)

## Summary

This pass used the repo's delete-first protocol:

1. question the requirement
2. delete dead or bundled surface first
3. simplify only the surviving owner
4. verify the narrower contract
5. record the new seam

Current Nushell baseline:

- `tokei nushell/scripts -s code`
- `159` files
- `30,467` code lines

This is still large, but the cheap compatibility and wrapper pass is mostly exhausted. The remaining work is now mostly real ownership seams, not obvious dead helpers.

## What Was Removed

The streamlining lane already deleted or internalized:

- dead wrapper exports with no repo callers
- compatibility-only aliases that no longer matched the supported command surface
- stale helper modules whose surviving logic belonged to an existing owner
- old shell-hook migration behavior for generations `v1` through `v3`
- duplicate Zellij/Yazi merger logic that treated raw text surgery as a generic requirement
- broad fake-public helper exports that only served same-file use

Representative deletions:

- `nushell/scripts/utils/config_manager.nu`
- `nushell/scripts/utils/nix_env_helper.nu`
- `nushell/scripts/utils/constants_with_helpers.nu`
- `nushell/scripts/utils/profile.nu`
- multiple dead wrapper exports in `ascii_art.nu`, `zellij.nu`, `terminal_launcher.nu`, `config_contract.nu`, and adjacent helpers

## Large Files That Are Acceptable For Now

These are large, but currently justified enough to avoid reflexive splitting:

- `nushell/scripts/dev/test_yzx_workspace_commands.nu`
  - wide default-lane workspace contract coverage
- `nushell/scripts/dev/test_yzx_generated_configs.nu`
  - broad generated-config invariants with real regressions
- `nushell/scripts/dev/test_yzx_core_commands.nu`
  - config/import/edit command behavior coverage
- `nushell/scripts/utils/ascii_art.nu`
  - concentrated asset/renderer logic rather than mixed subsystem ownership
- `nushell/scripts/maintainer/update_workflow.nu`
  - one maintainer workflow family with a coherent owner

These may still shrink later, but they are not the highest-value refactor targets right now.

## High-Value Remaining Seams

These are the worthwhile remaining refactor targets after the cheap cleanup pass:

1. `nushell/scripts/utils/devenv_backend.nu`
   - runtime activation, refresh intent, backend-shell execution, and launch-profile reuse still live close together
2. `nushell/scripts/utils/launch_state.nu`
   - recorded profile freshness, profile activation env, and launch-state persistence remain a dense runtime owner
3. `nushell/scripts/utils/config_migrations.nu`
   - the plan/apply engine is smaller than before, but the migration engine and transaction model are still a meaningful seam
4. `nushell/scripts/utils/config_migration_transactions.nu`
   - durable managed-write behavior and rollback rules deserve continued hardening
5. `nushell/scripts/utils/terminal_launcher.nu`
   - host terminal detection, wrapper preference, and detached launch transport are still braided together
6. `nushell/scripts/utils/config_surfaces.nu`
   - canonical surface reconciliation is legitimate, but pack-sidecar merge and relocation behavior are still dense enough to deserve care

## What This Means

- `yazelix-jxh` can stop as a broad delete-only lane once the current inventory is recorded.
- `yazelix-790` can stop as an umbrella once the shortlist above is accepted and later work happens in narrower beads.
- Future streamlining work should prefer these real seams over more micro-pruning.

## Traceability

- Bead: `yazelix-790`
- Bead: `yazelix-jxh`
