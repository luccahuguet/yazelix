# Cross-Language Runtime Ownership

## Summary

Yazelix keeps typed product behavior in Rust, live workspace truth in the pane orchestrator, interactive tool-local behavior in the tool that owns it, and shell code at stable process boundaries

The former Nushell command and materialization bridge has been deleted. Nushell is an interactive shell surface, not a control-plane owner

## Ownership Map

| Layer | Owns | Must not own |
| --- | --- | --- |
| Rust `yzx` and `yzx_control` | Public commands, help, human rendering, machine reports, launch and maintainer routing | A second shell-owned command registry |
| Rust `yazelix_core` | Config, state, runtime environment, materialization, diagnostics, startup facts, profiles, editor integration, and command metadata | Live Zellij pane truth or native terminal policy |
| Private Rust `yzx_core` | Three packaged subprocess seams required by POSIX or managed-tool launchers | Public command UX or ambient-path discovery |
| POSIX shell | Stable bootstrap, runtime-root handoff, security-wrapper preservation, and narrow managed-tool launchers | Config semantics, workspace state, or generated-runtime policy |
| Nushell | Interactive shell configuration, prompt behavior, and generated extern consumption | Public Yazelix commands, startup ownership, or duplicated runtime logic |
| Rust pane orchestrator | Per-tab workspace root, managed pane identity, focus, layout, and tab-local state | Package, config, or update policy |
| Lua Yazi plugins | In-Yazi keymaps, UI, and thin events to Yazelix owners | Durable workspace or runtime truth |
| Zellij KDL and CLI | Static layout/config shape and command transport | Business logic or config ownership |

## Runtime Activation

Rust owns config normalization, startup facts, runtime-environment construction, materialization planning and repair, status/doctor reports, and human-facing command results. POSIX bootstrap passes explicit roots and preserves host security wrappers before handing off to Rust and the packaged runtime

There is no surviving Nushell activation bridge. Interactive Nushell starts after runtime activation and consumes the environment it receives

## Generated Runtime State

Rust owns generated Zellij/Yazi configuration, shell initializers, recorded materialization state, and repair decisions. Writers are limited to Yazelix state paths and must not absorb user-owned native configuration

Zellij KDL and CLI remain transport and declarative shape, not an alternate state model

## Live Workspace State

The pane orchestrator owns current tab roots, managed pane identities, focus, and layout state. Rust public commands and Lua adapters may request or read that state; they must not infer a competing workspace model from process cwd or pane order

Yazi directory retargeting intentionally does not rename every surrounding workspace label. New panes may still inherit the changed cwd through the managed workspace path without recreating tighter Classic bookkeeping

## Native And Tool-Local State

- Mars owns terminal appearance, opacity, fonts, effects, and terminal cursor consumption
- Zellij owns native preferences and transport
- Helix and Yazi own their native files under the Yazelix config root
- Nushell owns interactive shell behavior
- Lua plugins remain adapters inside Yazi

Main Yazelix coordinates paths and runtime boundaries without duplicating each tool's config language

## Source-Swap Boundary

The final Classic runtime still projects the Nova-shaped semantic root into fixed Classic workspace internals. The source swap deletes that projection and the bounded Classic migration transaction. It does not recreate a Nushell bridge or port Classic implementation owners into Nova

## Non-goals

- moving tool-native behavior into the main semantic root
- removing POSIX launch boundaries that preserve host behavior safely
- treating Lua, KDL, or Nushell as alternate product control planes
- duplicating child-owned logic in a main-repo adapter
- tightening intentionally relaxed workspace-label synchronization before a real user need exists

## Verification

- `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core --test yzx_control_runtime_surface`
- `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core --test yzx_control_workspace_surface`
- `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core runtime_materialization`
- `yzx_repo_validator validate-contracts`

## Traceability
- Defended by: `yzx_repo_validator validate-contracts`
