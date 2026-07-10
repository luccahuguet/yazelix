# CHANGELOG

Short, upgrade-facing release notes live here. The longer narrative history remains in [docs/history.md](./docs/history.md).

## Unreleased

Post-v17.9 work in progress

Upgrade impact: no user action required

Highlights:
- Repaired the published Yazi assets Cargo vendor hash so fresh Nix installs build on Linux and macOS

## v17.9 - 2026-06-27

Post-v17.8 work in progress

Upgrade impact: manual action may be required

Highlights:
- Adopted the grouped Zellij mode layer from Yazelix Next: `Ctrl p/t/n/q` keep pane, tab, resize, and quit fast, `Ctrl Alt g/s/o` keep lock, search, and session grouped off plain Ctrl, `Ctrl h` stays unbound from move mode, and the consumed Zellij fork groups mixed modifier hints cleanly in the bottom bar
- Kept every capable terminal supported through `yzx enter` while making Mars the deepest integrated packaged path, Ghostty the most tested mature host-terminal path, and non-Mars terminal config native to each terminal
- Added `yzx cursors ghostty setup` so host Ghostty users can generate the Yazelix cursor include from the normal Yazelix package without installing `#yazelix_cursors` separately
- Gave the runtime smoke CI lane enough time for cache-miss builds

Manual action:
- Replace non-Mars flake outputs such as `#yazelix_ghostty`, `#yazelix_kitty`, `#runtime_kitty`, `#runtime_wezterm`, `#runtime_rio`, `#runtime_foot`, or `#runtime_ratty` with `#yazelix` or `#yazelix_mars`
- Remove Home Manager `programs.yazelix.extra_terminal_launchers` or older `programs.yazelix.extra_terminal_variants` settings, and set `programs.yazelix.terminal = "mars"` if the option is present; terminals other than packaged Mars should start Yazelix with `yzx enter`


## v17.8 - 2026-06-22

Mars default, status-bar chrome, and startup polish

Upgrade impact: no user action required

Highlights:
- Made Mars the default packaged terminal and finished the Mars rename across package outputs, Home Manager `programs.yazelix.terminal`, runtime metadata, launch environment, desktop entries, window class, icon assets, and docs; Ghostty remains the first-class mature alternate, with Rio, WezTerm, Kitty, Foot, and Ratty still available through explicit runtime variants
- Integrated the Zellij config-pack renderer into the Rust workspace so generated layouts, keybind rendering, swap fragments, and status-bar plugin wiring share one tested source instead of duplicated main-repo layout ownership
- Made the session name a normal status-bar widget, added configurable status widget chrome with `none`, `square`, and `round` frames plus `dot`, `pipe`, `empty`, and `space` separators, moved CPU/RAM out of the default tray while keeping them opt-in, and updated the compact Codex usage shape
- Improved tab creation UX: Zellij tab-mode `n` now opens a home-scoped tab named with the home marker, and the startup home tab uses the same marker instead of the generic `Tab #1`
- Rendered tutor content from Markdown and enriched onboarding with explicit key hints for `Alt Shift H/J/K/L`, `Ctrl y`, `Ctrl Shift Y`, and `Alt r`, plus clearer `yzx env` tool-path guidance
- Added runtime/install reliability checks including the cold-install runtime verifier, bootstrap install check, runtime self-description surface, generated-state diagnostics, Home Manager settings-contract state repair, and fresher Home Manager desktop launchers
- Tamed Zellij plugin helper churn by moving terminal activity refresh away from timer-driven shell-outs, consuming cold-stable system usage widgets, adding plugin doctor health reporting, and adding `yzx dev perf` planning for bounded Zellij/plugin snapshots
- Updated Mars, Yazelix Cursors, pane-orchestrator, Helix grammar, and Zellij bar child pins for terminal package identity, standard desktop icon sizes, cursor shader target contracts, compact AI tab markers, archive-backed grammar fetchers, and compact status-bar rendering
- Hardened maintainer and CI infrastructure with Darwin wasm smoke coverage, branch-push CI, scheduled runtime-env helper fixes, longer integration smoke timeouts, release child-transaction validation, and clearer cache setup docs


## v17.7 - 2026-06-15

Spartan runtime thinning, child config packs, and tab activity fixes

Upgrade impact: no user action required

Highlights:
- Cut roughly 9.6k net Rust lines from the main repo by deleting stale `yzx_core` wrappers, broad duplicate tests, one-use launch/materialization scaffolding, doctor bridge layers, and local render-plan ownership that no longer belonged in Yazelix
- Consumed child-owned Zellij and Yazi config-pack renderers, removed copied Zellij layout fragments and Yazi render-plan files from the main repo, and documented the child repo promotion gates that keep future extractions honest
- Fixed pane-orchestrator tab activity decorations so inactive tabs update when agents finish thinking or pane commands start and stop, instead of waiting for the tab to be visited
- Refreshed the screen runtime and prey sprite package pins through the current composite sprite set
- Repaired mars/Rio transparency and opacity handling for TUI backgrounds while keeping the generated welcome and ANSI colors readable
- Removed the obsolete NixCI file and tightened maintainer/runtime safety guidance, including the rule that agents must not run `yzx restart` against a live user session


