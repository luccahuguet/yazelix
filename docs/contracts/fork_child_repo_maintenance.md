# Fork And Child-Repo Maintenance

## Summary

Yazelix uses first-party forks and child repositories only when they create a clearer product or artifact boundary than keeping the code in the main repository.

The main repository owns the integrated Yazelix runtime, the inventory tables below, release-transaction policy, and the shared maintenance rules. Child repositories own their local source, local README delta, package artifacts, and standalone behavior where the child is advertised as standalone.

Forks are reviewed periodically. They are not permanent by default unless their standalone product value remains stronger than returning to upstream.

## Categories

| Category | Meaning | README obligation |
| --- | --- | --- |
| Active fork | A fork with continuing standalone product value or a long-lived Yazelix-specific runtime need | Full fork delta section |
| Temporary fork | A fork that exists to bridge an upstream gap with a named removal gate | Full fork delta section with removal gate first-class |
| Experimental fork | A fork used for dogfooding or unstable product exploration | Full fork delta section with stability caveat and promotion gate |
| Ordinary child repo | A first-party repo that is not an upstream fork and owns a package, plugin, crate, or asset boundary | Child README explains the local package surface; no upstream-delta block required |
| Archived fork | A fork that is no longer selected by the current runtime | README points to archive reason and replacement; no active maintenance cadence |

## Fork Inventory

