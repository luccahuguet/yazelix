# Architecture

Yazelix Next is a small Nix/Lix flake with one front-door command: `yzn`.
`yzn help` prints help, `yzn config` opens the Ratconfig UI, `yzn status`
summarizes owned runtime/config state, `yzn doctor` checks owned startup setup,
`yzn sponsor` opens or prints the Sponsors URL, `yzn enter` starts Yazelix
inside the current terminal, `yzn launch` opens Mars first, and `yzn menu`
prints the compact command/key reference. Bare `yzn` defaults to `yzn launch`.
The runtime paths are intentionally narrow:

```text
yzn launch -> Mars -> Yazelix Zellij fork -> Yazi sidebar + stacked work panes
yzn enter -> Yazelix Zellij fork -> Yazi sidebar + stacked work panes
```

The repo owns the glue that makes those pieces behave like one runtime. It does
not try to be a general terminal distribution, Home Manager module, or Yazelix
compatibility layer.

## Owners

`flake.nix` is the package graph and composition owner. It pins external inputs,
builds small local Rust helpers, substitutes local config templates, installs
the desktop entry, and exposes the `yzn` package/app.

`crates/yzn-config/` is the config host owner. It opens the Ratconfig UI,
creates `~/.config/yazelix-next/config.toml` with defaults and joined contract
state when missing, creates simple managed Mars and Zellij config files when
missing, routes source-backed edits to the correct file, exposes Advanced rows
for native Nu, Starship, and managed Yazi sidecar files, lists read-only packaged
bindings as a Keys table, and exposes one hidden package-internal read path used
by launch wrappers.
The contracted root config fields are `open.log_level`, which controls
`YZN_OPEN_LOG` for managed Yazi-to-Helix opens, `shell.program`, which selects
the packaged shell for new Zellij panes, and `popup.size`, which sets generated
managed popup width and height. The Mars and Zellij tabs are render/edit
surfaces without contracts or migrations. The Advanced tab is an open-file
surface: Ratconfig renders rows and emits file-open intents, while Yazelix Next
owns path selection, missing-file creation, and editor launch.

`mars.toml` is the packaged terminal visual config owner. It sets the default
Mars window, font, cursor, bell, quit, and theme behavior used by `yzn`. A user
can replace the Mars config with `~/.config/yazelix-next/mars/config.toml`;
`yzn` still owns the launch command and runtime environment.

`config.kdl`, `layout.kdl`, and `layout.swap.kdl` are the Zellij behavior
owners. They point `default_shell` at the packaged shell dispatcher, set
Zellij-native `Ctrl Alt` mode keys, direct `Ctrl Alt h/j/k/l` movement, the
Tab-mode new-tab layout binding, the `Alt m` pane binding, the `Alt Shift h`
sidebar toggle, the `Alt Shift J` LazyGit popup binding, the `Alt Shift K`
config popup binding, the persistent `Alt Shift L` guarded Codex resume popup
binding, the `Alt Shift M` menu popup binding, `Alt h/l` horizontal focus
walking, the Yazelix Zellij Bar top bar, the Yazi sidebar tab, the open/closed
sidebar swap layouts, and explicit Kitty keyboard protocol support.
The standalone `yzpp` plugin owns popup lifecycle, and the standalone pane
orchestrator owns horizontal focus policy. Yazelix Next only packages them and
binds `Alt h/l` to the orchestrator's visible-pane walker.

`yazelix-zellij-bar` owns the rendered top bar KDL, widget command logic, and
`zjstatus.wasm`. Yazelix Next owns only the selected semantic tray at
`[bar].widgets`, uses the packaged pre-rendered layout for the default tray,
and materializes a runtime layout through the pinned renderer when the user
chooses a custom tray. The runtime config points managed new tabs at that
active layout. It ships `tu` for usage widget refreshes, exports a yzn-owned
status cache path, names the initial and tab-mode-created tabs with the Yazelix
home marker, home-scopes tab-mode-created tabs, and keeps the native bottom
Zellij `status-bar` for key hints.

