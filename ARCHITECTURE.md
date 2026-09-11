# Architecture

Yazelix Nova is a small Nix/Lix flake with one front door: **`yzx`**.

This repo owns the glue that makes Rio, Nova Zellij, Yazi, and
Helix feel like one runtime. It is not a general terminal distro, a broad Home
Manager config system, or a main-Yazelix compatibility layer.

## Runtime chain

```text
yzx launch  →  Rio  →  yzx-welcome  →  Nova Zellij  →  Radar + work panes
yzx enter   →  yzx-welcome  →  Nova Zellij  →  same layout
yzx run     →  prepared Yazelix environment  →  exact child argv/status
yzx yazi-config materialize  →  private materializer  →  effective Yazi config path
```

Bare `yzx` prints help. `launch` is the only Rio route.
`enter` is the headless/SSH route and requires an interactive host terminal, not
a display server. `yazelix-no-helix` retains the Rio route and delegates
editing to an installed host command.
`no-yazi` variants keep the managed Yazi launcher and integration while
resolving a matching host `yazi`/`ya` pair. The two omission suffixes compose
into four explicit package and app outputs.

## Platforms

| Surface | Support |
| --- | --- |
| Four Rio/managed-Helix/managed-Yazi package and app combinations | `x86_64` / `aarch64` × Linux / Darwin |
| Headless / SSH floor | `enter` in a capable interactive host terminal; managed TUI only |
| macOS build evidence | Real `aarch64-darwin` runner builds all four packages and the Home Manager closure; no desktop entry |
| macOS interactive floor | `help`, `status`, `doctor`, `enter`, managed workspace, and host-editor delegation remain unverified |
| macOS full-package `launch` | Rio is packaged; GUI behavior remains unverified |
| Out of repo | App bundles, Homebrew, Ghostty packaging, broad terminal matrices |

## Commands

| Command | Role |
| --- | --- |
| `yzx` / `help` | Concise help; no implicit launch |
| `--version` | Package-owned exact Nova version |
| `launch` | Rio then managed session |
| `enter` | Managed session in current terminal |
| `run` | Structured command in the prepared runtime environment |
| `config` | Ratconfig UI |
| `yazi-config materialize` | Explicit-path Yazi config materialization for automation |
| `menu` | Curated command palette |
| `tutor` | Guided lessons / native tutor hints |
| `anima` | Terminal animations / welcome styles |
| `reveal` | Path in the active tab's persistent Yazi popup |
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
| `flake.nix` | Four fixed Rio/managed-Helix/managed-Yazi compositions, inputs, helpers, desktop entry, HM export |
| `home-manager/module.nix` | `programs.yazelix.enable` / package; optional config files; no default generation |

### Front door and helpers

| Path | Owns |
| --- | --- |
| `runtime/yzx/` | CLI, public Yazi materializer grammar/delegation, startup env, host-Yazi pair resolution, launch/enter handoff |
| `runtime/yzx-menu.rs` | Menu palette |
| `runtime/yzx-agent.rs` | Initial agent title, custom-command exec, provider bootstrap (`codex resume` → `grok` → `opencode` → `pi` → `claude --resume`), and one-time optional Codex Radar setup |
| `runtime/yzx-yazi.rs` | Managed Yazi process/env launch, active session appearance lookup, editor resolve, startup-picker and workspace-popup roles |
| `runtime/yzx-nu.rs` | Managed Nu layering; runtime-effective Starship config request |
| `runtime/yzx-zellij-config.rs` | Packaged + guarded Zellij scalar sidecar merge |
| `runtime/yzx/zellij.rs` | Plugin sidecar inject; launch materialize/patches |
| `crates/yzx-open/` | Editor open, Helix bridge, successful startup-picker removal, target-at-launch popup reveal, bounded diagnostics |
| `crates/yzx-yazi-config/` | Managed Yazi config-home materialization, native TOML layering, and runtime-only flavor projection |
| `crates/yzx-tutor/` | Tutor CLI and lessons |
| `runtime/yzx-helix.sh` (`yzx-hx`) | Effective Helix config + Steel wiring |
| `checks/` | Build-time contract guards |

### Config UI

`crates/yzx-config/` is the Ratconfig host.

