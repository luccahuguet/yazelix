# Architecture

Yazelix Nova is a small Nix/Lix flake with one development front door:
**`yzx`**. The temporary command and `yazelix` paths let it coexist with
public Yazelix v17 until the canonical swap.

This repo owns the glue that makes Mars, the Yazelix Zellij fork, Yazi, and
Helix feel like one runtime. It is not a general terminal distro, a broad Home
Manager config system, or a main-Yazelix compatibility layer.

## Runtime chain

```text
yzx launch  →  Mars  →  yzx-welcome  →  Yazelix Zellij  →  Yazi sidebar + work panes
yzx enter   →  yzx-welcome  →  Yazelix Zellij  →  same layout
yzx run     →  prepared Yazelix environment  →  exact child argv/status
```

Bare `yzx` prints help. `launch` is the only Mars route.
`enter` is the headless/SSH route and requires an interactive host terminal, not
a display server. The fixed `runtime` package compiles the Mars route out while
retaining the same command, config schema, and managed workspace.

## Platforms

| Surface | Support |
| --- | --- |
| Full/runtime package and app outputs | `x86_64` / `aarch64` × Linux / Darwin |
| Headless / SSH floor | `enter` in a capable interactive host terminal; managed TUI only |
| macOS build evidence | Real `aarch64-darwin` runner builds full/runtime packages and the Home Manager closure; no desktop entry |
| macOS interactive floor | `help`, `status`, `doctor`, `enter`, managed workspace, and host-editor delegation remain unverified |
| macOS full-package `launch` | Mars is packaged; GUI behavior remains unverified |
| Out of repo | App bundles, Homebrew, Ghostty packaging, broad terminal matrices |

## Commands

| Command | Role |
| --- | --- |
| `yzx` / `help` | Concise help; no implicit launch |
| `--version` | Package-owned exact Nova version |
| `launch` | Mars then managed session; unavailable in the runtime package |
| `enter` | Managed session in current terminal |
| `run` | Structured command in the prepared runtime environment |
| `config` | Ratconfig UI |
| `menu` | Curated command palette |
| `tutor` | Guided lessons / native tutor hints |
| `screen` | Terminal screens / welcome styles |
| `reveal` | Path in managed Yazi sidebar |
| `status` / `status --json` | Human and schema-versioned runtime status |
| `doctor` | Owned setup checks |
| `env` | Managed shell only |
| `help` | Help |

---

## Owners

One owner per concern. Paths are the durable map.

### Package and install

| Path | Owns |
| --- | --- |
| `flake.nix` | Fixed full/runtime composition, inputs, helpers, desktop entry, HM export |
| `home-manager/module.nix` | `programs.yazelix.enable` / package; optional config files; no default generation |

### Front door and helpers

| Path | Owns |
| --- | --- |
| `runtime/yzx/` | CLI, startup env, launch/enter handoff |
| `runtime/yzx-menu.rs` | Menu palette |
| `runtime/yzx-agent.rs` | Agent provider bootstrap (`codex resume` → `grok` → `opencode` → `pi` → `claude --resume`) |
| `runtime/yzx-yazi.rs` | Managed Yazi process/env launch, editor resolve |
| `runtime/yzx-nu.rs` | Managed Nu layering; runtime-effective Starship config request |
| `runtime/yzx-zellij-config.rs` | Packaged + guarded Zellij scalar sidecar merge |
| `runtime/yzx/zellij.rs` | Plugin sidecar inject; launch materialize/patches |
| `crates/yzx-open/` | Editor open, Helix bridge, reveal, bounded open diagnostics |
| `crates/yzx-yazi-config/` | Managed Yazi config-home materialization and native TOML layering |
| `crates/yzx-tutor/` | Tutor CLI and lessons |
| `runtime/yzx-helix.sh` (`yzx-hx`) | Effective Helix config + Steel wiring |
| `yazelix-screen` (child) | Screen styles; packaged as `yzx screen` |
| `checks/` | Build-time contract guards |

