# CHANGELOG

Short, upgrade-facing release notes live here. The longer narrative history remains in [docs/history.md](./docs/history.md).

## Unreleased

Post-v14 work in progress

Upgrade impact: no user action required

Highlights:
- Reserved for post-release changes after v14 lands.

## v14 - 2026-04-10

Boundary hardening, honest update ownership, and runtime cleanup.

Upgrade impact: manual action required

Highlights:
- Applied a broad delete-first Nushell cleanup pass that removed stale compatibility surfaces, broad helper aliases, the old config-manager layer, the standalone startup-profile module, and many one-caller wrappers that no longer justified their own seams.
- Refined Zellij, Yazi, terminal, and shell-hook ownership by splitting semantic merger blocks, centralizing Yazelix-owned Zellij settings, tightening Yazi override boundaries, replacing wrapper `nu -c` calls with runtime scripts, and failing fast on legacy override paths.
- Matured the Home Manager path with clean-room validation, root-flake consolidation, tighter activation ordering, support for symlinked generated config surfaces, profile-owned `yzx`, `yzx home_manager prepare` to preview or archive manual-install artifacts before Home Manager takeover, and clearer doctor validation around takeover and desktop shadowing.
- Tightened everyday workspace UX with direct `Ctrl+y` focus toggling, `Alt+number` tab jumps, and better zjstatus tab-window overflow behavior before truncation.
- Hardened runtime and refresh internals with shared atomic file writes, managed-root cleanup guards, a clearer backend adapter seam, explicit runtime entry transitions, no-op refresh recovery, canonical-contract config parsing, and launch-profile freshness diagnostics in `yzx doctor`.
- Fixed real runtime identity regressions around Nix-store symlink cleanup, fresh launch-profile imports after rebuilds, desktop bootstrap env setup, bundled Yazi asset refresh cleanup, and manual desktop icon installation.
- Moved sidebar identity and workspace retargeting deeper into the pane orchestrator so managed editor/sidebar routing depends less on shell-side cache heuristics and more on live Zellij truth, then documented the backend-free workspace slice and the v14 boundary-hardening gate explicitly.
- Simplified the packaged-runtime story by making the flake package surface primary, removing `runtime/current` from the Home Manager identity path, dropping installed-runtime fallbacks from shared runtime logic, and trimming most installer-artifact doctoring.
- Replaced the late-series runtime updater experiment with explicit owner commands: `yzx update upstream` refreshes upstream/manual installs, `yzx update home_manager` refreshes the current Home Manager flake input, desktop launch now targets the active runtime launcher directly, and the transitional `yzx update runtime` / `yzx update all` flow is gone again.
- Made `yzx run` a real argv passthrough for one-shot Yazelix-environment commands, so child args like `yzx run rg --files` pass through unchanged, and documented the package-runtime simplification path, config dependence matrix, subsystem code inventory, and the trim-first v15 roadmap that follows this transition release.

Migration notes:
- Replace `yzx update runtime` with `yzx update upstream` for upstream/manual installs.
- Replace `yzx update all` with exactly one owner path: `yzx update upstream` for upstream/manual installs or `yzx update home_manager` for Home Manager installs.

Yazelix Classic, v15, and the `yzx` surface:

v14 is the last feature release of what I now think of as Yazelix Classic.

Yazelix Classic is the broad, heavily integrated, `devenv`-era shape of the project: `yazelix packs`, dynamic runtime management, rich shell and terminal integration, multiple ownership paths, a large `yzx` surface that includes commands like `yzx packs`, and the unusually wide power-user workflow that made Yazelix one of a kind.

That line is not dead. The `v14` tag will continue to be maintained for bug fixes and stability fixes if real issues are found. What changes now is scope: v14 is entering a feature freeze.

Most new design and implementation work will now shift to v15.

