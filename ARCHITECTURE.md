# Architecture

Yazelix Next is a small Nix/Lix flake with one front door: **`yzn`**.

This repo owns the glue that makes Mars, the Yazelix Zellij fork, Yazi, and
Helix feel like one runtime. It is not a general terminal distro, a broad Home
Manager config system, or a main-Yazelix compatibility layer.

## Runtime chain

```text
yzn launch  →  Mars  →  yzn-welcome  →  Yazelix Zellij  →  Yazi sidebar + work panes
yzn enter   →  yzn-welcome  →  Yazelix Zellij  →  same layout
```

Bare `yzn` is `yzn launch`.

## Platforms

| Surface | Support |
| --- | --- |
| Package / app outputs | `x86_64` / `aarch64` × Linux / Darwin |
| macOS floor | `help`, `status`, `doctor`, `enter` |
| macOS `launch` | Mars path; issue-driven until hardware validation |
| Out of repo | App bundles, Homebrew, Ghostty packaging, broad terminal matrices |

## Commands

| Command | Role |
| --- | --- |
| `yzn` / `launch` | Mars then managed session |
| `enter` | Managed session in current terminal |
| `config` | Ratconfig UI |
| `menu` | Curated command palette |
| `tutor` | Guided lessons / native tutor hints |
| `screen` | Terminal screens / welcome styles |
| `reveal` | Path in managed Yazi sidebar |
| `status` / `doctor` | Paths and owned setup checks |
| `sponsor` | Sponsor URL |
| `env` | Managed shell only |
| `help` | Help |

---

## Owners

One owner per concern. Paths are the durable map.

### Package and install

| Path | Owns |
| --- | --- |
| `flake.nix` | Inputs, package graph, helpers, desktop entry, HM export |
| `home-manager/module.nix` | `programs.yazelix.enable` / package; optional config files; no default generation |

### Front door and helpers

| Path | Owns |
| --- | --- |
| `runtime/yzn/` | CLI, startup env, launch/enter handoff |
| `runtime/yzn-menu.rs` | Menu palette |
| `runtime/yzn-agent.rs` | Agent provider bootstrap |
| `runtime/yzn-yazi.rs` | Managed Yazi env, user init/keymap/plugins overlay |
| `runtime/yzn-nu.rs` | Managed Nu env/config + optional mise |
| `runtime/yzn-zellij-config.rs` | Packaged + guarded Zellij sidecar merge |
| `crates/yzn-open/` | Editor open, Helix bridge, reveal |
| `crates/yzn-tutor/` | Tutor CLI and lessons |
| `yazelix-screen` (child) | Screen styles; packaged as `yzn screen` |
| `checks/` | Build-time contract guards |

### Config UI

`crates/yzn-config/` is the Ratconfig host.

- Creates root, Mars, Zellij, Starship sources when missing
- Routes edits to the right file
- Helix / Advanced open-file rows; Keys read-only catalog
- Hidden reads for launch and custom-popup KDL render
- `KEY_BINDINGS` is the human key reference; `config.kdl` is the runtime owner

**Root semantic fields:** `open.log_level`, `shell.program`, `editor.command`,
`welcome.*`, `popup.*` margins, `keybindings.{config,agent,git,menu}`,
`[popups.<id>]`, `bar.widgets`.

### Packaged layout and tools

| Path | Owns |
| --- | --- |
| `mars.toml` | Default Mars window/font/appearance |
| `config.kdl` | Zellij keys, plugins load, popup wiring |
| `layout.kdl` / `layout.swap.kdl` | Sidebar + stacked panes, swap variants |
| `nu/` | Packaged Nu: carapace, zoxide, Starship |
| `yazi/` | Packaged Yazi: open via `yzn-open`, plugins, `Alt z` |
| `helix/config.toml` | Packaged Helix defaults |

### Child packages (not owned here)

