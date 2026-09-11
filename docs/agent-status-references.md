# Agent-status references

This file owns Nova's agent-status reference and decision evidence. It records
the accepted `yazelix-nova-agent-command-center-srmq-2hd.1` decision and its
implementation boundaries. The original sources were rechecked on **2026-09-07**;
the zj-agent-mob reference was reviewed on **2026-09-11**. Commit links identify
reviewed snapshots, not promises about later releases.

## Accepted model

**Keep the narrow Yazelix Radar fork as the sole activity and attention rail.**
Providers report through explicit hooks or plugins. Those integrations are
optional for launching an agent and necessary for supported detailed activity
reporting. Codex without enabled, trusted hooks has no equivalent hookless
status fallback. Ordinary pane/command information does not prove agent activity.

The current [flake lock](../flake.lock) selects [Yazelix Radar `1be7d734`][radar-fork].
The comparison baseline is [upstream Radar v0.6.0 `0b167260`][radar-upstream].
They are different revisions. The fork retains the producer protocol and owns
Nova's rail behavior, including its cadence and long-running marker.

[`yzx radar-setup`](../runtime/yzx/cli.rs) delegates to the packaged
`zj-radar setup codex claude opencode`, without `--yes`. Radar owns provider
detection, sequential consent, configuration changes and failure reporting.
The command appears in the [Alt Shift M palette](../runtime/yzx-menu.rs).
The [Codex first-launch offer](../runtime/yzx-agent.rs) remains Codex-specific;
declining it is remembered. The explicit command works after that decline.
Codex retains hook trust; Nova's [doctor](../runtime/yzx/doctor.rs) is read-only.
See [Runtime Notes](runtime-notes.md#agent-popup) for operating guidance.

Ownership follows [Architecture](../ARCHITECTURE.md): Rio owns the terminal
frontend; Zellij owns multiplexer sessions, panes and terminal processes;
Radar owns activity and attention navigation; the pane orchestrator owns tab
workspace/focus/visibility policy; Nova Bar/zjstatus owns tab/widget rendering.
Nova composes these children. Git owns worktree truth, Beads owns durable task
planning, and each agent owns its conversations. Existing Zellij bells and Radar
presentation remain notification surfaces; this decision selects no desktop
notification service or additional session manager.

## Evidence states

- **Proposed:** an accepted constraint for possible work, without implementation proof.
- **Source-proven:** the cited revision contains the mechanism; no runtime claim follows.
- **Mechanically proven:** a recorded command or scripted interaction exercised a named artifact and environment.
- **Dogfooded:** a person exercised the interaction and reported the result. Agent-operated PTY checks remain mechanical evidence.

## Comparison by product layer

The first five candidates came from the [Zellij discussion][discussion]. The
remaining rows retain the strongest mechanism references from 2hd.1. Every row
is **source-proven** only, except Radar's separately recorded Nova proofs below.
Linux/macOS entries describe source/package fit, not Nova installation tests.
Cost and disposition are Nova ownership judgments. No candidate code was copied.

| Layer / reference | Ownership | Status source | Onboarding | Permissions / data | Linux / macOS fit | Cost / Nova disposition |
| --- | --- | --- | --- | --- | --- | --- |
| Runtime: [Herdr `94f6d9c0`][herdr] | Terminal server, sessions, agent panes | Agent detection in its owned terminals | Run sessions inside Herdr | Owns terminal processes and output | Unix runtime; no Nova platform proof | Apache-2.0; replaces the runtime/session boundary. Reference only. |
| Rail: [zj-radar `0b167260`][radar-upstream] | Activity, attention, rail | Versioned push pipe from provider producers | Explicit provider setup | App state, CLI pipes, app changes, commands; event payloads can contain task/message text | Nix CLI/WASM; Linux and macOS paths | MIT; retain the existing narrow fork and one owner. |
| Popup/controller: [zj-agent-mob `0b2d4fc`][agent-mob] | Cross-session agent monitor, navigation and control | Claude/Codex hooks, per-agent spool files and optional process discovery | Installer edits agent hook configs; agents must restart | App state and commands; task/detail data in pipes and user-private temporary files; optional prompt approval, follow-up and injected peer context | Zellij 0.44+ WASM with Linux/macOS shell seams; no Nova platform proof | Apache-2.0; its one movable session instance and status-bar summary are useful lifecycle references. Its broader control and prompt ownership overlap Nova; reference only. |
| Passive detector: [zj-agents `39795f26`][zj-agents] | Background classifier plus sidebar | Roughly one-second pane-text and running-command inspection | Install two WASM plugins and grant permissions | `ReadPaneContents`, app state, commands, plugin messaging | Stock Zellij ≥0.44.3; no Nova platform proof | MIT OR Apache-2.0; extra tracker and UI parsing. Conditional evaluation only. |
| Dashboard: [Captain Miao `07e4a478`][miao] | Managed launches, profiles, local/remote sessions | Hooks and agent session/transcript data | Managed agent profiles/configuration | Codex profile contains precomputed hook trust; transcript previews | Linux/macOS; backend-specific limits | MIT; reserves configuration and session ownership. Reject dependency. |
| Rail/bar: [Zellaude `1d1d56c6`][zellaude] | Tab bar, Claude state, notifications | Claude hooks | First load registers hooks automatically | Writes Claude settings; host commands; optional notifications | Zellij WASM; notification path is macOS-specific | MIT; duplicates the bar and does not solve Codex. Reject dependency. |
| Dashboard: [Fleet `30450f09`][fleet] | tmux dashboard and status fusion | Hook records plus process, title and screen inference | Discovery with optional installed hooks | tmux output, process scans, local event files | Linux/macOS paths; tmux boundary | MIT; useful precedence reference, additional scanner/dashboard rejected. |
| Orchestrator: [RimZ `cb8c0dce`][rimz] | Control room, workspace, durable state, sidebar | Provider hooks, transcripts, APIs and process stats | Consented first-run hook installation | Agent stores/APIs and multiplexer control | Zellij/tmux; Linux/macOS installation paths | MIT; adapter reference, overlapping owners prevent adoption. |
| Runtime: [Rook `7cb77a38`][rook] | Terminal and sessions | Foreground-process identity separate from reported state/title markers | Explicit status-hook install menu | Process identity, terminal titles, reported payloads | Native macOS 14+; arm64 releases | MIT; useful identity/state separation, outside Nova's runtime boundary. |
| Producer: [gentle-agent-state `963e8537`][gentle] | Cross-agent state and terminal decorations | Claude/Codex hooks; OpenCode/Pi plugins | Install adapters, restart agents | Agent config, local state, pane/tab/title changes | tmux/Zellij/Ghostty seams; no Nova platform proof | MIT; useful plugin transport reference, duplicates Radar policy. |
| History reader: [AgentHUD `262d1c54`][agenthud] | Cross-agent history, HUD and reports | Agent JSONL and OpenCode SQLite stores | Run the reader over existing stores | Private conversation/session data | Node; macOS/Linux development; no Nova proof | Manifest declares MIT; additional store readers without a Zellij-pane identity contract. Reject live-rail role. |

Mechanisms worth retaining as references:

- [Radar producers][radar-producers] send `zj_radar.status.v1`; [rail commands][radar-commands]
  use `zj_radar.cmd.v1`. Its [permission boundary][radar-permissions]
  does not request pane-content access. Provider payloads still carry user data;
  event-driven reporting is not a claim that only status enums cross the boundary.
- [zj-agent-mob's design][agent-mob-design] uses one on-demand movable plugin
  instance and publishes a stable fleet-summary file and pipe for status bars.
  Its default process discovery, prompt approval, follow-up and injected peer
  context exceed Nova's activity-observation boundary; only the lifecycle and
  summary mechanisms remain references.
- [Fleet discovery][fleet-discovery] gives a fresh title match precedence over
  a stale screen scrape. [Its refresh path][fleet-refresh] selects hook-backed
  tracking when a hook record exists, with further signal fusion inside that
  path. This is a design reference, not proof of Nova's proposed precedence rule.
- [Captain Miao's Codex adapter][miao-codex] owns a named profile and writes
  precomputed trust hashes. It preserves other profile data, but Nova rejects
  taking over that profile/trust boundary.
- [zj-agents' engine][zj-agents-engine] requests pane-content access and schedules
  a one-second timer. A failed push-signal proof does not authorize this fallback.
- [Rook's foreground monitor][rook-monitor] separates agent identity from turn
  status. [gentle-agent-state's adapters][gentle-adapters] show a small native
  plugin route, but its decoration/state ownership would compete with Radar.

## Provider-native sources

**Codex, source-proven.** The original review used [rust-v0.152.1
`5adb68a4`][codex-browser]: `/hooks` toggles the selected hook, while the
[startup review][codex-review] offers bulk trust for hooks needing review.
Trust and enablement are separate states; this is not a general bulk-enable API.
The failed signal investigation used [rust-v0.153.0 `41e22fee`][codex-title-tests]
and its [OSC title writer][codex-title]. Activity and action-required titles
exist at that revision, but their existence does not prove delivery to Radar.
[Radar's Codex producer][radar-producers] maps explicit prompt/tool/subagent,
permission and stop events to running, pending and done. Its legacy `notify`
mode occupies Codex's single notifier slot and reports completion only; Nova
does not select it as the setup default.

**Claude Code, documentation evidence.** The official [hooks reference][claude-hooks],
accessed 2026-09-07, documents lifecycle JSON input and plugin-bundled hooks.
This is a mutable documentation source, not an audit of the closed client.
The exact [Radar Claude hook bundle][radar-claude] and [setup implementation][radar-claude-setup]
show the consumed events and marketplace CLI route. Installing/enabling a
provider plugin is an explicit setup action; Nova does not seed Claude hooks
merely because its binary exists.

**OpenCode, source-proven.** Supplementary native-source review at
[`57ef3828`][opencode-api] confirms the typed plugin interface; the
[loader][opencode-loader] supplies runtime integration. The official
[plugin documentation][opencode-docs], accessed 2026-09-07, describes local
plugin discovery. Radar v0.6.0's exact [bridge][radar-opencode] consumes
permission/question, session idle/error and tool/message events. Restart is
required after installation. Its trailing-question remap from done to pending
is a heuristic, and remote attach attributes status to the server's pane.
This supplementary source review is not a compatibility test of every OpenCode
release; the pinned [producer documentation][radar-producers] records 1.18.x
as its minimum verified version.

**Pi, source-proven but unsupported by bundled Radar.** At [`4e69b0c2`][pi-events],
extensions receive `agent_start`, `agent_settled`, turn, tool and shutdown events.
The [loader][pi-loader] discovers project and user extensions. This is a possible
future producer route, with no Nova adapter or runtime proof implied.

**Grok Build, unsupported by bundled Radar.** Its [native README at
`72a61251`][grok] advertises hooks. That alone proves neither a Radar adapter nor
Nova permission to configure them. The packaged [setup targets][radar-setup]
remain Codex, Claude Code and OpenCode for `yzx radar-setup`.

## Blocked passive proposal

`yazelix-5vl-radar-first-sidebar-xj0.14` records the **2026-09-03 source-proven
negative result**: OSC title edges do not reliably reach plugins in Nova's
normal multi-pane topology with managed tab names. Rechecking the current
[Nova Zellij `796a30c4` screen path][zellij-screen] and
[single-pane naming condition][zellij-tab] preserves that constraint: rendering
reports changed derived single-pane tab names, rather than every changed pane
title. Radar can receive `PaneInfo.title` in `PaneUpdate`; missing update edges
cannot be repaired by a unit-tested parser. This was a source failure, not a
successful installed hookless experiment.

**Proposed only:** command events establish Codex identity until shell/exit;
useful activity titles indicate running or pending; missing/custom titles
degrade to presence/idle. Valid producer reports must override inference.
An unfocused busy-to-idle transition could only suggest done heuristically and
would clear on focus; it cannot establish exact completion or error. None of
this fallback or precedence is delivered by the current pin.

Resume only after a reliable upstream push event is proven in normal Nova
topology, or the user explicitly accepts coordinated Zellij/Radar fork work.
Recheck exact producer, transport and consumer revisions then. A separately
authorized, isolated zj-agents comparison remains possible; its polling,
permissions and additional owner require their own decision.

## Retained proofs and limits

- **Historical mechanical proof, xj0.1 (2026-08-04):** upstream Radar v0.2.2
  [`b73e3ce3`][radar-original] built as Nix CLI/WASM and ran in an isolated Linux
  Zellij 0.44.3 session. Producer delivery, rendering and running/pending/done
  state passed; bounded sends observed 54–107 ms within a five-second deadline.
  **Attention-next failed** to switch tabs in both managed and stripped controls.
  Preserve that failure; neither the passes nor failure certify today's pin.
- **Installed proof and human dogfood, xj0.16:** the Codex first-offer flow was
  accepted. In the separately authorized hook-removal test, the user declined
  setup and observed pane information without Codex activity. This confirms the
  reporting requirement, not hookless detection. The proof record is
  `yazelix-5vl-radar-first-sidebar-xj0.16`.
- **Mechanical installed proof, xj0.19:** Nova [`72e37087`][nova-setup-revision]
  passed isolated setup checks for mixed consent, repeats, alternate config
  roots, preserved foreign hooks/trust, absent providers, noninteractive
  no-write behavior and failure propagation. The installed Linux artifact was
  `/nix/store/1akxl3dvvqgjv4hi4jm25anc2s58jdza-yazelix`. A fresh agent-operated
  Zellij PTY exercised Alt Shift M and the native setup prompts. Claude
  marketplace calls used a recording substitute; this was not a real
  marketplace installation or human GUI acceptance. Exact-revision
  [Linux CI][setup-linux] and [Darwin Package Smoke][setup-darwin] passed;
  Darwin build evidence does not imply Darwin interaction dogfood.

Read the decision with `br show yazelix-nova-agent-command-center-srmq-2hd.1 --json`. xj0.19's full ID
is `yazelix-5vl-radar-first-sidebar-xj0.19`; its local scripts/results are under
`~/.local/state/codex/proofs/nova-radar-setup-xj019`. The completed setup evidence
supersedes 2hd.1's historical wording that the command was still planned.

## Rejected defaults and maintenance

Unconsented hook seeding, copied trust hashes and hook-trust bypass would make
Nova decide for the user. Explicit, child-owned setup remains supported.
A Nova transcript reader, process poller, pane parser or second tracker would
expand data access and duplicate Radar. Full terminal/session replacements
would reopen an ownership boundary the user has retained. The blocked title
route changes none of those decisions.

Update this ledger in the same change whenever provider/Radar selection,
status derivation, relevant ownership or a reviewed revision changes. Record
the replacement SHA, mechanism and proof scope. Label invalidated evidence
**stale** or **superseded**, retain material negative results and reversal
conditions, and link replacement proof. Do not silently convert source review
into runtime support or start another competing research note.

[discussion]: https://github.com/zellij-org/zellij/discussions/5391#discussioncomment-17908201
[radar-fork]: https://github.com/Yazelix/zj-radar/tree/1be7d73417fc15d5403734e2e2312296573a111e
[radar-upstream]: https://github.com/marktoda/zj-radar/blob/0b1672606ca118f6241df687889142c8b52d03fe/README.md
[radar-producers]: https://github.com/marktoda/zj-radar/blob/0b1672606ca118f6241df687889142c8b52d03fe/docs/producers.md
[radar-commands]: https://github.com/marktoda/zj-radar/blob/0b1672606ca118f6241df687889142c8b52d03fe/docs/using.md
[radar-permissions]: https://github.com/marktoda/zj-radar/blob/0b1672606ca118f6241df687889142c8b52d03fe/crates/plugin/src/lib.rs#L191-L196
[agent-mob]: https://github.com/mohseenrm/zj-agent-mob/tree/0b2d4fc23d26a88cf94acd07b137d04c1adf06c4
[agent-mob-design]: https://github.com/mohseenrm/zj-agent-mob/blob/0b2d4fc23d26a88cf94acd07b137d04c1adf06c4/docs/how-it-works.md
[herdr]: https://github.com/herdrdev/herdr/blob/94f6d9c0d9bb9cf9ffae99d8bbfb09e9bf2fc9e0/README.md
[zj-agents]: https://github.com/kaankoken/zj-agents/blob/39795f26960505e378c619ffaf0963efd712bfeb/README.md
[zj-agents-engine]: https://github.com/kaankoken/zj-agents/blob/39795f26960505e378c619ffaf0963efd712bfeb/crates/zj-agents-engine/src/main.rs#L62-L92
[miao]: https://github.com/hyperlogue/captain-miao/blob/07e4a47886f5861294511ae28553201a16276f1f/README.md
[miao-codex]: https://github.com/hyperlogue/captain-miao/blob/07e4a47886f5861294511ae28553201a16276f1f/crates/cm-core/src/agents/codex.rs#L1-L35
[zellaude]: https://github.com/ishefi/zellaude/blob/1d1d56c6d29155d59ed9bd1386f970168f781bb8/README.md
[fleet]: https://github.com/nicknisi/fleet/blob/30450f0970bbc1e45ed2ff8488fe00908644cfe0/README.md
[fleet-discovery]: https://github.com/nicknisi/fleet/blob/30450f0970bbc1e45ed2ff8488fe00908644cfe0/src/agents/discovery.ts#L163-L225
[fleet-refresh]: https://github.com/nicknisi/fleet/blob/30450f0970bbc1e45ed2ff8488fe00908644cfe0/src/state/refresh.ts#L445-L480
[rimz]: https://github.com/rimio-ai/rimz/blob/cb8c0dce559177305e53a2f061a85bd449d922ce/README.md
[rook]: https://github.com/jokius/rook/blob/7cb77a38dc7e180305fbee43cec9ca65fb50383c/README.md
[rook-monitor]: https://github.com/jokius/rook/blob/7cb77a38dc7e180305fbee43cec9ca65fb50383c/rook/Ghostty/AgentMonitor.swift
[gentle]: https://github.com/Gentleman-Programming/gentle-agent-state/blob/963e85379ebf66f5304797b015327e133e6f4b6a/README.md
[gentle-adapters]: https://github.com/Gentleman-Programming/gentle-agent-state/tree/963e85379ebf66f5304797b015327e133e6f4b6a/adapters
[agenthud]: https://github.com/neochoon/agenthud/blob/262d1c540e416b3c8aafa9e61fdec3d6232cd579/README.md
[codex-browser]: https://github.com/openai/codex/blob/5adb68a49933ae446bf11935662c83dba55a0804/codex-rs/tui/src/bottom_pane/hooks_browser_view.rs
[codex-review]: https://github.com/openai/codex/blob/5adb68a49933ae446bf11935662c83dba55a0804/codex-rs/tui/src/startup_hooks_review.rs
[codex-title-tests]: https://github.com/openai/codex/blob/41e22fee981a63b3698df7ed36bad393cda24715/codex-rs/tui/src/chatwidget/tests/terminal_title.rs
[codex-title]: https://github.com/openai/codex/blob/41e22fee981a63b3698df7ed36bad393cda24715/codex-rs/tui/src/terminal_title.rs
[claude-hooks]: https://code.claude.com/docs/en/hooks
[radar-claude]: https://github.com/marktoda/zj-radar/blob/0b1672606ca118f6241df687889142c8b52d03fe/plugins/zj-radar-claude/hooks/hooks.json
[radar-claude-setup]: https://github.com/Yazelix/zj-radar/blob/1be7d73417fc15d5403734e2e2312296573a111e/crates/cli/src/setup/claude.rs
[opencode-api]: https://github.com/anomalyco/opencode/blob/57ef3828431790c53f8f333c7ffbfe88770a1812/packages/plugin/src/index.ts
[opencode-loader]: https://github.com/anomalyco/opencode/blob/57ef3828431790c53f8f333c7ffbfe88770a1812/packages/opencode/src/plugin/index.ts
[opencode-docs]: https://opencode.ai/docs/plugins/
[radar-opencode]: https://github.com/marktoda/zj-radar/blob/0b1672606ca118f6241df687889142c8b52d03fe/crates/cli/src/setup/opencode_plugin.js
[pi-events]: https://github.com/badlogic/pi-mono/blob/4e69b0c28060f0f02fbe38bfa7c21a2e2eb25057/packages/coding-agent/src/core/extensions/types.ts
[pi-loader]: https://github.com/badlogic/pi-mono/blob/4e69b0c28060f0f02fbe38bfa7c21a2e2eb25057/packages/coding-agent/src/core/extensions/loader.ts
[grok]: https://github.com/xai-org/grok-build/blob/72a61251fcffb464bcc687aeb5a998e5a98ec0c9/README.md
[radar-setup]: https://github.com/Yazelix/zj-radar/blob/1be7d73417fc15d5403734e2e2312296573a111e/crates/cli/src/setup/mod.rs
[zellij-screen]: https://github.com/Yazelix/nova-zellij/blob/796a30c44c8c4369021e7bb91e2b6c62cfc257de/zellij-server/src/screen.rs#L4234-L4237
[zellij-tab]: https://github.com/Yazelix/nova-zellij/blob/796a30c44c8c4369021e7bb91e2b6c62cfc257de/zellij-server/src/tab/mod.rs#L2176-L2200
[radar-original]: https://github.com/marktoda/zj-radar/tree/b73e3ce31ee49b62eff50287c50b11fae416d767
[nova-setup-revision]: https://github.com/Yazelix/nova/commit/72e37087f2d4bbb656da673066af899a0d1fc56a
[setup-linux]: https://github.com/Yazelix/nova/actions/runs/34127980578
[setup-darwin]: https://github.com/Yazelix/nova/actions/runs/34127982410