`runtime/yzn-zellij-config.rs` is the launch-time guarded Zellij sidecar owner.
It appends `~/.config/yazelix-next/zellij/config.kdl` to the packaged config
after a small first-token denylist rejects obvious attempts to take over the
managed shell, keymap, layout, plugin loading, Kitty keyboard protocol,
environment, or session startup behavior. `crates/yzn-config/` owns the config
UI renderer/parser for the small exposed scalar sidecar subset.

`nu/` is the packaged Nushell config owner. It initializes carapace, zoxide,
and Starship left and right prompts, and disables the normal Nushell banner and
prompt indicators.

`yazi/` is the packaged Yazi config owner. It enables the selected Yazi
plugins, keeps file opens routed through `yzn-open`, binds `Alt z` to a zoxide
picker that moves the sidebar and opens the chosen directory in the managed
editor path, and avoids broad Yazi config merging.

`runtime/yzn-yazi.rs` is the managed Yazi launcher owner. It sets scoped image
preview/session environment, appends optional managed user `yazi/init.lua` and
`yazi/keymap.toml`, overlays managed user `plugins/*.yazi` directories when
that init exists, and then execs packaged Yazi.

`runtime/yzn-nu.rs` is the Nushell runtime-config owner. It writes the runtime
`env.nu` and `config.nu` files, layers optional user config from
`~/.config/yazelix-next/nu`, chooses the Starship config path, and then execs
Nushell.

The flake owns the `yzn-shell` dispatcher. It reads `shell.program` through
`yzn-config` and execs packaged `nu`, `bash`, `zsh`, or `fish`. The `nu` path
delegates to `runtime/yzn-nu.rs`; other shells are intentionally plain packaged
interactive shells with their normal startup-file behavior. Unsupported shell
values fail in `yzn-config` before dispatch, so shell schema policy stays in the
config owner rather than the shell wrapper.

`crates/yzn-open/` is the editor-open owner. It sends file and directory open
requests to the live Yazelix Helix bridge when available, and otherwise opens a
managed `yzn-hx` pane through Zellij. Directory opens use the workspace root for
editor cwd and tab naming while keeping the selected directory as the Helix
picker directory. It also owns bounded diagnostics for Yazi-to-Helix opens.

`checks/` owns build-time contract guards. `zellij-layout.rs` validates Zellij
layout swaps, and `yzn-contracts.rs` validates the packaged runtime contracts.

## Config Layering

Packaged config comes first unless a surface explicitly opts into native
replacement. User config is narrow and explicit:

```text
~/.config/yazelix-next/config.toml
~/.config/yazelix-next/mars/config.toml
~/.config/yazelix-next/zellij/config.kdl
~/.config/yazelix-next/starship.toml
~/.config/yazelix-next/nu/env.nu
~/.config/yazelix-next/nu/config.nu
```

`YAZELIX_NEXT_CONFIG_HOME` can point at another config root. `config.toml` is
the Yazelix-owned semantic config file and is created by `yzn config` or the
package-internal config read path when missing. It controls bounded runtime
settings such as open diagnostics, the packaged shell choice, managed popup
size, and the ordered bar widget tray. `yzn config` also creates the managed
Mars and Zellij native files when missing. Mars uses full native replacement
when its `config.toml` exists. Nushell uses packaged config first, then
optional user `env.nu` and `config.nu`. For managed Nu, Starship uses the user
`starship.toml` when present, otherwise an empty config
that preserves Starship defaults. `yzn config` exposes the Nu and Starship
files through the Advanced tab and creates them only after explicit row
activation. Normal Nushell and Starship config files are not loaded by default,
which keeps the default `nu` path reproducible and avoids ambient user shell
behavior changing that runtime. Zellij uses packaged config first, then a
guarded sidecar for safe
native preferences. The sidecar is a guardrail rather than a KDL parser: it
rejects uncommented lines whose first token is known to own integration-critical
behavior such as `keybinds`, `default_shell`, layout, plugins, Kitty keyboard
protocol, environment, or session startup.

## Runtime Startup Boundary

`yzn enter` and `yzn launch` are owned by the Rust front door in
`runtime/yzn.rs`. Nix substitutes store paths into that binary source, but Rust
owns the startup wiring and final `exec` handoff:

