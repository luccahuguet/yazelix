# Helix Action Bridge Contract

## Summary

Yazelix-to-Helix automation should become typed editor actions instead of
simulated terminal input.

The bridge is the managed Helix integration seam for typed editor actions in
the `yazelix-helix` fork and Yazelix control client.

## Why

The current managed editor path can reuse a Helix pane by focusing it, sending
Escape, writing Helix command-mode text such as `:cd` and `:open`, and pressing
Enter. That works, but it makes a terminal multiplexer pretend to be an editor
API:

- focus timing becomes part of correctness
- user input can race command injection
- Helix command-mode keybindings become transport preconditions
- multi-file opens are strings instead of typed payloads
- stale or missing editor panes fail late and indirectly

The bridge should keep the useful ownership split:

- Helix owns editor state, buffers, selections, cwd, command execution, and
  Steel/plugin execution
- Zellij owns panes, tabs, focus, layout, terminal creation, and workspace
  routing

## Decision

The bridge is a Helix-native local IPC endpoint in
`luccahuguet/yazelix-helix`, transported over native per-instance local IPC:
Unix sockets on Unix-like systems and best-effort named pipes on native
Windows.

Rejected transports:

- Zellij pipe routing: it can find the managed editor pane, but it still cannot
  execute editor actions without sending terminal input to Helix
- state-dir action queue: it avoids key injection but adds polling, stale
  response files, and unclear timeout ownership
- generic terminal keystroke helper: it preserves the bug class under a new name
- loopback TCP: it is cross-platform, but it adds firewall prompts, port
  allocation, bind-address policy, and a broader network-facing threat model for
  a same-user local editor control plane

The bridge may be implemented inside the Yazelix Helix fork directly, or behind
a thin Helix extension module in that fork, but the supported public seam is the
native local IPC protocol described here.

## Contract Items

#### HAB-001
- Type: boundary
- Status: live
- Owner: `yazelix-helix` action bridge endpoint
- Statement: Yazelix-to-Helix editor actions use a Helix-owned local IPC endpoint
  instead of Zellij terminal input for migrated Helix actions
- Verification: automated Linux behavior tests in `yazelix-helix` plus
  `rust_core/yazelix_core` bridge tests and `yzx_repo_validator
  validate-contracts`

#### HAB-002
- Type: ownership
- Status: live
- Owner: Yazelix state root and Helix wrapper
- Statement: Each managed Helix process gets one opaque instance id, one native
  IPC transport, and one registry record under `YAZELIX_STATE_DIR`; no bridge
  client may target a global "current hx"
- Verification: automated tests for wrapper materialization and `yzx_control
  helix` address resolution

#### HAB-003
- Type: invariant
- Status: live
- Owner: Helix bridge client
- Statement: A bridge request identifies the target instance explicitly or by a
  Zellij pane id resolved through the current Yazelix session; ambiguous or stale
  matches are typed errors, not best-effort guesses
- Verification: automated tests in `rust_core/yazelix_core`

#### HAB-004
- Type: boundary
- Status: live
- Owner: pane orchestrator plus Helix bridge client
- Statement: Zellij remains the owner of panes, tabs, focus, layout, and
  workspace routing; the Helix bridge owns only editor-local actions after the
  managed Helix instance has been selected
- Verification: automated tests across `yzx_control_workspace_surface` and
  pane-orchestrator command tests

#### HAB-005
- Type: failure_mode
- Status: live
- Owner: migrated Helix action callers
- Statement: After a Helix action migrates to the bridge, bridge-missing,
  unsupported-action, timeout, authorization, and stale-instance failures are
  reported directly and must not fall back to simulated keystrokes
- Verification: automated bridge client and workspace surface tests

#### HAB-006
- Type: behavior
- Status: live
- Owner: first Helix action slice
- Statement: The first migrated action slice is `helix.open_files`,
  `helix.set_cwd`, and `helix.get_context`
- Verification: automated behavior tests in the `yazelix-helix` bridge and
  `yzx_control helix` client

#### HAB-007
- Type: failure_mode
- Status: planning
- Owner: `yzx doctor helix-steel`
- Statement: Doctor must detect bridge feature support, generated bridge env,
  registry liveness, socket liveness, instance/action schema mismatches, stale
  entries, and missing migrated actions before bridge-backed behavior is treated
  as healthy