- Supplies one stable source/path identity per field and reviewed root, Zellij,
  and Helix recommendation allowlists. Ratconfig owns Overview/All filtering,
  meaningful reduction thresholds, attention-state visibility, toggling, and
  All-scope search
- Exposes Rio as one exact native-file action. Rio owns its schema, validation,
  and reload behavior; Nova reserves only top-level `force-theme`
- Joins the packaged Helix baseline with observed native `config.toml` and
  `languages.toml` values as read-only generic rows. Helix publishes no stable
  machine-readable config catalog, so Yazelix does not mirror one or infer edit
  authority from TOML shape; exact file actions remain the persistence surface
- Resolves sparse override intent separately from baseline and effective values,
  declares editor capabilities independently of display types, and completes
  reloads by field identity rather than stale row position
- Seeds the complete native Rio TOML and any absent referenced adaptive themes
  at first use; root, Zellij, and Starship stay sparse
- Routes edits and true unset operations to the right file; Rio's user surface,
  Helix/Advanced open-file rows, and Keys remain read-only
- Resolves known config targets against the packaged Nix store root so
  Home Manager-owned sources stay read-only with exact module-option guidance
- Keeps localized invalid values as field intent. Wholly unsafe root documents
  become source diagnostics with an exact `config.toml` file action instead of
  preventing the UI from opening
- Hidden package-internal reads for launch + custom-popup KDL render
- `agent.popup.kdl` is an internal render path that routes custom managed agent
  argv through the packaged launcher
- `KEY_BINDINGS` is the human key reference; `defaults/zellij/config.kdl` is the runtime owner

#### Nova root schema inventory

Packaged `defaults/config.toml` owns every default below. The optional user file
stores only explicit overrides; `CONFIG_FIELDS` and `root_config.rs` own the
bounded catalog, validation, and sparse persistence unless another owner is named.
The root validator derives fixed leaves from that catalog, rejects unknown paths
before runtime or Ratconfig use, and delegates only `popups.<id>` to its dynamic
field validator.

`ROOT_CONFIG_RECOMMENDED_PATHS` contains `appearance.mode`, `shell.program`,
`shell.atuin`, `editor.command`, `agent.command`, the two ordinary welcome controls, the
managed action keys, and `bar.widgets`. Diagnostics (`open.log_level`), argument
and duration fine tuning, and popup geometry are All-only until they require
attention.
Configured custom popup leaves use Ratconfig's generic TOML rows and belong to
All; because every discovered leaf is explicit, Ratconfig also keeps it visible
in Overview. Absent optional leaves and unconfigured popup ids are not synthesized

| Root path | Type | Default | Effect | Applies |
| --- | --- | --- | --- | --- |
| `appearance.mode` | string enum | `dark` | Integrated Ratconfig, Rio, Zellij/bar, and new-Yazi appearance; writable Rio config uses native reload, while read-only Rio config freezes the current session and applies the saved mode everywhere on the next session | live as one unit or next session as one unit |
| `open.log_level` | string enum | `info` | `YZX_OPEN_LOG` diagnostics for managed opens | new opens |
| `shell.program` | string enum | `nu` | Packaged shell for new panes | new panes |
| `shell.atuin` | boolean | `true` | Atuin history and `Ctrl+r` search in managed shells | new shells |
| `editor.command` | executable string | `yzx-hx` | Yazi opens, config text edits, and Git clients | new opens |
| `forest.side` | string enum | `right` | Forest placement in managed Helix | next launch |
| `agent.command` | executable string or `auto` | `auto` | Managed agent popup command | next launch |
| `agent.args` | string array | `[]` | Arguments for a custom agent command | next launch |
| `welcome.enabled` | boolean | `true` | Enables the pre-Zellij splash | next launch |
| `welcome.style` | string enum | `random` | Selects the packaged splash style | next launch |
| `welcome.duration_seconds` | integer, 1–60 | `3` | Sets splash duration | next launch |
| `popup.side_margin` | non-negative integer | `1` | Left/right managed popup margin in cells | next launch |
| `popup.vertical_margin` | non-negative integer | `0` | Top/bottom managed popup margin in cells | next launch |
| `keybindings.config` | key chord or `false` | `Alt Shift K` | Config popup trigger | next launch |
| `keybindings.agent` | key chord or `false` | `Alt Shift L` | Agent popup trigger | next launch |
| `keybindings.git` | key chord or `false` | `Alt Shift J` | Git popup trigger | next launch |
| `keybindings.menu` | key chord or `false` | `Alt Shift M` | Menu popup trigger | next launch |
| `keybindings.screen` | key chord or `false` | `Alt Shift A` | Random full-screen visual trigger | next launch |
| `keybindings.sidebar` | key chord or `false` | `Alt Shift H` | Radar visibility | next launch |
| `keybindings.sidebar_focus` | key chord or `false` | `Ctrl y` | Forest open/refocus in managed Helix | next launch |
| `bar.widgets` | ordered string array | `editor`, `shell`, `term`, `codex_usage`, `cpu`, `ram` | Top-bar tray order; `BAR_WIDGET_VALUES` and `bar_widgets` own validation | next launch |