| Fork repo | Upstream project | Why forked | Current local delta | Status | Review cadence | Removal/upstreaming gate | Primary contract |
| --- | --- | --- | --- | --- | --- | --- | --- |
| [`yazelix-helix`](https://github.com/luccahuguet/yazelix-helix) | Helix Steel / Helix | Ship a standalone Steel-enabled Helix fork that can also satisfy Yazelix-managed config-directory launches | `--config-dir`, reusable Steel plugin defaults, and Yazelix bridge hooks behind explicit runtime flags | Active fork | Monthly or before Helix-sensitive releases | Upstream reusable pieces when accepted, but do not remove the fork while its standalone Steel/defaults value is higher than upstream plus main-repo adapters | [`helix_managed_config_contract.md`](./helix_managed_config_contract.md), [`helix_action_bridge_contract.md`](./helix_action_bridge_contract.md) |
| [`yazelix-terminal`](https://github.com/luccahuguet/yazelix-terminal) | Rio | Dogfood a first-party terminal path with Yazelix-controlled package metadata and profile/shader boundaries | yzxterm package profile, wrapper commands, metadata passthru, profile templates, and shader ABI | Experimental active fork | Monthly while dogfooding, and before yzxterm runtime release evidence | Graduate to stable only after release-profile validation and real user value; upstream or delete local terminal deltas that become Rio-owned | [`yzxterm_package_boundary.md`](./yzxterm_package_boundary.md), [`yzxterm_fast_dogfooding.md`](../yzxterm_fast_dogfooding.md) |
| [`yazelix-zellij`](https://github.com/luccahuguet/yazelix-zellij) | Zellij | Restore Yazi image previews through Kitty graphics passthrough in the default Ghostty runtime | Temporary KGP preview branch selected by Yazelix package outputs | Temporary fork | Monthly and whenever upstream Zellij changes Kitty graphics behavior | Drop and archive once upstream Zellij supports the required Kitty graphics path directly enough for Yazelix to return to upstream packages | [`v15_trimmed_runtime_contract.md`](./v15_trimmed_runtime_contract.md) |
| Yazelix Yazi graphics fork | Yazi | Pair with the temporary Zellij graphics path so managed Yazi previews work in the default runtime | Temporary Yazi-side graphics integration branch selected by Yazelix package outputs | Temporary fork | Monthly and whenever upstream Yazi/Zellij graphics integration changes | Drop and archive with `yazelix-zellij` when upstream support is sufficient for the managed preview path | [`yazi_integration_boundary.md`](./yazi_integration_boundary.md) |

## Child Repo Inventory

| Child repo | Owned artifact | Consumed as | Runtime role | Update responsibility | User-visible |
| --- | --- | --- | --- | --- | --- |
| [`yazelix-screen`](https://github.com/luccahuguet/yazelix-screen) | `yzs` terminal animation engine and Rust crate | Flake input `yazelixScreen`, package `#yazelix_screen`, app `#yzs`, Cargo git dependency | Welcome/screen rendering and standalone screen playback | Child owns animation engine and package; main repo owns integrated welcome/session policy | Yes, standalone |
| [`yazelix-cursors`](https://github.com/luccahuguet/yazelix-cursors) | `yzc` CLI, cursor registry, generated shader assets, cursor Rust crate | Flake input `yazelixCursors`, package `#yazelix_cursors`, app `#yzc`, Cargo git dependency | Cursor settings, Ghostty shader materialization, yzxterm cursor shader inputs | Child owns cursor schemes and shader generation; main repo owns per-window/runtime integration | Yes, standalone |
| [`yazelix-zellij-bar`](https://github.com/luccahuguet/yazelix-zellij-bar) | Standalone bar preset, `zjstatus.wasm`, `yazelix_zellij_bar_widget` | Flake input `yazelixZellijBar`, package `#yazelix_zellij_bar` | Integrated status-bar KDL rendering and standalone Zellij bar package | Child owns widget rendering and package-local wasm; main repo owns session-specific adapter paths | Yes, standalone |
| [`yazelix-zellij-pane-orchestrator`](https://github.com/luccahuguet/yazelix-zellij-pane-orchestrator) | `yazelix_zellij_pane_orchestrator.wasm` | Flake input `yazelixZellijPaneOrchestrator`, package `#yazelix_zellij_pane_orchestrator` | Managed pane identity, focus, editor/sidebar handoff, layout-family commands | Child owns Zellij plugin source/API; main repo owns generated layouts and runtime packaging | Indirectly; advanced standalone plugin commands |
| [`yazelix-zellij-popup`](https://github.com/luccahuguet/yazelix-zellij-popup) | `yzpp.wasm` and KDL-native popup plugin | Flake input `yazelixZellijPopup`, package `#yazelix_zellij_popup` | Popup panes, command menu, config UI floating panes | Child owns generic popup lifecycle; main repo owns generated popup specs and close hooks | Yes, standalone for Zellij users |
| [`yazelix-yazi-assets`](https://github.com/luccahuguet/yazelix-yazi-assets) | Yazi flavors, reusable plugins, Starship config | Flake input `yazelixYaziAssets`, package `#yazelix_yazi_assets` | Managed Yazi flavors/plugins and reusable asset pack | Child owns reusable assets and vendored plugin refreshes; main repo owns managed sidebar/editor integration | Yes, asset package |
| [`yazelix-ratconfig`](https://github.com/luccahuguet/yazelix-ratconfig) | Reusable Ratatui config editor crate | Cargo git dependency and Nix Cargo dependency metadata | `yzx config ui` generic editor core, JSONC/TOML adapter primitives, migration primitives | Child owns generic config UI/editing primitives; main repo owns Yazelix settings schema, validation, and apply policy | Yes, crate/API |

## Fork README Delta Protocol

Each Yazelix-maintained fork README must start with a short Yazelix-owned section before inherited upstream README content. The section must be local to that child repo and must not copy this whole contract.

Required fields:

- Upstream project and fork base
- Fork category: active, temporary, experimental, or archived
- Why the fork exists
- Current Yazelix-owned local delta
- What the fork intentionally does not own
- Standalone support level
- Upstream sync cadence
- Removal, upstreaming, or promotion gate
- Links to the main maintenance contract and any fork-specific contract

Active forks should make their standalone value obvious. Temporary forks should make the removal gate obvious. Experimental forks should make stability and promotion criteria obvious. Archived forks should point users to the replacement path.

Ordinary child repos do not need an upstream fork delta block, but their README should still name the standalone artifact, supported install surface, and what the main Yazelix repo owns instead.

Shared agent workflow belongs in the main repository `AGENTS.md`. A child repo `AGENTS.md` should point back to the main instructions and keep only child-specific guidance.

## README Delta Audit

| Repo | Current protocol state | Follow-up |
| --- | --- | --- |
| `yazelix-helix` | Partial: the README says the fork is thin, standalone-usable, and points to `YAZELIX.md`, but it does not yet expose the full structured delta/cadence/gate block | Direct child README update required |
| `yazelix-terminal` | Needs structured experimental fork delta block | Direct child README update required |
| `yazelix-zellij` | Needs structured temporary fork delta block | Direct child README update required |
| Yazelix Yazi graphics fork | Needs structured temporary fork delta block | Direct child README update required |

## Review Evidence

A fork-maintenance review is complete when the review record includes:

- upstream commit and PR scan scope
- local delta scan scope
- update, defer, upstream, remove, or archive decision
- package/lock impact
- follow-up work item for any direct repo edit, upstream PR, lock update, or contract change

Ordinary child-repo lock bumps follow the cross-repo release transaction rule in `AGENTS.md`: publish the child commit first, update the main lock to the published revision, validate without overrides, then complete the review record before pushing the main repo after required manual testing.

## Verification

- `yzx_repo_validator validate-contracts`