- Verification: unverified until implementation; planned automated doctor tests

## Transport

Transport is native local IPC, never terminal input and never loopback TCP by
default.

Unix-like systems use Unix stream sockets. Socket files live below:

```text
$YAZELIX_STATE_DIR/helix_bridge/<session_id>/<instance_id>.sock
```

Native Windows uses named pipes:

```text
\\.\pipe\yazelix-helix-<session_id>-<instance_id>
```

Windows support is best effort until Windows CI or a native Windows maintainer
smoke proves the named-pipe backend.

The bridge directory should be owned by the current user and mode `0700`.
Registry records should be owned by the current user and mode `0600`.

The Helix wrapper generates:

- `session_id`: launch-scoped Yazelix session identity derived from the session
  config snapshot or a generated launch id
- `instance_id`: opaque random id, unique per Helix process
- `auth_token`: opaque random token used to reject accidental cross-instance
  requests

The Helix process owns the transport lifecycle. It creates the transport after
the editor action loop is ready, removes Unix socket files on graceful exit, and
treats failed cleanup as stale state for doctor to report.

## Addressing

Every registry record contains:

```json
{
  "schema_version": 2,
  "session_id": "opaque-session",
  "instance_id": "opaque-instance",
  "transport": {
    "kind": "unix_socket",
    "path": "/home/user/.local/share/yazelix/helix_bridge/opaque-session/opaque-instance.sock"
  },
  "auth_token_path": "/home/user/.local/share/yazelix/helix_bridge/opaque-session/opaque-instance.token",
  "pid": 12345,
  "zellij_session_name": "optional",
  "zellij_tab_position": "optional",
  "zellij_pane_id": "optional",
  "started_at_unix_ms": 1780330000000,
  "managed_config_path": "/home/user/.local/share/yazelix/configs/helix/config.toml"
}
```

Clients may address a bridge target by:

- explicit `instance_id`
- explicit `zellij_pane_id` inside the current Yazelix session
- the current tab's managed editor pane id returned by the pane orchestrator

Multiple Helix instances in one Yazelix session are therefore normal. They are
not disambiguated by process title, cwd, or "most recent editor" heuristics.

If address resolution finds no live registry entry, the client returns
`stale_instance` or `missing_instance`. If it finds more than one live entry for
the same requested address, the client returns `ambiguous_instance`.

## Payload Format

Requests are newline-delimited JSON:

```json
{
  "schema_version": 2,
  "request_id": "uuid-or-random-id",
  "auth_token": "opaque-token",
  "action": "helix.open_files",
  "timeout_ms": 1500,
  "payload": {
    "working_dir": "/home/user/project",
    "file_paths": ["/home/user/project/src/main.rs"],
    "focus": true
  }
}
```

Responses are newline-delimited JSON on the same connection:

```json
{
  "schema_version": 2,
  "request_id": "uuid-or-random-id",
  "status": "ok",
  "data": {}
}
```

Errors use a typed envelope:

```json
{
  "schema_version": 2,
  "request_id": "uuid-or-random-id",
  "status": "error",
  "error": {
    "class": "unsupported_action",
    "message": "Helix bridge action is not supported by this runtime"
  }
}
```

Supported error classes:

- `invalid_payload`
- `unsupported_action`
- `permission_denied`
- `stale_instance`
- `ambiguous_instance`
- `editor_busy`
- `timeout`
- `internal_error`

The default client timeout is `1500` ms. Callers may use shorter timeouts for
read-only context queries and longer timeouts for multi-file opens, but the
request must carry the effective timeout so failures are visible and bounded.

## First Slice Actions

### `helix.open_files`

Open one or more files in the addressed Helix instance.

Payload:

- `working_dir`: directory Helix should use before opening files
- `file_paths`: non-empty array of absolute paths
- `focus`: whether Yazelix should ask Zellij to focus the editor pane before or
  after the action

Helix owns buffer creation and path handling. Zellij may focus the pane, but it
must not send `:open` text for this action once migrated.

### `helix.set_cwd`