## v17.6 - 2026-06-13

Upstream Yazi, adaptive appearance, and release hardening

Upgrade impact: manual action may be required

Highlights:
- Moved the shared cursor settings namespace from `~/.config/yazelix_ghostty_cursors/settings.jsonc` to `~/.config/yazelix_cursors/settings.jsonc`, and renamed the packaged default template to `yazelix_cursors_default.toml`
- Added adaptive appearance support across Yazelix and Mars Terminal, including dark, light, and auto terminal modes, appearance-aware welcome surfaces, `ice` and `midnight` cursor presets, and safer light-mode random cursor selection
- Replaced the default process monitor popup and bundled runtime tool from `bottom` / `btm` to `zenith`, with the Zenith information popup on `Alt Shift I`; bottom and SysWatch remain good custom-popup alternatives
- Routed the default right sidebar through `yzx agent`, so missing Codex opens an actionable shell placeholder instead of failing; users can configure another agent or any other terminal command for the managed right sidebar
- Added host-owned `xonsh` shell support for generated initializer hooks and optional `shell.default_shell = "xonsh"` without bundling xonsh in the Yazelix runtime
- Dropped the temporary Yazelix Yazi fork from the runtime package graph; managed Yazi launches use upstream/nixpkgs Yazi with a scoped Kitty adapter environment while Yazelix restores the real Zellij session for sidebar state and `Alt z` zoxide-to-editor control commands
- Kept `Shift Z` on native Yazi zoxide behavior while preserving Yazelix's separate `Alt z` editor-pane jump flow
- Removed the cursor widget from the status bar and guarded retired status-bar widget config paths with clearer errors
- Used Zellij session names in terminal window titles so desktop switchers distinguish sessions
- Hardened `yzx dev release` so it auto-syncs GitHub issues into Beads when needed, pushes `main` before the release tag, waits for `CI` plus `Publish Nix Cache`, and splits slow release validators across parallel CI jobs
- Fixed mars welcome rendering so startup-size races no longer leave the welcome card tiny or corrupt the banner when a key is pressed during the animation
- Kept mars window transparency while making generated welcome and ANSI colors render crisp instead of washed out by per-cell alpha blending
- Published Darwin package outputs to Cachix, tightened wasm plugin package toolchain preservation, and kept the Yazelix wrapper substitutable in Nix package overrides
- Guarded ratconfig/settings contract migration changes so unsupported or stale config fields fail with clearer reset guidance

Manual action:
- Move existing cursor settings from `~/.config/yazelix_ghostty_cursors/settings.jsonc` to `~/.config/yazelix_cursors/settings.jsonc`, or run `yzc init` / relaunch Yazelix to create a fresh cursor settings file


## v17.5 - 2026-06-07

Popup reliability, keyboard repair, and child-owned runtime cleanup

Upgrade impact: manual action may be required

Highlights:
- Made `zellij.custom_popups` fully editable in `yzx config ui`, including popup id, command argv, keybindings, and `keep_alive`, while the default `btm` process viewer keeps its process state and restores the larger popup geometry across focused toggles
- Repaired native Zellij movement defaults so `Ctrl Alt h` and `Ctrl Alt l` move tabs, `Ctrl Alt j` and `Ctrl Alt k` move panes, `Ctrl h` is removed from the default Zellij move-mode entry, and `zellij.support_kitty_keyboard_protocol = true` is the default for unambiguous modified-letter input
- Added `terminal.emoji_style` and Home Manager `programs.yazelix.mars_emoji_font` for Mars Terminal emoji fallback selection, with `noto`, `twitter`, and `serenityos` values, and refreshed mars runtime pins, desktop identity, polished icons, and fast dogfooding cache guidance
- Removed the image-backed `magician` welcome, `yzx screen`, standalone `yzs`, runtime asset, Home Manager, and idle screen-saver surfaces so retained screen styles are text/animation-engine owned instead of GIF-backed
- Moved more shipped artifacts and package contracts to child-owned outputs, including cursor shader generation, Zellij plugin wasm metadata, Steel plugin defaults, and minimal runtime docs, while deleting stale copied wasm, shader trees, media assets, and runtime artifact repair paths from the main repo
- Split Nix and Home Manager ownership into smaller modules for flake outputs, terminal variant metadata, runtime integration, settings rendering, KGP package contracts, and package customization surfaces
- Hardened maintainer automation with Rust-owned release workflow helpers, GitHub/Beads issue contract validation, stronger child-release validators, readme/version/upgrade-note checks, and CI/cache behavior that avoids duplicate release-tag runs while publishing the fast mars runtime cache output
- Refactored Zellij materialization around smaller plugin/materialization modules, stricter runtime metadata parsing, and clearer stale generated-layout cleanup