### Config UI

`crates/yzx-config/` is the Ratconfig host.

- Seeds only the child-owned cursor TOML; root, Mars, Zellij, and Starship stay
  sparse
- Routes edits to the right file; Helix/Advanced open-file rows; Keys read-only
- Resolves known config targets against the packaged Nix store root so
  Home Manager-owned sources stay read-only with exact module-option guidance
- Hidden package-internal reads for launch + custom-popup KDL render
- `agent.popup.kdl` is an internal render path for custom managed agent command
  KDL
- `KEY_BINDINGS` is the human key reference; `defaults/zellij/config.kdl` is the runtime owner

#### Nova root schema inventory

Packaged `defaults/config.toml` owns every default below. The optional user file
stores only explicit overrides; `CONFIG_FIELDS` and `root_config.rs` own the
bounded catalog, validation, and sparse persistence unless another owner is named.
The root validator derives fixed leaves from that catalog, rejects unknown paths
before runtime or Ratconfig use, and delegates only `popups.<id>` to its dynamic
field validator.

| Root path | Type | Default | Effect | Applies |
| --- | --- | --- | --- | --- |
| `open.log_level` | string enum | `info` | `YZX_OPEN_LOG` diagnostics for managed opens | new opens |
| `shell.program` | string enum | `nu` | Packaged shell for new panes | new panes |
| `editor.command` | executable string | `yzx-hx` | Yazi opens, config text edits, and Git clients | new opens |
| `agent.command` | executable string or `auto` | `auto` | Managed agent popup command | next launch |
| `agent.args` | string array | `[]` | Arguments for a custom agent command | next launch |
| `welcome.enabled` | boolean | `true` | Enables the pre-Zellij splash | next launch |
| `welcome.style` | string enum | `random` | Selects the packaged splash style | next launch |
| `welcome.duration_seconds` | integer, 1–60 | `3` | Sets splash duration | next launch |
| `popup.side_margin` | non-negative integer | `1` | Left/right managed popup margin in cells | next launch |
| `popup.vertical_margin` | non-negative integer | `0` | Top/bottom managed popup margin in cells | next launch |
| `keybindings.config` | key chord | `Alt Shift K` | Config popup trigger | next launch |
| `keybindings.agent` | key chord | `Alt Shift L` | Agent popup trigger | next launch |
| `keybindings.git` | key chord | `Alt Shift J` | Git popup trigger | next launch |
| `keybindings.menu` | key chord | `Alt Shift M` | Menu popup trigger | next launch |
| `keybindings.sidebar` | key chord | `Alt Shift H` | Sidebar visibility | next launch |
| `keybindings.sidebar_focus` | key chord | `Ctrl y` | Editor/sidebar focus | next launch |
| `bar.widgets` | ordered string array | `editor`, `shell`, `term`, `codex_usage`, `cpu`, `ram` | Top-bar tray order; `BAR_WIDGET_VALUES` and `bar_widgets` own validation | next launch |

`custom_popups.rs` owns the dynamic `[popups.<id>]` namespace. An id starts
with an ASCII letter or `_`, then uses ASCII letters, digits, `_`, or `-`; the
packaged ids `config`, `agent`, `git`, and `menu` are reserved.

| Dynamic path | Type | Required/default | Meaning |
| --- | --- | --- | --- |
| `popups.<id>.command` | non-empty executable string without whitespace | required | Popup executable; arguments stay separate |
| `popups.<id>.args` | non-empty string array | `[]` | Structured argv |
| `popups.<id>.title` | non-empty string | `<id>_popup` | Unique pane title; packaged popup titles are reserved |
| `popups.<id>.keybinding` | key chord | required | Unique trigger that cannot collide with packaged bindings |
| `popups.<id>.keep_alive` | boolean | `false` | Hides rather than closes the popup when toggled |

