# RuVector blueprint provenance ledger — Yazelix execution slice

This is the durable requirement-to-owner-to-proof ledger for the four
independent Yazelix repositories registered by Meta. It covers every blueprint
requirement that assigns implementation or runtime ownership to Yazelix,
Zellij, Helix, Yazi, terminal-support metadata, or the profile-carried operator
tools. System-wide requirements retain their canonical non-Yazelix owners in
the boundary ledger below; this repository does not claim to implement their
PostgreSQL, LifeOS, redb, CodeDB, or envctl responsibilities.

## Immutable authority receipt

| Field | Receipt |
| --- | --- |
| Owner-named path | `/home/flexnetos/Downloads/Architecture_Data_Pipeline_Blueprint_RUVECTOR_FULLY_EXPANDED_VERIFIED (1).md` |
| SHA-256 | `c54063110be8bebb07469cbc0f76fecab142cd636e98950a36a3ee02b766a62c` |
| Size and coverage | 974,321 bytes; 6,340 lines; read completely as lines 1–2200, 2201–4400, and 4401–6340 |
| Replacement authority | The owner-named duplicate pathname disappeared during execution. `/home/flexnetos/Downloads/Architecture_Data_Pipeline_Blueprint_RUVECTOR_FULLY_EXPANDED_VERIFIED.md` is byte-identical at the same SHA-256 and is the established read authority. The immutable LifeOS planning-spine copy is also recorded by GitKB. |
| Normative graph | `Architecture_Data_Pipeline_Graph_ANCHORED_VERIFIED.md`, SHA-256 `abd36f1c2bd9d62e4fdb522e5290d93d4e7017b1b478c13dbf0a5da939c5b663` |
| Durable context | GitKB `tasks/architecture-data-pipeline-blueprint`, immutable-source receipt and Yazelix crosswalk |

The applicable blueprint anchors are §§1–3.4, D03–D09, D21, D23, §§4.3,
4.8–4.9, the source register at lines 688–698, bootstrap/release steps at lines
5574–5587, the component register at lines 5733–5736 and 6070–6093, the release
contract at line 6262, evidence corrections R02/R04/R06/R07, and the closing
ownership summary at lines 6312–6318.

## Repository ownership

| Repository | Meta path | Single role | Relationship |
| --- | --- | --- | --- |
| `FlexNetOS/yazelix` | `/home/flexnetos/meta/src/yazelix` | Nix/profile owner; `yzx`; Zellij, Nu, editor/navigation composition; installed contracts | Consumes the other three as immutable flake/package inputs |
| `FlexNetOS/yazelix-helix` | `/home/flexnetos/meta/src/yazelix-helix` | Helix fork plus the local Yazelix action bridge | Standalone package; consumed with `nixpkgs.follows = "nixpkgs"` |
| `FlexNetOS/yazelix-terminal-support` | `/home/flexnetos/meta/src/yazelix-terminal-support` | Versioned terminal metadata only | Supplies schema-2 Mars metadata; never launches or renders a terminal |
| `FlexNetOS/yazelix-yazi-assets` | `/home/flexnetos/meta/src/yazelix-yazi-assets` | Reusable Yazi assets and packaged ccboard/CodeDB tools | Supplies one asset/package root to the main flake |

## Requirement and proof crosswalk