| Child | Domain |
| --- | --- |
| Mars | Terminal |
| yazelix-zellij / helix | Multiplexer / editor forks |
| yazelix-zellij-popup (`yzpp`) | Popup lifecycle |
| yazelix-zellij-pane-orchestrator | Focus / sidebar walk |
| yazelix-zellij-bar | Top bar render + widgets |
| ratconfig | Config UI toolkit |
| yazelix-screen | Welcome / screen animations |

This repo packages them and applies product policy only.

### Shell dispatch

`yzn-shell` reads `shell.program` via `yzn-config`, then runs packaged `nu`
(through `yzn-nu`) or plain `bash` / `zsh` / `fish`.

---

## Module and repo boundaries

| Boundary | When |
| --- | --- |
| Local module | Large code, still ships with `yzn`, no independent user |
| Local crate | Cargo/binary/test isolation, still product glue |
| Separate repo | Independent users + release + stable artifact + low `yzn` coupling + real owner deletion here |

**Extraction counts only when this repo deletes or relinquishes an owner.**

Edge trim is smaller: child already owns a generic concept, and local glue can
die without a pure adapter layer.

---

## Config layering

Packaged first, unless a surface opts into native replacement.

```text
~/.config/yazelix-next/
  config.toml              # semantic root (Yazelix-owned)
  mars/config.toml         # full Mars native replace when present
  zellij/config.kdl        # guarded scalar sidecar
  zellij/plugins.kdl       # extra plugins only
  starship.toml
  nu/{env,config}.nu       # after packaged Nu
  helix/*                  # lazy; created on tab use
  yazi/{init.lua,keymap.toml,plugins/}
```

Override root with `YAZELIX_NEXT_CONFIG_HOME`.

| Surface | Layering |
| --- | --- |
| Root TOML | Created with defaults + contract state |
| Mars | User file replaces packaged when present |
| Nu | Packaged → optional mise → optional user Nu |
| Starship | User file if present; else empty/defaults for managed Nu |
| Helix | Packaged merge + optional user dir; `Alt r` reclaimed |
| Zellij | Packaged → guarded sidecar → runtime materialize; session saves can patch active file |
| Host `~/.config/{helix,yazi,starship}` | Not loaded by default |

---

## Startup boundary

Owned by `runtime/yzn/` (Nix substitutes paths; Rust owns wiring and `exec`).

1. State dir + optional Helix bridge session id  
2. Editor env from `editor.command`  
3. Config home resolution  
4. Read root settings (open log, welcome, keys, custom popups, bar)  
5. Mars config selection  
6. Zellij config materialize + bar cache + plugin permissions  

Pre-`exec` failures → Yazelix diagnostics.  
After `exec` → Mars / Zellij / child tool.

`status` and `doctor` reuse this boundary without launching UI.

---

## Session isolation

| Mechanism | Purpose |
| --- | --- |
| `YAZELIX_HELIX_BRIDGE_SESSION_ID` | Per top-level `yzn` launch with `yzn-hx` |
| `ZELLIJ_SESSION_NAME` / `YAZELIX_ZELLIJ_SESSION_NAME` | Session match; Yazi may blank Zellij name for previews |
| Pane → tab id checks in `yzn-open` | Same-tab bridge reuse only |
| Helper-derived ids outside `yzn` | No accidental bridge to a live window |

Host editors (`hx`, `nvim`, …) skip the Helix bridge.

---

## Runtime contracts

**Index only:** behavior · owner · check · gap.  
Detail lives in Owners, checks, and the notes below.

`C9*` / `C11*` are splits of former mega-rows. Letter suffixes keep `C10` /
`C12` / `C13` stable. Checks may still share `checks/yzn-contracts.rs`.

### Front door and desktop

| ID | Contract | Owner | Check | Gap |
| --- | --- | --- | --- | --- |
| C1 | Front-door CLI and pre-exec diagnostics | `runtime/yzn/`, menu/tutor/config/open, screen, flake | `yzn-contracts`, helix/key parity, unit tests, `nix build .#yzn` | GUI launch |
| C8 | Desktop entry starts `yzn` | `flake.nix` | `nix build .#yzn` | Desktop launch |

