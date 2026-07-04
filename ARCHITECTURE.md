# Architecture

Yazelix Next is a small Nix/Lix flake with one front-door command: `yzn`.
`yzn help` prints help, `yzn config` opens the Ratconfig UI, `yzn status`
summarizes owned runtime/config state, `yzn doctor` checks owned startup setup,
`yzn sponsor` opens or prints the Sponsors URL, `yzn enter` starts Yazelix
inside the current terminal, `yzn launch` opens Mars first, and `yzn menu`
opens the curated live-filter command palette. `yzn tutor` prints guided
workspace, discovery, recovery, and tool-tutor lessons. `yzn screen [style]` shows a
Yazelix terminal screen, and `yzn reveal <target>` reveals a path in the
managed Yazi sidebar. Bare `yzn` defaults to `yzn launch`.
The runtime paths are intentionally narrow:

```text
yzn launch -> Mars -> yzn-welcome -> Yazelix Zellij fork -> Yazi sidebar + stacked work panes
yzn enter -> yzn-welcome -> Yazelix Zellij fork -> Yazi sidebar + stacked work panes
```

The repo owns the glue that makes those pieces behave like one runtime. It does
not try to be a general terminal distribution, broad Home Manager runtime
config module, or Yazelix compatibility layer.

The flake exposes package and app outputs for Linux and Darwin on x86_64 and
aarch64. The macOS contract is package-first: command surfaces such as `help`,
`status`, `doctor`, and terminal-local `enter` are the supported floor, while
Mars-backed `launch` is issue-driven best-effort until trusted macOS hardware
validation proves it. App bundles, Homebrew, Ghostty packaging, and broad
terminal matrices stay outside this repo.

## Owners

`flake.nix` is the package graph and composition owner. It pins external inputs,
builds small local Rust helpers, substitutes local config templates, installs
the desktop entry, exposes the `yzn` package/app, and exports the narrow Home
Manager module.

`home-manager/module.nix` is the declarative install and config-file ownership
owner. It exposes `programs.yazelix.enable` and `programs.yazelix.package`,
installs the selected package into `home.packages`, relies on that package for
the desktop entry, can render root Yazelix settings to `config.toml`, and can
install native Mars, Zellij, Starship, Helix, Yazi, and Nu files from text or
source paths. It does not generate runtime config files by default or
semantically model native config formats.

`crates/yzn-tutor/` is the tutor owner. It parses the small `yzn tutor`
argument surface, renders bundled lessons through a strict terminal Markdown
subset, and prints the packaged Helix and Nushell tutor commands without
launching either tool.

`crates/yzn-config/` is the config host owner. It opens the Ratconfig UI,
creates `~/.config/yazelix-next/config.toml` with defaults and joined contract
state when missing, creates simple managed Mars, Zellij, and Starship config
files when missing, routes source-backed edits to the correct file, exposes
Helix rows for managed Helix native files, creates the Steel pair together,
exposes Advanced rows for native Nu, managed Yazi sidecar files, and the
managed Zellij plugin sidecar, lists read-only packaged bindings as a Keys
table, and exposes one hidden
package-internal read path used by launch wrappers plus hidden custom-popup KDL
render paths used by the runtime materializer. Its `KEY_BINDINGS` catalog is
the human-facing key reference owner for Ratconfig and future discovery
surfaces; `config.kdl` remains the runtime binding owner, with flake checks
guarding the catalog-to-runtime link and the tutor-to-catalog key hints.
Inside the crate, `main.rs` owns CLI dispatch and the test harness, while
focused modules own common helpers, paths, root config, custom popups, native
Mars/Starship fields, Zellij sidecars, file actions, model construction, and UI
terminal handling.
The contracted root config fields are `open.log_level`, which controls
`YZN_OPEN_LOG` for managed Yazi-to-Helix opens, `shell.program`, which selects
the packaged shell for new Zellij panes, `appearance.mode`, which selects the
packaged dark/light Mars config and child-rendered Zellij bar palette,
`welcome.enabled`, `welcome.style`, and `welcome.duration_seconds`, which
control the startup splash,
`popup.side_margin` and `popup.vertical_margin`, which set the `yzpp` default
left/right and top/bottom cell margins for generated managed popups,
`keybindings.config`/`agent`/`git`/`menu`, which remap managed popup
triggers, semantic `[popups.<id>]` specs with required keybindings for managed
custom popups, and `bar.widgets`, which selects the
top-bar tray. The Mars, Zellij, and Starship tabs are render/edit surfaces
without contracts or migrations. The Starship tab
edits `format`, `right_format`, and `add_newline`, with `format` defaulting to
colon-colon-space (`:: `). The Helix tab is an open-file surface for `helix/config.toml`,
`helix/languages.toml`, `helix/helix.scm`, and `helix/init.scm`. The Advanced
tab is an open-file surface for Nu and managed Yazi files. Ratconfig renders
rows and emits file-open intents, while Yazelix Next owns path selection,
missing-file creation, and editor launch.