Manual action:
- If your config explicitly sets `core.welcome_style = "magician"` or `zellij.screen_saver_style = "magician"`, or your Home Manager settings set `welcome_style = "magician"` or `screen_saver_style = "magician"`, choose a retained style such as `random`, `mandelbrot`, `boids`, `boids_predator`, `boids_schools`, `game_of_life_gliders`, `game_of_life_oscillators`, `game_of_life_bloom`, or `logo`


## v17.4 - 2026-06-05

Launch reliability, terminal variants, and managed Helix compatibility

Upgrade impact: manual action may be required

Highlights:
- Added `zellij.custom_popups`, moved the default `btm` process viewer popup to that surface on `Alt Shift B`, kept `btm` alive across popup toggles, and made sidebar-launched popups inherit the active Yazi cwd
- Restored and simplified terminal variants with separate packages and desktop entries for Ghostty, Mars Terminal, WezTerm, Rio, and Linux-only Foot, plus Home Manager `programs.yazelix.extra_terminal_variants` for installing additional terminal launchers beside the primary runtime
- Moved startup handoff, welcome sequencing, runtime environment generation, and release metadata ownership into Rust, deleting the obsolete Nushell startup bridge and `constants.nu` version source
- Repaired desktop and Home Manager launch freshness by refreshing generated terminal configs, mars shader assets, and launcher-visible feedback before sessions open
- Added full runtime identity diagnostics through `yzx --version-full`, clarified active runtime ownership reporting, and documented lower-CPU Home Manager activation settings
- Added the Helix action bridge path for reliable existing-editor-pane Yazi opens, reveals, cwd sync, directory picker behavior, and raw pane-id routing
- Clarified that `helix.external` is only for Yazelix-compatible Helix forks with `--config-dir` support; arbitrary upstream/vanilla Helix binaries are not supported as managed Yazelix Helix targets
- Fixed LazyGit edits so they use the Yazelix editor wrapper, respect `editor.command`, strip stale runtime-owned LazyGit config, and preserve useful cwd behavior from Yazi/sidebar popups
- Fixed `Backspace` in Mars Terminal so it does not enter Zellij move mode through `Ctrl H`, and fixed the welcome prompt so any key really continues
- Restored Linux Foot and vanilla Rio terminal support, reduced Foot's generated font size, hid the Foot titlebar, and differentiated Ghostty/terminal launcher window names for desktop switchers
- Improved cursor and shader packaging across Ghostty and Mars Terminal, including shader activation refresh, shader launch-snapshot isolation, Ghostty shader compilation fixes, and fast mars dogfooding outputs
- Hardened packaging, CI, and maintainer flows with runtime closure size reporting, Cachix cache advertisement, child release validation, macOS package smoke contracts, KGP Yazi warning suppression, and stronger guidance against Linux-only assumptions in shared surfaces

Manual action:
- If `helix.external` points at upstream or vanilla Helix, remove that setting to use bundled Yazelix Helix, point it at a fork based on `luccahuguet/yazelix-helix`, or pin `v17.1` temporarily for the pre-`--config-dir` behavior


## v17.3 - 2026-06-01

Helix Steel plugins, Mars Terminal runtime, and shared cursors

Upgrade impact: no user action required

Highlights:
- Added the experimental packaged Mars Terminal runtime as `#mars` and `#runtime_mars`, backed by the `mars` child repo and exposed through `programs.yazelix.runtime_variant = "mars"`
- Renamed the standalone cursor child repository from `yazelix-ghostty-cursors` to `yazelix-cursors` now that Mars Terminal consumes the same cursor shader package; the old cursor flake output names are not kept as aliases
- Updated the Mars Terminal runtime input, split terminal cursor materialization, and made `mars` honor the shared cursor settings path for shader/trail behavior
- Dropped Alacritty and Foot from the maintained terminal set, polished Mars Terminal docs wording, and refreshed the terminal emulator comparison matrix around protocols, packaging, and 3D graphics
- Added `programs.yazelix.extra_terminal_variants` for Home Manager users who want to install additional bundled terminal emulator packages, such as Ghostty beside a `mars` primary runtime, without adding duplicate `yzx` wrappers
- Added Kitty as a packaged runtime variant through `#yazelix_kitty`, `#runtime_kitty`, and `programs.yazelix.runtime_variant = "kitty"`
- Repaired the KGP graphics runtime path with wrapped nixpkgs Zellij support, KGP Zellij/Yazi package-base and hook fixes, Zellij cursor replay consumption, and clearer host-Yazi graphics diagnostics
- Added the Helix Steel plugin system through the `yazelix-helix` fork, including the managed Steel plugin surface, curated managed Helix defaults, bundled Steel authoring tools, steel-enabled Helix pinning, and non-Steel managed-Helix diagnostics
- Improved maintainer and repo tooling with the Rust 1.96 toolchain bump, guarded doctor repair framework, split update/validator/core dispatch, cargo-crap and mutation/coverage tooling, and stale shell-initializer runtime repair tracking

