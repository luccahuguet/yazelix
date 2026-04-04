# Open Window Update Transition Contract

## Summary

Yazelix should treat `yzx refresh`, `yzx update runtime`, and `yzx restart` as different transition surfaces across already-open windows. `yzx refresh` updates shared materialized state, `yzx update runtime` updates the installed runtime on disk, and `yzx restart` is the explicit live-session transition surface. None of those commands should silently hot-swap unrelated already-open non-persistent windows in place.

## Why

The current runtime model is much clearer than before, but the user-visible update story is still spread across CLI notes, runtime helpers, and a few tests. The real confusing cases are cross-window transitions:

- a refresh builds a newer launch profile while other windows remain open
- a runtime update replaces the installed runtime on disk while other windows remain open
- a restart moves one live session onto newer state
- persistent-session mode changes the scope of what counts as "one session"

Without an explicit contract, future fixes will keep rediscovering whether a given transition should be local, global, or session-scoped.

## Scope

- define what `yzx refresh` means for already-open windows
- define what `yzx update runtime` means for already-open windows
- define what `yzx restart` means for already-open windows
- define how the restart scope differs between default non-persistent windows and persistent-session mode

## Behavior

- `yzx refresh` updates shared materialized state.
  - It may rebuild the `devenv` shell/profile, refresh launch-profile state, and regenerate runtime-owned generated state.
  - It does not hot-replace unrelated already-open windows.
  - After refresh completes, the current window remains on its existing live session until explicitly restarted or relaunched.
- `yzx update runtime` updates the installed runtime on disk.
  - It may replace `runtime/current`, the installed `yzx` CLI wrapper, runtime-local tools, shell hooks, and generated runtime configs.
  - It does not silently upgrade unrelated already-open live sessions in place.
  - Future launches should use the updated installed runtime.
- `yzx restart` is the explicit live-session transition surface.
  - In the default non-persistent contract, restarting one window should move that window onto newer durable state without pretending other already-open windows also transitioned.
  - In persistent-session mode, restart behavior is evaluated at the logical-session scope because multiple windows may be clients of one named persistent session.
- Version skew across open windows is allowed.
  - After refresh or runtime update, a newly launched window may legitimately use newer state than an older still-open window.
  - That is expected until the older window is explicitly restarted or relaunched.
- The command meanings stay intentionally distinct.
  - `yzx refresh` is a durable-state/materialization operation.
  - `yzx update runtime` is an installed-runtime/distribution operation.
  - `yzx restart` is a live-session transition operation.

## Non-goals

- redefining the non-persistent or persistent session contracts themselves
- forcing all open windows to upgrade immediately after refresh or runtime update
- specifying every internal helper call or launch-state write path
- defining the final UX for prompts or confirmations around restarting shared persistent sessions

## Acceptance Cases

1. When `yzx refresh` completes, it may produce newer shared materialized state, but unrelated already-open windows are not silently hot-swapped in place.
2. When `yzx update runtime` completes, future launches use the updated installed runtime, but unrelated already-open live sessions are not silently upgraded in place.
3. When `yzx restart` is invoked from one non-persistent window, that window transitions explicitly and other open non-persistent windows remain separate live sessions until they are restarted or relaunched.
4. When `yzx restart` is invoked while multiple windows are attached to one persistent session, the resulting behavior is evaluated at the shared logical-session scope rather than by pretending each client window is independent.
5. A user-facing explanation of these commands can distinguish durable-state updates from live-session transitions without relying on implementation trivia.

## Verification

- CI checks: `nu nushell/scripts/dev/validate_specs.nu`
- integration tests: `nu nushell/scripts/dev/test_yzx_workspace_commands.nu`
- manual verification:
  - run `yzx refresh` with one window open and confirm the window is not silently replaced
  - run `yzx update runtime` and confirm a fresh launch uses newer runtime state while an older open window remains on its current live session until restart
  - compare non-persistent and persistent restart behavior across multiple open windows

## Traceability

- Bead: `yazelix-jehj`
- Defended by: `nu nushell/scripts/dev/validate_specs.nu`
- Defended by: `nu nushell/scripts/dev/test_yzx_workspace_commands.nu`

## Open Questions

- Should Yazelix eventually expose a clearer status surface that shows when one open window is on older durable state than another?
- Should `yzx update runtime --restart` grow more explicit UX around persistent-session scope so attached clients are less surprising?