`yazelix-screen` is the terminal screen owner. Its `yzs` package owns screen
style parsing, the static Yazelix welcome card, timed playback, animation
rendering, and the fixed random pool that excludes `static`. Yazelix Next
packages that child command as `yzn screen` and keeps only the semantic welcome
settings plus the pre-Zellij `yzn-welcome` handoff wrapper. It does not expose
a configurable pool.

`mars.toml` is the packaged terminal visual config owner. It sets the default
Mars window, font, cursor, bell, quit, and theme behavior used by `yzn`.
`flake.nix` renders dark and light packaged Mars config roots from that
template; light colors follow main Yazelix's Mars light theme. A user can
replace the Mars config with `~/.config/yazelix-next/mars/config.toml`; an
untouched generated copy of the packaged dark scaffold does not count as a user
override. `yzn` still owns the launch command and runtime environment.

`config.kdl`, `layout.kdl`, and `layout.swap.kdl` are the Zellij behavior
owners. They point `default_shell` at the packaged shell dispatcher, set
Zellij-native `Ctrl Alt` mode keys, direct `Ctrl Alt h/j/k/l` movement, the
Tab-mode new-tab layout binding, the `Alt m` pane binding, the `Alt Shift h`
pane-orchestrator sidebar toggle, the default `Alt Shift J/K/L/M` Git, config,
agent, and menu popup bindings rendered from semantic popup role keybindings,
generated custom popup specs and keybindings rendered from `[popups.<id>]`,
`Alt h/l` horizontal focus
walking, `Alt 1-9` tab jumps, `Alt r` smart reveal, the Yazelix Zellij Bar top bar, the Yazi
sidebar tab, the open/closed sidebar swap layouts, and explicit Kitty keyboard
protocol support.
The standalone `yzpp` plugin owns popup lifecycle, and the standalone pane
orchestrator owns horizontal focus policy. Yazelix Next only packages them and
binds `Alt h/l` to the orchestrator's visible-pane walker.

`yazelix-zellij-bar` owns the rendered top bar KDL, widget command logic, and
`zjstatus.wasm`. Yazelix Next owns only the selected semantic tray at
`[bar].widgets`, uses the packaged pre-rendered layout for the default tray,
passes `[appearance].mode` to the renderer, and materializes a runtime layout
through the pinned renderer when light mode or a custom tray needs a
non-default bar. The runtime config points managed new tabs at that active
layout and the user's home cwd. It ships `tu` for usage widget
refreshes, exports a yzn-owned status cache path, names the initial and
tab-mode-created tabs with the Yazelix home marker, home-scopes
tab-mode-created tabs, and keeps the native bottom Zellij `status-bar` for key
hints.

`runtime/yzn-zellij-config.rs` is the launch-time guarded Zellij sidecar owner.
It appends `~/.config/yazelix-next/zellij/config.kdl` to the packaged config
after a small first-token denylist rejects obvious attempts to take over the
managed shell, keymap, layout, plugin loading, Kitty keyboard protocol,
environment, or session startup behavior. `runtime/yzn/zellij.rs` owns the
separate `zellij/plugins.kdl` plugin-only sidecar; it accepts only `plugins`
and `load_plugins` blocks and rejects Yazelix-owned plugin ids before
injecting entries into the managed config. `crates/yzn-config/` owns the config
UI renderer/parser for the small exposed scalar sidecar subset and the Advanced
file action for the plugin sidecar.

