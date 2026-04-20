# Yazelix Core Boundary

## Summary

Yazelix should not ship or support a separate `Yazelix Core` edition yet.

Current recommendation:

- do not create a separate Core branch or package now
- do not market a no-backend/no-package-management edition as a supported product
- keep `Core` only as a future boundary concept that can be revisited after more runtime and workspace seams are reduced

If revisited later, `Core` should be a narrower support mode built from the same repo and command-family contracts, not a speculative fork.

## Why

The current architecture already tells us what survives a backend reduction and what does not:

- workspace actions and config-surface management are comparatively backend-agnostic
- backend control-plane commands are not
- runtime/distribution surfaces still depend on the current install/runtime model
- cross-language ownership is cleaner than before, but still not small enough to justify a second product promise

The delete-first answer is to make main cleaner first, not to create a second edition that duplicates support burden.

## Scope

- decide whether `Yazelix Core` should exist now
- define the likely keep/drop boundary if revisited later
- define the support-story recommendation

## Decision

### Decision Now

Do not pursue a separate `Yazelix Core` edition now.

### Why Not Now

1. The backend boundary is cleaner, but still active work.
2. Runtime/distribution surfaces still assume the current managed runtime story.
3. Supporting a second edition now would widen the support matrix before the remaining seams are small enough.
4. The current product value still comes from the integrated runtime plus workspace experience together.

## If Revisited Later

The coherent future boundary would look like this:

### Likely Keep

- informational commands such as `yzx`, `yzx why`, `yzx sponsor`, `yzx whats_new`
- workspace actions such as `yzx cwd`, `yzx reveal`, `yzx popup`, `yzx screen`
- discoverability/training commands such as `yzx keys` and `yzx tutor`
- config-surface management such as `yzx config`, `yzx edit`, `yzx import`

### Likely Drop Or Redefine

- `yzx env`
- `yzx run`
- `yzx packs`
- generic Nix housekeeping or other non-workspace utility commands
- any promise that Yazelix provisions the tool/runtime environment itself

### User Responsibilities In That Future Mode

- provide required tools on `PATH`
- own package/runtime provisioning
- accept narrower support around launch/runtime repair
- accept that some current `doctor` or install/update guarantees no longer apply

## Support Story Recommendation

If `Core` is ever revisited:

- keep it in the same repo
- make it a later supported mode or packaging/profile choice
- do not fork the command surface into a separate branch first
- only introduce it after the backend/runtime/distribution seams are narrow enough to support honestly

## Non-goals

- designing a new backend now
- promising a separate package today
- replacing full Yazelix with a lighter edition
- treating `Core` as branding without a support story

## Acceptance Cases

1. There is a clear decision on whether `Yazelix Core` should exist now.
2. The answer is grounded in the current delete-first runtime inventory and runtime contracts rather than intuition alone.
3. The likely keep/drop boundary is explicit enough to guide later work if the idea returns.
4. The current recommendation is specific enough to close the planning bead instead of leaving it vague.

## Verification

- manual review against [v15_trimmed_runtime_contract.md](./v15_trimmed_runtime_contract.md)
- manual review against [rust_migration_matrix.md](./rust_migration_matrix.md)
- manual review against [cross_language_runtime_ownership.md](./cross_language_runtime_ownership.md)
- spec validation: `nu nushell/scripts/dev/validate_specs.nu`

## Traceability

- Bead: `yazelix-qv8c`
- Defended by: `nu nushell/scripts/dev/validate_specs.nu`