`custom_popups.rs` owns the dynamic `[popups.<id>]` namespace. An id starts
with an ASCII letter or `_`, then uses ASCII letters, digits, `_`, or `-`; the
packaged ids `config`, `agent`, `git`, `menu`, and `yazi` are reserved.

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
| `defaults/rio/` | Complete seed-once Rio window, font, cursor, effects, and adaptive-theme configuration; Nova reserves only top-level `force-theme` for integrated appearance |
| `defaults/zellij/config.kdl` | Zellij keys, plugin loads, popup wiring, Kitty protocol; leaves application-local `Alt r` routing to Helix and Yazi |
| `defaults/zellij/layout*.kdl` | Radar + stacked panes, open/closed swap |
| `defaults/nu/` | Packaged Nu: carapace, zoxide, and Starship invocation |
| `defaults/yazi/` | Popup initialization, opens via `yzx-open`, plugins, `Alt z` workspace retarget, and `Alt r` popup hide |
| `defaults/helix/config.toml` | Packaged defaults; `Alt r` reveal and `Ctrl r` reload; runtime composition adds the Forest binding |

### Packaged components (not owned here)

| Child | Domain |
| --- | --- |
| Nova Rio | Terminal and native configuration schema |
| Nova Zellij | Multiplexer fork |
| Nova Helix | Editor fork |
| Yazelix Forest | Managed Helix file tree and its native interaction behavior |
| zj-radar | Session and agent-attention rail, command pipe, and producer CLI |
| Zellij Popup (`yzpp`) | Popup lifecycle |
| Zellij Pane Orchestrator | Pane focus, Radar visibility, workspace state, and popup request routing |
| Nova Bar | Top bar rendering, widgets, palettes, and integrated KDL |
| zjstatus | WebAssembly renderer used by Nova Bar; host-theme and workspace-pipe fixes |
| Ratconfig | Config UI toolkit |
| Anima | Welcome / screen animations |
| Yazi Bistro | Complete pinned Yazi flavors, provenance, licenses, dark/light classification, and the packaged light default |
| auto-layout.yazi | Adaptive Yazi column layout |

This repo packages them and applies product policy only.

### Installed closure topology

`flake.nix` owns the package graph. The full package includes the terminal,
workspace, managed editor, and managed Yazi. Variant outputs remove managed
Helix and/or the bundled Yazi executable without changing the
remaining integration contracts. Nova's top-level outputs are thin command,
desktop-entry, and asset joins whose references pull in their selected runtime
graph.

### Shell dispatch

`yzx-shell` reads `shell.program` via `yzx-config`, then runs packaged `nu`
(through `yzx-nu`) or plain `bash` / `zsh` / `fish`.

