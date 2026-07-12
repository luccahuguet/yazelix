# Yazelix Workspace Extraction

## Summary

`yazelix_workspace` is the selected future name for a reusable managed workspace engine built from Yazelix's Zellij, Yazi, and editor orchestration.

The extraction is not ready for a separate distribution. The correct next step is an internal module boundary that keeps Yazelix as the proving ground while separating reusable workspace contracts from product runtime policy.

Alternatives considered:

- `yazelix_zellij_workspace`: precise about the Zellij host, but too narrow because the value also depends on Yazi and editor adapters
- `yazelix_editor_flow`: captures the editor/sidebar feel, but misses workspace root, tab-local session state, and terminal-pane behavior
- keeping all workspace orchestration internal: safest short term, but it would leave the most reusable managed-pane/session concepts buried in broad product modules
- extracting popup primitives first: useful for configured floating panes, but not a substitute for the managed workspace/editor/sidebar contract

## Readiness Decision

The extraction readiness state is `internal_boundary_only`.

`yazelix_workspace` should not become a public crate, plugin, or separate repository until the internal boundary proves smaller than the integrated product surface. Workspace orchestration is closest to Yazelix's identity, so extraction must preserve the current tab-local session model instead of exposing a half-product API.

The internal boundary may move code, but it must not change supported runtime behavior by itself. The post-launch and post-Zellij-materialization shrink evaluation keeps this as a no-go for public extraction: the reusable session types are small, and the private Zellij/workspace command split is still surrounded by Yazelix runtime facts, generated Zellij config, Yazi `emit-to`, editor runtime env construction, popup cwd policy, status/cache paths, and pane-orchestrator plugin aliases.

## Zellij Layout Ownership Gate

The current layout ownership decision is intentionally narrow:

- Yazelix core owns built-in layout family metadata, generated layout assets, runtime placeholder substitution, and startup/swap layout file selection
- The pane orchestrator owns live tab-local layout state, sidebar collapsed/open state, and managed pane identity after Zellij starts
- Users may customize top-level KDL layout files and the managed agent command, but brand-new sidebar families are not first-class until the pane orchestrator and layout metadata learn them explicitly
- Home Manager renders the same Yazelix-owned settings surface; it does not own a second layout profile language
- Public `yazelix_workspace` extraction remains blocked until workspace/editor/session command ownership shrinks inside the main repo. Zellij materialization has shed stale no-op layout fragments, but it still owns generated config, plugin permission seeding, built-in layout rendering, and current status-bar integration.

Rejected or deferred layout branches:

- Override-layout resurrection through owned explicit-run pane creation is rejected for this pass. It would require Yazelix to replace too many native Zellij pane-creation surfaces just to make live layout transitions preserve anonymous panes
- The bottom-bar/zen-mode POC based on override-layout is deferred. Existing Zellij fullscreen behavior remains the supported focused-work fallback, and any future barless component toggle should start from the status/layout ownership model instead of this POC
- User-declared declarative Zellij layout profiles are deferred. The supported customization boundary remains explicit KDL files plus `agent.command` and `agent.args`; sidebar geometry is fixed in the Classic bridge

This gate unblocks deletion-first Zellij cleanup. `zellij_materialization.rs`, `zellij_commands.rs`, and Zellij validators should consolidate around the built-in family metadata plus copied top-level custom KDL files instead of preserving duplicate render paths for override-layout or declarative profiles that are not accepted product surfaces.

## In-Scope Flow

The candidate reusable workspace surface includes:

- tab-local workspace root tracking with `bootstrap` versus `explicit` source
- managed editor and sidebar pane identity
- focus transitions between editor, sidebars, and ordinary panes
- sidebar show, hide, and focus-toggle policy
- workspace retargeting through one mutation seam
- opening files into a managed editor pane
- setting the managed editor cwd
- opening a terminal at the active tab workspace root
- reading active-tab session truth through one typed snapshot
- validating current-tab sidebar Yazi identity before reveal or refresh
- layout-family awareness needed to make sidebar visibility coherent

The reusable surface does not include:

- Yazelix install, update, Home Manager, or package management
- terminal launcher selection
- config UI behavior
- status bar, cursor presets, or AI usage widgets
- broad runtime materialization
- command-palette command text
- session persistence or resurrection behavior

Session persistence and resurrection are deliberately out of scope for this decision. A future persistent-session contract can consume the workspace boundary after it exists, but it should not drive this extraction.

## Prerequisites

A future standalone `yazelix_workspace` distribution would require:

