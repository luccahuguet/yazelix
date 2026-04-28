# Streamlining Audit (April 2026)

> Status: Historical audit note
> This document records an earlier delete-first pass and still references files
> that have since been removed
> Current planning should start from
> [subsystem_code_inventory.md](./subsystem_code_inventory.md) and
> [architecture_map.md](./architecture_map.md)

## Summary

This pass used the repo's delete-first protocol:

1. question the requirement
2. delete dead or bundled surface first
3. simplify only the surviving owner
4. verify the narrower contract
5. record the new seam

Historical Nushell baseline at the time:

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

## Historical Remaining-Seam List

This earlier shortlist is now superseded and is kept only as audit history:

1. `nushell/scripts/utils/devenv_backend.nu`
   - later deleted during the runtime trim
2. `nushell/scripts/utils/launch_state.nu`
   - later deleted; do not treat it as a current target
3. `nushell/scripts/utils/config_migrations.nu`
   - later deleted with the migration engine
4. `nushell/scripts/utils/config_migration_transactions.nu`
   - later deleted with the migration engine
5. `nushell/scripts/utils/terminal_launcher.nu`
   - still live, but the current targeted lane is the detached-launch probe cut,
     not the older broad shortlist framing
6. `nushell/scripts/utils/config_surfaces.nu`
   - later deleted as the active-config owner; surviving helper behavior moved
     to narrower path/file surfaces

## What This Means

- `yazelix-jxh` can stop as a broad delete-only lane once the current inventory is recorded.
- `yazelix-790` can stop as an umbrella once the shortlist above is accepted and later work happens in narrower beads.
- Future streamlining work should not reuse this historical shortlist. Current
  work should start from the delete-first inventory and ranked Nu deletion
  budget instead.

## Traceability

- Bead: `yazelix-790`
- Bead: `yazelix-jxh`