Nix generates locked Atuin init files for Nushell, Bash, Zsh, and Fish.
`yzx-nu` appends the Nushell form after packaged config, successful host Mise
output, and user `nu/config.nu`. `yzx-shell` composes the other forms after
their native user startup files through Bash's explicit rc file, a temporary
Zsh `ZDOTDIR` layer, or Fish's post-config command. A shell-specific Atuin
search function makes user initialization the sole hook owner. The generated
forms disable Up-arrow and AI bindings, keep `Ctrl+r`, and honor
`ATUIN_NOBIND`. Managed init errors reach stderr without preventing shell
startup. Atuin owns its history database and native config. Carapace owns
Nushell external completion. Nova owns package selection, startup composition,
and the default-on policy.

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
  rio/config.toml          # complete native Rio config; seeded once with themes/
  zellij/config.kdl        # guarded scalar sidecar
  zellij/plugins.kdl       # extra plugins only
  starship.toml            # optional sparse prompt overrides
  nu/{env,config}.nu       # after packaged Nu
  helix/*                  # lazy; created on tab use
  yazi/{yazi.toml,theme.toml,package.toml,starship.toml,init.lua,keymap.toml,plugins/,flavors/}
```

Override root with `YAZELIX_CONFIG_HOME`.
Runtime state defaults to `$XDG_DATA_HOME/yazelix` or `YAZELIX_STATE_DIR`.

| Surface | Layering |
| --- | --- |
| Root TOML | Packaged semantic defaults → sparse explicit user overrides |
| Rio | Packaged complete config and themes → one-time copy to native files; `RIO_CONFIG_HOME` selects them; every field is user-owned except Nova's top-level `force-theme` projection |
| Nu | Packaged → optional host `mise activate nu` → optional user Nu |
| Starship | Packaged generated schema + `print-config --default` → Ratconfig discovery and bounded scalar editing → sparse user overrides over Nova's `character.format` marker → runtime-effective TOML |
| Helix | Packaged Yazelix default → read-only Ratconfig baseline/observed rows → recursive sparse user override → reserved `keys.normal.A-r` restoration; dynamic languages and Steel stay native-file surfaces |
| Yazi | Packaged TOML → recursive user tables + replacing scalars/arrays → managed opener/Git fetchers → runtime-only root-mode flavor projection; optional complete Starship config replacement |
| Zellij | Packaged → guarded scalar sidecar → runtime materialize under state dir |
| Host `~/.config/{helix,yazi,starship}` | Not loaded by default |

### Zellij sidecars

`zellij/config.kdl` is a **first-token denylist**, not a full KDL parser. Uncommented
top-level ownership nodes are rejected, including:

`keybinds`, `default_shell`, `default_layout`, `layout_dir`, `layout`, `plugins`,
`load_plugins`, `support_kitty_keyboard_protocol`, `env`, `session_name`,
`attach_to_session`.

The sidecar is optional and sparse. Ratconfig displays ten effective scalar
defaults without creating it, including separate dark and light theme pickers
derived from the same 41 identities embedded by the pinned Zellij package.
Packaged configuration owns the inherited `ansi` / `gruvbox-light` pair, while
custom native names remain accepted per field. Assignment presence is explicit
intent, and removing the final assignment removes the sidecar. A legacy static
`theme` assignment remains untouched in the user file but is diagnosed and
omitted from runtime materialization.

Yazelix recommends six high-use fields in the Zellij Overview and leaves four
fine-tuning fields in All. Ratconfig keeps explicit, invalid, externally
managed, and diagnosed values in Overview and searches the All inventory.
Safe opaque leaves remain byte-for-byte unchanged and are reached through the
exact `zellij/config.kdl` file action without a synthetic field or informational
diagnostic per leaf. Advanced diagnostics are reserved for ignored, invalid,
structurally unsafe, or integration-owned state. Guarded diagnostics name the
Yazelix integration owner. This uses Ratconfig's existing views, search, file
actions, and diagnostics without extending its API or Yazelix's typed schema.

`zellij/plugins.kdl` accepts only `plugins` / `load_plugins` and must not
redeclare Yazelix-owned plugin ids (`yzpp`, `yazelix_pane_orchestrator`,
`radar`, or `radar_controller`).

Inside a managed session, `yzx config` Zellij scalar saves and resets also patch
`$YAZELIX_STATE_DIR/zellij/config.kdl` (watched active file) without wiping
launch patches. Pane frames, rounded corners, copy-on-select, and clipboard
target reconfigure the active session; mouse mode, scrollback size, styled
underlines, and startup tips need a new session. Live-capable root appearance
uses the fork's native dark/light action for the addressed session. The fork
sends the resulting mode event to the bar, which chooses its internal palette.

### Helix

- `yzx-hx` writes effective config under `$YAZELIX_STATE_DIR/helix/config.toml`
  each launch: packaged template deep-merged with optional sparse overrides from
  `~/.config/yazelix/helix/config.toml`, then `keys.normal.A-r` reclaimed
  for `yzx reveal`.
- If the user Helix dir has `config.toml`, `languages.toml`, and/or a Steel pair
  (`helix.scm` + `init.scm`), that directory is native config. Packaged Steel
  remains active unless the complete user pair is composed through the private
  overlay below.
- Packaged Steel exposes `:yzx-new-shell` and starts the fork's bounded
  authenticated loopback transport. Yazelix owns token generation, atomic
  private registry publication and lookup under validated session path components,
  same-session/tab instance selection that prefers the caller's own Helix pane,
  and the TCP client; the fork passes authenticated actions to Steel on the
  editor thread without owning Zellij, Yazi, or registry policy. Closed bridges,
  stale-token authentication failures, and invalid or mismatched response frames
  leave later registry candidates eligible.
- An explicit user Steel pair runs through a private state overlay. Its
  `helix.scm` remains the command module and its `init.scm` loads after the
  packaged bridge init, so user plugins compose without replacing the managed
  bridge.
- Packaged bindings: `Alt r` reveal (reserved), `Ctrl r` reload (user-overridable).
- Ratconfig recommends eight packaged or integration-owned rows and searches
  every packaged or explicitly observed native row. It makes no complete-schema
  claim and routes edits to exact Helix files.
- `hx` and `yzx-hx` select managed Helix; other `editor.command` values skip its
  bridge.
- The no-Helix package replaces both managed names with an unavailable command;
  another executable name or an absolute host Helix path stays host-owned.

### Git editor boundary

- `yzx-editor` resolves `editor.command`, disables the Helix bridge, waits for
  the executable, and restores Zellij's default background when it exits. It
  never calls `yzx-open` or a Zellij pane action.
- Config file actions and Git clients invoke `yzx-editor`; the config UI drops
  the session's editor snapshot so each action resolves the current setting.
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
4. Root settings → env and launch args (`YZX_OPEN_LOG`, welcome, Zellij theme mode, popup chords/custom KDL, bar tray)
5. Rio native config home
6. Zellij materialize (sidecar + patches) + status-bar cache path + isolated
   permission seeds for the exact bundled plugin artifacts, including Radar

Pre-`exec` failures → Yazelix diagnostics.  
After `exec` → Rio / Zellij / child tool.

`status` and `doctor` reuse configuration resolution and validation without
materializing runtime files, initializing Rio or seeding plugin permissions
(`DIAGNOSTICS-READ-ONLY-001`). The scalar Zellij helper returns merged text;
runtime preparation owns its final write. Existing launch/run commands retain
initialization. `doctor` reports missing runtime files and warns if managed
Helix TOML overrides reserved `Alt r`; `status --json` retains schema version 1.

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
| C8 | Desktop entry starts `yzx`; Rio app id matches `yzx.desktop` | `flake.nix`, `runtime/yzx/` | `yzx-contracts`, `nix build .#yazelix` | Desktop launch |

### Terminal, layout, shell, editor bridge

| ID | Contract | Owner | Check | Gap |
| --- | --- | --- | --- | --- |
| C2 | Complete Rio config is seeded once and selected through `RIO_CONFIG_HOME`; Nova owns only top-level `force-theme`, using native reload when writable and coherent next-session fallback when read-only | `defaults/rio/config.toml`, runtime, `yzx-config` | launcher/config unit tests, `yzx-contracts` | Visual dogfood |
| C3 | One Radar controller with per-tab views across layout swaps | `defaults/zellij/config.kdl`, `defaults/zellij/layout*.kdl`, zj-radar | `zellij-layout`, `yzx-contracts`, isolated startup benchmark | Fresh-session state-retention dogfood |
| C4 | Packaged keys + guarded Zellij sidecar | `defaults/zellij/config.kdl`, `yzx-zellij-config` | `yzx-contracts` | Full keys |
| C5 | Managed Nu layering | `yzx-nu`, `defaults/nu/` | `yzx-contracts` | — |
| C6 | Managed Yazi layering, public noninteractive materialization, `yzx-open`, and zoxide | `defaults/yazi/`, `runtime/yzx-yazi.rs`, `runtime/yzx/`, `crates/yzx-yazi-config/`, `crates/yzx-open/` | host-Yazi contracts + materialization + open tests | Yazi UI |
| C7 | Helix bridge window/tab isolation (`session` + `tab_id`) | `yzx-open`, flake | `yzx-open` tests | Multi-window |
| C10 | Top bar tray, home-marker tabs, home-scoped new tabs, usage `tu` + cache | layout, config, runtime, tokenusage | layout + contracts | Visual bar |
| C12 | Welcome defaults and random pool | Anima, runtime, root config | screen tests + contracts | Animation |

### Popups (`C9*`)

| ID | Contract | Owner | Check | Gap |
| --- | --- | --- | --- | --- |
| C9a | Kitty protocol + `yzpp` packaged/loaded | `defaults/zellij/config.kdl`, flake | `yzx-contracts` | Visual |
| C9b | Role popups + popups tab remaps + margins + refresh hooks | config, runtime, `yzx-config` | contracts + keybinding tests | Visual |
| C9c | Custom `[popups.<id>]` argv + unique titles | `yzx-config`, runtime | custom popup tests + contracts | Visual |
| C9d | Agent hide keep-alive + custom command or provider bootstrap + supported stock Radar setup | `yzx-agent`, config | `yzx-contracts` | Provider UX |
| C9e | Git LazyGit + editor env + close-on-toggle | config, runtime | `yzx-contracts` | Visual |

### Config UI (`C11*`)

| ID | Contract | Owner | Check | Gap |
| --- | --- | --- | --- | --- |
| C11a | Root semantic schema + sparse persistence | `yzx-config`, `defaults/config.toml` | config tests + contracts | UI |
| C11b | Popups; Rio exact-file action; owner-complete Starship/Yazi inventories; curated Zellij with exact-file fallback and session patching; Nushell selection plus Advanced file actions | Owner catalogs + Ratconfig + `yzx-config` | config tests + contracts | Session live scalars |
| C11c | Helix observed native rows + exact files + `yzx-hx` merge / `Alt r` / Steel | `yzx-config`, Helix, `yzx-hx` | `helix-contracts` + config tests | Owner catalog unavailable |
| C11d | Keys read-only + Advanced open-file | `yzx-config` | Keys/Advanced tests, key parity | UI |

### Install

| ID | Contract | Owner | Check | Gap |
| --- | --- | --- | --- | --- |
| C13 | Four fixed Rio/managed-Helix/managed-Yazi package combinations + narrow Home Manager enable/package/optional files | `home-manager/`, flake, `yzx-config` | Rio/no-Helix/host-Yazi contracts + `checks.home_manager` | Full HM switch |

### Notes

**C1:** Bare `yzx` → help; `launch` is explicit. Menu is a curated allowlist,
`run` reuses the prepared environment, and reveal is tab-local. `enter` reaches
the managed Zellij/Yazi workspace and selected editor without starting Rio or using display
variables; terminal-specific graphics and clipboard behavior remain host-owned.
Diagnostics stop before Rio/Zellij handoff.

**C2:** The first config or runtime preparation copies the complete packaged
Rio config and its adaptive themes only when `rio/config.toml` is absent.
Ratconfig exposes only the exact file; later edits are user-owned except for
Nova's top-level `force-theme` projection. With a writable file, `yzx launch`
omits Rio's theme override and a Ratconfig save updates Rio through its native
watcher, Zellij through its native action, and the bar through Zellij's mode
event. Each new managed Yazi reads the active session mode and projects the
selected flavor into generated `theme.toml`, leaving user and Home Manager
sources unchanged. With a read-only Rio file, the session captures one mode,
Rio receives it at launch, and every managed component waits for the next
session before changing. The packaged light fallback and flavor classification
remain owned by Yazi Bistro. A custom complete Rio file must provide an
adaptive theme pair to participate.

**C9:** Protocol/packaging (a), shared role wiring (b), user custom (c),
agent hide + bootstrap (d), Git close-on-toggle + editor env (e). Agent
cwd-mismatch restart is owned by `yzpp` and consumed via flake pin.

**C11:** Root schema (a), native tabs + session Zellij active-file patch (b),
Helix merge/Steel/`Alt r` (c), Keys/Advanced (d).

---

## Tradeoffs

**Pros:** small public surface; concrete semantic config; one Nix-composed
runtime; Rio isolated; Rust where process/files matter; layout checks in build.

**Cons:** `flake.nix` is heavy; fork deps; Yazi
integration surface; user layering is intentionally incomplete.

**Current bet:** one owner per contract beats minimal file count. Nix is the
awkward but reproducible composition layer while children evolve.