- Zellij with plugin pipe support and permission handling compatible with the pane-orchestrator model
- a loaded workspace/orchestrator plugin instance addressed by stable alias
- Yazi only for sidebar reveal, refresh, and selected-file flows
- a supported managed editor adapter, initially Helix and Neovim command sequences
- a runtime-independent way to configure wrapper commands, cwd policy, layout names, and pane titles
- plain-Zellij examples that do not require the full Yazelix runtime root

Inside full Yazelix, those prerequisites are currently provided by generated Zellij config, runtime scripts, config metadata, Yazi plugins, and the pane orchestrator wasm.

## Current Coupling Snapshot

The current extraction gate is blocked by adapter thickness, not by the name or repository shape.

2026-06-06 decision refresh: public extraction remains rejected for now. The
current internal request model is `workspace_session.rs`, which owns pure
payload shaping and response parsing for workspace retargeting, active-tab
workspace roots, sidebar identity, managed editor kind resolution, and terminal
open requests. Existing Yazelix flows already consume that seam from
`workspace_commands.rs`, `workspace_commands/popup.rs`,
`workspace_commands/yazi_sidebar.rs`, `zellij_commands/pipe.rs`, and
`zellij_commands/workspace.rs`.

Current direct adapter thickness measured in this pass:

- `workspace_session.rs`: `340` lines
- `workspace_commands.rs`: `278` lines
- `workspace_asset_contract.rs`: `504` lines
- `zellij_commands/workspace.rs`: `1005` lines
- `pane_orchestrator_client.rs`: `86` lines
- Direct total: `2213` lines before tests, status-cache callers, and contract docs

That shape is not a child-repo extraction candidate. The reusable request seam is
small, while the behavior users invoke still depends on Yazelix runtime paths,
Yazi `emit-to`, editor environment construction, generated Zellij config,
popup cwd policy, status/cache behavior, and pane-orchestrator aliases.

The smallest reusable pieces are already visible:

- `workspace_session.rs` owns typed parsing for active-tab workspace roots, retarget responses, sidebar identity, and pure workspace request payload shaping
- `pane_orchestrator_client.rs` owns the Zellij plugin pipe transport and aliases

Those pieces are not enough for a standalone package. The surrounding product adapters still own the behavior users actually invoke:

- `zellij_commands/pipe.rs` owns Zellij pipe diagnostics and workspace-root reads, but still assumes Yazelix's pane-orchestrator alias
- `zellij_commands/workspace.rs` owns workspace command orchestration, Yazi-to-editor open flow, editor pane creation, terminal pane opening, sidebar hiding, and runtime editor env construction while delegating pure request payload shaping to `workspace_session.rs`
- `zellij_commands.rs` is now a public command export shell; status/cache command code and regressions live under `zellij_commands/status.rs` and `zellij_commands/status/tests.rs`
- `workspace_commands.rs` owns public `yzx cwd`, workspace command config loading, zoxide/path resolution, managed editor kind detection, and current-tab retargeting
- `workspace_commands/popup.rs` owns the yzpp-backed `yzx popup` adapter, popup cwd policy, runtime environment forwarding, and the sidebar refresh close hook
- `workspace_commands/yazi_sidebar.rs` owns `yzx reveal`, `yzx sidebar refresh`, active sidebar lookup, sidebar focus, command availability, and Yazi `emit-to`
- launch and restart adapters still provide the environment and session facts that workspace commands consume
- `zellij_materialization.rs` still wires generated layouts, plugin artifact paths, permissions, keybindings, and status-bar command widgets
- status cache and AI usage widgets are deliberately out of the workspace package, but their tests and command exports still pass through the same broad Zellij command surface

Expected main-repo LOC movement from extracting only the small reusable parsers and pipe client would be minor and would not create a useful non-Yazelix package. Extracting the larger command surface now would pull thousands of lines of Yazelix runtime policy into a child repo and make both repos harder to maintain.

## Public API Shape

If extraction eventually proceeds, the public API should be action/schema based rather than a second CLI parser.

Candidate request families:

- `read_active_tab_session_state`
- `retarget_workspace`
- `focus_editor`
- `focus_sidebar`
- `toggle_editor_sidebar_focus`
- `toggle_editor_right_sidebar_focus`
- `toggle_sidebar`
- `hide_sidebar`
- `open_file`
- `set_managed_editor_cwd`
- `open_workspace_terminal`
- `register_sidebar_yazi_state`

Candidate shared data types:

- workspace root and source
- managed pane ids
- focus context
- layout/sidebar visibility state
- sidebar Yazi identity and cwd
- editor adapter kind
- file-open target list and working directory

The API should accept structured payloads and return structured results. It should not require callers to know Yazelix command names, runtime directory layout, Home Manager state, or generated config file paths.

## Internal-First Plan

