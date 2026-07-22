# Yazelix Nova Beta

<div align="center">
  <img src="assets/logo.png" alt="Yazelix logo" width="200"/>
</div>

**What is the best possible terminal experience?**

Yazelix tries to answer that question. Nova brings the terminal, multiplexer,
file manager, editor, shell, Git tools, configuration, and an AI agent
together as one coherent workspace

Yazelix Nova is a Nix-packaged terminal workspace built around
[Mars](https://github.com/luccahuguet/mars) (a Rio-derived fork), a thin
[Yazelix-owned Zellij fork](https://github.com/luccahuguet/yazelix-zellij),
Yazi, Nushell (with packaged Bash, Zsh, and Fish alternatives), a lazygit popup (but you can configure other git clients!), and
an optional coding agent popup. It uses the
[Yazelix Helix fork](https://github.com/luccahuguet/yazelix-helix) by default
(but `editor.command` can select your preferred terminal editor). `yzx launch`
opens the desktop workspace through Mars, while `yzx enter` will open Yazelix in any capable terminal emulator (Mars
provides tighter Yazelix integration, though) or over SSH. Great defaults out of the box!

Nova packages the workspace as a Nix flake with an optional Home Manager
module. The repo keeps one launcher, one config root, one packaged layout, and
focused checks for its contracts

## Preview

![Yazelix Nova workspace](assets/screenshots/nova_workspace.png)

## Nova vs Classic

Classic was bloated and built on the wrong ownership model. Its main repository
acted as the product runtime, component control plane, configuration repair
system, compatibility layer, and maintainer toolbox.

Classic's child repositories did not create firm boundaries. The main repo
still carried their maintenance machinery and overlapping runtime logic. Nova
gives [first-party Yazelix components](#first-party-components) firm package
boundaries. Each component owns its implementation and contract. Nova pins and
composes their package outputs.

| Measure | Nova | Classic |
| --- | --- | --- |
| Code and configuration (Rust, Nix, shell, TOML, etc.) | **18,197 LOC** | **91,545 LOC** |
| Rust | **15,215 LOC** | **80,957 LOC** |
| Ownership model | One owner per concern | Overlapping responsibilities across layers |
| Yazelix component boundaries | Independent, versioned packages | Child repos mixed with main-repo ownership |
| Product experience | More features, stronger defaults, tighter integration, and polished UX | Fewer features and a less cohesive workspace |
| Status | Recommended | Frozen migration and rollback path |

Nova owns **73,348 fewer lines**, an **80% reduction**. Classic's Rust code
alone is 4.4 times larger than Nova's entire code and configuration surface.

Nova delivers more features in 20% of the code. It has a clearer configuration
model, tighter editor and Yazi integration, stronger diagnostics, and a
coherent popup-oriented interface. The smaller architecture makes Yazelix
easier to improve and better to use.

Classic proved the idea. Nova is the better product and the architecture
Yazelix should have had from the start.

## Install and launch

Yazelix requires Nix with flakes enabled. `launch` opens the packaged Mars window
in a graphical session, while `enter` starts the same workspace in the current
terminal or over SSH.

The `stable` branch advances from a checked
and dogfooded `main` revision at most once per week. Use `main` for more constant updates or an
immutable `nova-v*` tag for an exact release.

Linux is the dogfooded platform. CI builds all packages and a Home Manager
activation on `aarch64-darwin`, while interactive macOS use and the Mars GUI
remain unverified.

### Try without installing

```nu
nix run github:FlexNetOS/yazelix -- launch
nix run github:FlexNetOS/yazelix#yazelix-no-mars -- enter
```

If the one-off launch fails, inspect the owned runtime setup with:

```nu
nix run github:FlexNetOS/yazelix -- doctor
nix run github:FlexNetOS/yazelix -- inspect
```

### Install in a Nix profile

On a fresh machine where the literal profile is absent or empty, install the
single FlexNetOS foundation element with:

```nu
nix profile add --profile /home/flexnetos/.nix-profile --refresh github:FlexNetOS/yazelix#lifeos_foundation_yzx
yzx launch
```

Do not add `#yazelix` or any `yazelix-no-*` variant beside the foundation element. Existing
foundation installations update through the checked migration described in
[Installation](docs/installation.md#updates), which archives prior selectors
and verifies the exact replacement closure before declaring success.

### Install with Home Manager

Use the [Home Manager module](docs/installation.md#home-manager) for a
declarative install.

### Moving from Yazelix Classic

Classic v17.12 translates mutable Classic `settings.jsonc` or `config.toml`
files into Nova configuration. It does not rewrite Home Manager declarations
or Home Manager-owned files. Run upstream's bridge once when you need to
preserve mutable Classic settings, then install Nova from the authoritative
FlexNetOS repository. The recovery-only tag below belongs to the upstream
repository; it is not published on the FlexNetOS remote:

```nu
nix run github:luccahuguet/yazelix/v17.12#yazelix -- launch
```

If your Classic settings match packaged defaults, start with Nova's packaged
defaults and move straight to Nova. Home Manager users must replace
Classic-only options with Nova's narrow module surface before switching.

The original Nova cutover replaced the old `main` history. The FlexNetOS
recovery joins canonical Nova as first parent to the complete FlexNetOS history
as second parent through one reviewable unrelated-history merge. It does not
discard, cherry-pick, replay, rebase, squash, or force-push either lineage.
Classic remains available at the frozen `classic` branch, while the immutable
`v17.12` tag remains the migration and rollback bridge

## First five minutes

Start the guided tour after launching Yazelix:

```nu
yzx tutor begin
```

`yzx help` lists every command. `yzx doctor` checks the owned runtime setup
without opening Mars or Zellij. Inside Yazelix, press `Alt Shift M` to open the
command palette, which includes both help and tutor entries.

### Ratconfig

Press `Alt Shift K` to open Ratconfig:

| Key | Action |
| --- | --- |
| `1`-`9` | Jump to a tab |
| `Tab` / `Shift-Tab`, `h` / `l` | Change tabs |
| `j` / `k`, `/` | Move through rows or search All settings |
| `a` | Switch between Core and All |
| `e`, `Enter`, `Space` | Run the selected row's contextual action |
| `u`, `q` | Reset the selected setting or quit |

The footer lists the selected row's controls.

### Workspace keys

Yazelix extends Helix/Vim's `h/j/k/l` motion model into a workspace key grid.
The `Alt` and `Ctrl Alt` layers move focus, tabs, or panes, while `Alt Shift`
groups four workspace surfaces:

| Layer | `h` | `j` | `k` | `l` |
| --- | --- | --- | --- | --- |
| `Alt` | Focus left or previous tab | Focus down | Focus up | Focus right or next tab |
| `Ctrl Alt` | Move tab left | Move pane down | Move pane up | Move tab right |
| `Alt Shift` | Sidebar | Git | Ratconfig | Agent |

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
| `yzx inspect [--json]` | Report runtime, profile-frontdoor, shadow, and session provenance |
| `yzx status` | Print config/runtime paths and selected settings |
| `yzx status --json` | Print the versioned machine-readable status record |
| `yzx desktop [--print-path]` | Report the profile-owned desktop entry without copying it |
| `yzx env` | Open the managed shell without launching the UI |
| `yzx tutor [lesson]` | Print guided Yazelix lessons |
| `yzx screen [style]` | Show a terminal welcome screen |
| `yzx reveal <target>` | Reveal a file or directory in the managed Yazi sidebar |

Status JSON contains numeric `schema_version = 1`, plus `name`, `version`,
`package`, `config_home`, `state_dir`, `shell`, `editor_command`, `editor`,
`agent_command`, and `inside_zellij`. The sponsor URL remains in `yzx help`
without a public `sponsor` command

Inspect JSON is also schema 1 and adds the invoked and symlink-resolved
frontdoor, profile manifest, expected single profile root, local shadow state,
runtime identity, and optional session identity. It is read-only and works
outside Zellij

The top-right Zellij corner shows the compact release line derived from the
same version: `NOVA DEV` in development, `NOVA βN` during the v1 beta line,
and `NOVA 1.0` for the stable release

Screen styles are `static`, `logo`, `boids`, `boids_predator`,
`boids_schools`, `mandelbrot`, `game_of_life_gliders`,
`game_of_life_oscillators`, `game_of_life_bloom`, and `random`

Tutor lessons are `workspace`, `discovery`, `troubleshooting`, and
`tool_tutors`. `yzx tutor hx` and `yzx tutor nu` print the native tool tutor
commands

## Packages and platforms

Package names follow `yazelix[-no-mars][-no-helix][-no-yazi]`. Each suffix
removes that managed package while retaining the integration around it.
`no-helix` uses the configured host editor; `no-yazi` requires matching host
`yazi` and `ya` commands.

| Package | Mars | Managed Helix | Managed Yazi |
| --- | --- | --- | --- |
| `yazelix` | Yes | Yes | Yes |
| `yazelix-no-helix` | Yes | No | Yes |
| `yazelix-no-yazi` | Yes | Yes | No |
| `yazelix-no-helix-no-yazi` | Yes | No | No |
| `yazelix-no-mars` | No | Yes | Yes |
| `yazelix-no-mars-no-helix` | No | No | Yes |
| `yazelix-no-mars-no-yazi` | No | Yes | No |
| `yazelix-no-mars-no-helix-no-yazi` | No | No | No |

See [Installation and packages](docs/installation.md) for package variants,
platform support, SSH use, measured sizes, Home Manager, and updates.

## First-party components

Yazelix assembles focused first-party forks, plugins, libraries, and commands:

| Component | Yazelix role |
| --- | --- |
| [Mars](https://github.com/luccahuguet/mars) | GUI terminal used by `yzx launch`, with Kitty graphics, cursor shaders, and Yazelix session integration |
| [Yazelix Zellij](https://github.com/luccahuguet/yazelix-zellij) | Multiplexer fork with Kitty graphics passthrough for the workspace |
| [Yazelix Helix](https://github.com/luccahuguet/yazelix-helix) | Steel-enabled editor fork with isolated configuration and explicit workspace bridge hooks |
| [Yazelix Zellij Pane Orchestrator](https://github.com/luccahuguet/yazelix-zellij-pane-orchestrator) | Zellij plugin that owns tab-local workspace roots and coordinates panes, focus, popups, the editor, and agent activity |
| [Yazelix Zellij Popup](https://github.com/luccahuguet/yazelix-zellij-popup) | Zellij plugin that opens, focuses, hides, and closes configured floating TUI panes |
| [Yazelix Zellij Bar](https://github.com/luccahuguet/yazelix-zellij-bar) | Zellij plugin package for the compact top bar, tabs, modes, session details, and status widgets |
| [Ratconfig](https://github.com/luccahuguet/ratconfig) | Reusable Ratatui configuration editor and TOML patching and migration library |
| [Yazelix Screen](https://github.com/luccahuguet/yazelix-screen) | Terminal welcome animations exposed through `yzx screen` |
| [Yazelix Cursors](https://github.com/luccahuguet/yazelix-cursors) | Shared cursor presets and validation for Ratconfig, plus palettes and shader assets for Mars |
| [auto-layout.yazi](https://github.com/luccahuguet/auto-layout.yazi) | Yazi plugin that changes the column layout to match the available pane width |
| [zjstatus](https://github.com/luccahuguet/zjstatus) | Fork that gives the bar activity-aware tab markers without changing native Zellij tab names |

## Configuration

`yzx config` opens Ratconfig over the managed tree at
`~/.config/yazelix/`. Yazelix inherits packaged defaults and persists only
explicit overrides. Core shows the settings most users need. All includes the
complete inventory.

Set `shell.program` in Ratconfig or `config.toml` to choose packaged Nushell
(default), Bash, Zsh, or Fish for new panes and sessions.
Yazelix initializes Starship, Carapace completions, and zoxide for managed
Nushell. Bash, Zsh, and Fish use their normal interactive startup files.

See [Configuration](docs/configuration.md) for settings, popups, native files,
Yazi plugins, cursor ownership, and editor behavior.

## Development

From a local checkout, use:

```sh
nix run .#yazelix -- launch
nix run .#yazelix-no-mars -- enter
```

See [Development](docs/development.md) for CI and local checks,
[Architecture](ARCHITECTURE.md) for ownership boundaries, and
[Runtime Notes](docs/runtime-notes.md) for launch and integration contracts.

The [RuVector blueprint provenance ledger](docs/ruvector_blueprint_provenance.md)
maps every applicable Engine Room and formerly optional capability to its one
repository owner and verification surface

## LOC Scorecard

Nova owns **23,476 lines** of tracked text project files. The
[reproducible scorecard](docs/development.md#loc-scorecard) excludes Beads,
lockfiles, and binary assets.