## v17.2 - 2026-05-31

Steel-backed Helix, managed Helix config tree, and plugin manifests

Upgrade impact: manual action may be required

Highlights:
- Made bundled Helix the thin first-party `yazelix-helix` Steel fork with `--config-dir`, consumed as a child-owned package, so managed sessions can use Yazelix's Helix config root without taking over `~/.config/helix`
- Moved the managed Helix source config to `~/.config/yazelix/helix/config.toml` and made `~/.config/yazelix/helix/` the root for languages, themes, ignore, Steel plugin files, and future managed Helix surfaces; the old flat `~/.config/yazelix/helix.toml` path is unsupported
- Added `helix.steel_plugins` with bundled `recentf`, `splash`, and `spacemacs_theme` defaults plus `extra` manifests for user-owned Steel plugins under `~/.config/yazelix/helix/steel_plugins`
- Added `helix.external` for users who intentionally run their own Helix fork, requiring both `binary` and `runtime_path` so binary/runtime mismatches stay explicit
- Added the `:yzx-new-shell` Helix Steel command for opening a new Yazelix shell pane from the current editor context
- Added `yzx import helix`, Steel-focused doctor/config checks, and documentation for the `v17.1` tag as the pre-Steel escape hatch
- Kept the default keybinding story stable: `Alt+r` remains the managed Helix reveal binding, and `Ctrl+y`, `Ctrl+Shift+Y`, plus `Alt+Shift+H/J/K/L` remain the workspace navigation layer

Manual action:
- If you keep custom Yazelix-managed Helix config in `~/.config/yazelix/helix.toml`, move it to `~/.config/yazelix/helix/config.toml` before launching v17.2
- If you use a custom Helix fork, configure both `helix.external.binary` and `helix.external.runtime_path`; bare binary-only custom Helix config is rejected
- If you are not ready for the bundled Steel-backed Helix line, pin `v17.1` temporarily


## v17.1 - 2026-05-30

Pre-Steel escape hatch, Rust-owned runtime polish, and terminal/VFX planning

Upgrade impact: no user action required

Highlights:
- Moved native tab and pane movement defaults to `Ctrl Alt H/L` and `Ctrl Alt J/K` so Ghostty does not intercept pane-down movement through its `Ctrl Shift J` screen-file shortcut
- Added `zellij.codex_usage_periods` with a default Codex 5h/week status window and bundled `tokenusage` in packaged runtimes so default Codex and Claude usage widgets have their helper available
- Restored popup editor runtime environment propagation, fixed sidebar runtime helper packaging, reported no-op `yzx update upstream` runs clearly, repaired desktop icon reinstalls over read-only copied icons, and added runtime diagnostics plus maintainer validators for packaged-runtime behavior
- Rust-owned `yzx menu`, launch setup preflight, canary session validation, and guarded doctor config repair while continuing to shrink shell-owned runtime paths
- Updated `yzx screen` and welcome visuals after v17, including recursive Mandelbrot work, the magician GIF/Kitty-graphics renderer, a four-second welcome default, boids height fixes, and experimental Ratty runtime/cursor probes
- Added the Yazelix-owned `yazelix-zellij` package fork and a short-lived `yazelix-yazi` experiment after `v17` to validate the Kitty Graphics Protocol passthrough path that restores Yazi image previews through Zellij; the current runtime keeps the Zellij bridge and uses upstream/nixpkgs Yazi
- Recorded the Rio-based `mars` direction, including cursor-shader parity as the first gate, protocol parity planning, and Rio/Yazi image-preview risk
- Verified that stock Helix and the Steel branch still lack a true config-dir override, recorded the scoped-XDG caveat, and planned the bundled Yazelix Helix Steel fork plus external binary/runtime-pair escape hatch
- Treat `v17.1` as the pre-Steel pin for users who are not ready to move to the future bundled Yazelix Helix Steel editor line


## v17 - 2026-05-22

First-party child repos, Ghostty image previews, and JSONC workspace config

Upgrade impact: no user action required

