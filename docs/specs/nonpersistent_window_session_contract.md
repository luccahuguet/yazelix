# Nonpersistent Window Session Contract

## Summary

When `zellij.persistent_sessions = false`, each Yazelix launch creates an independent live session even when multiple windows consume the same user config, runtime code, and materialized/generated state. Shared durable state may advance between launches, but already-open windows must not be silently hot-swapped or treated as one logical session.

## Why

Yazelix now has clearer runtime, materialized-state, and activation-state boundaries, but the default multi-window behavior is still mostly implicit. In practice, users can have multiple Yazelix windows open at once, and the risky cases are not ordinary pane/workspace spillover so much as special transitions:

- one window is still running an older built profile while a newer launch uses fresher state
- generated-state repair updates shared materialized state while other windows remain open
- a package-manager upgrade or compatibility-installer rerun changes the runtime on disk while other windows remain open
- closing a non-persistent window may leave a detached zero-client Zellij server behind

Without a written contract, future fixes will keep rediscovering the same questions.

## Scope

- define the default cross-window behavior when `zellij.persistent_sessions = false`
- define what is shared across non-persistent windows and what remains session-local
- define how generated-state repair, external runtime replacement, and `yzx restart` should behave for already-open non-persistent windows
- define the expected lifecycle of the last client in a non-persistent Yazelix session

## Behavior

- The shipped default is non-persistent mode.
  - `persistent_sessions = false` means new Yazelix windows do not intentionally reattach to one logical session.
- Each non-persistent Yazelix entrypoint creates its own live session.
  - This includes new-window launch flows such as `yzx launch` and desktop launch.
  - This also includes current-terminal startup through `yzx enter`.
- Non-persistent windows share durable state, not one live session.
  - Shared durable state includes user config, installed/runtime code, generated configs, recorded launch-profile state, and rebuild hashes.
  - Live session state includes the active Zellij session, attached clients, in-session activation markers, and tab-local workspace state.
- Session-local workspace behavior must stay local to the window that owns that live session.
  - Opening or restarting one non-persistent window must not implicitly reuse another non-persistent window's live Zellij session.
  - Tab-local workspace state, sidebar/editor state, and current activation markers are session-local, not global.
- New launches may use newer durable state than already-open non-persistent windows.
  - That is expected behavior, not cross-window corruption.
  - Older open windows may remain on an older activated profile until explicitly restarted or relaunched.
- generated-state repair updates shared materialized state only.
  - It may build a newer launch profile and update generated/runtime-owned artifacts.
  - It must not silently hot-replace unrelated already-open non-persistent windows.
- external runtime replacement updates the installed runtime for future launches.
  - It may replace the active package root or compatibility-installer runtime path on disk.
  - It must not silently upgrade unrelated already-open non-persistent windows in place.
- `yzx restart` is the explicit live-session transition for a non-persistent window.
  - Restarting one non-persistent window should move that window onto newer durable state without pretending that other open non-persistent windows also transitioned.
  - Other already-open non-persistent windows remain separate live sessions until they are explicitly restarted or relaunched.
- Closing the last client for a non-persistent Yazelix session should end that session.
  - Leaving behind a long-lived detached zero-client Zellij server in default non-persistent mode is a bug, not an intentional persistence feature.

## Non-goals

- defining the persistent-session contract when `persistent_sessions = true`
- requiring all already-open windows to switch profiles immediately after refresh or external runtime replacement
- specifying the full internal implementation of Zellij attach/spawn mechanics
- defining desktop-launch fallback terminal selection in this spec

## Acceptance Cases

1. When a user opens two Yazelix windows in non-persistent mode, each window owns its own live session even if both consume the same user config and runtime.
2. When one non-persistent window uses fresher durable state than another, that is treated as normal version skew between independent live sessions rather than as cross-window spillover.
3. When generated-state repair or external runtime replacement happens while non-persistent windows are already open, those windows are not silently hot-swapped in place; future launches may use newer state, and explicit restart remains the transition surface for existing windows.
4. When `yzx restart` is invoked from one non-persistent window, that window transitions independently and other already-open non-persistent windows remain separate live sessions.
5. When the last client for a non-persistent Yazelix session closes, the session should not remain behind as a hot detached zero-client server.

## Verification

- integration tests: `nu nushell/scripts/dev/test_yzx_workspace_commands.nu`
- CI checks: `nu nushell/scripts/dev/validate_specs.nu`
- manual verification:
  - open two non-persistent Yazelix windows, confirm they do not reattach to one shared session
  - run generated-state repair through startup/doctor or replace the runtime from a package-manager or compatibility-installer context and verify the other open window is not silently replaced
  - close the last client for a non-persistent session and confirm no detached zero-client server remains

## Traceability

- Bead: `yazelix-1zti`
- Defended by: `nu nushell/scripts/dev/test_yzx_workspace_commands.nu`
- Defended by: `nu nushell/scripts/dev/validate_specs.nu`

## Open Questions

- Should Yazelix expose a clearer user-facing surface to show which open windows are on older durable state versus newer durable state?
- Should non-persistent session cleanup be enforced by Yazelix directly, by Zellij configuration, or by a narrower lifecycle helper around launch/restart/exit?