- establish `YAZELIX_STATE_DIR` and the top-level Helix bridge session id
- set managed `EDITOR` and `VISUAL`
- resolve the Yazelix config home from `YAZELIX_NEXT_CONFIG_HOME`,
  `XDG_CONFIG_HOME`, or `HOME`
- read `open.log_level` through `yzn-config` and export `YZN_OPEN_LOG`
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
summary; `doctor` renders ok/warn/fail rows for the owned startup surface.

After this boundary, Yazelix gives control to the target process. `yzn enter`
execs the Yazelix Zellij fork directly. `yzn launch` execs Mars with Zellij as
the child command. Startup failures before this handoff are Yazelix-owned;
failures after the handoff belong to Mars, Zellij, or the child tool.

## Session Isolation

Each top-level `yzn` launch creates one opaque
`YAZELIX_HELIX_BRIDGE_SESSION_ID`. Zellij, Yazi, Helix, and `yzn-open` inherit
that id inside the window, so Yazi opens can only reuse Helix bridge registries
from the same session. `yzn-open` also compares `ZELLIJ_SESSION_NAME` when a
registry records it, which prevents two Zellij windows with copied session state
from sharing an editor pane. Managed Yazi saves that real session in
`YAZELIX_ZELLIJ_SESSION_NAME` before blanking `ZELLIJ_SESSION_NAME` for image
previews, and `yzn-open` restores the saved name for Zellij control commands.
For tab isolation, `yzn-open` reads the invoking Yazi pane id from
`ZELLIJ_PANE_ID`, asks Zellij for live pane membership, and reuses only a Helix
registry whose recorded pane id is in the same stable `tab_id`.

Helpers launched outside `yzn` do not fall back to the shared literal `yzn`
session id. `yzn-hx`, `yzn-yazi`, and `yzn-open` derive isolated helper ids, so
standalone helper use cannot accidentally target a bridge from a live `yzn`
window.

## Runtime Contracts