Highlights:
- Established the first-party child-repo architecture across `yazelix-screen`, `yazelix-ghostty-cursors`, `yazelix-zellij-popup`, `yazelix-zellij-bar`, `yazelix-zellij-pane-orchestrator`, `yazelix-yazi-assets`, and `ratconfig`
- Replaced copied source, copied wasm, duplicated widget code, and vendored Yazi assets with locked child-owned packages and artifacts consumed by the main runtime
- Promoted Ghostty back to the default packaged terminal and made restored Yazi image previews through Zellij the default-terminal goal; the current graphics path uses upstream Yazi, `yazelix-yazi-assets`, and the `yazelix-zellij` Kitty passthrough bridge
- Switched the package baseline to `nixpkgs-unstable` and pulled in newer Yazi/Chafa behavior that avoids the Chafa terminal-probe ghost-keypress regression
- Made `settings.jsonc` the canonical user config, backed by `settings_default.jsonc`, JSON schema coverage, strict unknown-field diagnostics, additive repair, and complete Home Manager rendering
- Upgraded `yzx config ui` into a structured JSONC settings editor with scalar pickers, keybinding rows, safer parse-error behavior, popup launch through `Alt Shift C`, and generic config UI machinery owned by `ratconfig`
- Added the directional workspace keymap: `Alt Shift H` toggles the left sidebar, `Alt Shift J` opens the bottom popup, `Alt Shift K` opens the top popup, `Alt Shift L` opens the right Codex agent sidebar, and `Alt Shift M` opens the menu popup
- Added managed focus/reveal keys: `Ctrl y` switches between editor and left sidebar, `Ctrl Shift Y` switches between editor and right sidebar, and `Alt r` smart-reveals in the editor or falls back to editor/left-sidebar focus
- Made Yazelix and native Zellij key policies data-driven through `settings.jsonc`, including remappable native defaults such as `Ctrl Alt g/s/o` for locked/scroll/session modes, `Ctrl Shift H/L` for tab movement, `Alt 1..9` for tab jumps, pane-group controls, and `Alt Shift F` for focus fullscreen
- Converged generated runtime state before launch so stale Zellij layouts, plugin permission caches, terminal configs, copied native config files, and Yazi static assets are repaired or diagnosed deterministically
- Moved status-bar and widget ownership into `yazelix-zellij-bar`, including Codex, Claude, OpenCode Go, CPU/RAM, cursor, cached facts, throttled refresh, and first-paint hydration
- Exposed standalone subsystem packages for screen rendering, Ghostty cursors, popup panes, the Zellij bar, and Yazi assets while keeping normal Yazelix installs wired automatically
- Matured public Nix customization with `mkYazelix`, overlays, runtime tool sources, component toggles, child package outputs, Home Manager integration, and Cachix publishing
- Migrated maintainer issue tracking from Go/Dolt `bd` to Rust `br`, with tracked JSONL state, ignored local SQLite cache, Nix packaging, CI initialization, and GitHub issue sync support
- Users jumping straight from early v16 should still read the v16.2 and v16.3 notes for cursor-sidecar and flat-config-path manual actions


## v16.5 - 2026-05-21

Managed JSONC settings, Codex sidebars, and strict Zellij ownership

Upgrade impact: no user action required

Highlights:
- `settings.jsonc` is the canonical Yazelix settings file, with a complete `settings_default.jsonc` template, schema coverage, strict unknown-field diagnostics, and additive repair for missing supported fields
- Home Manager renders complete managed `settings.jsonc` files while still letting users keep tiny declarative Yazelix settings in Nix
- Yazelix ships a host-installed Codex agent path with a managed right sidebar and explicit directional defaults: `Alt Shift H/J/K/L` toggle the left sidebar, bottom popup, top popup, and right Codex sidebar; `Ctrl Shift Y` switches focus between editor and right sidebar
- Sidebar launchers are configured symmetrically through `workspace.left_sidebar.*` and `workspace.right_sidebar.*`; the left sidebar defaults to `yzx sidebar yazi`, the right Codex sidebar defaults to `40%` width, and both sidebar widths accept `1` to `48`
- Writable configs that still contain the old `editor.sidebar_command`, `editor.sidebar_args`, or `editor.sidebar_width_percent` fields are repaired once into `workspace.left_sidebar.*` before strict validation
- Generated Zellij runtime state is re-rendered and validated before launch, so stale configs, missing plugin permissions, and copied native config files are detected or repaired deterministically
- Managed `~/.config/yazelix/zellij.kdl` is reserved for native Zellij settings that Yazelix does not render; Yazelix-owned keybindings, theme, pane frames, and rounded corners stay in `settings.jsonc`
- `yzx config ui` renders scalar select fields as single-choice option lists; `Space` opens the picker for selects and remains a direct toggle only for booleans
- The pane-orchestrator and popup wasm artifacts are consumed from locked first-party child packages instead of copied main-repo binaries


## v16.4 - 2026-05-09

Standalone bar/cursor ownership, structured config editing, and first-launch reliability

Upgrade impact: no user action required

Highlights:
- `yazelix-zellij-bar` owns more of its standalone widget surface so non-Yazelix Zellij users can run the bar with its bundled fact, cursor, and usage widget helpers
- The Ghostty cursor package was consistently named `yazelix-ghostty-cursors`, and that release exposed the `yzc`, `yazelix_ghostty_cursors`, and `ghostty_cursor_shaders` package outputs
- `yzx config ui` presents `zellij.keybindings` and `yazi.keybindings` as structured action rows instead of forcing one-line JSON object edits
- Complex config arrays and objects without a dedicated structured editor no longer open unreadable raw JSON edit buffers
- Desktop launches after a Home Manager or flake runtime update refresh terminal configs before opening the managed terminal, so the first launch does not depend on stale generated files
- The Rust inventory describes the current code ownership shape and future extraction opportunities without bead-specific or legacy implementation notes