The goal for v15 is not to rewrite Yazelix line for line in Rust. The goal is a smaller, slimmer, faster, more opinionated Yazelix with a much clearer product boundary. In practice, that means dropping `devenv`, stopping the project from also trying to be a broad package-and-environment manager, trimming the command and config surface, and then rewriting the smaller surviving product in Rust.

That is the main architectural lesson of v14: Yazelix had clearly become two products in one.

One product was the broad environment-management system: rebuild and refresh semantics, package and pack ownership, shell and terminal breadth, multiple install and update ownership modes, launch-profile state, and a large amount of dynamic runtime machinery.

The other product was the narrower workspace tool that had been trying to emerge inside it: fast entry into a built runtime, explicit ownership, predictable workspace behavior, stronger editor/sidebar orchestration, and a smaller core that can eventually be made much more robust and performant in Rust.

v14 is the release where that split became impossible to ignore. It did not fully resolve it, but it made the problem visible enough to finally treat it honestly.

A lot of the current `yzx` surface belongs to Yazelix Classic. That includes the parts of `yzx` that are tightly tied to the older `devenv` hot-path and cold-path model: explicit refresh semantics, dynamic runtime entry behavior, launch-profile reuse, first-class `yazelix packs` / `yzx packs` package selection and inspection, broad pack and package-graph ownership, wider shell and terminal policy, and the idea that Yazelix should also act as a fairly general environment-management layer.

In that sense, commands like `yzx refresh` and much of the older meaning carried by `yzx run` belong much more to the v14 Classic world than to the slimmer v15 direction.

The current v15 lean is to keep the core `yzx` product surface and trim away the parts that mainly exist to support the older `devenv` machinery. The backbone most likely to survive is `yzx launch`, `yzx env`, `yzx update`, and `yzx desktop`. Beyond that backbone, workspace-facing commands such as `yzx cwd`, `yzx reveal`, `yzx popup`, `yzx menu`, `yzx keys`, `yzx tutor`, `yzx whats_new`, and `yzx doctor` still fit the actual product much better than the older backend-management surface does.

What is much less likely to survive in its current form are commands and semantics that mainly exist because Yazelix was also trying to manage a large dynamic `devenv` lifecycle. That is why the current v15 direction leans toward dropping or heavily narrowing `yzx refresh`, `yzx run`, launch-profile reuse semantics, explicit backend/materialization entry logic on the hot path, and the broader `yazelix packs` / `yzx packs` package-graph ownership model unless a much narrower replacement survives.

There is also a real chance that the broader `devenv` runtime and terminal-environment layer will continue as a separate project, forked from Yazelix Classic. That would let the broader environment-management direction evolve on its own terms instead of staying entangled with the slimmer v15 product.

If that separate project proves valuable, it may be reintegrated later, potentially around v16, but only with much cleaner boundaries: separate codebases, clear separation of concerns, and an explicit integration seam between the two products.

v14 may also end up being the last heavily Nushell-based release of Yazelix before most of the system is rewritten in Rust. If that happens, the `v14` line will remain useful not just as a product, but also as a substantial real-world Nushell codebase for people who want to study how a larger Nushell-heavy project is structured.

I still strongly recommend using v14 for the time being, especially if you are a power user. Yazelix Classic remains unusually powerful, highly customizable, and very much alive. If you find bugs, please open issues. If the Classic direction is the one you care about most, you can also keep using it, customize it heavily, or fork it and take it further yourself.

## v13.13 - 2026-04-05

Yazi git refresh hardening, fresh launch-profile recording, and maintainer input updates.

Upgrade impact: no user action required

Highlights:
- Refreshed Yazi sidebar git decorations more reliably on focus return, open, navigation, and explicit sidebar refresh so the managed sidebar stops carrying stale git state.
- Recorded the fresh built launch profile after Yazelix-owned rebuilds so desktop-entry and restart flows stop reactivating stale `DEVENV_PROFILE` paths.
- Updated maintainer inputs, including the packaged `beads-rust` build used for `br`, and verified the real issue-mutation path after the bump.

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