| ID | Blueprint or source requirement | Owner and files | Implementation/configuration | Verification evidence | Status |
| --- | --- | --- | --- | --- | --- |
| YZX-BP-001 | Nix-owned Yazelix Engine Room (§3.1, §4.3, D03, component row 6072) | `yazelix`: `flake.nix`, `runtime/yzx/`, Zellij defaults/layout | `lifeos_foundation_yzx` contains the one `yzx` frontdoor, pinned Zellij, Nushell, Helix, Yazi, Neovim, agents, and build tools | `checks.contracts`, `checks.runtime_contracts`, `checks.flexnetos_foundation_contracts`, package build | Implemented |
| YZX-BP-002 | Repository entrypoint is `yzx enter [zellij-args...]` and replaces itself with pinned Zellij (§3.1, §4.3, D04) | `yazelix`: `runtime/yzx/cli.rs`, `runtime/yzx/zellij.rs`, `checks/yzx-contracts.rs` | Exact argv forwarding, managed config/layout, `exec`, session attach/create flags | focused Rust contract plus `yzx enter --version` and installed `yzx doctor` | Implemented |
| YZX-BP-003 | Database/session-derived identities must fit the real PTY/session route without reimplementing terminal execution (D04, R02) | `yazelix` owns Engine endpoint; LifeOS owns PTY controller | `yzx enter` accepts and forwards all Zellij session options; terminal-support remains data-only | option-forwarding contract and terminal metadata architecture test | Implemented at the Engine boundary |
| YZX-BP-004 | Editable input, generated proof, and installed frontdoor have one owner | `yazelix`: paths/runtime contracts, profile tooling | editable `~/.config/yazelix`; generated under the profile runtime link `~/.nix-profile/runtime`; executable `/home/flexnetos/.nix-profile/bin/yzx` | single-profile contract, `yzx inspect --json`, command/type/symlink/profile receipts | Implemented |
| YZX-BP-005 | One standard profile desktop entry; terminal metadata must not become a launcher (R04, row 6073) | `yazelix` `flexnetosDesktopSource`; terminal-support TOML | profile `share/applications/com.flexnetos.Yazelix.Agent.desktop`; read-only `yzx desktop`; Mars-only metadata relation | desktop-file validation, installed TOML parse, exact `Exec`/WM class/count, no user-local write | Implemented |
| YZX-BP-006 | Runtime provenance must be inspectable without launching UI | `yazelix`: `runtime/yzx/inspect.rs`, `status.rs`, `doctor.rs` | human and schema-1 JSON receipts include invoked/resolved frontdoor, profile manifest, roots, local shadow, runtime and session | focused RED/GREEN contracts; installed `yzx inspect`, `status`, and `doctor` | Implemented |
| YZX-BP-007 | Helix remains an editor surface and its local action bridge is selectable (§4.3, row 6090) | `yazelix-helix`: `helix-term/src/yazelix_bridge.rs`; main `runtime.rs`, `yzx-open` | latest upstream merged into the fork; explicit short `YAZELIX_HELIX_BRIDGE_ROOT`; enabled and disabled paths; session/tab isolation | child bridge tests, grammar lock, Cargo check, Nix package build, main Helix/open/contracts | Implemented |
| YZX-BP-008 | Yazi remains the navigation surface; every documented plugin/config sidecar path is available (§4.3, row 6089) | child owns reusable auto-layout/Git/Starship assets; main owns sidebar-state/sidebar-status/zoxide-editor composition and materialization | one composed `yzxYaziConfig` is both the runtime input and installed share tree; all required plugins/config plus optional native TOML/Lua/keymap/theme/package layers are materialized | child package tests; installed share-tree contract; `yzx_yazi_materialization`; main config/open contracts | Implemented |
| YZX-BP-009 | ccboard and CodeDB described as optional runtime tools are required | `yazelix-yazi-assets` manifests/binaries; main executable map | both packages and `nu_plugin_codedb` are exposed through the foundation profile | installed executable/manifests and foundation contract | Implemented |
| YZX-BP-010 | Optional AI-agent surface and all supported provider selections are required | `yazelix`: `runtime/yzx-agent.rs`, config and contracts | `codex`, Grok, OpenCode, Pi, and Claude selection/fallback behavior; profile packages Codex and Claude | provider matrix tests and foundation executable checks | Implemented |
| YZX-BP-011 | Optional Home Manager module and every package/config ownership path are required | `yazelix`: `home-manager/module.nix`, flake fixtures | enable/default, package override, runtime package, and managed config-file paths | `checks.home_manager` plus all-system evaluation | Implemented |
| YZX-BP-012 | Optional sparse root, Mars, Starship, Nu/mise/user, Helix, Yazi, and Zellij sidecars must all work | `yazelix`: config defaults/materializers and `checks/` | every supported absent/present/override/reject path remains contract-driven | `checks.contracts`, sidecar parity, config crate tests, runtime materialization | Implemented |
| YZX-BP-013 | Neovim remains an alternate Engine Room editor (lines 45, 66, 596–598, row 6091) | `yazelix` foundation executable map | profile-owned `nvim`; host/editor selection bypasses the Helix bridge without losing the managed environment | `nvim --version`, editor-selection unit and integration tests | Implemented |
| YZX-BP-014 | Bun owns JavaScript/TypeScript package execution | `yazelix` foundation executable map | pinned Bun 1.3.14 and Bunx; npm, npx, pnpm, Yarn, and Corepack are not profile frontdoors | exact Bun version and profile absence contract | Implemented |
| YZX-BP-015 | RTK is pinned and native; `rtk_nu` supplies byte-exact pre-transform envelopes (§3.4, R06/R07, row 5733) | `FlexNetOS/rtk-tokenkill` latest develop consumed by `yazelix` | one source provides `rtk` 0.43.0 and merged `rtk_nu`; profile exports both; canonical Nu dispatcher remains packaged | RTK version/help/config, proxy operation, `rtk_nu` format fixtures, fresh-shell resolution | Implemented |
| YZX-BP-016 | GitKB is durable context, graph, task, provenance, and code intelligence (D21, row 6070) | Meta KB plus profile `git-kb` | exact four repos indexed; task/crosswalk current; document board/graph/search and deep Rust/Lua call graph exercised | `git-kb doctor`, `fsck`, `status`, board, graph, code stats/doctor/symbol/caller/impact operations | Implemented |
| YZX-BP-017 | ICM supplies persistent memory/retrieval surfaces (D21, row 6068) | profile ICM pinned to `03d63a9…`; `.codex/config.toml` MCP owner | recall/contextual recall, topics/list, health, store/update, decisions, resolved errors, progress and completion memories; service/MCP inventory | version/help/health plus real operations and fresh-shell resolution | Implemented |
| YZX-BP-018 | Beads task atoms and graph-aware triage are active (D21, row 6071) | main profile `br` and `bv`; `.beads` | `br` remains issue writer; `bv` 0.16.1 is package-owned and used only with robot flags | `br` version/doctor/show/sync and `bv --robot-triage`/plan/insights | Implemented |
| YZX-BP-019 | Durable Codex rules follow editable-input/generated-output/profile ownership (lines 1228–1229, 6312–6318) | `yazelix`: `agent_configs/codex`, Nu materializer, provenance tests | reviewed config/rules, transactional mode-0644 generated pair, exact profile selector, preserved user preferences/hooks | focused materializer, staged/live provenance, foundation installed-artifact contract | Implemented |
| YZX-BP-020 | Latest authoritative children and one nixpkgs universe | all four repositories and main `flake.lock` | exact FlexNetOS commit pins; child inputs follow main nixpkgs; no duplicate plugin/Yazi/terminal owner | lock inspection, child package builds, main flake show/check | Implemented |
| YZX-BP-021 | Old Yazelix remains recoverable but inactive; current Nova is installed | Meta archive owner plus main Git/Nix profile | archive manifest preserves Git/object/provenance; one profile generation selects latest merged Nova | archive checksum/manifest, Meta inventory, profile history and resolved store target | Completion gate |
| YZX-BP-022 | PR #77 has a definitive provenance-preserving disposition | `FlexNetOS/yazelix` GitHub history | exact semantic and attribution merged through PR #81; deleted fork-head provenance retained in pull ref and archive evidence | PR #81 merge `cd7379ff…`; [PR #77 resolution comment](https://github.com/FlexNetOS/yazelix/pull/77#issuecomment-5027850215); PR #77 closed as superseded on 2026-07-20 | Implemented |
| YZX-BP-023 | Release ends in clean Git/GitHub/profile state (D23, line 6262) | all four repositories, Meta/profile owner | reviewed PRs, green checks, merges, synchronized mains, branch/worktree cleanup, installed closure | per-repo status/open-PR checks, build/profile/runtime receipts | Completion gate |