Custom popups apply on the next launch. No other fields are accepted inside a
custom popup entry.

### Packaged layout and tools

| Path | Owns |
| --- | --- |
| `defaults/mars/config.toml` | Default Mars window/font/appearance; `mars.appearance.preset` is also Ratconfig UI theme (live palette) |
| `defaults/zellij/config.kdl` | Zellij keys, plugins load, popup wiring, Kitty protocol |
| `defaults/zellij/layout*.kdl` | Sidebar + stacked panes, open/closed swap |
| `defaults/nu/` | Packaged Nu: carapace, zoxide, and Starship invocation |
| `defaults/yazi/` | Opens via `yzx-open`, plugins, `Alt z` workspace retarget |
| `defaults/helix/config.toml` | Packaged defaults; `Alt r` reveal, `Ctrl r` reload (overridable) |

### Child packages (not owned here)

| Child | Domain |
| --- | --- |
| Mars | Terminal |
| yazelix-cursors | Cursor TOML schema, validation, definitions, and resolution |
| yazelix-zellij / helix | Multiplexer / editor forks |
| yazelix-zellij-popup (`yzpp`) | Popup lifecycle |
| yazelix-zellij-pane-orchestrator | Focus / sidebar walk |
| yazelix-zellij-bar | Top bar render + widgets |
| ratconfig | Config UI toolkit |
| yazelix-screen | Welcome / screen animations |

This repo packages them and applies product policy only.

### Installed closure topology

`flake.nix` owns the package graph. On `x86_64-linux`, the 2026-07-12 locked
graph realizes to **2.28 GiB across 619 store paths**. The fixed Mars-free
variant is **1.37 GiB across 591 paths**, a measured 927 MiB reduction; its
source-build graph contains 2,407 fewer derivations. Nova's top-level full output
contains only 46.1 KiB of NAR data; it is a thin command, desktop-entry, and
asset join whose references pull in the runtime.

The individual package closures below explain the architectural weight. They
share libraries and tools, so no row is additive and removing one root does not
necessarily save its complete closure size.

| Layer | Package roots and complete individual closures |
| --- | --- |
| Terminal | Mars 1.13 GiB, including Rio, graphics, Python, and fonts |
| Workspace | Yazi + preview tools 503.2 MiB; Yazelix Zellij 101.9 MiB |
| Editor | Yazelix Helix 327.6 MiB, including runtime queries and grammars |
| Source control | Git 373.8 MiB; LazyGit 59.4 MiB |
| Config | Ratconfig / `yzx-config` 124.4 MiB |
| Shell and navigation | Carapace 105.9 MiB; Nushell 104.1 MiB; zoxide 60.8 MiB; Starship 58.9 MiB; fzf 49.5 MiB |
| Status and welcome | tokenusage 75.5 MiB; Yazelix Zellij bar 43.0 MiB; Yazelix Screen 36.7 MiB |
| Zellij control plugins | Pane orchestrator 2.1 MiB; popup 1.9 MiB |

