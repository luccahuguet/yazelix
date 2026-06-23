# Zellij Config Pack Boundary

## Summary

`yazelix_zellij_config_pack` is the in-tree Rust crate for deterministic Zellij
config and layout rendering. Main Yazelix consumes the workspace crate and writes
the rendered output into the active state directory.

The crate owns pure rendering. The `yazelix_core` materializer keeps product
policy, runtime integration, filesystem materialization, plugin artifact
resolution, doctor and repair behavior, and live workspace/session behavior.

## Ownership

| Surface | Owner |
| --- | --- |
| Settings normalization and defaults | Main Yazelix |
| Runtime identity, package paths, and generated state roots | Main Yazelix |
| Zellij action registry policy and supported keybinding semantics | Main Yazelix |
| Yazelix default native settings, including `scroll_buffer_size 5000` when the user has not set an active value | Main Yazelix |
| Pane-orchestrator, popup, and status-bar package/artifact resolution | Main Yazelix plus existing child repos |
| Live tab, pane, workspace, status-bus, and session facts | Pane orchestrator plus Main Yazelix adapters |
| Filesystem writes, repair decisions, doctor checks, and permission seeding | Main Yazelix |
| Layout templates and layout-family metadata that are not product policy | `rust_core/yazelix_zellij_config_pack` |
| Deterministic KDL config/layout rendering and merge mechanics | `rust_core/yazelix_zellij_config_pack` |
| Config-pack fixture and equivalence tests | `rust_core/yazelix_zellij_config_pack`, with `yazelix_core` keeping integration tests |

## Boundary

The crate boundary must be a pure function:

```text
render_zellij_config_pack(request) -> output
```

The request is structured data supplied by main Yazelix. It may include:

- normalized Zellij-relevant settings
- managed native Zellij config text or an already-parsed neutral representation
- runtime paths selected by main
- resolved plugin URLs or package paths selected by main
- child-rendered status-bar plugin block text from `yazelix_zellij_bar`
- popup command/block data selected by main
- deterministic generation metadata such as schema version and fingerprint input

The output is structured data returned to main. It may include:

- rendered `config.kdl`
- rendered layout files keyed by layout name
- warnings or validation errors for malformed render input
- a renderer schema/version field

## Forbidden Dependencies

The renderer must not read or infer:

- `~/.config/yazelix`
- `~/.local/share/yazelix`
- `YAZELIX_RUNTIME_DIR` or other Yazelix runtime environment variables
- Home Manager ownership state
- main-repo source paths
- Zellij live session state
- pane ids, tab ids, active workspaces, status cache files, or orchestrator pipes
- first-party plugin source trees or mutable adjacent checkouts

If the renderer needs one of those facts, main must resolve it before the
request is built or the boundary is not ready for extraction.

## Consumption Gates

`yazelix_core` may consume config-pack behavior only when all gates pass:

1. The renderer accepts explicit structured input and produces
   deterministic config/layout output
2. Generated Zellij output has equivalence tests for representative
   default, custom popup, native merge, status-bar, and layout-fragment cases
3. The renderer has no hidden filesystem, environment, runtime, Home Manager,
   live Zellij, or adjacent-checkout reads
4. Status-bar rendering is consumed from `yazelix_zellij_bar`; popup and
   pane-orchestrator artifacts are consumed from their existing child owners
5. `yazelix_core` keeps only policy, path resolution, artifact resolution, writes,
   doctor/repair, and integration validation
6. `yazelix_core` does not absorb renderer internals back into
   `zellij_materialization.rs`

## Deletion Bar

The crate boundary should keep `zellij_materialization.rs` from owning these
groups:

- pure KDL rendering and merge mechanics
- layout template rendering that does not encode main product policy
- config-pack fixture tests that can run without a Yazelix checkout
- renderer-owned layout-family metadata

It is not a valid boundary if `yazelix_core` keeps parallel renderer code,
fallback config/layout templates, generated mirrors, or validators of similar
size inside the materializer.

## Verification

Verification:

- `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core zellij_materialization`
- `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_zellij_config_pack`
- `yzx_repo_validator validate-workspace-session-contract` when workspace
  layouts or plugin placement are touched
- `shells/posix/yazelix_loc_scorecard.sh <base> HEAD` for deletion evidence

## Traceability

- Related boundary: `docs/contracts/status_bar_ownership.md`
- Related boundary: `docs/contracts/first_party_zellij_plugin_wasm_ownership.md`
- Related policy: `docs/child_repo_ownership_boundaries.md`