Set the addressed Helix instance cwd.

Payload:

- `working_dir`: absolute directory path

This replaces the Helix part of workspace retargeting. Zellij still owns the
workspace mutation and sidebar state; the bridge only updates the editor cwd.

### `helix.get_context`

Return editor context needed by Yazelix-owned integrations.

Response data:

- `cwd`
- `current_file`
- `selection_count`
- `mode`, if cheaply available

This is read-only. It should not become a generic editor introspection API until
a later contract expands it.

## Zellij-Owned Actions

These actions remain Zellij or pane-orchestrator owned:

- `focus_editor`
- `focus_sidebar`
- `toggle_editor_sidebar_focus`
- `toggle_editor_right_sidebar_focus`
- `toggle_sidebar`
- `move_focus_left_or_tab`
- `move_focus_right_or_tab`
- `next_family`
- `previous_family`
- `open_workspace_terminal`
- `open_terminal_in_cwd`
- popup, menu, config UI, and `btm` pane lifecycle
- status-bar state and screen-saver launch
- workspace retargeting as the top-level user action

When workspace retargeting needs to update Helix cwd, the pane orchestrator or
Rust control client should call `helix.set_cwd` after the managed Helix instance
has been resolved. The workspace action itself remains Zellij-owned.

## No Keystroke Fallback

For a migrated Helix action, these are invalid fallbacks:

- send Escape, write `:open`, press Enter
- send Escape, write `:cd`, press Enter
- depend on `:` being bound to `command_mode`
- retry through `debug_write_literal` or `debug_send_escape`

If the bridge is unavailable, the caller fails with the typed bridge error.
Non-Helix editors may keep their current backend until they get their own
contract; that is not a fallback for Helix.

## Doctor And Validation

Before implementation starts, the planned doctor surface is:

- verify the bundled `hx` advertises the Yazelix bridge feature and schema
  version
- verify `yazelix_hx.sh` exports the bridge env and uses the Yazelix state root
- verify bridge directories use owner-only permissions
- verify the current managed editor pane has exactly one live bridge registry
  entry when Helix is the managed editor
- verify registry pid and socket liveness
- verify `ping` or `helix.get_context` succeeds against the addressed instance
- verify the action list includes all actions required by the current Yazelix
  runtime
- warn about stale registry files and remove them only through an explicit
  doctor repair path
- report non-Steel or non-bridge external Helix as incompatible with bridge
  actions unless the external fork advertises the same bridge schema

Implementation validators should also reject main-repo claims that a Helix
action is migrated while the pane orchestrator or Rust control path still sends
terminal input for that same Helix action.

## Non-goals

- generic remote control of arbitrary user Helix instances
- TCP sockets or network-visible editor control
- a generic "send keys" replacement API
- moving Zellij layout, pane, or focus ownership into Helix
- making Neovim use the Helix bridge
- supporting unmanaged `hx` launched outside Yazelix unless it explicitly opts
  into the same bridge contract

## Acceptance Cases

1. A Yazelix session with two Helix panes can open a file in the intended pane
   without relying on focus state.
2. A Yazi file-open action can update Helix cwd and open selected files through
   typed JSON payloads instead of command-mode text.
3. A workspace retarget can keep Zellij as the workspace owner while updating
   Helix cwd through `helix.set_cwd`.
4. A missing, stale, or bridge-incompatible Helix instance produces a typed
   error and does not inject terminal input.
5. `yzx doctor helix-steel` can explain whether the bridge is ready before
   migrated actions depend on it.

## Verification

Current contract verification:

- `yzx_repo_validator validate-contracts`

Planned implementation verification:

- `cargo test --manifest-path ../yazelix-helix/Cargo.toml` for bridge request,
  response, dispatch, and lifecycle behavior
- `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core helix_bridge`
  for client addressing and error handling
- `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core --test
  yzx_control_workspace_surface` for Yazi/workspace integration
- `cargo test --manifest-path ../yazelix-zellij-pane-orchestrator/Cargo.toml
  --lib` for removing Helix keystroke dispatch from migrated paths
- `yzx doctor helix-steel --json` against a fresh Yazelix session

## Traceability

- Defended by: `yzx_repo_validator validate-contracts`