`nu/` is the packaged Nushell config owner. It initializes carapace, zoxide,
and Starship left and right prompts, and disables the normal Nushell banner and
prompt indicators.

`yazi/` is the packaged Yazi config owner. It enables the selected Yazi
plugins, keeps file opens routed through `yzn-open`, binds `Alt z` to a zoxide
picker that moves the sidebar and opens the chosen directory in the configured
editor command, publishes the sidebar Yazi id for tab-local reveal, and avoids
broad Yazi config merging.

`runtime/yzn-yazi.rs` is the managed Yazi launcher owner. It sets scoped image
preview/session environment, appends optional managed user `yazi/init.lua` and
`yazi/keymap.toml`, overlays managed user `plugins/*.yazi` directories when
that init exists, resolves `editor.command` for managed opens, and then execs
packaged Yazi.

`runtime/yzn-nu.rs` is the Nushell runtime-config owner. It writes the runtime
`env.nu` and `config.nu` files, inserts host `mise activate nu` output when
`mise` is available, layers optional user config from `~/.config/yazelix-next/nu`,
chooses the Starship config path, and then execs Nushell.

The flake owns the `yzn-shell` dispatcher. It reads `shell.program` through
`yzn-config` and execs packaged `nu`, `bash`, `zsh`, or `fish`. The `nu` path
delegates to `runtime/yzn-nu.rs`; other shells are intentionally plain packaged
interactive shells with their normal startup-file behavior. Unsupported shell
values fail in `yzn-config` before dispatch, so shell schema policy stays in the
config owner rather than the shell wrapper.

`crates/yzn-open/` is the editor-open and reveal owner. For bridge-enabled
`yzn-hx`, it sends file and directory open requests to the live Yazelix Helix
bridge when available, and otherwise opens a managed `editor` pane through
Zellij. Host-owned commands such as `hx` or `nvim` bypass the bridge and are
run as executables from `PATH`. Directory opens use the workspace root for
editor cwd and tab naming while keeping the selected directory as the Helix
picker directory. `yzn-reveal` asks the pane
orchestrator for the active tab's registered sidebar Yazi id, emits Yazi
`reveal`, and focuses the sidebar. It also owns bounded diagnostics for
Yazi-to-Helix opens.

`checks/` owns build-time contract guards. `zellij-layout.rs` validates Zellij
layout swaps, and `yzn-contracts.rs` validates the packaged runtime contracts.

## Module And Repo Boundaries

Yazelix Next uses the smallest boundary that keeps one clear owner. A local
module is the default split for large implementation code when the behavior
still releases atomically with `yzn` and has no independent user. A local crate
is for code that needs Cargo packaging, separate binary ownership, or focused
Rust checks while remaining product glue.

A separate repo needs more than a large file. It needs independent users, an
independent release cadence, a stable artifact or API contract, low coupling to
`yzn` runtime paths/config/session state, low duplicate-owner risk, tolerable
release friction, and a local test path that proves both the child artifact and
the integrated runtime. Moving code out only counts as extraction when this repo
deletes or relinquishes a real owner.

Two thresholds apply. Big extraction needs independent users, a stable API or
artifact contract, a release cadence, and real deletion of an owner from this
repo. Edge trimming can be much smaller: it only needs a child-owned concept
that is already generic, already parsed or validated by the child, and can
delete repeated local glue without creating an adapter that exists only to
translate between duplicate owners.

Child repos keep reusable behavior: Mars, the Yazelix Zellij and Helix forks,
Ratconfig, `yazelix-screen`, `yazelix-zellij-bar`,
`yazelix-zellij-popup`, and the pane orchestrator own their standalone domains
and artifacts. Yazelix Next keeps adapters and product policy:
`runtime/yzn/`, `crates/yzn-config`, `crates/yzn-open`, local runtime
helpers, contract checks, path selection, semantic config, launch wiring, and
generated integration KDL stay local until they meet the separate-repo criteria.