## v16.3 - 2026-05-07

First-party child repos, standalone popup panes, and launch/config hardening

Upgrade impact: manual action may be required

Highlights:
- Published first-party child repository links for `yazelix-screen`, `yazelix-ghostty-cursors`, and `yazelix-zellij-popup` so the extracted projects are easier to find and use independently
- Extracted the reusable Zellij popup pane plugin into `yazelix-zellij-popup`, with `yzpp.wasm` usable from plain Zellij layouts while Yazelix keeps its integrated `yzx popup` command
- Desktop launch keeps its starter terminal for visible preflight and failure feedback, but defers the managed Yazelix window until after the starter exits so tiling window managers do not size the real session around the bootstrap window
- The default Zellij status widget tray includes `codex_usage` so new configs show the Codex quota/reset widget without manual opt-in
- Home Manager desktop-entry generation is Linux-only so Darwin Home Manager evaluations do not require Linux `xdg.desktopEntries` options
- User-editable config surfaces are flat under `~/.config/yazelix/`, with unambiguous old regular files migrated into the flat names on first use

Manual action:
- If both the old nested config path and the new flat path exist with different contents, keep the new flat path or move the old file aside before launching
- If the old nested path is a Home Manager or Nix symlink, move that ownership to the flat path instead of relying on automatic migration


## v16.2 - 2026-05-02

Home Manager leaves yazelix.toml mutable, cursor presets move to a sidecar, and status widgets use cached facts

Upgrade impact: manual action required

Highlights:
- Changed the Home Manager module default so it installs the Yazelix package/runtime/desktop integration while leaving `~/.config/yazelix/user_configs/yazelix.toml` as a normal mutable file
- Moved Ghostty cursor selection, effects, duration, glow, Kitty fallback, and cursor preset definitions out of `yazelix.toml` and into `~/.config/yazelix/user_configs/yazelix_ghostty_cursors.toml`
- Simplified Ghostty cursor definitions to `mono` and `split` families with `divider`/`transition` split controls, renamed the black/red horizontal preset from inferno to magma, intensified reef's green, and added the `yzx cursors` inspection command for resolved colors
- Replaced `editor.initial_sidebar_state` with `editor.hide_sidebar_on_file_open` so tabs start consistently and file-open behavior owns sidebar hiding
- Restored workspace and agent-usage status-bar widgets through window-local cached facts instead of direct zjstatus pane-orchestrator or provider polling
- Kept `programs.yazelix.manage_config = true` as the explicit opt-in for users who want Home Manager to generate and own `yazelix.toml`

Manual action:
- If your `user_configs/yazelix.toml` still contains `terminal.ghostty_*` cursor fields, move those cursor choices and effects into `user_configs/yazelix_ghostty_cursors.toml` and remove the old fields
- If your `user_configs/yazelix_ghostty_cursors.toml` uses `family = "simple_dual"` or `family = "axis_gradient"`, update those cursor definitions to `family = "mono"` or `family = "split"` and replace split `direction`/`blend` with `divider`/`transition`
- If your `user_configs/yazelix.toml` uses `editor.initial_sidebar_state`, remove it and set `editor.hide_sidebar_on_file_open = true` when you want the Yazi sidebar hidden after opening a file


## v16.1 - 2026-04-25

v16.1 stabilizes Home Manager installs and screen rendering

Upgrade impact: no user action required

Highlights:
- Fixed the welcome and `yzx screen` Game of Life renderers so gliders keep their intended shape without terminal row-gap artifacts, and added the configurable `core.game_of_life_cell_style` option with `full_block` and `dotted` styles
- Added `programs.yazelix.manage_config` for Home Manager users who want Home Manager to own the package/runtime/desktop integration while leaving `~/.config/yazelix/user_configs/yazelix.toml` as a normal mutable file
- Made Home Manager local-input updates faster and clearer by warning about `path:` snapshot semantics and filtering package/runtime source trees away from local build artifacts
- Fixed empty Zellij status widgets by removing missing dynamic identity helper commands and rendering stable editor, shell, and terminal labels directly in the generated status payload


## v16 - 2026-04-24

v16 Rust-forward control plane with an irreducible Nushell core

Upgrade impact: no user action required

Highlights:
- Finished the Rust owner cuts across the remaining deterministic control-plane and editor/Yazi integration surfaces, so the public `yzx` story is now much more clearly Rust-owned
- Reduced Nushell to the explicit shell and UI core, documented the surviving floor, and kept popup/menu wrappers on Nushell where that boundary is the clearest fit
- Moved maintainer, update, and sweep ownership further out of Nushell, including repo-maintainer flows and pane-orchestrator sync semantics, so the remaining Nu surface is much smaller and more intentional
- Unified the human CLI rendering for `yzx status`, `yzx status --versions`, and `yzx keys` around one shared Rust styling layer with cleaner grouped output and better contrast


## v15.4 - 2026-04-21

