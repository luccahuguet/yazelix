# Yazelix Next

Small start: a Nix flake that installs `yzn`, a conflict-free dev command that
opens Mars with a Yazi-first layout that becomes a sidebar plus stacked work
panes, a bridge-enabled Yazelix Helix editor, reef cursor colors, and the
Yazelix Zellij fork. The top bar uses a rendered Yazelix Zellij Bar tray with
editor, shell, terminal, Codex, CPU, RAM, and version widgets, with bundled
`tu` for the Codex usage widget. `Alt Shift K` toggles the config popup, and
`Alt Shift J` toggles a managed LazyGit popup through the standalone Yazelix
Zellij Popup plugin.

## Run

```sh
nix run
nix run .#yzn
nix run .#yzn -- help
nix run .#yzn -- config
nix run .#yzn -- enter
nix run .#yzn -- launch
```

`yzn help` prints help, `yzn config` opens the Ratconfig UI, `yzn
enter` starts the managed Zellij runtime inside the current terminal, and `yzn
launch` opens Mars first. Bare `yzn` defaults to `yzn launch`.

## Install

```sh
nix profile add --refresh /absolute/path/to/yazelix-next
yzn
```

Profile installs include `bin/yzn` and a `Yazelix Next` desktop entry.

## Update

```sh
nix profile upgrade --refresh yazelix-next
```

## Config

`yzn config` opens the Ratconfig UI in the current terminal and creates
this owned semantic config file when it is missing:

```text
~/.config/yazelix-next/config.toml
```

The first config slice controls `open.log_level`, which sets the managed
`YZN_OPEN_LOG` level used by Yazi-to-Helix opens. Values are `off`, `error`,
`info`, and `debug`.

## Mars Config

`yzn` uses the packaged Mars config unless this native Mars config exists:

```text
~/.config/yazelix-next/mars/config.toml
```

Set `YAZELIX_NEXT_CONFIG_HOME` to use a different Yazelix Next config root.
The Mars config controls terminal preferences; `yzn` still owns the Mars launch
command and the managed Zellij runtime.

## Zellij Config

`yzn` owns the Zellij shell, keybindings, layout, and plugin/runtime spine. Safe
native Zellij preferences can be appended with this sidecar:

```text
~/.config/yazelix-next/zellij/config.kdl
```

The sidecar is a simple guardrail, not a KDL merge engine. It is rejected before
launch when an uncommented line starts with integration-critical ownership such
as `keybinds`, `default_shell`, `default_layout`, `layout`, `plugins`,
`load_plugins`, `session_name`, or `attach_to_session`.

The packaged config enables Zellij's Kitty keyboard protocol for modified key
chords such as `Alt Shift J` and `Alt Shift K`.

## Nushell Config

`yzn` does not read normal Nushell config. It loads packaged Yazelix Next
`nu/env.nu` and `nu/config.nu` first, then optional user files:

```text
~/.config/yazelix-next/nu/env.nu
~/.config/yazelix-next/nu/config.nu
```

The same `YAZELIX_NEXT_CONFIG_HOME` root applies here.

## Starship Config

`yzn` sets `STARSHIP_CONFIG` to this native Starship config when it exists:

```text
~/.config/yazelix-next/starship.toml
```

Otherwise it uses an empty config, so normal `~/.config/starship.toml` does not
affect managed shells. The file uses Starship TOML; user `nu/config.nu` can
still override prompt variables for advanced cases. `format` controls the left
prompt, and `right_format` controls the right prompt.

## Editor Opens

Yazi opens files through the packaged `yzn-open` Rust helper. If no Helix bridge
is live, `yzn-open` opens `yzn-hx` in a Zellij pane. If the Helix bridge is
live, `yzn-open` sends the file or directory open request to that editor.
Inside the packaged Yazi sidebar, `Alt z` opens a zoxide picker and sends the
selected directory through `yzn-open`.

`yzn-open` writes bounded diagnostics to
`${YAZELIX_STATE_DIR}/logs/yzn-open.log` and keeps one rotated
`yzn-open.log.1` file. Managed `yzn` sessions set `YZN_OPEN_LOG` from
`open.log_level` in `config.toml`; the default is `info`.

## Keybindings

`Ctrl p/t/n/q` are the high-frequency Zellij controls. The rest of the native
Zellij layer uses `Ctrl Alt`, leaving most plain `Ctrl` keys available to
Helix, Nushell, Yazi, and terminal programs.

| Key | Action |
| --- | --- |
| `Ctrl Alt g/s/o` | lock, search, session |
| `Ctrl p/t/n/q` | pane, tab, resize, quit |
| `Ctrl Alt h/j/k/l` | move tab left, move pane down/up, move tab right |
| `Alt m` | new pane in the stacked layout |
| `Alt z` | Yazi zoxide jump into the managed editor |
| `Alt Shift J` | toggle the LazyGit popup |
| `Alt Shift K` | toggle the config popup |
| `Alt Shift h` | show or hide the Yazi sidebar |

Move mode is intentionally unbound.

## Hack On Mars

```sh
nix run --override-input mars ../mars
nix run --override-input yazelixZellij ../yazelix-zellij
nix run --override-input yazelixHelix ../yazelix-helix
nix run --override-input yazelixZellijPopup ../yazelix-zellij-popup
nix run --override-input yazelixZellijBar ../yazelix-zellij-bar
```

## LOC Scorecard

Counts owned project files by language with `wc -l`.

```sh
wc -l .gitignore AGENTS.md README.md CHANGELOG.md ARCHITECTURE.md flake.nix packaging/tokenusage.nix config.toml mars.toml config.kdl layout.kdl layout.swap.kdl nu/config.nu nu/env.nu helix/config.toml yazi/init.lua yazi/keymap.toml yazi/plugins/sidebar-status.yazi/main.lua yazi/plugins/zoxide-editor.yazi/main.lua yazi/yazi.toml crates/yzn-config/Cargo.toml crates/yzn-config/src/main.rs crates/yzn-open/Cargo.toml crates/yzn-open/src/main.rs checks/zellij-layout.rs checks/yzn-contracts.rs runtime/yzn-nu.rs runtime/yzn-zellij-config.rs
```

| Language | Files | Lines |
| --- | --- | ---: |
| Ignore | `.gitignore` | 1 |
| Markdown | `AGENTS.md`, `README.md`, `CHANGELOG.md`, `ARCHITECTURE.md` | 510 |
| Nix | `flake.nix`, `packaging/tokenusage.nix` | 456 |
| TOML | `config.toml`, `mars.toml`, `helix/config.toml`, `yazi/yazi.toml`, `yazi/keymap.toml`, `crates/yzn-config/Cargo.toml`, `crates/yzn-open/Cargo.toml` | 127 |
| KDL | `config.kdl`, `layout.kdl`, `layout.swap.kdl` | 153 |
| Nu | `nu/config.nu`, `nu/env.nu` | 11 |
| Lua | `yazi/init.lua`, `yazi/plugins/sidebar-status.yazi/main.lua`, `yazi/plugins/zoxide-editor.yazi/main.lua` | 131 |
| Rust | `crates/yzn-config/src/main.rs`, `crates/yzn-open/src/main.rs`, `checks/zellij-layout.rs`, `checks/yzn-contracts.rs`, `runtime/yzn-nu.rs`, `runtime/yzn-zellij-config.rs` | 2109 |
| Total | owned project files | 3498 |
