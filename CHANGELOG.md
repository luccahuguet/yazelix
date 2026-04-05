# CHANGELOG

Short, upgrade-facing release notes live here. The longer narrative history remains in [docs/history.md](./docs/history.md).

## v13.12 - 2026-04-05

Simpler runtime updates and stronger update-path hardening.

Upgrade impact: no user action required

Highlights:
- Replaced the clone-oriented `yzx update repo` flow with `yzx update runtime`, and redefined `yzx update all` around the packaged runtime plus the runtime-owned `devenv` CLI.
- Hardened `yzx launch` and `yzx restart` so they stop trusting stale current shells that do not contain the newly configured terminal.
- Fixed `yzx dev update --canary-only` regressions around canary selection syntax and read-only temporary config copies.
- Deleted the copied `PINNED_DEVENV_VERSION` contract so maintainer pins and doctor output stop pretending there is a third authoritative `devenv` version source.
- Restored `[zellij].pane_frames` as a canonical managed config option while keeping direct Zellij pane-frame ownership and rounded-corner control.
- Reverted the managed Yazi default theme to Yazi's upstream built-in default instead of forcing the bundled `tokyo-night` flavor unless you opt into a flavor explicitly.
- Fixed Home Manager runtime-source evaluation so the standalone `home_manager` flake can validate and install the lock-derived runtime without tripping invalid parent-source paths.
- Dropped the broken Home Manager source-input workaround and kept the simpler module wiring now that the lock-derived `devenv` source import is stable again.

## Unreleased

Post-v13.12 work in progress

Upgrade impact: no user action required

Highlights:
- Reserved for post-release changes after v13.12 lands.

## v13.11 - 2026-03-31

Configurable Yazi command overrides, Ghostty neon tuning, and maintainer tooling polish.

Upgrade impact: no user action required

Highlights:
- Added `[yazi].command` and `[yazi].ya_command` so managed Yazi launches and sidebar/reveal actions can use explicit binaries instead of only relying on `PATH`.
- Added `nu-lint` to the `maintainer` pack and exposed the same maintainer-tooling surface through Home Manager's pack definitions.
- Tuned the Ghostty neon cursor-trail base color from violet toward a brighter cyan-blue so the shipped neon variant matches its intended palette.
- Clarified that custom Yazi plugin initialization lives in `user_configs/yazi/init.lua`, and updated user-facing docs to the current nested config shape and `user_configs/` paths.
- Refreshed maintainer input pins and updated the runtime-owned `devenv` CLI to `2.0.7`.

## v13.10 - 2026-03-30

Automatic safe migrations, sidebar/layout polish, and cleaner config commands.

Upgrade impact: manual action required

Highlights:
- Added automatic safe config-migration preflight on startup, launch, restart, and interactive env flows, while keeping `yzx config migrate` as the explicit preview and repair surface.
- Stabilized sidebar-aware layout control with pane-orchestrator fixes, removed stale swap-layout widget output, added the missing no-side bottom-terminal family, and made `editor.sidebar_width_percent` configurable.
- Simplified config-facing commands by splitting downstream inspection into `yzx open hx|yazi|zellij`, managed config editing into `yzx edit config|packs`, and adding `yzx config reset --no-backup`, while polishing welcome-screen skip behavior.

Migration notes:
- Replace `yzx config hx`, `yzx config yazi`, and `yzx config zellij` with `yzx open hx`, `yzx open yazi`, and `yzx open zellij`.
- Replace `yzx config open config` and `yzx config open packs` with `yzx edit config` and `yzx edit packs`.

## v13.9 - 2026-03-29

Managed config boundaries, richer welcomes, and cleaner terminal ownership.

Upgrade impact: migration available

Highlights:
- Split pack settings into `yazelix_packs.toml`, tightened migration-aware config ownership, and made `user_configs/` the canonical managed input boundary for Zellij, Yazi, and the main Yazelix config surfaces.
- Added terminal override layers for Ghostty, Kitty, and Alacritty while keeping launch-critical startup behavior Yazelix-owned and moving Ghostty shader generation toward runtime state.
- Expanded the front-door UX with per-user desktop entries, `yzx tutor`, `yzx screen`, explicit `yzx config open` targets, and a much richer welcome-screen system with multiple styles and animations.

Migration notes:
- `yzx config migrate` can move legacy `[packs]` settings into `yazelix_packs.toml`, replace `[ascii].mode` with `[core].welcome_style`, and rename `core.welcome_style = "life"` to `"game_of_life"`.
- If you still use `terminal.config_mode = "auto"`, replace it with either `"yazelix"` or `"user"` after deciding which config owner you want.
- Yazelix now treats `user_configs/` as the canonical managed input boundary and relocates legacy root or native config files into that structure when it can do so safely.

## v13.8 - 2026-03-27

Zellij 0.44, startup recovery, and migration-aware upgrade UX.

Upgrade impact: migration available

Highlights:
- Upgraded Yazelix to Zellij `0.44`, hardened the startup handoff, and added an explicit permission recovery path for broken plugin state.
- Added migration-aware stale-config diagnostics plus `yzx config migrate` and `yzx doctor --fix` for safe deterministic rewrites.
- Added a root `CHANGELOG`, structured upgrade notes, first-run upgrade summaries, and historical `v12`/`v13` note backfill.

Migration notes:
- Removed the broken `layout` value from `zellij.widget_tray`; safe migration is available.
- Removed the obsolete `shell.enable_atuin` toggle; safe migration is available.
- Legacy Ghostty cursor-trail settings still require manual review because the old intent is not always deterministic.

## v13.7 - 2026-03-26

Popup polish, Ghostty controls, and stronger validation.