## Config Layering

Packaged config comes first unless a surface explicitly opts into native
replacement. User config is narrow and explicit:

```text
~/.config/yazelix-next/config.toml
~/.config/yazelix-next/mars/config.toml
~/.config/yazelix-next/zellij/config.kdl
~/.config/yazelix-next/zellij/plugins.kdl
~/.config/yazelix-next/starship.toml
~/.config/yazelix-next/nu/env.nu
~/.config/yazelix-next/nu/config.nu
```

`YAZELIX_NEXT_CONFIG_HOME` can point at another config root. `config.toml` is
the Yazelix-owned semantic config file and is created by `yzn config` or the
package-internal config read path when missing. It controls bounded runtime
settings such as open diagnostics, the packaged shell choice, managed popup
size, managed popup role keybindings, semantic custom popup specs, startup
welcome style/timing, and the ordered bar widget tray. `yzn config` also creates the managed Mars, Zellij,
and Starship native files when missing.
Mars uses full native replacement when its `config.toml` exists. Nushell uses
packaged config first, then optional user `env.nu` and `config.nu`. For managed
Nu, Starship uses the user `starship.toml` when present, otherwise an empty config
that preserves Starship defaults. `yzn config` exposes Starship as a structured
tab and exposes the Nu files through the Advanced tab. `yzn config` exposes
Helix TOML and Steel files through the Helix tab and creates them only after
row activation. `yzn-hx` uses the packaged Helix config dir by default; when
`config.toml`, `languages.toml`, or the Steel pair exists under
`~/.config/yazelix-next/helix`, it uses that directory for Helix native config,
points `HELIX_STEEL_CONFIG` there only when the Steel pair exists, points Steel
at an internal state fallback for TOML-only managed config, and keeps the
packaged `config.toml` as TOML fallback if no user `helix/config.toml` exists.
Without user Steel files, `yzn-hx` points Steel at a packaged module that
exposes `:yzn-new-shell` for opening a new Yazelix terminal pane at the current
Helix file directory or workspace. The packaged Helix TOML binds `Alt r` to
Yazi reveal and `Ctrl r` to reload Helix config plus the current buffer.
Normal Nushell, Starship, and Helix config files are not loaded by default,
which keeps the default managed paths reproducible and avoids ambient user shell
behavior changing that runtime. Zellij uses packaged config first, then a
guarded sidecar for safe
native preferences. The sidecar is a guardrail rather than a KDL parser: it
rejects uncommented lines whose first token is known to own integration-critical
behavior such as `keybinds`, `default_shell`, layout, plugins, Kitty keyboard
protocol, environment, or session startup.

## Runtime Startup Boundary

`yzn enter` and `yzn launch` are owned by the Rust front door in
`runtime/yzn/`. Nix substitutes store paths into `main.rs` and compiles the
module directory as one binary, but Rust owns the startup wiring and final
`exec` handoff:

- establish XDG-data-backed `YAZELIX_STATE_DIR` and, for `yzn-hx`, the
  top-level Helix bridge session id
- set managed `EDITOR`, `VISUAL`, `YZN_EDITOR`, `YAZELIX_NEXT_EDITOR`, and
  `GIT_EDITOR` from semantic `editor.command`
- resolve the Yazelix config home from `YAZELIX_NEXT_CONFIG_HOME`,
  `XDG_CONFIG_HOME`, or `HOME`
- read `open.log_level` through `yzn-config` and export `YZN_OPEN_LOG`
- read `welcome.enabled`, `welcome.style`, and `welcome.duration_seconds` and
  export them for the pre-Zellij welcome wrapper
- read `keybindings.config`/`agent`/`git`/`menu` and render changed popup
  role chords into the managed Zellij config
- read semantic `[popups.<id>]` specs and render custom `yzpp` popup blocks and
  managed keybindings into the active Zellij config
- select packaged Mars config or the managed user Mars config
- run `yzn-zellij-config` to pick or materialize the active Zellij config
- export the Yazelix Zellij Bar cache path
- seed Zellij plugin permissions for the popup plugin, pane orchestrator, and
  top bar

