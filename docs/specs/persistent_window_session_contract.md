# Persistent Window Session Contract

## Summary

When `zellij.persistent_sessions = true`, multiple Yazelix windows are clients of one named logical Zellij session rather than independent live sessions. That changes the semantics of launch, path/bootstrap intent, restart scope, and last-client lifecycle compared with the default non-persistent mode.

## Why

Persistent sessions are not just "normal launch, but reused more." They are a different product contract. The current runtime already behaves that way:

- if the named session exists, Yazelix reattaches instead of creating a fresh live session
- fresh `--path` intent is ignored once that persistent session already exists
- later windows become additional clients of the same logical session

Without stating that explicitly, users and future fixes will keep treating persistent-mode surprises as bugs when some of them are really the point of persistence.

## Scope

- define how Yazelix behaves when `zellij.persistent_sessions = true`
- define what later windows do once the named session already exists
- define what happens to fresh-start intent such as `yzx enter --path ...`
- define the intended difference between persistent-session lifecycle and default non-persistent lifecycle
- define the session-scoped meaning of restart/update transitions in persistent mode

## Contract Items

#### PWS-001
- Type: behavior
- Status: live
- Owner: persistent-session launch/session boundary
- Statement: When `zellij.persistent_sessions = true`, Yazelix windows are
  clients of one named logical session. Later launches attach to the existing
  named session instead of creating independent live sessions
- Verification: automated
  `nu nushell/scripts/dev/test_yzx_workspace_commands.nu`

#### PWS-002
- Type: failure_mode
- Status: live
- Owner: persistent-session reuse warning path
- Statement: Once the named persistent session already exists, fresh bootstrap
  intent such as `--path` does not silently override it. Yazelix warns that the
  existing session is being reused and that the new path will not take effect
  until the session is explicitly killed and recreated
- Verification: automated
  `nu nushell/scripts/dev/test_yzx_workspace_commands.nu`

#### PWS-003
- Type: boundary
- Status: live
- Owner: persistent-session lifecycle semantics
- Statement: In-session state, restart semantics, and update effects are
  session-scoped in persistent mode rather than being modeled as independent
  per-window behavior
- Verification: automated
  `nu nushell/scripts/dev/test_yzx_workspace_commands.nu`; validator
  `nu nushell/scripts/dev/validate_specs.nu`

#### PWS-004
- Type: behavior
- Status: live
- Owner: persistent-session lifecycle
- Statement: Last-client lifecycle may intentionally differ from non-persistent
  mode. A detached named persistent session may be an intended persistence
  feature rather than a leak by default
- Verification: manual persistent-session lifecycle review; validator
  `nu nushell/scripts/dev/validate_specs.nu`

## Behavior

- Persistent mode means one named logical Yazelix session may have multiple clients.
  - The first launch creates the named session.
  - Later launches attach to that existing session instead of creating independent live sessions.
- Persistent mode intentionally diverges from the default non-persistent contract.
  - In non-persistent mode, each window is its own live session.
  - In persistent mode, multiple windows are clients of one live session.
- Once the named persistent session already exists, fresh bootstrap intent does not override it silently.
  - If a user launches with `--path` or similar fresh-start intent after the persistent session exists, Yazelix should warn clearly that the existing session is being reused and the fresh path is ignored.
  - Users who want a fresh starting directory must explicitly kill the named session first.
- Session-local state is shared across persistent-mode windows because those windows are attached to one logical session.
  - Tab state, workspace roots, and in-session behavior belong to that shared session rather than to each client window separately.
- Restart and update transitions are session-scoped in persistent mode.
  - generated-state repair still updates shared durable/materialized state rather than mutating every running process in place.
  - `yzx restart` should be understood as a transition of the named logical session, not as an independent restart of just one attached client.
  - If restart behavior affects other attached clients, that is part of the persistent-session model rather than a cross-window spillover bug.
- Last-client lifecycle may intentionally differ from non-persistent mode.
  - In default non-persistent mode, leaving behind a detached zero-client session is a bug.
  - In persistent mode, keeping the named session alive after clients disconnect may be an intentional persistence feature.
  - Future lifecycle fixes must distinguish those two cases instead of applying one cleanup rule everywhere.

## Non-goals

- redefining the default non-persistent multi-window contract
- specifying the final UI/UX for prompting users before restarting a shared persistent session
- deciding every implementation detail of Zellij attach, detach, or resurrection internals
- defining desktop-launch fallback behavior in this spec

## Acceptance Cases

1. When `persistent_sessions = true` and the named session already exists, a later Yazelix window reattaches to that session instead of creating a new independent session.
2. When a user provides `--path` after the named persistent session already exists, Yazelix warns that the existing session is being reused and that the new path will not take effect until the session is explicitly killed and recreated.
3. When two windows are attached to the same persistent session, shared in-session state is treated as intentional shared-session behavior rather than as cross-window spillover.
4. When restart/update behavior affects a persistent session, it is evaluated at the logical-session scope rather than by pretending each attached client is an independent live session.
5. Lifecycle fixes can distinguish an intentional persistent detached session from a default non-persistent leaked detached session.

## Verification

- integration tests: `nu nushell/scripts/dev/test_yzx_workspace_commands.nu`
- CI checks: `nu nushell/scripts/dev/validate_specs.nu`
- manual verification:
  - launch a persistent Yazelix session, then open another window and confirm it reattaches
  - invoke `yzx enter --path ...` or similar fresh-start intent against an existing persistent session and confirm the warning path

## Traceability

- Bead: `yazelix-vxrb`
- Defended by: `nu nushell/scripts/dev/test_yzx_workspace_commands.nu`
- Defended by: `nu nushell/scripts/dev/validate_specs.nu`

## Open Questions

- Should Yazelix eventually offer an explicit command for restarting or replacing the named persistent session with less surprise than a general `yzx restart` call from one attached client?
- Should persistent-session detachment behavior remain entirely session-owned, or should Yazelix still enforce some cleanup guardrails around obviously broken zero-client persistent sessions?
