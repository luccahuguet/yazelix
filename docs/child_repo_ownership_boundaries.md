# Child Repo Ownership Boundary Audit

## Summary

This is an evaluation record for the current first-party child repositories consumed by Yazelix. It is not a contract and does not implement repository moves.

The current child-repo shape is justified overall. No repository should be merged back into the main repo now. The best next pressure is boundary discipline: keep standalone child surfaces genuinely runnable outside Yazelix, keep Yazelix-only adapters in the main repo, and treat every main-repo lock update that consumes child changes as a release transaction.

The pane orchestrator is the highest-risk boundary because it owns real workspace behavior and contains the most Yazelix-specific integration. It should remain separate because the runtime ABI is a Zellij plugin wasm artifact, but it needs the strictest standalone/API discipline.

## Inputs Reviewed

- `flake.nix` child inputs and package forwarding
- `packaging/mk_runtime_tree.nix` runtime artifact wiring
- `packaging/rust_core_helper.nix` Cargo output hashes for linked child crates
- `docs/yazelix_collection.md`
- `docs/contracts/artifact_first_child_integration.md`
- `docs/contracts/first_party_zellij_plugin_wasm_ownership.md`
- `docs/contracts/standalone_yazelix_screen_distribution.md`
- `docs/contracts/standalone_cursor_distribution.md`
- `docs/contracts/standalone_yazelix_zellij_bar_distribution.md`
- `docs/contracts/status_bar_ownership.md`
- `docs/contracts/floating_tui_panes.md`
- `docs/contracts/yazelix_zellij_pane_orchestrator_extraction.md`
- `docs/contracts/yazi_integration_boundary.md`
- Adjacent checkouts for `yazelix-screen`, `yazelix-cursors`, `yazelix-terminal`, `yazelix-ratconfig`, `yazelix-zellij-bar`, `yazelix-zellij-pane-orchestrator`, `yazelix-zellij-popup`, and `yazelix-yazi-assets`

## Scoring

Scores use `1..5`, where `5` is the healthier result for a separate child repository.

- Standalone value: how useful the child is without installing Yazelix.
- Low coupling: how little the child needs Yazelix runtime, config, session, or wrapper state.
- Artifact clarity: how clear the package output and consumption seam are.
- Low duplicate risk: how unlikely the split is to create duplicate implementation owners.
- Low release friction: how easy the child is to publish, consume, and validate without unpublished commits.
- Local testability: how directly a maintainer can validate the child and the integrated main runtime.

## Decision Matrix

| Child repo | Standalone value | Low coupling | Artifact clarity | Low duplicate risk | Low release friction | Local testability | Recommendation |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `yazelix-screen` | 4 | 5 | 4 | 4 | 3 | 5 | Keep separate |
| `yazelix-cursors` | 5 | 4 | 4 | 4 | 3 | 5 | Keep separate |
| `yazelix-terminal` | 4 | 3 | 4 | 4 | 2 | 4 | Keep separate while experimental |
| `yazelix-ratconfig` | 4 | 4 | 3 | 5 | 3 | 5 | Keep separate with Yazelix adapter discipline |
| `yazelix-zellij-bar` | 4 | 3 | 4 | 4 | 3 | 4 | Keep separate with adapter discipline |
| `yazelix-zellij-pane-orchestrator` | 3 | 2 | 5 | 3 | 2 | 4 | Keep separate, revise boundary discipline |
| `yazelix-zellij-popup` | 5 | 5 | 5 | 5 | 4 | 4 | Keep separate |
| `yazelix-yazi-assets` | 4 | 5 | 5 | 5 | 4 | 5 | Keep separate |

## Per-Repo Evaluation

### `yazelix-screen`

Recommendation: keep separate.

The screen repo has a real standalone product surface: the `yzs` command, terminal animation engines, examples, and a Nix package. The main repo consumes the same Rust crate for integrated welcome and `yzx screen` behavior, but the child does not need Yazelix config, Zellij, Home Manager, or session state.

The main cost is release friction because Yazelix consumes it through both a flake input and a Cargo git dependency with a package hash. That is acceptable because the implementation is small, strongly testable, and reusable outside the workspace.