If a Yazelix-owned pre-exec setup step fails, the front door prints a concise
startup diagnostic with the reason and, when applicable, a concrete config path
to check. It does not catch errors after `exec`.

`yzn status` and `yzn doctor` reuse the same startup boundary to validate
owned inputs without launching Mars or Zellij. `status` renders a compact text
summary; `doctor` renders ok/warn/fail rows for the owned startup surface and
warns when a managed Helix TOML override omits the packaged reveal command.

After this boundary, Yazelix gives control to the target process. `yzn enter`
execs `yzn-welcome` with the Yazelix Zellij fork as its child. `yzn launch`
execs Mars with `yzn-welcome` and then Zellij as the child command. Startup
failures before this handoff are Yazelix-owned; failures after the handoff
belong to Mars, Zellij, or the child tool.

## Session Isolation

Each top-level `yzn` launch with the bridge-enabled `yzn-hx` editor creates one
opaque `YAZELIX_HELIX_BRIDGE_SESSION_ID`. Zellij, Yazi, Helix, and `yzn-open`
inherit that id inside the window, so Yazi opens can only reuse Helix bridge
registries from the same session. Non-Helix configured editors do not receive
the Helix bridge environment, so `yzn-open` treats them as host-owned editor
commands. `yzn-open` also compares `ZELLIJ_SESSION_NAME` when a registry records
it, which prevents two Zellij windows with copied session state from sharing an
editor pane. Managed Yazi saves that real session in
`YAZELIX_ZELLIJ_SESSION_NAME` before blanking `ZELLIJ_SESSION_NAME` for image
previews, and `yzn-open` restores the saved name for Zellij control commands.
`yzn-reveal` uses the same saved session path for sidebar focus commands. For
tab isolation, `yzn-open` reads the invoking Yazi pane id from `ZELLIJ_PANE_ID`,
asks Zellij for live pane membership, and reuses only a Helix registry whose
recorded pane id is in the same stable `tab_id`.

Helpers launched outside `yzn` do not fall back to the shared literal `yzn`
session id. `yzn-hx`, `yzn-yazi`, and `yzn-open` derive isolated helper ids, so
standalone helper use cannot accidentally target a bridge from a live `yzn`
window.

## Runtime Contracts

