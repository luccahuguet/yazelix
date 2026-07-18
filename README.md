# Yazelix Nova Beta

<div align="center">
  <img src="assets/logo.png" alt="Yazelix logo" width="200"/>
</div>

Yazelix Nova is a Nix-packaged terminal workspace built around Mars, the
Yazelix Zellij fork, Yazi, Nushell, Git tools, and an optional coding agent. It
uses managed Helix by default (`editor.command` can select an installed terminal
editor). `yzx launch` opens the desktop workspace through Mars, while
`yzx enter` runs the same Yazi-first workspace in a capable terminal or over SSH.
Packaged defaults make the first run configuration-free.

## Preview

![Yazelix Nova workspace](assets/screenshots/nova_workspace.png)

## Install and launch

Yazelix requires Nix with flakes enabled. `launch` opens the packaged Mars window
in a graphical session, while `enter` starts the same workspace in the current
terminal or over SSH.

Nova remains in the v1 beta series. The `stable` branch is the recommended
install and update channel, not the final v1 release. It advances from a checked
and dogfooded `main` revision at most once per week, with earlier promotions
reserved for urgent fixes. Use `main` for the development channel or an
immutable `nova-v*` tag for an exact release.

Linux is the dogfooded platform. CI builds all packages and a Home Manager
activation on `aarch64-darwin`, while interactive macOS use and the Mars GUI
remain unverified.

### Try without installing

```sh
nix run github:luccahuguet/yazelix/stable -- launch
nix run github:luccahuguet/yazelix/stable#runtime -- enter
```

If the one-off launch fails, inspect the owned runtime setup with:

```sh
nix run github:luccahuguet/yazelix/stable -- doctor
```

### Install in a Nix profile

```sh
nix profile add --refresh github:luccahuguet/yazelix/stable
yzx launch
```

### Install with Home Manager

Use the [Home Manager module](docs/installation.md#home-manager) for a
declarative install.

### Moving from Yazelix Classic

Classic v17.12 translates mutable Classic `settings.jsonc` or `config.toml`
files into Nova configuration. It does not rewrite Home Manager declarations
or Home Manager-owned files. Run the bridge once when you need to preserve
mutable Classic settings:

```sh
nix run github:luccahuguet/yazelix/v17.12#yazelix -- launch
```

If your Classic settings match packaged defaults, start with Nova's packaged
defaults and move straight to `stable`. Home Manager users must replace
Classic-only options with Nova's narrow module surface before switching.

The Nova cutover intentionally replaces the old `main` history. Existing Git
clones should be replaced with a fresh clone rather than updated with an
ordinary pull. Classic remains available at the frozen `classic` branch, while
the immutable `v17.12` tag remains the migration and rollback bridge.

## First five minutes

Start the guided tour after launching Yazelix:

```sh
yzx tutor begin
```

`yzx help` lists every command. `yzx doctor` checks the owned runtime setup
without opening Mars or Zellij. Inside Yazelix, press `Alt Shift M` to open the
command palette, which includes both help and tutor entries.

### Ratconfig

Press `Alt Shift K` to open Ratconfig:

| Key | Action |
| --- | --- |
| `1`-`9` | Jump to a tab; `8` opens Yazi settings and `9` opens the key reference |
| `Tab` / `Shift-Tab`, `h` / `l` | Change tabs |
| `j` / `k`, `/` | Move through rows or search All settings |
| `a` | Switch between Core and All |
| `e`, `Enter`, `Space` | Run the selected row's contextual action |
| `u`, `q` | Reset the selected setting or quit |

The footer lists the selected row's controls.

### Workspace keys

Yazelix carries Helix/Vim's `h/j/k/l` motion model through the workspace:

| Layer | `h` | `j` | `k` | `l` |
| --- | --- | --- | --- | --- |
| Helix/Vim normal mode | Move cursor left | Move cursor down | Move cursor up | Move cursor right |
| `Alt` | Focus left or previous tab | Focus down | Focus up | Focus right or next tab |
| `Ctrl Alt` | Move tab left | Move pane down | Move pane up | Move tab right |

The default `Alt Shift H/J/K/L` row groups four workspace surfaces:

```text
H          J      K          L
sidebar    Git    Ratconfig  agent
```

Yazi and the menu use their initials:

- `Alt Shift Y` toggles the full Yazi popup.
- `Alt Shift M` toggles the command menu.

Press a popup's key again to close or hide it. Other useful bindings are:

| Scope | Key | Action |
| --- | --- | --- |
| Workspace | `Ctrl q` | Quit the Yazelix session |
| Workspace | `Alt m` | Open a new pane |
| Workspace | `Alt Shift F` | Toggle the focused pane fullscreen |
| Workspace | `Ctrl y` | Toggle focus between the editor and Yazi sidebar |
| Workspace | `Alt 1-9` | Go directly to tab 1-9 |
| Editor | `Alt r` | Reveal the current editor file in Yazi |
| Yazi | `Alt z` | Retarget the tab workspace with zoxide |

Managed Helix supplies the editor binding. Terminal editors can bind the same
`yzx reveal` command; see [Configuration](docs/configuration.md#editor-and-file-opens)
for Neovim and terminal Emacs examples.

Ratconfig's Keys tab is the complete packaged reference, and
`defaults/zellij/config.kdl` remains the runtime source.

## Commands

| Command | Purpose |
| --- | --- |
| `yzx`, `yzx help` | Print command help |
| `yzx --version` | Print the exact package-owned Yazelix version |
| `yzx launch [zellij-args...]` | Open Mars first, then start managed Zellij |
| `yzx enter [zellij-args...]` | Start managed Zellij in the current terminal |
| `yzx run <program> [args...]` | Run exact argv inside the prepared Yazelix environment |
| `yzx config` | Open the Ratconfig-backed config UI |
| `yzx menu` | Open the command palette |
| `yzx doctor` | Check owned runtime setup without launching Mars or Zellij |
| `yzx status` | Print config/runtime paths and selected settings |
| `yzx status --json` | Print the versioned machine-readable status record |
| `yzx env` | Open the managed shell without launching the UI |
| `yzx tutor [lesson]` | Print guided Yazelix lessons |
| `yzx screen [style]` | Show a terminal welcome screen |
| `yzx reveal <target>` | Reveal a file or directory in the managed Yazi sidebar |

## Packages and platforms

The default package includes Mars and managed Helix, and opens the full
graphical workspace with `yzx launch`. `yazelix-no-helix` keeps Mars and the
workspace while delegating editing to an installed terminal editor. The fixed
`runtime` package keeps the same `yzx` command, managed tools, and configuration
without Mars or desktop assets.

See [Installation and packages](docs/installation.md) for package variants,
platform support, SSH use, measured sizes, Home Manager, and updates.

## Configuration

`yzx config` opens Ratconfig over the managed tree at
`~/.config/yazelix/`. Yazelix inherits packaged defaults and persists only
explicit overrides. Core shows the settings most users need. All includes the
complete inventory.

See [Configuration](docs/configuration.md) for settings, popups, native files,
Yazi plugins, cursor ownership, and editor behavior.

## Development

From a local checkout, use:

```sh
nix run .#yazelix -- launch
nix run .#runtime -- enter
```

See [Development](docs/development.md) for CI and local checks,
[Architecture](ARCHITECTURE.md) for ownership boundaries, and
[Runtime Notes](docs/runtime-notes.md) for launch and integration contracts.

## LOC Scorecard

Yazelix owns **19,746 lines** of tracked text project files. The
[reproducible scorecard](docs/development.md#loc-scorecard) excludes Beads,
lockfiles, and binary assets.