Boundary rule: animation engines, automata, generation logic, random animation-family policy, terminal frame primitives, and generated screen assets stay in the child. Yazelix-specific welcome copy, settings, skip behavior, startup logging, package linking, and session integration stay in the main repo.

### `yazelix-cursors`

Recommendation: keep separate.

This is one of the strongest standalone boundaries. It owns a real cursor workflow through `yzc init`, `yzc generate ghostty`, generated shader assets, standalone JSONC settings, and examples. Yazelix consumes the same registry and shader logic for config UI, settings rendering, Ghostty materialization, Yazelix Terminal shader assets, and `yzx cursors`.

The main risks are dual consumption through flake and Cargo, plus the fact that cursor facts also feed the status bar. Those are manageable because the ownership line is clear: cursor schemes, shader generation, standalone cursor config, and `yzc` belong to the child; Yazelix owns per-window randomization, integrated terminal materialization, and config UI composition.

Boundary rule: do not broaden this repo into generic terminal config. Keep it cursor-preset and shader-output owned; terminal launch, windowing, and config materialization stay in the terminal-specific owners.

### `yazelix-ratconfig`

Recommendation: keep separate with Yazelix adapter discipline.

This repo owns the reusable Ratatui config editor core: project-agnostic model types, navigation/edit state, generic rendering, comment-preserving JSONC patching, and deterministic migration primitives. That gives other projects a useful crate without installing Yazelix, and it deletes the old duplicate reusable implementation from the main repo.

The coupling score is not perfect because Yazelix still has a rich adapter: settings schema metadata, Home Manager read-only state, native config status, keybinding registry details, validation, file writes, and runtime apply modes all remain product-specific. That is the right split. Moving those into the child would turn a reusable editor crate into a hidden Yazelix runtime dependency.

Boundary rule: generic config UI mechanics and JSONC/migration primitives stay in the child. Yazelix settings schema, Home Manager/native status, keybinding action metadata, generated runtime refresh, and post-save apply behavior stay in the main repo.

### `yazelix-zellij-bar`

Recommendation: keep separate with adapter discipline.

The bar repo is justified because it owns standalone Zellij/zjstatus preset packaging, package-local `zjstatus.wasm`, the `yazelix_zellij_bar_widget` command, non-workspace widget rendering, provider cache behavior, and integrated runtime KDL template rendering. That is enough standalone value for plain Zellij users and enough duplicated-owner reduction for Yazelix.

The coupling score is lower than screen/cursors because integrated Yazelix layout materialization calls `yazelix_zellij_bar_widget render-yazelix-runtime` and supplies runtime paths, widget settings, and session-specific cache locations. That is still a healthy artifact/command boundary as long as the main repo remains an adapter and does not link the child crate or reimplement widget internals.

Boundary rule: generic bar rendering, non-workspace widget commands, cache/backoff, and runtime KDL template rendering stay in the child. Workspace facts, session cache path selection, layout insertion, and pane-orchestrator status payloads stay in Yazelix.

### `yazelix-zellij-pane-orchestrator`

Recommendation: keep separate, revise boundary discipline.

This is the riskiest child boundary, but still worth keeping separate. The runtime form is a Zellij plugin wasm artifact, and the child repo owns the correct build target, package artifact, Zellij plugin API, and standalone command subset. Merging it back would make the main repo own wasm plugin source and increase runtime/package complexity.

The risk is that the plugin is also Yazelix's live workspace brain: editor/sidebar identity, workspace retargeting, sidebar state, screen saver launch, status-cache facts, and runtime-config reload behavior. That makes it easy for "standalone plugin" to become a label over a Yazelix-only plugin.

Boundary rule: keep the plugin separate, but enforce a public API split between standalone Zellij commands and Yazelix integration commands. Standalone behavior must remain testable without `YAZELIX_RUNTIME_DIR`, `YAZELIX_SESSION_CONFIG_PATH`, `yzx_control`, or Yazelix-managed config paths. Yazelix-only commands should be explicit integrations, not hidden requirements for core focus/sidebar behavior.

### `yazelix-zellij-popup`

Recommendation: keep separate.