## Formerly optional capability matrix

| Capability described as optional | Required paths exercised | Proof owner |
| --- | --- | --- |
| AI agent | auto selection plus Codex, Grok, OpenCode, Pi, and Claude fallbacks | main agent contracts |
| Home Manager | default full package, explicit override, Mars-free runtime, and config-file ownership | main Home Manager check |
| Sparse config and native sidecars | absent, valid present, merge/replace, reset, invalid/rejected, and Home Manager-owned paths | main config/runtime checks |
| Helix action bridge | disabled, enabled, explicit short root, default short root, session identity, stale registry, same-tab reuse, alternate-editor bypass | child bridge and main open/Helix contracts |
| Yazi native files/plugins | TOML, Lua, keymap, package, theme, Starship, auto-layout, Git, sidebar state/status, zoxide editor | child asset checks plus main composition/materialization contracts |
| ccboard and CodeDB tools | manifests, binaries, plugin, and profile resolution | child package plus foundation contract |
| Session/runtime switches | launch and enter; attach/create; named session; bridge session; full and Mars-free packages | main runtime contracts |
| Neovim alternate editor | profile binary and editor-command bypass path | foundation and open contracts |
| JavaScript package route | Bun and Bunx only | foundation ownership contract |

## Non-Yazelix owner boundary ledger

