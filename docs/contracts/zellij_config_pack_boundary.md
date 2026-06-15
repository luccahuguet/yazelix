# Zellij Config Pack Boundary

## Summary

`yazelix-zellij-config-pack` is the child repository for deterministic Zellij
config and layout rendering. Main Yazelix consumes the published child crate and
flake package, then writes the rendered output into the active state directory.

The child owns pure rendering. Main Yazelix keeps product policy, runtime
integration, filesystem materialization, plugin artifact resolution, doctor and
repair behavior, and live workspace/session behavior.

## Ownership

| Surface | Owner |
| --- | --- |
| Settings normalization and defaults | Main Yazelix |
| Runtime identity, package paths, and generated state roots | Main Yazelix |
| Zellij action registry policy and supported keybinding semantics | Main Yazelix |
| Pane-orchestrator, popup, and status-bar package/artifact resolution | Main Yazelix plus existing child repos |
| Live tab, pane, workspace, status-bus, and session facts | Pane orchestrator plus Main Yazelix adapters |
| Filesystem writes, repair decisions, doctor checks, and permission seeding | Main Yazelix |
| Layout templates and layout-family metadata that are not product policy | `yazelix-zellij-config-pack` |
| Deterministic KDL config/layout rendering and merge mechanics | `yazelix-zellij-config-pack` |
| Config-pack fixture and equivalence tests | `yazelix-zellij-config-pack`, with main keeping integration tests |

## Boundary

The child boundary must be a pure function:

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

The child renderer must not read or infer:

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

Main may consume a config-pack revision only when all gates pass:

1. The child renderer accepts explicit structured input and produces
   deterministic config/layout output
2. Generated Zellij output has equivalence tests for representative
   default, custom popup, native merge, status-bar, and layout-fragment cases
3. The renderer has no hidden filesystem, environment, runtime, Home Manager,
   live Zellij, or adjacent-checkout reads
4. Status-bar rendering is consumed from `yazelix_zellij_bar`; popup and
   pane-orchestrator artifacts are consumed from their existing child owners
5. Main keeps only policy, path resolution, artifact resolution, writes,
   doctor/repair, and integration validation
6. Main deletes Rust/assets/tests/metadata ownership instead of adding wrappers,
   mirrors, or fallback copies
7. Local override validation is treated only as a smoke test; main closes the
   integration only after consuming a published child revision without overrides

## Deletion Bar

Consumption must remove real main ownership. A valid child update should keep
main from owning these groups:

- pure KDL rendering and merge mechanics
- layout template rendering that does not encode main product policy
- config-pack fixture tests that can run without a Yazelix checkout
- renderer-owned layout-family metadata

It is not a valid child consumption if main keeps parallel renderer code,
fallback config/layout templates, generated mirrors, or validators of similar
size to the code moved out.

## Verification

Main verification:

- `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core zellij_materialization`
- `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core zellij_render_plan`
- `yzx_repo_validator validate-workspace-session-contract` when workspace
  layouts or plugin placement are touched
- `shells/posix/yazelix_loc_scorecard.sh <base> HEAD` for deletion evidence

Child release transaction verification:

- child repo checks pass locally and in CI
- main consumes a published child revision through `flake.lock`
- main validation passes without local overrides
- main scorecard records deleted ownership in the release transaction

## Traceability

- Related boundary: `docs/contracts/status_bar_ownership.md`
- Related boundary: `docs/contracts/first_party_zellij_plugin_wasm_ownership.md`
- Related policy: `docs/child_repo_ownership_boundaries.md`