| ID | Contract | Owner | Check | Missing Coverage |
| --- | --- | --- | --- | --- |
| C1 | `yzn` defaults to `yzn launch`, `yzn help` prints help, `yzn config` opens Ratconfig config, `yzn menu` opens a curated live-filter command palette for `config`, `doctor`, `status`, `screen`, `sponsor`, `launch`, `help`, and `tutor`, `yzn tutor` prints guided Yazelix lessons and packaged native tutor commands, `yzn screen [style]` shows a Yazelix terminal screen, `yzn reveal <target>` reveals a path in the active tab's managed Yazi sidebar, `yzn status` prints a runtime/config summary, `yzn doctor` checks owned startup setup and warns about Helix overrides missing the packaged reveal command, `yzn sponsor` opens or prints the sponsor URL, `yzn enter` starts managed Zellij in the current terminal, and `yzn launch` starts Mars first; pre-exec setup failures print a readable Yazelix diagnostic | `flake.nix`, `runtime/yzn/`, `runtime/yzn-menu.rs`, `crates/yzn-tutor/`, `crates/yzn-config/`, `crates/yzn-open/`, `yazelix-screen` | `checks/yzn-contracts.rs` validates help, menu allowlist, packaged fzf wiring, menu dispatch, tutor output, screen help output, front-door wiring, runtime setup wiring, status/doctor/sponsor behavior, Helix override warnings, reveal helper packaging, and representative startup diagnostics; `checks/key-reference-parity.rs` validates tutor key hints against the `KEY_BINDINGS` owner; `crates/yzn-tutor` unit tests cover parsing, Markdown rendering, lesson coverage, and native tutor command printing; `cargo test` covers reveal command routing; `nix build .#yzn` packages the runtime | GUI launch remains manual dogfooding |
| C2 | Mars uses the packaged dark or light visual config from `appearance.mode` unless a changed user native Mars config exists | `mars.toml`, `flake.nix`, `runtime/yzn/` | `checks/yzn-contracts.rs` validates packaged dark/light config availability, generated Mars scaffolding, and launcher selection | Visual correctness remains manual dogfooding |
| C3 | Zellij layout has the sidebar template required by swaps | `layout.kdl`, `layout.swap.kdl` | `checks/zellij-layout.rs` runs during build | None for the current template/swap contract |
| C4 | Zellij-native mode keys use `Ctrl Alt`, Tab-mode new tabs use the packaged Yazelix sidebar layout, move mode is unbound, `Alt m` opens a pane for the swap layout to stack, `Alt 1-9` jumps directly to tabs 1-9, `Alt Shift h` toggles the managed sidebar through the pane orchestrator, `Alt h/l` walk visible panes through the pane orchestrator, `Alt r` smart-reveals through the editor or sidebar focus path, and obvious sidecar ownership lines are rejected | `config.kdl`, `runtime/yzn-zellij-config.rs` | `checks/yzn-contracts.rs` validates the packaged config, pane-orchestrator binding, and accepted/rejected sidecars | Full key behavior remains manual dogfooding |
| C5 | When `shell.program` is `nu`, Nushell loads packaged config first, optional host `mise activate nu` output when `mise` is available, optional user config after it, and controlled Starship left/right prompt config | `runtime/yzn-nu.rs`, `nu/` | `checks/yzn-contracts.rs` validates Nushell layering through the managed shell dispatcher, host `mise` detection, Starship config selection, and right prompt rendering | None for current layering behavior |
| C6 | Yazi launches with scoped image-preview environment, optionally appends managed user `yazi/init.lua` and `yazi/keymap.toml`, overlays managed user `plugins/*.yazi` directories when init exists, opens paths through `yzn-open` with bounded diagnostics, resolves `editor.command` for managed opens, registers the tab-local sidebar Yazi id for reveal, and `Alt z` jumps through zoxide into the configured editor command while using the workspace root for editor cwd and tab naming | `yazi/`, `runtime/yzn-yazi.rs`, `crates/yzn-open/` | `checks/yzn-contracts.rs` validates packaged Yazi keymap/plugin wiring, sidebar-state plugin wiring, opener/env ownership, and launcher environment; `yzn_yazi_materialization` covers init/keymap/plugin materialization; `cargo test` covers `yzn-open` bridge/fallback, host-owned bridge bypass, missing editor command diagnostics, reveal routing, and tab rename behavior | Full Yazi UI behavior remains manual dogfooding |
| C7 | Helix bridge reuse stays inside the current `yzn` window and current Zellij tab | `crates/yzn-open/`, `flake.nix` | `yzn-open` Rust tests cover session, Zellij-window, and Zellij-tab mismatch | Full multi-window GUI behavior remains manual dogfooding |
| C8 | Desktop entry starts `yzn` | `flake.nix` | `nix build .#yzn` packages the desktop file | Desktop environment launch remains manual dogfooding |
| C9 | Kitty keyboard protocol is explicitly enabled, `Alt Shift J/K/L/M` toggle Git, config, agent, and menu popups through `yzpp`; the Git popup defaults to LazyGit, starts through a managed wrapper that exports the configured Yazelix editor to Git editor environment, and closes on toggle so the next open follows the current tab cwd after workspace retargeting; `keybindings.config`/`agent`/`git`/`menu` default those popup role triggers and render changed chords without changing popup payloads; semantic `[popups.<id>]` specs render custom `yzpp` popup blocks and required managed keybindings without raw Zellij `keybinds`/`plugins`, using unique titles for pane identity; the agent popup bootstraps once from `codex resume`, `grok`, `opencode`, `pi`, then `claude --resume`, leaving the first-run pane empty when no provider is available; popup margins default to side `1` and vertical `0` and are written once as `yzpp` popup defaults that built-in and custom popup specs inherit; popup close/hide hooks refresh the managed Yazi sidebar git decorations | `config.kdl`, `runtime/yzn/`, `runtime/yzn-agent.rs`, `crates/yzn-config/`, `crates/yzn-open/`, `flake.nix` | `checks/yzn-contracts.rs` validates Kitty protocol, the packaged popup plugin, commands, popup ids, payloads, default and overridden popup role key bindings, managed custom popup rendering and keybindings, managed Git popup editor env wiring, agent hide behavior, Git close behavior, popup default geometry and refresh hooks, empty no-provider output, provider order, configured-missing diagnostics, and persisted-provider behavior | Visual popup behavior remains manual dogfooding |
| C10 | Top bars use the child-rendered Yazelix Zellij Bar tray, tabs use the home marker, tab-mode-created tabs open in home, light mode renders the bar with the child light palette, Codex usage has bundled `tu` and a yzn-owned cache path, and bottom bars keep native Zellij key hints | `layout.kdl`, `config.kdl`, `runtime/yzn/`, `flake.nix`, `packaging/tokenusage.nix` | `checks/zellij-layout.rs` validates packaged child bar usage, no-mode formatting, declared yzn widgets, the startup home tab marker, the home-scoped new-tab template, and native bottom status bars; `checks/yzn-contracts.rs` validates the tab-mode new-tab marker, runtime home cwd, terminal-label wiring, bundled tokenusage path, light-mode runtime bar rendering, and status-cache export | Visual bar behavior remains manual dogfooding |
| C11 | `yzn config` auto-creates root, Mars, Zellij, and Starship config sources; root `config.toml` has defaults and joined Ratconfig contract state; `open.log_level` controls managed `YZN_OPEN_LOG`; `shell.program` controls the packaged default-shell dispatcher; `editor.command` controls managed Yazi opens and Ratconfig text edits as one executable name or path, with `yzn-hx` resolving to packaged Yazelix Helix and host commands staying host-owned without bridge reuse; `appearance.mode` controls packaged Mars and Zellij bar dark/light surfaces; `welcome.enabled`, `welcome.style`, and `welcome.duration_seconds` control the startup welcome on new launches; `popup.side_margin` and `popup.vertical_margin` control managed popup margins on new launches; `keybindings.config`/`agent`/`git`/`menu` control semantic popup role triggers and reject invalid, duplicate, or conflicting packaged chords; semantic `[popups.<id>]` entries require `command` and `keybinding`, accept optional `args`, `title`, and `keep_alive`, reject shell-string commands, conflicting chords, and duplicate or packaged popup titles, and render through hidden package-internal KDL read paths; `bar.widgets` controls the ordered top-bar widget tray through Ratconfig's string-list picker; Mars/Zellij tabs route writes to their native files; `zellij/plugins.kdl` accepts only plugin declarations for injection into the managed Zellij config; text edit buffers can round-trip through the config UI's configured editor environment before save; the Starship tab edits `format`, `right_format`, and `add_newline` in `starship.toml` with a colon-colon-space (`:: `) default left prompt; the Helix tab opens managed `config.toml`, `languages.toml`, `helix.scm`, and `init.scm` files through the managed editor, creates TOML files after activation, and creates the Steel pair when either Steel row is activated; packaged managed Helix exposes `:yzn-new-shell` unless user Steel files override it, and packaged TOML binds `Alt r`/`Ctrl r` unless user `config.toml` overrides it; Keys table columns list packaged bindings as read-only group, key, action, and owner metadata with source paths in details; Advanced rows open Nu, managed Yazi sidecar files, and `zellij/plugins.kdl` through the managed editor and create them only after activation | `crates/yzn-config/`, `config.toml`, `mars.toml`, `helix/config.toml`, `flake.nix` | `crates/yzn-config` unit tests cover create/edit validation, source routing, editor command validation, appearance mode validation, welcome field validation, popup margin validation, popup role keybinding validation, custom popup rendering and validation, bar widget validation, Starship field rendering, Zellij scalar rendering, guarded-node diagnostics, Keys read-only table rows, native file action rows, Helix native file rows, external text editor round-tripping, the Steel pair action, and owned missing-file creation; `checks/yzn-contracts.rs` validates packaged defaults, helper install, creation, `--get`, dispatcher wiring, popup role and custom popup key rendering, Zellij plugin sidecar injection and rejection, config UI editor wrapper wiring, packaged `:yzn-new-shell` Steel command wiring, managed Helix reload/reveal bindings, managed Helix runtime selection for packaged, TOML-only, languages-only, and Steel-pair cases | Interactive Ratconfig UI behavior remains manual dogfooding |
| C12 | The welcome screen defaults to enabled, random, and 3 seconds; a user can choose a fixed style instead of random, and random chooses only from the animated screen-style pool, excluding the explicit `static` card | `yazelix-screen`, `runtime/yzn/`, `config.toml` | `yazelix-screen` unit tests validate the fixed pool, argument parsing, timed playback mode, and static-card copy; `checks/yzn-contracts.rs` validates packaged helper wiring and config/status/doctor exposure | Visual animation behavior remains manual dogfooding |
| C13 | `homeManagerModules.default` provides a narrow declarative Home Manager surface: `programs.yazelix.enable = true` installs the selected package, package override is supported, the package desktop entry is present, no Yazelix runtime config files are generated by default, optional `config.settings` renders root `config.toml` with defaults and contract state, and optional native Mars, Zellij, Starship, Helix, Yazi, and Nu files are installed from text or source paths | `home-manager/module.nix`, `flake.nix`, `crates/yzn-config/` | `checks.home_manager` evaluates Home Manager with the default package and an override package, validates `bin/yzn`, validates the desktop entry, rejects default generated `~/.config/yazelix-next` files, validates rendered root settings/defaults/contract state through `yzn-config --get`, and validates native text/source files; `crates/yzn-config` unit tests allow read-only complete root TOML with format-only drift and still reject incomplete read-only sources | Full Home Manager switch behavior remains external to the module evaluation check |

