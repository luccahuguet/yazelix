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
the packaged shell for new Zellij panes, `welcome.enabled`, `welcome.style`,
and `welcome.duration_seconds`, which control the startup splash,
`popup.side_margin` and `popup.vertical_margin`, which set the `yzpp` default
left/right and top/bottom cell margins for generated managed popups,
`keybindings.config`/`agent`/`git`/`menu`, which remap managed popup
triggers, semantic `[popups.<id>]` specs with required keybindings for managed
custom popups, and `bar.widgets`, which selects the
top-bar tray while the shell widget label follows `shell.program`. The Mars,
Zellij, and Starship tabs are render/edit surfaces
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
Mars window, font, cursor, bell, quit, and appearance preset used by `yzn`.
`yzn config` exposes selected Mars-native fields, including
`mars.appearance.preset`, through
`~/.config/yazelix-next/mars/config.toml`; root `config.toml` does not own a
global appearance mode. `mars.appearance.preset` is also the Ratconfig UI
theme source, so saves through the config UI switch the config palette live.
Low-level `force-theme` and `[colors]` entries remain manual native Mars TOML.
`yzn` still owns the launch command and runtime environment.

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
and materializes a runtime layout through the pinned renderer when the user
chooses a custom tray. The runtime config points managed new tabs at that
active layout and the user's home cwd. It ships `tu` for usage widget
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
layout swaps, `helix-contracts.rs` validates managed Helix packaging and
override behavior, and `yzn-contracts.rs` validates the remaining packaged
runtime contracts.

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
row activation. `yzn-hx` writes an effective Helix config under
`YAZELIX_STATE_DIR/helix/config.toml` on each launch by deep-merging the
packaged template with optional `~/.config/yazelix-next/helix/config.toml`, then
reclaiming `keys.normal.A-r` for `yzn reveal`. When `config.toml`,
`languages.toml`, or the Steel pair exists under
`~/.config/yazelix-next/helix`, it uses that directory for Helix native config,
points `HELIX_STEEL_CONFIG` there only when the Steel pair exists, and points
Steel at an internal state fallback for TOML-only managed config.
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
warns when a managed Helix TOML override tries to replace reserved `Alt r`.

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

The table is an **index**: one row per contract with behavior, owner, proving
check, and gap. Implementation detail lives in the owner sections above, the
checks, and the short notes after the table for the densest rows.

| ID | Contract | Owner | Check | Missing Coverage |
| --- | --- | --- | --- | --- |
| C1 | Front-door `yzn` CLI: default `launch`, plus `help`, `config`, `menu`, `tutor`, `screen`, `reveal`, `status`, `doctor`, `sponsor`, `enter`, and pre-exec diagnostics | `runtime/yzn/`, `runtime/yzn-menu.rs`, `crates/yzn-tutor/`, `crates/yzn-config/`, `crates/yzn-open/`, `yazelix-screen`, `flake.nix` | `checks/yzn-contracts.rs`, `checks/helix-contracts.rs`, `checks/key-reference-parity.rs`, crate unit tests, `nix build .#yzn` | GUI launch dogfooding |
| C2 | Mars uses packaged visual config unless user native Mars config exists; config UI exposes `mars.appearance.preset` as Ratconfig theme source; low-level Mars colors/cursors stay native | `mars.toml`, `flake.nix`, `crates/yzn-config/` | `checks/yzn-contracts.rs`, `yzn-config` tests | Visual dogfooding |
| C3 | Zellij layout has the sidebar template required by swaps | `layout.kdl`, `layout.swap.kdl` | `checks/zellij-layout.rs` | None for current swap contract |
| C4 | Packaged Zellij keys, sidebar/orchestrator bindings, and guarded sidecar ownership (no integration-owned top-level takeover) | `config.kdl`, `runtime/yzn-zellij-config.rs` | `checks/yzn-contracts.rs` | Full key dogfooding |
| C5 | Managed Nu loads packaged config, optional host `mise`, optional user Nu, and controlled Starship prompts | `runtime/yzn-nu.rs`, `nu/` | `checks/yzn-contracts.rs` | None for current layering |
| C6 | Managed Yazi: scoped preview env, optional user init/keymap/plugins, opens via `yzn-open`, sidebar id/reveal, zoxide jump | `yazi/`, `runtime/yzn-yazi.rs`, `crates/yzn-open/` | `checks/yzn-contracts.rs`, `yzn_yazi_materialization`, `yzn-open` tests | Full Yazi UI dogfooding |
| C7 | Helix bridge reuse stays in the current `yzn` window and Zellij tab | `crates/yzn-open/`, `flake.nix` | `yzn-open` tests | Multi-window GUI dogfooding |
| C8 | Desktop entry starts `yzn` | `flake.nix` | `nix build .#yzn` packages the desktop file | Desktop launch dogfooding |
| C9 | Managed popups via `yzpp` (roles, custom specs, agent/Git behavior, margins, sidebar refresh hooks) with Kitty keyboard protocol | `config.kdl`, `runtime/yzn/`, `runtime/yzn-agent.rs`, `crates/yzn-config/`, `flake.nix` | `checks/yzn-contracts.rs` | Visual popup dogfooding |
| C10 | Top bar tray from child bar package, home-scoped tabs, usage widget cache, native bottom key hints | `layout.kdl`, `config.kdl`, `runtime/yzn/`, `flake.nix`, `packaging/tokenusage.nix` | `checks/zellij-layout.rs`, `checks/yzn-contracts.rs` | Visual bar dogfooding |
| C11 | `yzn config` owns semantic root settings, native source files, and session-aware Zellij sidecar apply; Helix effective config and Keys/Advanced surfaces | `crates/yzn-config/`, `config.toml`, `mars.toml`, `helix/config.toml`, `flake.nix` | `yzn-config` tests, `checks/yzn-contracts.rs`, `checks/helix-contracts.rs` | Interactive Ratconfig dogfooding |
| C12 | Welcome defaults (enabled, random, 3s); random excludes static/logo card styles | `yazelix-screen`, `runtime/yzn/`, `config.toml` | `yazelix-screen` tests, `checks/yzn-contracts.rs` | Animation dogfooding |
| C13 | Narrow Home Manager module: install/override package, optional settings and native file passthroughs, no default config generation | `home-manager/module.nix`, `flake.nix`, `crates/yzn-config/` | `checks.home_manager`, `yzn-config` read-only root tests | Full HM switch external |

### C1 Front door

Bare `yzn` is `launch`. Non-launch commands stay non-GUI where possible
(`help`, `status`, `doctor`, `tutor`, `screen` help paths). Menu is a curated
allowlist. Reveal targets the active tab sidebar. Pre-exec failures print a
Yazelix diagnostic before Mars/Zellij handoff.

### C9 Popups

Role keys default to `Alt Shift J/K/L/M` and remaps stay semantic. Custom
`[popups.<id>]` is argv-based with unique titles. Agent uses hide keep-alive and
provider bootstrap; Git closes on toggle and uses the managed editor env.
Margins come from `yzpp` popup defaults; close/hide hooks refresh sidebar git
decorations.

### C11 Config UI

Auto-creates root/Mars/Zellij/Starship sources. Root owns shell, editor,
welcome, popup margins/roles, custom popups, bar widgets, and open log level.
Mars/Zellij/Starship tabs write native files; Helix and Advanced are open-file
surfaces. Inside a managed session, Zellij scalar saves also patch the active
runtime config file. Host editors bypass the Helix bridge; `yzn-hx` does not.

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
