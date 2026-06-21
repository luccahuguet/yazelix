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
| [`yazelix-zellij`](https://github.com/luccahuguet/yazelix-zellij) | Zellij | Restore Yazi image previews through Kitty graphics passthrough in the default Ghostty runtime | Temporary KGP preview branch selected by Yazelix package outputs | Temporary fork | Monthly and whenever upstream Zellij changes Kitty graphics behavior | Drop and archive once upstream Zellij supports the required Kitty graphics path directly enough for Yazelix to return to upstream packages | [`v15_trimmed_runtime_contract.md`](./v15_trimmed_runtime_contract.md) |
| Yazelix Yazi graphics fork | Yazi | Pair with the temporary Zellij graphics path so managed Yazi previews work in the default runtime | Temporary Yazi-side graphics integration branch selected by Yazelix package outputs | Temporary fork | Monthly and whenever upstream Yazi/Zellij graphics integration changes | Drop and archive with `yazelix-zellij` when upstream support is sufficient for the managed preview path | [`yazi_integration_boundary.md`](./yazi_integration_boundary.md) |

## Upstream Review Cadence

Fork reviews are maintainer-owned and evidence-driven. The default cadence is monthly, but release-sensitive forks should also be reviewed before a Yazelix release that changes their runtime behavior, and quiet forks may defer a month when the review evidence says there is no user-value, security, build, or fork-delta reason to update.

Minimum review checklist:

- inspect upstream commits, releases, and relevant PRs since the last reviewed point
- inspect the Yazelix local delta and whether any patches can be upstreamed, deleted, or narrowed
- decide update, defer, upstream, remove, or archive
- record package or lock impact
- create follow-up work only for concrete repo edits, upstream PRs, lock updates, or contract changes

| Fork repo | Upstream source | Owner | Cadence | Review evidence location |
| --- | --- | --- | --- | --- |
| `yazelix-helix` | `helix-editor/helix` plus the active Helix Steel branch used by the fork | Yazelix maintainer | Monthly or before Helix-sensitive Yazelix releases | Maintainer review record or child README/`YAZELIX.md` update when the fork delta changes |
| `yazelix-zellij` | `zellij-org/zellij` | Yazelix maintainer | Monthly and whenever upstream Zellij changes Kitty graphics behavior | Maintainer review record and child README update when the temporary graphics delta changes |
| Yazi graphics fork | `sxyazi/yazi` | Yazelix maintainer | Monthly with `yazelix-zellij`, and whenever upstream Yazi preview behavior changes | Maintainer review record and child README update when the temporary graphics delta changes |

## Child Repo Inventory

| Child repo | Owned artifact | Consumed as | Runtime role | Update responsibility | User-visible |
| --- | --- | --- | --- | --- | --- |
| [`yazelix-screen`](https://github.com/luccahuguet/yazelix-screen) | `yzs` terminal animation engine and Rust crate | Flake input `yazelixScreen`, package `#yazelix_screen`, app `#yzs`, Cargo git dependency | Welcome/screen rendering and standalone screen playback | Child owns animation engine and package; main repo owns integrated welcome/session policy | Yes, standalone |
| [`yazelix-cursors`](https://github.com/luccahuguet/yazelix-cursors) | `yzc` CLI, cursor registry, generated shader assets, cursor Rust crate | Flake input `yazelixCursors`, package `#yazelix_cursors`, app `#yzc`, Cargo git dependency | Cursor settings and Ghostty shader materialization | Child owns cursor schemes and shader generation; main repo owns per-window/runtime integration | Yes, standalone |
| [`yazelix-zellij-bar`](https://github.com/luccahuguet/yazelix-zellij-bar) | Standalone bar preset, `zjstatus.wasm`, `yazelix_zellij_bar_widget` | Flake input `yazelixZellijBar`, package `#yazelix_zellij_bar` | Integrated status-bar KDL rendering and standalone Zellij bar package | Child owns widget rendering and package-local wasm; main repo owns session-specific adapter paths | Yes, standalone |
| [`yazelix-zellij-pane-orchestrator`](https://github.com/luccahuguet/yazelix-zellij-pane-orchestrator) | `yazelix_zellij_pane_orchestrator.wasm` | Flake input `yazelixZellijPaneOrchestrator`, package `#yazelix_zellij_pane_orchestrator` | Managed pane identity, focus, editor/sidebar handoff, layout-family commands | Child owns Zellij plugin source/API; main repo owns generated layouts and runtime packaging | Indirectly; advanced standalone plugin commands |
| [`yazelix-zellij-popup`](https://github.com/luccahuguet/yazelix-zellij-popup) | `yzpp.wasm` and KDL-native popup plugin | Flake input `yazelixZellijPopup`, package `#yazelix_zellij_popup` | Popup panes, command menu, config UI floating panes | Child owns generic popup lifecycle; main repo owns generated popup specs and close hooks | Yes, standalone for Zellij users |
| [`yazelix-yazi-assets`](https://github.com/luccahuguet/yazelix-yazi-assets) | Yazi flavors, reusable plugins, Starship config | Flake input `yazelixYaziAssets`, package `#yazelix_yazi_assets` | Managed Yazi flavors/plugins and reusable asset pack | Child owns reusable assets and vendored plugin refreshes; main repo owns managed sidebar/editor integration | Yes, asset package |
| [`ratconfig`](https://github.com/luccahuguet/ratconfig) | Reusable Ratatui config editor crate | Cargo git dependency and Nix Cargo dependency metadata | `yzx config ui` generic editor core, JSONC/TOML adapter primitives, migration primitives | Child owns generic config UI/editing primitives; main repo owns Yazelix settings schema, validation, and apply policy | Yes, crate/API |

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

## Child AGENTS.md Pointer Policy

The main Yazelix `AGENTS.md` is the canonical shared policy source for agent workflow, issue-tracker usage, verification philosophy, command-surface policy, and cross-repo release transactions.

Every active fork and ordinary child repo should carry a short local `AGENTS.md` that:

- links to `https://github.com/luccahuguet/yazelix/blob/main/AGENTS.md`
- tells agents to read `../yazelix/AGENTS.md` first when working from sibling local checkouts
- keeps only child-specific scope, build commands, artifact names, upstream-sync notes, release gates, or caveats
- avoids copying long shared main-policy sections that would drift, such as issue-tracker workflow, verification requirements, command-surface policy, or cross-repo release transactions

If a child repo has no local exceptions, its `AGENTS.md` should contain only the pointer and a short statement that the main policy applies.

## README Delta Audit

| Repo | Current protocol state | Follow-up |
| --- | --- | --- |
| `yazelix-helix` | Structured active-fork delta block present in the child README | Keep current during monthly fork reviews |
| `yazelix-zellij` | Structured temporary-fork delta block present in the child README | Keep current until the upstream-removal gate closes |
| Yazelix Yazi graphics fork | Structured temporary-fork delta block present in the child README | Keep current with the paired `yazelix-zellij` fork |

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