Highlights:
- Added configurable popup sizing and a configurable popup program for Zellij floating panes.
- Added Ghostty trail glow controls plus a `ghostty_trail_color = "none"` option.
- Tightened config-schema validation, ignored Yazelix config backup files, and improved test/spec governance.

## v13.6 - 2026-03-23

Managed popup runner workflow and configurable popup commands.

Highlights:
- Shipped the managed popup-runner workflow for Zellij floating panes.
- Added configurable popup commands and trimmed the preview asset surface around popup flows.

## v13.5 - 2026-03-22

Pane-walking polish and Zellij config merge path fixes.

Highlights:
- Fixed Zellij config merge path resolution and removed residual runtime path callers.
- Stabilized pane-orchestrator updates and made pane walking skip the closed managed sidebar.

## v13.4 - 2026-03-22

Reliability hardening, issue automation, and spec workflow foundations.

Highlights:
- Hardened launch, config, CI, and Home Manager reliability around the modern runtime model.
- Added Beads/GitHub issue contract automation, architecture docs, and the spec-driven workflow foundation.

## v13.3 - 2026-03-15

Restart workspace bootstrap follow-through and roadmap reset.

Highlights:
- Bootstrapped the first-tab workspace from restarted Yazi state and added restart-only Yazi cwd inheritance.
- Refined the maintainer update flow, restored Gemini CLI support, and reset the rewrite roadmap in Beads.

## v13.2 - 2026-03-15

Workspace opening polish and the Ghostty cursor-effect transition.

Highlights:
- Made `Alt+p` set the tab workspace root and fixed `Alt+m` cwd regressions and startup cwd leaks.
- Started the Ghostty cursor-effect transition that split older cursor-trail intent across newer trail and mode fields.

Migration notes:
- If you customized older Ghostty cursor-trail settings, review them against the newer trail and mode fields because the old intent is not always safe to rewrite automatically.

## v13.1 - 2026-03-14

Safer multi-tab cwd routing, stronger workspace sync, and better diagnostics.

Highlights:
- Scoped Yazi sidebar cwd sync to the current tab and strengthened the `yzx cwd` workspace flow.
- Improved Zellij and workspace diagnostics while cleaning up pane-handling and update-canary behavior.

## v13 - 2026-03-07

Plugin-managed editor/sidebar orchestration and deterministic workspace controls.

Highlights:
- Replaced fragile pane-scanning with plugin-managed editor and sidebar orchestration plus deterministic layout controls.
- Improved launch freshness on restart and unified workspace-navigation behavior around the pane orchestrator.

## v12.11 - 2026-03-06

TUI rebuild restoration and restart refresh fixes.

Highlights:
- Restored the TUI rebuild flow for launch and fixed restart refresh quoting.
- Reduced sweep subprocess cost, showed live sweep progress, and removed dead launch IPC plumbing.

## v12.10 - 2026-03-03

Refresh command and shell-initializer simplification.

Highlights:
- Added `yzx refresh` and shared devenv refresh helpers across the main command surface.
- Removed the `shell.enable_atuin` toggle and simplified shell initializers to rely on direct `PATH` detection.

Migration notes:
- `shell.enable_atuin` was removed in this release; remove the field or use a migration-aware repair flow on newer versions.

## v12.9 - 2026-02-08

Ghostty input fixes and sidebar/layout cleanup.

Highlights:
- Added an automatic Ghostty IM-module fallback for dead keys on Wayland.
- Cleaned up sidebar and zjstatus layout handling around restart and startup.

## v12.8 - 2026-02-08

Command palette and restart hardening.

Highlights:
- Added the `yzx` command palette.
- Hardened restart launch behavior around the refreshed command flow.

## v12.7 - 2026-02-07

Safer `yzx env` shell supervision and better version reporting.

Highlights:
- Reworked `yzx env` shell supervision to prevent orphaned shells and prefer the stronger `setpriv` path where available.
- Improved version reporting and locked tool-version resolution more tightly to repo state.

## v12.6 - 2026-02-01

Update-nix flow and desktop launcher hardening.

Highlights:
- Added `yzx update nix` and strengthened version pin reporting around the maintainer flow.
- Hardened the desktop launcher shell environment and clarified Yazi config merge behavior.

## v12.5 - 2026-01-29

Garbage collection command and stronger pack reporting.

Highlights:
- Added `yzx gc` for Nix-store cleanup.
- Improved `yzx packs` output and CLI documentation around the pack surface.

## v12.4 - 2026-01-29

Dynamic Yazelix version display in zjstatus.

Highlights:
- Made the zjstatus bar read the live Yazelix version from `constants.nu`.

## v12.3 - 2026-01-29

llm-agents integration, AI packs, and `yzx packs`.

Highlights:
- Integrated `llm-agents.nix`, added AI-focused packs, and added the `yzx packs` inspection command.
- Added Cachix binary caches and tightened unfree-package handling around the AI tooling surface.

## v12.2 - 2026-01-29

AI pack experiment and system-mode rollback.

Highlights:
- Added the first AI pack experiment for the newer tooling era.
- Backed out the short-lived system-mode path and documented the experiment more clearly.

## v12.1 - 2026-01-25

`yzx run` and Helix utility-prefix polish.

Highlights:
- Added `yzx run` for running a single command inside the Yazelix environment.
- Polished the Helix utility prefix and refined the release notes around the v12 surface.

## v12 - 2026-01-25

User-declared packs, Yazi config merging, and tighter terminal integration.

Highlights:
- Added user-declared packs, Yazi user-config merging, and cleaner terminal and persistent-session behavior.
- Fixed default-shell persistence, hot-reload behavior, and Kitty cursor-trail fallback handling.

See also:
- Full narrative history: [docs/history.md](./docs/history.md)
