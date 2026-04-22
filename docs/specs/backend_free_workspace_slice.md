# Backend-Free Workspace Slice

## Summary

Yazelix should define one explicit backend-free workspace slice that already works in `runtime_root_only` mode with host-provided tools on `PATH`.

This is not a new product edition. It is an executable proof that the surviving workspace UX can run cleanly after deleting backend ownership, as long as the user still has:

- a valid Yazelix runtime root with the shipped scripts and assets
- a host-provided Nushell binary
- the session-local tools required by the specific workspace flow

## Why

The repo has already separated a large amount of workspace truth from backend control-plane logic:

- tab root, sidebar identity, and managed-editor targeting now live behind the pane orchestrator boundary
- install/update/doctor now have explicit runtime/distribution tiers

But those architectural claims are not enough by themselves. There needs to be a concrete proof slice showing that real workspace commands still behave coherently when Yazelix is not provisioning the backend/runtime environment for you.

## Scope

- define the proven backend-free workspace slice
- define the required host/runtime assumptions for that slice
- define what is intentionally out of scope because it still belongs to backend ownership

## Proof Mode

The proof mode for this slice is `runtime_root_only`:

- there is a real Yazelix runtime root
- there is no installer-owned `runtime/current`
- there is no requirement that the runtime root ship `libexec/nu`
- host-provided `nu` is allowed and expected

This is the honest closest current analogue to a future narrower workspace-oriented mode.

## Proven Slice

### `yzx reveal`

- The stable CLI leaf path can dispatch directly to the lightweight reveal helper without bootstrapping the full command suite.
- Reveal behavior depends on the current Zellij/Yazi workspace state, not on backend activation.
- Required local conditions:
  - inside a Yazelix/Zellij session
  - sidebar enabled
  - `ya` available when Yazi actions must be emitted

### `yzx popup`

- The stable CLI leaf path can dispatch directly to the lightweight popup module without bootstrapping the full command suite.
- Popup wrappers can fall back to host-provided `nu` when the runtime root does not ship `libexec/nu`.
- Popup cwd derives from workspace/session state, not backend provisioning.
- Required local conditions:
  - inside Zellij
  - the shipped popup wrapper scripts exist in the runtime root
  - host-provided `nu` is available

### `yzx menu`

- The stable CLI leaf path can dispatch directly to the lightweight menu module without bootstrapping the full command suite.
- The palette UI itself is backend-free enough to run in `runtime_root_only`.
- Current limit:
  - `yzx menu` is still a mixed seam because the user may choose backend-bound commands from the palette

### `yzx edit`

- Managed config-target resolution is backend-free.
- Editor launch context can resolve the canonical managed Helix wrapper or ambient editor path without needing backend provisioning ownership.
- Required local conditions:
  - a usable editor command is available from the canonical launch env or ambient shell

### `yzx cwd`

- Tab-local workspace retargeting is owned by Zellij session state and the pane orchestrator, not by backend provisioning.
- The proof here is not “works outside Zellij”; it is that the workspace mutation path is session-local once a supported session exists.
- Required local conditions:
  - inside Zellij
  - pane orchestrator available

## Explicit Non-proof Areas

This slice does **not** prove that these families work without backend ownership:

- `yzx launch`
- `yzx enter`
- `yzx env`
- `yzx run`
- `yzx restart`
- distribution/update/repair commands

Those still depend on backend activation, rebuild, or runtime/distribution ownership by design.

## Remaining Mixed Seams

- `yzx menu` still spans backend-bound commands even though the palette UI itself is lightweight.
- Workspace flows still rely on shipped runtime scripts and assets; this proof deletes backend ownership, not the runtime root itself.
- Session-local requirements such as Zellij, pane-orchestrator permissions, and Yazi availability remain real prerequisites for the commands that need them.

## Acceptance Cases

1. There is a named `runtime_root_only` proof mode for backend-free workspace behavior.
2. The repo has real tests showing workspace leaf commands can run from that mode with host-provided `nu`.
3. The surviving workspace commands and their prerequisites are explicit.
4. The still-backend-coupled command families are called out directly instead of being left ambiguous.

## Verification

- integration tests:
  - `nu -c 'source nushell/scripts/dev/test_yzx_workspace_commands.nu; run_workspace_canonical_tests'`
  - `nu -c 'source nushell/scripts/dev/test_yzx_popup_commands.nu; run_popup_canonical_tests'`
- manual review against:
  - [workspace_session_contract.md](./workspace_session_contract.md)
  - [cross_language_runtime_ownership.md](./cross_language_runtime_ownership.md)
  - [runtime_distribution_capability_tiers.md](./runtime_distribution_capability_tiers.md)

## Traceability

- Bead: `yazelix-5wao`
- Defended by:
  - `nu -c 'source nushell/scripts/dev/test_yzx_workspace_commands.nu; run_workspace_canonical_tests'`
  - `nu -c 'source nushell/scripts/dev/test_yzx_popup_commands.nu; run_popup_canonical_tests'`
