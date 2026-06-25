# Architecture

Yazelix Next is a small Nix/Lix flake that installs one user-facing command,
`yzn`. The runtime path is intentionally narrow:

```text
yzn -> Mars -> Yazelix Zellij fork -> Yazi sidebar + stacked work panes
```

The repo owns the glue that makes those pieces behave like one runtime. It does
not try to be a general terminal distribution, Home Manager module, or Yazelix
compatibility layer.

## Owners

`flake.nix` is the package graph and composition owner. It pins external inputs,
builds small local Rust helpers, substitutes local config templates, installs
the desktop entry, and exposes the `yzn` package/app.

`mars.toml` is the terminal visual config owner. It sets the Mars window, font,
cursor, bell, quit, and theme behavior used by `yzn`.

`config.kdl`, `layout.kdl`, and `layout.swap.kdl` are the Zellij behavior
owners. They set the shell, Zellij-native `Ctrl Alt` mode keys, direct
`Ctrl Alt h/j/k/l` movement, the `Alt m` pane binding, the `Alt Shift h`
sidebar toggle, the Yazi sidebar tab, and the open/closed sidebar swap layouts.

`nu/` is the packaged Nushell config owner. It initializes carapace, zoxide, and
starship and disables the normal Nushell banner and prompt indicators.

`runtime/yzn-nu.rs` is the Nushell runtime-config owner. It writes the runtime
`env.nu` and `config.nu` files, layers optional user config from
`~/.config/yazelix-next/nu`, and then execs Nushell.

`yazi/` is the file-manager config owner. It enables the selected Yazi plugins
and routes file opens through `yzn-open`.

`crates/yzn-open/` is the editor-open owner. It sends file and directory open
requests to the live Yazelix Helix bridge when available, and otherwise opens a
managed `yzn-hx` pane through Zellij. It also owns bounded diagnostics for
Yazi-to-Helix open behavior.

`checks/` owns build-time contract guards. `zellij-layout.rs` validates Zellij
layout swaps, and `yzn-contracts.rs` validates Nushell config layering.

## Config Layering

Packaged config comes first. User config is narrow and explicit:

```text
~/.config/yazelix-next/nu/env.nu
~/.config/yazelix-next/nu/config.nu
```

`YAZELIX_NEXT_CONFIG_HOME` can point at another config root. Normal Nushell
config is not loaded by default, which keeps `yzn` reproducible and avoids
ambient user shell behavior changing the runtime.

## Session Isolation

Each top-level `yzn` launch creates one opaque
`YAZELIX_HELIX_BRIDGE_SESSION_ID`. Zellij, Yazi, Helix, and `yzn-open` inherit
that id inside the window, so Yazi opens can only reuse Helix bridge registries
from the same session. `yzn-open` also compares `ZELLIJ_SESSION_NAME` when a
registry records it, which prevents two Zellij windows with copied session state
from sharing an editor pane.

Helpers launched outside `yzn` do not fall back to the shared literal `yzn`
session id. `yzn-hx`, `yzn-yazi`, and `yzn-open` derive isolated helper ids, so
standalone helper use cannot accidentally target a bridge from a live `yzn`
window.

## Runtime Contracts

| ID | Contract | Owner | Check | Missing Coverage |
| --- | --- | --- | --- | --- |
| C1 | `yzn` launches the pinned Mars/Zellij runtime | `flake.nix` | `nix build .#yzn` | GUI launch remains manual dogfooding |
| C2 | Mars uses the packaged visual config | `mars.toml` | `nix build .#yzn` materializes `MARS_CONFIG_HOME` | Visual correctness remains manual dogfooding |
| C3 | Zellij layout has the sidebar template required by swaps | `layout.kdl`, `layout.swap.kdl` | `checks/zellij-layout.rs` runs during build | None for the current template/swap contract |
| C4 | Zellij-native mode keys use `Ctrl Alt`, move mode is unbound, `Alt m` opens a pane for the swap layout to stack, and `Alt Shift h` toggles the sidebar swap | `config.kdl` | `checks/yzn-contracts.rs` validates the packaged config | Full key behavior remains manual dogfooding |
| C5 | Nushell loads packaged config first, then optional user config | `runtime/yzn-nu.rs`, `nu/` | `checks/yzn-contracts.rs` runs in `nix flake check` | None for current layering behavior |
| C6 | Yazi opens paths through `yzn-open` with bounded diagnostics | `yazi/yazi.toml`, `crates/yzn-open/` | `cargo test` through `yzn-open` package build | Full Yazi UI behavior remains manual dogfooding |
| C7 | Helix bridge reuse stays inside the current `yzn` window | `crates/yzn-open/`, `flake.nix` | `yzn-open` Rust tests cover session and Zellij-window mismatch | Full multi-window GUI behavior remains manual dogfooding |
| C8 | Desktop entry starts `yzn` | `flake.nix` | `nix build .#yzn` packages the desktop file | Desktop environment launch remains manual dogfooding |

## Pros

- The public surface is small: install or run the flake, then start `yzn`.
- Nix owns dependency composition, so Mars, Zellij, Helix, Yazi, plugins, fonts,
  desktop entry assets, and helper binaries are versioned together.
- Mars is an isolated terminal concern. The rest of the runtime can stay focused
  on Zellij/Yazi/Helix behavior.
- Rust owns runtime glue where quoting, file writes, sockets, process execution,
  and fallback behavior matter.
- Zellij layout validation runs inside the build, so a broken swap/template
  relationship fails before the package is installed.
- User config layering is intentionally narrow, which keeps reproducibility
  higher than loading the user's normal shell and Yazi environments.

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
- User config layering exists for Nushell but not yet for Yazi or Zellij.

## Current Tradeoff

The architecture favors a small, working vertical slice over a minimal file
count. The biggest simplicity win is not fewer source lines in isolation; it is
having one owner for each contract and no duplicated compatibility surface. Nix
is the least elegant part of the stack, but it is also the part that keeps the
runtime installable and reproducible while Mars, Zellij, Helix, Yazi, and the
helpers evolve together.