The first extraction step stays inside the repository:

1. Keep shrinking the remaining workspace/editor/session adapters after the broad `zellij_commands.rs` and `workspace_commands.rs` splits.
2. Keep the pane orchestrator as the live state owner and keep its wasm build/sync workflow unchanged.
3. Move reusable request/response types and pure decision helpers into focused modules or a private workspace crate.
4. Keep Yazelix product adapters responsible for path resolution, zoxide, runtime wrapper paths, config facts, and Yazi `emit-to` execution.
5. Prove the boundary with existing Yazelix behavior before adding plain-Zellij examples.

No public package should be cut until the internal API can be used by Yazelix without exporting product-only assumptions.

The next useful private split is the remaining status/cache and Yazelix adapter boundary: keep status/cache widgets, popup commands, Yazi `emit-to`, and runtime env construction local, while continuing to make the workspace request layer small enough to measure independently. Popup and Yazi/sidebar code now have private adapters, but that is still an integrated Yazelix boundary rather than a public workspace package.

## Relationship To Other Components

### Pane Orchestrator

The pane orchestrator remains the authoritative owner for live tab-local state. `yazelix_workspace` cannot replace that state owner unless it is the same Zellij plugin boundary under a cleaner package name.

The current plugin command seam is the nearest implementation shape, but the public extraction must hide Yazelix runtime paths and debug commands.

### Yazelix Zellij Popup

Popup extraction is adjacent but separate, and the standalone plain-Zellij popup plugin source lives in the `yazelix-zellij-popup` child repository while Yazelix packages its `yzpp.wasm` artifact for integrated popup, menu, and config UI panes. `yzpp` remains the short Zellij plugin alias and wasm artifact.

The popup surface owns transient floating panes for configured commands. Workspace extraction owns persistent managed editor/sidebar/session behavior. Both may share structured Zellij plugin request conventions, geometry validation, and pane identity helpers, but neither should force the other's release schedule.

### Yazi And Editor Adapters

Yazi remains an adapter. It publishes selected files, cwd, and sidebar identity events; it does not own workspace state.

Editors remain adapters. Helix and Neovim command generation can be reusable, but deciding which editor is configured and when to sync it is Yazelix product policy until the standalone config surface exists.

## Migration Risk

The high-risk areas are:

- tab-local state leaking across Zellij tabs
- stale sidebar Yazi identity targeting the wrong pane
- editor cwd and workspace root drifting apart
- layout-family commands becoming tied to Yazelix-only KDL names
- public API compatibility promises around unstable debug or maintainer commands
- plugin wasm/source drift during refactors
- duplicated Rust/Nushell path resolution

The migration should delete duplicated heuristics and broad-module ownership before introducing a public package.

## Test Strategy

Use the existing Yazelix suite as the first proof:

- pane-orchestrator unit tests for workspace state, sidebar identity retention, focus policy, and editor command sequences
- `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core --test yzx_control_workspace_surface`
- `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core yazi_materialization`
- `yzx_repo_validator validate-workspace-session-contract`
- `yzx_repo_validator validate-workspace-session-contract`
- `nix build .#yazelix` after first-party plugin package changes
- `yzx_repo_validator validate-contracts`

Before public extraction, add a plain-Zellij proof that starts from structured request examples and does not require `yzx`, Home Manager, the config UI, status bar widgets, or the Yazelix runtime root.

## Go/No-Go Criteria

Go for public `yazelix_workspace` only when all of these are true:

- the internal workspace boundary is smaller and clearer than today's broad modules
- `zellij_commands.rs` and `workspace_commands.rs` no longer own mixed workspace/editor/popup/session concerns
- the public request/response schema excludes Yazelix runtime paths and maintainer debug commands
- plain-Zellij examples demonstrate at least focus toggle, workspace retarget, open file, and workspace terminal behavior
- Yazi and editor adapters are optional or clearly declared prerequisites
- tests cover tab-local state, sidebar identity, editor sync, and failure modes without relying on resurrection behavior
- the release and package boundary can be maintained without duplicating pane-orchestrator wasm or config materialization logic

No-go if the candidate API still requires full Yazelix generated config, Home Manager semantics, mutable runtime sidecars, or session resurrection.

## Verification

- `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core --test yzx_control_workspace_surface`
- `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core yazi_materialization`
- `cargo test --manifest-path ../yazelix-zellij-pane-orchestrator/Cargo.toml --lib`
- `yzx_repo_validator validate-workspace-session-contract`
- `yzx_repo_validator validate-contracts`

## Traceability

- Defended by: `yzx_repo_validator validate-contracts`
- Defended by: `yzx_repo_validator validate-workspace-session-contract`