Closure size describes distribution cost, not source ownership or local code
volume. Child packages and packaged tools carry most binary data; Nova keeps
their composition and policy in the small top-level join. The README [installed-size
ledger](docs/installation.md#installed-size) owns the complete per-module list, measurement
meaning, and reproduction commands.

### Shell dispatch

`yzx-shell` reads `shell.program` via `yzx-config`, then runs packaged `nu`
(through `yzx-nu`) or plain `bash` / `zsh` / `fish`.

---

## Module and repo boundaries

| Boundary | When |
| --- | --- |
| Local module | Large code, still ships with `yzx`, no independent user |
| Local crate | Cargo/binary/test isolation, still product glue |
| Separate repo | Independent users + release cadence + stable artifact/API + low `yzx` path/config coupling + low duplicate-owner risk |

**Extraction counts only when this repo deletes or relinquishes a real owner.**

| Threshold | Needs |
| --- | --- |
| Big extraction | Independent users, stable API/artifact, release cadence, real owner deletion here |
| Edge trim | Child already owns a generic validated concept; deleting local glue must not leave a pure adapter between duplicate owners |

---

## Config layering

Packaged first, unless a surface opts into native replacement.

```text
~/.config/yazelix/
  config.toml              # optional sparse semantic overrides
  cursors.toml             # shared cursor selection/effects; seeded once
  mars/config.toml         # optional sparse Mars overrides
  zellij/config.kdl        # guarded scalar sidecar
  zellij/plugins.kdl       # extra plugins only
  starship.toml            # optional sparse prompt overrides
  nu/{env,config}.nu       # after packaged Nu
  helix/*                  # lazy; created on tab use
  yazi/{yazi.toml,theme.toml,package.toml,init.lua,keymap.toml,plugins/,flavors/}
```

Override root with `YAZELIX_CONFIG_HOME`.
Runtime state defaults to `$XDG_DATA_HOME/yazelix` or `YAZELIX_STATE_DIR`.

| Surface | Layering |
| --- | --- |
| Root TOML | Packaged semantic defaults → sparse explicit user overrides |
| Cursors | Child-owned template → user file; Ratconfig edits bounded common fields and preserves custom definitions |
| Mars | Packaged base → recursive sparse user override; cursor selection arrives separately through `YAZELIX_CURSOR_CONFIG` |
| Nu | Packaged → optional host `mise activate nu` → optional user Nu |
| Starship | Native defaults + `yzx-config` `character.format` → sparse user overrides → runtime-effective TOML |
| Helix | See Helix notes below |
| Yazi | Packaged TOML → recursive user tables + replacing scalars/arrays → managed opener/Git fetchers |
| Zellij | Packaged → guarded scalar sidecar → runtime materialize under state dir |
| Host `~/.config/{helix,yazi,starship}` | Not loaded by default |

### Zellij sidecars

`zellij/config.kdl` is a **first-token denylist**, not a full KDL parser. Uncommented
top-level ownership nodes are rejected, including:

`keybinds`, `default_shell`, `default_layout`, `layout`, `plugins`,
`load_plugins`, `support_kitty_keyboard_protocol`, `env`, `session_name`,
`attach_to_session`.

The sidecar is optional and sparse. Ratconfig displays all eight effective
packaged scalar defaults without creating it, treats assignment presence as
explicit intent, and removes only the selected assignment on reset. Removing
the final assignment removes the sidecar.

`zellij/plugins.kdl` accepts only `plugins` / `load_plugins` and must not
redeclare Yazelix-owned plugin ids (`yzpp`, `yazelix_pane_orchestrator`, …).

Inside a managed session, `yzx config` Zellij scalar saves and resets also patch
`$YAZELIX_STATE_DIR/zellij/config.kdl` (watched active file) without wiping
launch patches. Many scalars apply live; some (e.g. `scroll_buffer_size`) need
a new session.

### Helix

- `yzx-hx` writes effective config under `$YAZELIX_STATE_DIR/helix/config.toml`
  each launch: packaged template deep-merged with optional sparse overrides from
  `~/.config/yazelix/helix/config.toml`, then `keys.normal.A-r` reclaimed
  for `yzx reveal`.
- If user Helix dir has `config.toml`, `languages.toml`, and/or Steel pair
  (`helix.scm` + `init.scm`), that dir is native config; `HELIX_STEEL_CONFIG`
  points at the Steel pair only when both exist.
- Without user Steel, packaged Steel exposes `:yzx-new-shell`.
- Packaged bindings: `Alt r` reveal (reserved), `Ctrl r` reload (user-overridable).
- `hx` and `yzx-hx` select managed Helix; other `editor.command` values skip its
  bridge.

### Git editor boundary

- `yzx-editor` resolves `editor.command`, disables the Helix bridge, waits for
  the executable, and restores Zellij's default background when it exits. It
  never calls `yzx-open` or a Zellij pane action.
- Managed sessions export `yzx-editor` through `EDITOR`, `VISUAL`, and
  `GIT_EDITOR`. `YZX_EDITOR` remains the effective editor for managed Yazi opens.
- `yzx-git` appends a LazyGit `os.edit*` overlay while retaining global and
  repository configuration.

---

## Startup boundary

Owned by `runtime/yzx/` (Nix substitutes paths; Rust owns wiring and `exec`).

1. `YAZELIX_STATE_DIR` + optional `YAZELIX_HELIX_BRIDGE_SESSION_ID` (when `yzx-hx`)
2. Effective `YZX_EDITOR` / `YAZELIX_EDITOR`; standard editor variables route through `yzx-editor`
3. Config home: `YAZELIX_CONFIG_HOME` → `XDG_CONFIG_HOME/yazelix` → `~/.config/yazelix`
4. Root settings → env (`YZX_OPEN_LOG`, welcome, popup chords/custom KDL, bar tray)
5. Mars packaged base + sparse user config homes
6. Zellij materialize (sidecar + patches) + status-bar cache path + plugin permission seeds  

Pre-`exec` failures → Yazelix diagnostics.  
After `exec` → Mars / Zellij / child tool.

`status` and `doctor` reuse this boundary without launching UI. `doctor` warns
if managed Helix TOML overrides reserved `Alt r`.

---

## Session isolation

| Mechanism | Purpose |
| --- | --- |
| `YAZELIX_HELIX_BRIDGE_SESSION_ID` | Opaque per top-level `yzx` launch with bridge-enabled `yzx-hx` |
| `ZELLIJ_SESSION_NAME` | Compared when a bridge registry recorded it |
| `YAZELIX_ZELLIJ_SESSION_NAME` | Yazi saves real session here before blanking `ZELLIJ_SESSION_NAME` for image previews; open/reveal restore it for Zellij control |
| `ZELLIJ_PANE_ID` → live tab membership | `yzx-open` reuses only a Helix registry whose pane is in the same `tab_id` |
| Helper-derived ids outside `yzx` | `yzx-hx` / `yzx-yazi` / `yzx-open` standalone must not hit a live window bridge |

Host editors (`nvim`, `/usr/bin/hx`, …) skip the Helix bridge entirely.

---

## Runtime contracts

**Index only:** behavior · owner · check · gap.  
Detail lives in Owners, checks, and the notes below.

`C9*` / `C11*` are splits of former mega-rows. Letter suffixes keep `C10` /
`C12` / `C13` stable. Checks may still share `checks/yzx-contracts.rs`.

### Front door and desktop

| ID | Contract | Owner | Check | Gap |
| --- | --- | --- | --- | --- |
| C1 | Front-door CLI, headless `enter`, and pre-exec diagnostics | `runtime/yzx/`, menu/tutor/config/open, screen, flake | launcher unit, `yzx-contracts`, manual PTY, helix/key parity, `nix build .#yazelix` | GUI launch |
| C8 | Desktop entry starts `yzx`; Mars app id matches `yzx.desktop` | `flake.nix`, `runtime/yzx/` | `yzx-contracts`, `nix build .#yazelix` | Desktop launch |

### Terminal, layout, shell, editor bridge

| ID | Contract | Owner | Check | Gap |
| --- | --- | --- | --- | --- |
| C2 | Mars packaged base + sparse user config; appearance preset as UI theme | `defaults/mars/config.toml`, flake, `yzx-config` | `yzx-contracts`, config tests | Visual |
| C3 | Layout sidebar template for swaps | `defaults/zellij/layout*.kdl` | `zellij-layout` | — |
| C4 | Packaged keys + guarded Zellij sidecar | `defaults/zellij/config.kdl`, `yzx-zellij-config` | `yzx-contracts` | Full keys |
| C5 | Managed Nu layering | `yzx-nu`, `defaults/nu/` | `yzx-contracts` | — |
| C6 | Managed Yazi (preview env, open logs, plugins) + `yzx-open` + zoxide | `defaults/yazi/`, `yzx-yazi`, `yzx-open` | contracts + materialization + open tests | Yazi UI |
| C7 | Helix bridge window/tab isolation (`session` + `tab_id`) | `yzx-open`, flake | `yzx-open` tests | Multi-window |
| C10 | Top bar tray, home-marker tabs, home-scoped new tabs, usage `tu` + cache | layout, config, runtime, tokenusage | layout + contracts | Visual bar |
| C12 | Welcome defaults and random pool | screen child, runtime, root config | screen tests + contracts | Animation |

### Popups (`C9*`)

| ID | Contract | Owner | Check | Gap |
| --- | --- | --- | --- | --- |
| C9a | Kitty protocol + `yzpp` packaged/loaded | `defaults/zellij/config.kdl`, flake | `yzx-contracts` | Visual |
| C9b | Role popups + popups tab remaps + margins + refresh hooks | config, runtime, `yzx-config` | contracts + keybinding tests | Visual |
| C9c | Custom `[popups.<id>]` argv + unique titles | `yzx-config`, runtime | custom popup tests + contracts | Visual |
| C9d | Agent hide keep-alive + custom command or provider bootstrap | `yzx-agent`, config | `yzx-contracts` | Provider UX |
| C9e | Git LazyGit + editor env + close-on-toggle | config, runtime | `yzx-contracts` | Visual |

### Config UI (`C11*`)

| ID | Contract | Owner | Check | Gap |
| --- | --- | --- | --- | --- |
| C11a | Root semantic schema + sparse persistence | `yzx-config`, `defaults/config.toml` | config tests + contracts | UI |
| C11b | Popups/Mars/Cursors/Zellij/Starship tabs; session Zellij active-file patch | `yzx-config` | config tests + contracts | Session live scalars |
| C11c | Helix tab + `yzx-hx` merge / `Alt r` / Steel | `yzx-config`, helix, `yzx-hx` | `helix-contracts` + config tests | UI |
| C11d | Keys read-only + Advanced open-file | `yzx-config` | Keys/Advanced tests, key parity | UI |

### Install

| ID | Contract | Owner | Check | Gap |
| --- | --- | --- | --- | --- |
| C13 | Fixed full/runtime packages + narrow Home Manager enable/package/optional files | `home-manager/`, flake, `yzx-config` | runtime contracts + `checks.home_manager` | Full HM switch |

### Notes

**C1:** Bare `yzx` → help; `launch` is explicit. Menu is a curated allowlist,
`run` reuses the prepared environment, and reveal is tab-local. `enter` reaches
the managed Zellij/Yazi/Helix workspace without Mars or display variables;
terminal-specific graphics and clipboard behavior remain host-owned.
Diagnostics stop before Mars/Zellij handoff.

**C2:** Saving `mars.appearance.preset` through `yzx config` switches the
Ratconfig palette live. Mars also reloads opacity, font size, line height,
scrollbar, and bell behavior in open windows; width and height apply to newly
created windows.

**C9:** Protocol/packaging (a), shared role wiring (b), user custom (c),
agent hide + bootstrap (d), Git close-on-toggle + editor env (e). Agent
cwd-mismatch restart is owned by `yzpp` and consumed via flake pin.

**C11:** Root schema (a), native tabs + session Zellij active-file patch (b),
Helix merge/Steel/`Alt r` (c), Keys/Advanced (d).

---

## Tradeoffs

**Pros:** small public surface; concrete semantic config; one Nix-composed
runtime; Mars isolated; Rust where process/files matter; layout checks in build.

**Cons:** `flake.nix` is heavy; Mars packaging weight; fork deps; Yazi
integration surface; user layering is intentionally incomplete.

**Current bet:** one owner per contract beats minimal file count. Nix is the
awkward but reproducible composition layer while children evolve.