### Terminal, layout, shell, editor bridge

| ID | Contract | Owner | Check | Gap |
| --- | --- | --- | --- | --- |
| C2 | Mars packaged vs user config; appearance preset as UI theme | `mars.toml`, flake, `yzn-config` | `yzn-contracts`, config tests | Visual |
| C3 | Layout sidebar template for swaps | `layout*.kdl` | `zellij-layout` | — |
| C4 | Packaged keys + guarded Zellij sidecar | `config.kdl`, `yzn-zellij-config` | `yzn-contracts` | Full keys |
| C5 | Managed Nu layering | `yzn-nu`, `nu/` | `yzn-contracts` | — |
| C6 | Managed Yazi + `yzn-open` + zoxide | `yazi/`, `yzn-yazi`, `yzn-open` | contracts + materialization + open tests | Yazi UI |
| C7 | Helix bridge window/tab isolation | `yzn-open`, flake | `yzn-open` tests | Multi-window |
| C10 | Top bar tray, home tabs, usage cache | layout, config, runtime, tokenusage | layout + contracts | Visual bar |
| C12 | Welcome defaults and random pool | screen child, runtime, root config | screen tests + contracts | Animation |

### Popups (`C9*`)

| ID | Contract | Owner | Check | Gap |
| --- | --- | --- | --- | --- |
| C9a | Kitty protocol + `yzpp` packaged/loaded | `config.kdl`, flake | `yzn-contracts` | Visual |
| C9b | Role popups + remaps + margins + refresh hooks | config, runtime, `yzn-config` | contracts + keybinding tests | Visual |
| C9c | Custom `[popups.<id>]` argv + unique titles | `yzn-config`, runtime | custom popup tests + contracts | Visual |
| C9d | Agent hide keep-alive + provider bootstrap | `yzn-agent`, config | `yzn-contracts` | Provider UX |
| C9e | Git LazyGit + editor env + close-on-toggle | config, runtime | `yzn-contracts` | Visual |

### Config UI (`C11*`)

| ID | Contract | Owner | Check | Gap |
| --- | --- | --- | --- | --- |
| C11a | Root semantic schema + source creation | `yzn-config`, `config.toml` | config tests + contracts | UI |
| C11b | Mars/Zellij/Starship tabs; session Zellij active-file patch | `yzn-config` | config tests + contracts | Session live scalars |
| C11c | Helix tab + `yzn-hx` merge / `Alt r` / Steel | `yzn-config`, helix, `yzn-hx` | `helix-contracts` + config tests | UI |
| C11d | Keys read-only + Advanced open-file | `yzn-config` | Keys/Advanced tests, key parity | UI |

### Install

| ID | Contract | Owner | Check | Gap |
| --- | --- | --- | --- | --- |
| C13 | Narrow Home Manager enable/package/optional files | `home-manager/`, flake, `yzn-config` | `checks.home_manager` | Full HM switch |

### Notes

**C1:** Bare `yzn` → `launch`. Menu is a curated allowlist. Reveal is
tab-local. Diagnostics stop before Mars/Zellij handoff.

**C9:** Protocol and packaging (a), shared role wiring (b), user custom (c),
agent (d), Git (e).

**C11:** Root schema (a), native tabs + live Zellij patch (b), Helix (c),
Keys/Advanced (d).

---

## Tradeoffs

**Pros:** small public surface; concrete semantic config; one Nix-composed
runtime; Mars isolated; Rust where process/files matter; layout checks in build.

**Cons:** `flake.nix` is heavy; Mars packaging weight; fork deps; Yazi
integration surface; user layering is intentionally incomplete.

**Current bet:** one owner per contract beats minimal file count. Nix is the
awkward but reproducible composition layer while children evolve.