| Blueprint contract | Canonical owner | Relationship proven here |
| --- | --- | --- |
| Tauri/Svelte Glass, portable-pty 0.9.0, xterm addons, ordered PTY capture | `FlexNetOS/lifeos` and GitKB child task `tasks/lifeos-postgresql-durable-storage-cutover` | Yazelix supplies the real `yzx enter` endpoint and exact Zellij option forwarding; terminal-support is explicitly rejected as the bridge |
| Single-writer redb owner, mmap projection, event/replay/spool | redb/CodeDB/LifeOS owners in the architecture task graph | Yazelix remains a client/execution projection and opens no competing redb database |
| PostgreSQL/RuVector canonical state, envctl-only durable commit, byte-complete CodeDB ingress | PostgreSQL/RuVector, envctl, and CodeDB implementation tasks | Yazelix exposes controlled execution surfaces and does not claim durable truth ownership |
| `rtk_nu` adapter implementation | `FlexNetOS/rtk-tokenkill` PR #11, merged `e822928…` and retained by latest develop | main profile consumes and exercises the merged binary |
| Figma Glass design node | LifeOS design/release owner | no UI implementation is invented in the four Yazelix repositories |

## Evidence command classes

All shell execution is routed through
`/home/flexnetos/.nix-profile/bin/rtk`; raw commands use `rtk proxy`.

- Child repositories: Cargo format/check/test/clippy where applicable, grammar
  lock validation, Nix package builds, `nix flake check`, and all-system show.
- Main: focused Rust contracts, Yazi/Helix/runtime/Home Manager/single-profile/
  foundation checks, full flake show/check, package build, temporary-profile
  materialization, installed runtime `doctor` and `inspect`.
- Tools: exact command path, symlink target, version/help inventory, health,
  successful workspace operation, and a fresh shell with only the profile path.
- Finish state: per-repository `git status --short --branch`, SSH remote and
  identity proof, `gh pr list --state open`, merge/check state, profile inventory,
  frontdoor/desktop resolution, archive manifest, Beads sync, and ICM completion
  memory.