The popup repo is the cleanest first-party Zellij plugin boundary. It owns one generic behavior: configured floating TUI popups with stable pane identity and toggle/focus/close semantics. Plain Zellij users can configure it directly with KDL, while Yazelix consumes the wasm artifact and generates popup/menu/config specs.

The main repo's role is correctly narrow: package `yzpp.wasm`, generate integrated specs, choose runtime commands, and add Yazelix-specific close hooks such as sidebar refresh. The child stays useful without Yazelix and does not need to know about `yzx`, config UI, or the command palette.

Boundary rule: configured popup lifecycle belongs in `yzpp`. Yazelix owns generated popup specs, command selection, semantic bindings, and integration hooks.

### `yazelix-yazi-assets`

Recommendation: keep separate.

The asset repo is a strong data/package boundary. It owns reusable Yazi flavors, reusable upstream or Yazelix-maintained plugins, the Starship config used by Yazi, vendored plugin metadata, and a package shape with install checks. The main repo now keeps the Yazelix-specific sidebar/editor plugins and materialization logic, which is the right split.

This repo should not grow into full Yazi integration unless the main repo adapter gets much thinner. A full Yazi integration child would need to own managed roots, output paths, editor opener preservation, semantic keybindings, pane-orchestrator registration, and legacy guardrails. Those are still Yazelix product behavior.

Boundary rule: reusable flavors and reusable plugins stay in the child. Sidebar state, zoxide-to-editor, managed editor opener preservation, semantic keymap expansion, and generated output ownership stay in Yazelix.

## Cross-Repo Risks

### Published Artifact Risk

The main risk is using a child commit locally before it is pushed and then updating the main lock to a revision other machines cannot fetch. The AGENTS.md cross-repo release transaction rule is correct: push the child first, update the main lock to the published GitHub revision, validate without overrides, then close beads and push main.

This risk is highest for `yazelix-zellij-pane-orchestrator` and `yazelix-zellij-popup`, because missing or unpublished wasm artifacts break runtime packaging directly. Before landing any main lock update that consumes those packages, run `yzx_repo_validator validate-child-release-transaction`; it instantiates their `aarch64-darwin` package derivations and rejects package shapes where `cargoBuildHook` can run before the Fenix `wasm32-wasip1` toolchain is exported.

### Dual-Pin Rust Crate Risk

`yazelix-screen` and `yazelix-cursors` are consumed as Rust git dependencies and flake inputs. That means a release can involve Cargo lock updates, Nix output hashes, and flake lock updates. This friction is acceptable because both have real standalone value, but it should stay explicit in review.

For `yazelix-screen`, `validate-child-release-transaction` is also the main Darwin smoke gate after a lock update: it rejects `aarch64-darwin` screen package derivations that reintroduce package-time ImageMagick or expanded magician frame generation.

### Boundary Creep Risk

The main repo should not create local mirrors of child behavior to "smooth" release friction. That would defeat the extraction. When a child boundary is painful, either improve the child artifact/API or deliberately revise the boundary. Do not add fallback copies, generated mirrors, or local-only source assumptions.

## Final Recommendation

Keep the current child repository set separate.

Do not merge any current child repo back now.

Do not create a new child repo for Yazi integration, workspace state, or runtime control until the main repo owner has first become a thin adapter with a concrete artifact/API seam. `yazelix-ratconfig` is the accepted config UI exception because the reusable editor/JSONC owner has moved out and Yazelix now keeps only product-specific adapter behavior.

Use this priority order for future boundary pressure:

1. Keep `yazelix-zellij-pane-orchestrator` honest about standalone versus Yazelix-only commands.
2. Keep `yazelix-ratconfig` as a reusable config editor crate and prevent Yazelix schema/apply policy from leaking into it.
3. Keep `yazelix-zellij-bar` as a command/artifact boundary and prevent widget implementation from returning to the main repo.
4. Keep `yazelix-yazi-assets` asset-only unless a real config-pack API emerges.
5. Treat `yazelix-screen`, `yazelix-cursors`, and Rust git child crates as release transactions, not local cleanup.
6. Keep `yazelix-zellij-popup` narrow and generic; it is the model child boundary.