v15.4 Rust-owns public yzx families and deletes bridge seams

Upgrade impact: no user action required

Highlights:
- Moved more public `yzx` command families onto Rust-owned execution paths, including `yzx config`, `yzx home_manager`, `yzx why`, `yzx sponsor`, `yzx keys`, `yzx doctor`, `yzx cwd`, and `yzx reveal`
- Collapsed more transitional Nushell bridge owners: the extern bridge, preflight bridge, runtime-materialization bridge, doctor report cluster, install-ownership bridge, and the surviving Yazi/Zellij compatibility wrappers are gone or demoted
- Centralized public command metadata and route planning around the Rust shared command-surface schema so help, menu, extern, and routing behavior stay aligned while duplicated hand-written tables shrink
- Updated the Rust migration and spec inventories around the real remaining seams, with the next planning track focused on canonical contracts, stronger test traceability, and ranked delete-first cleanup


## v15.3 - 2026-04-21

v15.3 Rust-owns more of the core and starts much faster

Upgrade impact: no user action required

Highlights:
- Moved more of the typed core into Rust: `yzx` root metadata, `yzx env`, `yzx run`, `yzx update*`, `yzx status`, doctor findings, startup preflight, runtime-env planning, and runtime-materialization planning now route through `yzx_core` or `yzx_control`
- Rust now owns Yazi, Zellij, terminal, Helix, and runtime-materialization generation or write lifecycles, while the old Nushell wrapper owners were deleted or collapsed
- Startup got much faster on the same maintainer machine: 75.6% faster warm current-terminal, 95.6% faster cold clear-cache, 55.6% faster desktop launch, and 59.0% faster managed new-window launch
- Rewrote the Rust migration inventory around the real remaining Nushell seams, so follow-up work now targets bridge collapse and honest shell-bound survivors instead of stale transition docs


## v15.2 - 2026-04-19

v15.2 hardens startup, desktop launch, and Ghostty polish

Upgrade impact: no user action required

Highlights:
- Upgraded `yzx menu` to a prettier `fzf`-backed command palette and fixed the popup selection crash path
- Rerolled Ghostty random cursor palettes and effects for each Yazelix Ghostty window, including desktop fast-path launches, while keeping fixed palettes stable
- Made managed desktop entries terminal-backed and surfaced desktop-launch pre-terminal progress and failures so desktop entry clicks no longer fail invisibly before terminal handoff
- Moved config parsing and generated-state hashing onto the packaged Rust `yzx_core` helper, deleted the old Nushell fallback and legacy raw-string rebuild cache path, and kept malformed cache state on the safe one-time-refresh path


## v15.1 - 2026-04-15

v15.1 hardens install ownership, popup env, and Home Manager packaging

Upgrade impact: no user action required

Highlights:
- Stopped packaged and one-off runtime entrypoints from rewriting host shell dotfiles, kept runtime setup inside `~/.local/share/yazelix`, and narrowed `yzx status` back to runtime/config inspection with clearer stale-shell recovery guidance
- Added the explicit `editor` token for `zellij.popup_program`, propagated the canonical runtime env into popup flows, and set `VISUAL` alongside `EDITOR` so popup editors and TUI tools reuse the configured Yazelix editor contract
- Narrowed the packaged public `bin/` surface to `yzx`, moved bundled runtime tools under `libexec/`, and kept packaged and Home Manager installs away from binary-collision traps while still shipping the full runtime toolset
- Hardened Linux Home Manager and desktop-launch reliability by passing the runtime-owned `nixGL` wrapper through the module build, improving Ghostty launch diagnostics, and documenting a minimal flake example plus clearer update-owner recovery
- Replaced the remaining popup and workflow examples that still referenced Claude with Codex examples


## v15 - 2026-04-13

v15 trims Yazelix down to the fast workspace core

Upgrade impact: manual action required

Highlights:
- v15 is the only supported Yazelix line now, and v14 is the final historical Classic snapshot rather than a maintained fallback.
- Dropped the out-of-scope Classic runtime-manager surface: no runtime-local `devenv`, no `yazelix_packs.toml`, no `yazelix packs` or `yzx packs`, no automatic config migrations, and no `yzx refresh`.
- Ghostty is now the first-party bundled terminal on Linux and macOS, while WezTerm, Kitty, Alacritty, and Foot remain supported when you provide them on the host `PATH`.
- Split current-terminal startup into `yzx enter`, kept `yzx launch` as the managed external-terminal entrypoint, and kept `yzx env` as the non-UI tool-environment surface.
- `yzx popup` and `yzx menu --popup` now share the fast floating-pane path with explicit pane identity, shared toggle semantics, and no helper-pane detour.
- Kept the workspace core around layouts, managed editor/sidebar orchestration, `yzx cwd`, `yzx reveal`, `yzx doctor`, `yzx whats_new`, and explicit update owners through `yzx update upstream` or `yzx update home_manager`.
- Continued the delete-first trim by replacing string-built runtime wrapper commands with direct runtime scripts, making maintainer pins explicit again, and keeping the runtime lock on the declared unstable input.