## Pros

- The public surface is small: `yzn help`, `yzn config`, `yzn menu`,
  `yzn tutor`, `yzn screen`, `yzn reveal`, `yzn status`, `yzn doctor`,
  `yzn sponsor`, `yzn enter`, and `yzn launch`.
- The semantic config surface stays small and concrete instead of becoming a
  broad command/config system.
- Nix owns dependency composition, so Mars, Zellij, Helix, Yazi, LazyGit, the
  Yazelix bar, plugins, fonts, desktop entry assets, and helper binaries are
  versioned together.
- Mars is an isolated terminal concern. The rest of the runtime can stay focused
  on Zellij/Yazi/Helix behavior.
- Rust owns runtime glue where quoting, file writes, sockets, process execution,
  and fallback behavior matter.
- Zellij layout validation runs inside the build, so a broken swap/template
  relationship fails before the package is installed.
- Default user config layering is intentionally narrow. Alternate shell startup
  behavior is opt-in through `shell.program`, while Yazi stays managed.

## Cons

- The flake is the main composition point, so `flake.nix` carries more plumbing
  than the project philosophy wants.
- Mars still brings Nix-specific packaging weight into a project that wants a
  minimal architecture.
- Small Rust helpers add more owned LOC than shell wrappers, even when they
  reduce quoting and process-lifecycle risk.
- The Zellij and Helix forks are hard runtime dependencies. This keeps behavior
  direct, but raises the cost of following upstream.
- Yazi is powerful but integration-heavy. Sidebar behavior, plugins, previews,
  opener routing, and editor bridge behavior create several contracts that need
  focused checks.
- User config layering exists for Mars, Starship, Nushell, guarded Zellij
  preferences, managed Yazi sidecars, and a few semantic config fields, but not
  broad native Yazi replacement.

## Current Tradeoff

The architecture favors a small, working vertical slice over a minimal file
count. The biggest simplicity win is not fewer source lines in isolation; it is
having one owner for each contract and no duplicated compatibility surface. Nix
is the least elegant part of the stack, but it is also the part that keeps the
runtime installable and reproducible while Mars, Zellij, Helix, Yazi, and the
helpers evolve together.