| ID | Contract | Owner | Check | Missing Coverage |
| --- | --- | --- | --- | --- |
| C1 | `yzn` defaults to `yzn launch`, `yzn help` prints help, `yzn config` opens Ratconfig config, `yzn menu` prints a compact command/key reference, `yzn status` prints a runtime/config summary, `yzn doctor` checks owned startup setup, `yzn sponsor` opens or prints the sponsor URL, `yzn enter` starts managed Zellij in the current terminal, and `yzn launch` starts Mars first; pre-exec setup failures print a readable Yazelix diagnostic | `flake.nix`, `runtime/yzn.rs`, `crates/yzn-config/` | `checks/yzn-contracts.rs` validates help, menu output, front-door wiring, runtime setup wiring, status/doctor/sponsor behavior, and representative startup diagnostics; `nix build .#yzn` packages the runtime | GUI launch remains manual dogfooding |
| C2 | Mars uses packaged visual config unless a user native Mars config exists | `mars.toml`, `flake.nix` | `checks/yzn-contracts.rs` validates packaged config and launcher selection | Visual correctness remains manual dogfooding |
| C3 | Zellij layout has the sidebar template required by swaps | `layout.kdl`, `layout.swap.kdl` | `checks/zellij-layout.rs` runs during build | None for the current template/swap contract |
| C4 | Zellij-native mode keys use `Ctrl Alt`, Tab-mode new tabs use the packaged Yazelix sidebar layout, move mode is unbound, `Alt m` opens a pane for the swap layout to stack, `Alt Shift h` toggles the sidebar swap, `Alt h/l` walk visible panes through the pane orchestrator, and obvious sidecar ownership lines are rejected | `config.kdl`, `runtime/yzn-zellij-config.rs` | `checks/yzn-contracts.rs` validates the packaged config, pane-orchestrator binding, and accepted/rejected sidecars | Full key behavior remains manual dogfooding |
| C5 | When `shell.program` is `nu`, Nushell loads packaged config first, optional user config after it, and controlled Starship left/right prompt config | `runtime/yzn-nu.rs`, `nu/` | `checks/yzn-contracts.rs` validates Nushell layering through the managed shell dispatcher, Starship config selection, and right prompt rendering | None for current layering behavior |
| C6 | Yazi launches with scoped image-preview environment, optionally appends managed user `yazi/init.lua` and `yazi/keymap.toml`, overlays managed user `plugins/*.yazi` directories when init exists, opens paths through `yzn-open` with bounded diagnostics, and `Alt z` jumps through zoxide into the managed editor path while using the workspace root for editor cwd and tab naming | `yazi/`, `runtime/yzn-yazi.rs`, `crates/yzn-open/` | `checks/yzn-contracts.rs` validates packaged Yazi keymap/plugin wiring and launcher environment; `yzn_yazi_materialization` covers init/keymap/plugin materialization; `cargo test` covers `yzn-open` bridge/fallback and tab rename behavior | Full Yazi UI behavior remains manual dogfooding |
| C7 | Helix bridge reuse stays inside the current `yzn` window and current Zellij tab | `crates/yzn-open/`, `flake.nix` | `yzn-open` Rust tests cover session, Zellij-window, and Zellij-tab mismatch | Full multi-window GUI behavior remains manual dogfooding |
| C8 | Desktop entry starts `yzn` | `flake.nix` | `nix build .#yzn` packages the desktop file | Desktop environment launch remains manual dogfooding |
| C9 | Kitty keyboard protocol is explicitly enabled, `Alt Shift J/K/M` toggle LazyGit, config, and menu popups through `yzpp`; `Alt Shift L` hides or shows a persistent guarded Codex resume popup; `popup.size` controls generated popup width and height | `config.kdl`, `runtime/yzn.rs`, `flake.nix` | `checks/yzn-contracts.rs` validates Kitty protocol, the packaged popup plugin, commands, popup ids, payloads, key bindings, agent hide behavior, popup geometry, and the missing-Codex guard | Visual popup behavior remains manual dogfooding |
| C10 | Top bars use the child-rendered Yazelix Zellij Bar tray, tabs use the home marker, tab-mode-created tabs open in home, Codex usage has bundled `tu` and a yzn-owned cache path, and bottom bars keep native Zellij key hints | `layout.kdl`, `config.kdl`, `flake.nix`, `packaging/tokenusage.nix` | `checks/zellij-layout.rs` validates packaged child bar usage, no-mode formatting, declared yzn widgets, the startup home tab marker, the home-scoped new-tab template, and native bottom status bars; `checks/yzn-contracts.rs` validates the tab-mode new-tab marker, terminal-label wiring, bundled tokenusage path, and status-cache export | Visual bar behavior remains manual dogfooding |
| C11 | `yzn config` auto-creates root, Mars, and Zellij config sources; root `config.toml` has defaults and joined Ratconfig contract state; `open.log_level` controls managed `YZN_OPEN_LOG`; `shell.program` controls the packaged default-shell dispatcher; `popup.size` controls managed popup geometry on new launches; `bar.widgets` controls the ordered top-bar widget tray through Ratconfig's string-list picker; Mars/Zellij tabs route writes to their native files; Keys table columns list packaged bindings as read-only group/key/action/owner metadata with source paths in details; Advanced rows open Nu, Starship, and managed Yazi sidecar files through the managed editor and create them only after activation | `crates/yzn-config/`, `config.toml`, `mars.toml`, `flake.nix` | `crates/yzn-config` unit tests cover create/edit validation, source routing, popup size validation, bar widget validation, Zellij scalar rendering, guarded-node diagnostics, Keys read-only table rows, native file action rows, and owned missing-file creation; `checks/yzn-contracts.rs` validates packaged defaults, helper install, creation, `--get`, dispatcher wiring, and the config UI editor wrapper | Interactive Ratconfig UI behavior remains manual dogfooding |

## Pros

- The public surface is small: `yzn help`, `yzn config`, `yzn menu`,
  `yzn status`, `yzn doctor`, `yzn sponsor`, `yzn enter`, and `yzn launch`.
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
  preferences, and a few semantic config fields, but not yet for Yazi.

## Current Tradeoff

The architecture favors a small, working vertical slice over a minimal file
count. The biggest simplicity win is not fewer source lines in isolation; it is
having one owner for each contract and no duplicated compatibility surface. Nix
is the least elegant part of the stack, but it is also the part that keeps the
runtime installable and reproducible while Mars, Zellij, Helix, Yazi, and the
helpers evolve together.