Command surface:
- `yzx launch`: open Yazelix in a managed terminal window.
- `yzx enter`: start Yazelix directly in the current terminal.
- `yzx env`: enter the Yazelix tool environment without the UI.
- `yzx popup`: toggle the configured popup program, usually `lazygit`.
- `yzx menu --popup`: toggle the popup command palette.

Migration notes:
- Compare your current config with `yazelix_default.toml` or run `yzx reset config` to start fresh; v15 does not ship automatic config migrations.
- If you relied on Classic-only surfaces such as `yazelix packs`, `yzx packs`, or `yzx refresh`, stay on the historical `v14` tag or adapt to the trimmed v15 command surface.


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

The `v14` tag remains available only as the final historical Classic snapshot for users who specifically need that broader product shape. It is no longer a supported line.

The active branch direction is now v15 rather than two maintained products in parallel.

v15.0 is the trimmed reboot with a narrower Rust scope, not a Rust-free release. The goal is a smaller, slimmer, faster, more opinionated Yazelix with a much clearer product boundary. In practice, that means dropping the old runtime-local `devenv` layer, stopping the project from also trying to be a broad package-and-environment manager, trimming the command and config surface, deleting the config-migration engine, focusing on fast workspace entry, and keeping the Rust pane orchestrator where it still owns live workspace and session state.

That is the main architectural lesson of v14: Yazelix had clearly become two products in one.

One product was the broad environment-management system: rebuild and refresh semantics, package and pack ownership, shell and terminal breadth, multiple install and update ownership modes, launch-profile state, and a large amount of dynamic runtime machinery.

The other product was the narrower workspace tool that had been trying to emerge inside it: fast entry into a built runtime, explicit ownership, predictable workspace behavior, stronger editor/sidebar orchestration, and a smaller core.

v14 is the release where that split became impossible to ignore. The v15 branch now resolves it by trimming first instead of trying to preserve both product shapes.

A lot of the current `yzx` surface belongs to Yazelix Classic. That includes the parts of `yzx` that are tightly tied to the older `devenv` hot-path and cold-path model: explicit refresh semantics, dynamic runtime entry behavior, launch-profile reuse, first-class `yazelix packs` / `yzx packs` package selection and inspection, broad pack and package-graph ownership, wider shell and terminal policy, and the idea that Yazelix should also act as a fairly general environment-management layer.

In that sense, commands like `yzx refresh` and much of the older meaning carried by `yzx run` belong much more to the v14 Classic world than to the slimmer v15 direction.

The v15 branch keeps the core `yzx` product surface and trims away the parts that mainly existed to support the older `devenv` machinery. The backbone is `yzx launch`, `yzx env`, `yzx update`, and `yzx desktop`. Beyond that backbone, workspace-facing commands such as `yzx cwd`, `yzx reveal`, `yzx popup`, `yzx menu`, `yzx keys`, `yzx tutor`, `yzx whats_new`, and `yzx doctor` fit the actual product much better than the older backend-management surface does.

Commands and semantics that mainly existed because Yazelix was also trying to manage a large dynamic `devenv` lifecycle are now historical Classic surfaces. That is why v15 drops or heavily narrows `yzx refresh`, `yzx run`, launch-profile reuse semantics, explicit backend/materialization entry logic on the hot path, the broader `yazelix packs` / `yzx packs` package-graph ownership model, and automatic config migrations.

There is still a chance that a broader runtime or terminal-environment project could be forked from Yazelix Classic later. That would let the broader environment-management direction evolve on its own terms instead of staying entangled with the slimmer v15 product.

If that separate project proves valuable, it should only be reintegrated with much cleaner boundaries: separate codebases, clear separation of concerns, and an explicit integration seam between the two products.

Rust remains a later implementation path. Selective Rust can land in v15.x point releases when it clearly pays for itself, while v16 is the Rust-forward release target.

v14 may also be the last heavily Nushell-based Classic snapshot. It remains useful as a substantial real-world Nushell codebase for people who want to study that older broader product shape.

## v13.13 - 2026-04-05

Yazi git refresh hardening, fresh launch-profile recording, and maintainer input updates.

Upgrade impact: no user action required

Highlights:
- Refreshed Yazi sidebar git decorations more reliably on focus return, open, navigation, and explicit sidebar refresh so the managed sidebar stops carrying stale git state.
- Recorded the fresh built launch profile after Yazelix-owned rebuilds so desktop-entry and restart flows stop reactivating stale `DEVENV_PROFILE` paths.
- Updated maintainer inputs, including the then-current Beads tracker build, and verified the real issue-mutation path after the bump.

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
- Simplified config-facing commands by splitting downstream inspection into `yzx open hx|yazi|zellij`, managed config editing into `yzx edit config|packs`, and adding `yzx reset config --no-backup`, while polishing welcome-screen skip behavior.

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
