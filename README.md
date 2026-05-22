# Yazelix v17

<div align="center">
  <img src="assets/logo.png" alt="Yazelix Logo" width="200"/>
</div>

## Preview
![Current Yazelix workspace](assets/screenshots/yazelix_current_example.png)

The repo keeps one maintained static preview

## Installation

```bash
nix profile add github:luccahuguet/yazelix#yazelix
yzx launch
```

Yazelix publishes an optional `x86_64-linux` Cachix binary cache for package installs and Home Manager switches; see the [installation guide](./docs/installation.md#optional-use-the-yazelix-binary-cache) for the Nix configuration snippet

> If you previously evaluated this flake, for example with `nix run` or `nix flake show`, Nix may have cached an older version. Add `--refresh` to force a fresh fetch:
> ```bash
> nix profile add --refresh github:luccahuguet/yazelix#yazelix
> ```

One-off use without installing also works:

```bash
nix run github:luccahuguet/yazelix#yazelix -- launch
```

Prefer declarative installs? Use the top-level Home Manager module in [home_manager/README.md](home_manager/README.md)

📖 **[Complete Installation Guide →](./docs/installation.md)** - Detailed step-by-step installation instructions

## Updating

Choose one update owner for each Yazelix install, and do not mix both update paths for the same installed runtime

- Profile installs: use `yzx update upstream`
- Home Manager installs: use `yzx update home_manager`

`yzx update upstream` prints and runs:

```bash
nix profile upgrade --refresh <matching-yazelix-profile-entry>
```

If the active runtime comes from an unmanaged Nix store path, such as `nix run` or a manually installed desktop entry, first install Yazelix into the default profile:

```bash
nix profile add --refresh github:luccahuguet/yazelix#yazelix
yzx desktop install
```

`yzx update home_manager` runs in your current flake directory and refreshes the `yazelix` input with:

```bash
nix flake update yazelix
```

Run it only from the Home Manager flake that owns this install

If your Home Manager flake uses a different Yazelix input name, run `nix flake update <your-input-name>` yourself instead

This still matters for `path:` inputs because `flake.lock` pins a snapshot of that local path until you refresh it

Then `yzx update home_manager` prints `home-manager switch` for you to run yourself

Updating replaces the installed runtime that future launches use, while already-open Yazelix windows keep running their current live runtime until you explicitly relaunch them or run `yzx restart`; Yazelix does not silently hot-swap live sessions in place

## Overview
Yazelix is a workspace-focused terminal environment built around [Yazi](https://github.com/sxyazi/yazi), [Zellij](https://github.com/zellij-org/zellij), and [Helix](https://helix-editor.com), with first-class [Neovim](https://neovim.io) support too

The supported product in this branch is the v17 Yazelix line

## Everyday Model

After installation, keep three things in mind:

- Edit `~/.config/yazelix/settings.jsonc` for the main workspace settings
- Treat generated runtime state under `~/.local/share/yazelix` as Yazelix-owned output
- Relaunch the window, or run `yzx restart`, after changing settings that affect live panes

Ghostty cursor presets use their own config at `~/.config/yazelix_ghostty_cursors/settings.jsonc`. Deeper Yazi, Zellij, Helix, terminal, and shell overrides also live under `~/.config/yazelix/`, but the main settings file is the first place to look

## Workspace Model

- Zellij orchestrates the workspace, with a managed sidebar and your chosen editor in the managed `editor` pane
- In Yazelix docs, `sidebar` means the generic side-surface slot; the default sidebar is a Yazi file tree
- `Alt+Shift+H/J/K/L` is the spatial surface layer: `H` toggles the left sidebar, `J` toggles the bottom popup, `K` toggles the top popup, and `L` toggles the right agent sidebar
- Use `Ctrl+y` to toggle focus between the left sidebar and editor, and `Ctrl+Shift+Y` to toggle focus between the editor and right agent sidebar
- `Alt+[` and `Alt+]` are reserved for previous/next layout-family cycling, but the packaged runtime ships one managed sidebar family, so those bindings usually keep the visible layout unchanged; see [Layouts](./docs/layouts.md)
- When you open something from the default Yazi file-tree sidebar with Helix or Neovim, Yazelix targets the managed `editor` pane through the pane orchestrator instead of relying on pane scanning heuristics; it reuses that pane when present and creates one titled `editor` when needed
- `yzx reveal` is the stable editor-integration surface for jumping the current file back into the managed Yazi file tree
- `Alt+Shift+J` toggles the bottom managed popup pane through `yzpp` and refreshes the Yazi file-tree sidebar git view when that popup closes; `Alt+Shift+K` toggles the top popup slot, `Alt+Shift+M` toggles the popup command menu, and `Alt+Shift+C` toggles the config UI popup
- Named popup commands live in `zellij.popup_commands`: bottom defaults to `lazygit`, top defaults to `yzx config ui` for Yazelix's ratconfig-backed JSONC settings editor, and menu defaults to `yzx menu`; the extra unbound personal popup slot uses `zellij.popup_program` plus the `zellij.keybindings.popup` action
- Configure the managed editor with `editor.command` in `settings.jsonc`

## Advanced: First-Party Child Repositories

Yazelix keeps this repo as the integrated workspace/runtime and splits focused subsystems into child repositories when they are reusable outside the full workspace or need their own build/release boundary

Regular Yazelix users do not need to install, configure, or understand these child repos separately; the normal Yazelix package already integrates the pieces it uses

Reusable child repos:

- [yazelix-screen](https://github.com/luccahuguet/yazelix-screen) — Terminal animation engine used by Yazelix welcome/screen styles and exposed here as `#yzs` and `#yazelix_screen`
- [yazelix-ghostty-cursors](https://github.com/luccahuguet/yazelix-ghostty-cursors) — Ghostty cursor preset and shader generator with the `yzc` CLI, exposed here as `#yzc`, `#yazelix_ghostty_cursors`, and `#ghostty_cursor_shaders`
- [yazelix-zellij-bar](https://github.com/luccahuguet/yazelix-zellij-bar) — Standalone Zellij bar plugin package and `yazelix_zellij_bar_widget` command, exposed here as `#yazelix_zellij_bar`
- [yazelix-zellij-pane-orchestrator](https://github.com/luccahuguet/yazelix-zellij-pane-orchestrator) — First-party Zellij plugin wasm that owns managed pane identity, editor/sidebar handoff, focus actions, and layout-family commands, exposed here as `#yazelix_zellij_pane_orchestrator`
- [yazelix-zellij-popup](https://github.com/luccahuguet/yazelix-zellij-popup) — Standalone Zellij popup plugin for plain-Zellij floating TUI panes, exposed here as `#yazelix_zellij_popup`; its plugin alias and wasm artifact are `yzpp`, and regular Yazelix sessions use it for the popup, command palette, and config UI panes
- [yazelix-yazi-assets](https://github.com/luccahuguet/yazelix-yazi-assets) — Standalone Yazi flavor and reusable plugin asset pack, exposed here as `#yazelix_yazi_assets` and integrated into the normal Yazelix Yazi runtime
- [yazelix-ratconfig](https://github.com/luccahuguet/yazelix-ratconfig) — Reusable Ratatui JSONC config editor crate consumed by `yzx config ui`; Yazelix keeps settings schema, Home Manager ownership, validation, and runtime apply behavior in this repo

Temporary integration forks:

- [yazelix-zellij](https://github.com/luccahuguet/yazelix-zellij) and [yazelix-yazi](https://github.com/luccahuguet/yazelix-yazi) — Default Ghostty-runtime source forks that restore Yazi image previews through Kitty graphics passthrough in Zellij; these forks are expected to be dropped and archived once upstream Zellij supports the required path directly enough for Yazelix to return to upstream packages

## Why Yazelix
Yazelix is a reproducible terminal IDE built around Zellij, Yazi, and your configured editor. It gives you one packaged workspace with a managed Yazi file tree, a stable editor pane, optional right agent sidebar, directional popup surfaces, and a fixed runtime toolset that behaves the same locally or over SSH

The workspace is managed by pane identity instead of pane-scanning guesses. Opening from Yazi targets the managed editor, `yzx reveal` jumps the current file back into the file tree, and the `Alt+Shift+H/J/K/L` layer maps naturally to left sidebar, bottom popup, top popup, and right sidebar

Configuration lives in JSONC at `~/.config/yazelix/settings.jsonc`, with `yzx config ui` providing Yazelix's ratconfig-backed settings editor for inspecting defaults, editing values, and understanding stale-field diagnostics

First-party child packages own focused pieces of the stack: screen rendering, Ghostty cursors, the Zellij bar, the popup plugin, the pane orchestrator wasm, and Yazi assets. The normal Yazelix package wires them together automatically

Ghostty is the default packaged terminal for cursor trails and Yazi image previews, with temporary Yazelix Zellij/Yazi forks carrying the Kitty graphics passthrough until upstream support is enough to drop them. WezTerm is an explicit packaged alternate, and Kitty, Alacritty, and Foot remain supported when present on the host `PATH`

Get everything running in less than 10 minutes with no extra dependencies beyond Nix

Install once, get the same managed workspace everywhere

Want the docs front door? See [Yazelix Docs](./docs/README.md)

Want the high-level product map? See [Architecture Map](./docs/architecture_map.md); want the current runtime boundary? See [Current Trimmed Runtime Contract](./docs/contracts/v15_trimmed_runtime_contract.md); want profiler details? See [Startup Performance](./docs/startup_performance.md)

## Acknowledgments
See [Yazelix Collection](./docs/yazelix_collection.md) for a full list of all projects, tools, and plugins Yazelix integrates, including links to each project and their homepages

Special thanks to [soderluk](https://github.com/soderluk) for grinding with me through unstable periods of Yazelix, when things that should work were not working. His many reports had very high value for the development of Yazelix

Special thanks to [tag-und-nacht](https://github.com/tag-und-nacht) for very detailed macOS, Home Manager, theming, and configuration reports that helped sharpen Yazelix's cross-platform support and user-config story

If Yazelix is useful to you, you can support its development on [GitHub Sponsors](https://github.com/sponsors/luccahuguet)

<!-- BEGIN GENERATED README LATEST SERIES -->
## Latest Tagged Releases

### v17

First-party child repos, Ghostty image previews, and JSONC workspace config

- Established the first-party child-repo architecture across `yazelix-screen`, `yazelix-ghostty-cursors`, `yazelix-zellij-popup`, `yazelix-zellij-bar`, `yazelix-zellij-pane-orchestrator`, `yazelix-yazi-assets`, and `yazelix-ratconfig`
- Replaced copied source, copied wasm, duplicated widget code, and vendored Yazi assets with locked child-owned packages and artifacts consumed by the main runtime
- Promoted Ghostty back to the default packaged terminal, with Yazi image previews restored through temporary first-party `yazelix-zellij` and `yazelix-yazi` Kitty-graphics passthrough forks while upstream Zellij support is still pending
- Switched the package baseline to `nixpkgs-unstable` and pulled in newer Yazi/Chafa behavior that avoids the Chafa terminal-probe ghost-keypress regression
- Made `settings.jsonc` the canonical user config, backed by `settings_default.jsonc`, JSON schema coverage, strict unknown-field diagnostics, additive repair, and complete Home Manager rendering
- Upgraded `yzx config ui` into a structured JSONC settings editor with scalar pickers, keybinding rows, safer parse-error behavior, popup launch through `Alt Shift C`, and generic config UI machinery owned by `yazelix-ratconfig`
- Added the directional workspace keymap: `Alt Shift H` toggles the left sidebar, `Alt Shift J` opens the bottom popup, `Alt Shift K` opens the top popup, `Alt Shift L` opens the right Codex agent sidebar, and `Alt Shift M` opens the menu popup
- Added managed focus/reveal keys: `Ctrl y` switches between editor and left sidebar, `Ctrl Shift Y` switches between editor and right sidebar, and `Alt r` smart-reveals in the editor or falls back to editor/left-sidebar focus
- Made Yazelix and native Zellij key policies data-driven through `settings.jsonc`, including remappable native defaults such as `Ctrl Alt g/s/o` for locked/scroll/session modes, `Ctrl Shift H/L` for tab movement, `Alt 1..9` for tab jumps, `Ctrl Alt p` for pane groups, and `Alt Shift F` for focus fullscreen
- Converged generated runtime state before launch so stale Zellij layouts, plugin permission caches, terminal configs, copied native config files, and Yazi static assets are repaired or diagnosed deterministically
- Moved status-bar and widget ownership into `yazelix-zellij-bar`, including Codex, Claude, OpenCode Go, CPU/RAM, cursor, cached facts, throttled refresh, and first-paint hydration
- Exposed standalone subsystem packages for screen rendering, Ghostty cursors, popup panes, the Zellij bar, and Yazi assets while keeping normal Yazelix installs wired automatically
- Matured public Nix customization with `mkYazelix`, overlays, runtime tool sources, component toggles, child package outputs, Home Manager integration, and Cachix publishing
- Migrated maintainer issue tracking from Go/Dolt `bd` to Rust `br`, with tracked JSONL state, ignored local SQLite cache, Nix packaging, CI initialization, and GitHub issue sync support
- Users jumping straight from early v16 should still read the v16.2 and v16.3 notes for cursor-sidecar and flat-config-path manual actions

### v16

v16 Rust-forward control plane with an irreducible Nushell core

- Finished the Rust owner cuts across the remaining deterministic control-plane and editor/Yazi integration surfaces, so the public `yzx` story is now much more clearly Rust-owned
- Reduced Nushell to the explicit shell and UI core, documented the surviving floor, and kept popup/menu wrappers on Nushell where that boundary is the clearest fit
- Moved maintainer, update, and sweep ownership further out of Nushell, including repo-maintainer flows and pane-orchestrator sync semantics, so the remaining Nu surface is much smaller and more intentional
- Unified the human CLI rendering for `yzx status`, `yzx status --versions`, and `yzx keys` around one shared Rust styling layer with cleaner grouped output and better contrast

For exact tagged release notes, see [CHANGELOG](./CHANGELOG.md) or run `yzx whats_new` after installing that release
For the longer project story, see [Version History](./docs/history.md)
<!-- END GENERATED README LATEST SERIES -->

## Compatibility
- **Platform**: Linux and macOS — see the [macOS support floor contract](docs/contracts/macos_support_floor.md) for the current guaranteed macOS surfaces
- **Terminal**: Ghostty is the default packaged terminal with Yazelix cursor trails and Yazi image previews, WezTerm is available through the explicit WezTerm package path, while Kitty and Alacritty remain supported PATH-provided alternatives and Foot remains a Linux-only PATH-provided alternative
- **Editor**: Any editor works, with Helix and Neovim getting first-class support (reveal in the Yazi file tree, open buffer in a running instance, managed editor-pane targeting) and configuration through `editor.command` in `settings.jsonc`
- **Shell**: Bash, Fish, Zsh, or Nushell - use whichever you prefer

### Helix Integration
Helix supports optional `yzx reveal` integration through `Alt+r`, and Yazelix reserves `Alt+r` globally: in the managed editor it forwards `Alt+r` into Helix for reveal, outside the editor it falls back to the editor/left-sidebar focus flow, and `Ctrl+y`, `Ctrl+Shift+Y`, plus `Alt+Shift+H` remain the dedicated workspace navigation keys

📖 **[Complete Helix Keybindings Guide →](./docs/helix_keybindings.md)** - Recommended keybindings for enhanced editing experience

### Neovim Integration
For Neovim-Yazi integration, bind `yzx reveal` to any editor-local shortcut that does not conflict with your terminal or Zellij bindings; a good default is `<M-r>`:

This assumes `yzx` is on your editor `PATH`

```lua
-- Yazelix Yazi file-tree integration - reveal current file in the managed sidebar
vim.keymap.set('n', '<M-r>', function()
  local buffer_path = vim.fn.expand('%:p')
  if buffer_path ~= '' then
    vim.fn.system({ 'yzx', 'reveal', buffer_path })
  end
end, { desc = 'Reveal in Yazi file tree' })
```

📖 **[Complete Neovim Keybindings Guide →](./docs/neovim_keybindings.md)** - Setup instructions and workflow tips

## Version Check
Check installed tool versions:
```bash
yzx status --versions
```

## POSIX/XDG Paths

Yazelix keeps user-edited config separate from generated runtime output:

- User config lives under `$XDG_CONFIG_HOME/yazelix`, usually `~/.config/yazelix`, with `settings.jsonc` as the canonical main config
- Generated runtime output lives under `$XDG_DATA_HOME/yazelix`, usually `~/.local/share/yazelix`, including generated Yazi, Zellij, Helix, terminal configs, logs, profiles, sessions, and freshness records
- Launchers may set `YAZELIX_CONFIG_DIR` and `YAZELIX_STATE_DIR` explicitly; Home Manager uses those owner-provided paths when it manages Yazelix

See [POSIX/XDG Paths](./docs/posix_xdg.md) for the full path contract

## SSH / Remote

Yazelix shines over SSH: the TUI stack (Zellij, Yazi, Helix) runs cleanly without any GUI, giving you a fully configured, consistent “superterminal” on barebones hosts such as an AWS EC2 instance, while the Yazelix environment delivers the same tools, keybindings, and layouts you use locally, minimizing drift on ephemeral servers

## Customization & Configuration

Yazelix uses a **layered configuration system** that safely merges your personal settings with Yazelix defaults:

- **Core settings**: Edit `~/.config/yazelix/settings.jsonc` for shell, editor, terminal, Zellij, and Yazi settings, edit `~/.config/yazelix_ghostty_cursors/settings.jsonc` for Ghostty cursor settings, run `yzx config set/unset` for safe scalar and string-list edits, or run `yzx config ui`, Yazelix's ratconfig-backed JSONC settings editor, to inspect and edit explicit/defaulted values and stale-field diagnostics
- **Yazi customization**: Use the built-in `yazi` settings in `settings.jsonc` for things like plugins, theme, sorting, and binary overrides, and use the managed Yazi home at `~/.config/yazelix/yazi/` for `yazi.toml`, `keymap.toml`, `init.lua`, packages, plugins, and flavors (see [Yazi Configuration](./docs/yazi-configuration.md))
- **Zellij customization**: Use the built-in `zellij` settings in `settings.jsonc` for Yazelix-owned Zellij knobs, keybindings, theme, and rounded corners, and use `~/.config/yazelix/zellij.kdl` for deeper native Zellij settings that Yazelix does not render (see [Zellij Configuration](./docs/zellij-configuration.md))
- **Status bar widgets**: Configure `[zellij].widget_tray` to order or hide `editor`, `shell`, `term`, `workspace`, `cursor`, usage, `cpu`, and `ram` widgets; the default cursor widget renders mono presets as colored `█ name` and split presets as one-cell split glyphs from the launch-scoped Ghostty cursor fact
- **Your configs persist** across Yazelix updates without git conflicts
- **Intelligent merging**: Generated Yazi and Zellij runtime configs are rebuilt from Yazelix defaults plus your managed overrides instead of forcing you to edit tracked runtime files
- **Launch-time config snapshots**: each Yazelix window keeps the `settings.jsonc` snapshot it launched with; edit config whenever you want, then open a new Yazelix window or run `yzx restart` to apply it to live panes. Use repeatable `--with KEY=VALUE` on `yzx launch`, `yzx enter`, or `yzx restart` for session-only settings overrides

📖 **[Complete Customization Guide →](./docs/customization.md)** - Detailed instructions for customizing every tool

### Editor Configuration

📝 **[Editor Configuration Guide →](./docs/editor_configuration.md)** - Complete guide for configuring editors

**Quick setup:**
- **Default (recommended)**:
  ```toml
  [editor]
  command = ""
  ```
- **Neovim**:
  ```toml
  [editor]
  command = "nvim"
  ```
- **System Helix**:
  ```toml
  [editor]
  command = "hx"

  [helix]
  runtime_path = "/path/to/runtime"  # Only when your Helix runtime is outside normal discovery paths
  ```
- **Other editors**:
  ```toml
  [editor]
  command = "vim"
  ```

### Alternative: CLI-Only Mode
To use Yazelix tools without starting the full interface (no sidebar, no Zellij), use:
```bash
yzx env
```
This loads the curated Yazelix tool surface into your current shell, with Yazelix env vars set and clean messaging, and automatically launches the shell configured in your `settings.jsonc`; if you prefer the legacy behavior, run `yzx env --no-shell` to stay in your current shell

Internal runtime helpers stay private under `libexec/` instead of leaking into your interactive PATH, so host-distributed apps launched from that shell do not accidentally inherit Yazelix-owned core userland tools ahead of the system PATH

If you want the Yazelix tool PATH without switching into your configured shell, use:
```bash
yzx env --no-shell
```

### Packages & Customization

**What Gets Installed:**
See the full catalog of tools and integrations in the Yazelix Collection:
[docs/yazelix_collection.md](./docs/yazelix_collection.md)
- **Essential tools**: [Yazi](https://github.com/sxyazi/yazi) (file manager), [Zellij](https://github.com/zellij-org/zellij) (terminal multiplexer), [Helix](https://helix-editor.com) (editor), shells (bash/nushell, plus your preferred shell), [fzf](https://github.com/junegunn/fzf), [zoxide](https://github.com/ajeetdsouza/zoxide), [Starship](https://starship.rs)
- **Bundled helpers**: [lazygit](https://github.com/jesseduffield/lazygit) (or `lg`), [mise](https://github.com/jdx/mise), [carapace](https://github.com/carapace-sh/carapace-bin), [macchina](https://github.com/Macchina-CLI/macchina), and the fixed helper tooling behind the packaged runtime
- **Yazi preview helpers**: `p7zip`, `jq`, `poppler`, `fd`, `ripgrep` are part of the fixed runtime surface
- **Environment setup**: Proper paths, variables, and shell configurations

**Customize Your Installation:**
If you followed [step 3 in the installation guide](./docs/installation.md#step-3-configure-your-installation-optional), you already have your `~/.config/yazelix/settings.jsonc` config file ready, you can modify it anytime and restart Yazelix to apply changes. Main options live in that file; Ghostty cursor presets live in `~/.config/yazelix_ghostty_cursors/settings.jsonc`

**Terminal Emulator Selection:**
- **Ghostty** (default packaged preference): Modern, fast terminal written in Zig with Yazelix cursor trails and Yazi image previews
- **WezTerm** (explicit packaged alternate path): Rust terminal with strong graphics support and Sixel compatibility
- **Kitty**: Fast, feature-rich, GPU-accelerated terminal
- **Alacritty**: Fast, GPU-accelerated terminal written in Rust
- **Foot**: Wayland-native terminal (Linux-only)
- **Auto-detection**: Fallback order follows your configured terminal list
- Configure your preference in `settings.jsonc` with `terminal.terminals = ["ghostty", "wezterm", ...]` (first item is primary)
- **Terminal package contract**: Yazelix ships one packaged terminal variant at a time; Ghostty is the default, and explicit Ghostty/WezTerm variants remain available

[See the full Customization Guide here.](./docs/customization.md)

---

## Home Manager Integration

Yazelix includes optional Home Manager support for declarative configuration management through the top-level flake's `homeManagerModules.default` output; see [home_manager/README.md](home_manager/README.md) for setup instructions

## When should you not use yazelix?
- If you hate having fun

## Initializer Scripts
Yazelix auto-generates initialization scripts for Starship, Zoxide, Mise, and Carapace for your configured shell set during environment setup and refresh; see [docs/initializer_scripts.md](./docs/initializer_scripts.md) for details

## yzx Command Line Interface

Run `yzx help` for the live command list

### Start Sessions

- `yzx launch` - Open Yazelix in a managed terminal window from the current directory
- `yzx enter` - Start Yazelix in the current terminal
- `yzx launch --path DIR` - Launch from a specific directory
- `yzx launch --home` - Launch from the home directory
- `yzx launch --terminal ghostty` - Force a supported terminal for this launch
- `yzx launch --config ./minimal.jsonc` - Start one window from an alternate complete settings file
- `yzx launch --with editor.command=nvim` - Override one settings field for this window only
- `yzx launch --verbose` - Print detailed launch diagnostics

### Use Tools Without the Workspace

- `yzx env [--no-shell]` - Load Yazelix tools without the UI; `--no-shell` keeps your current shell
- `yzx run <command> [args...]` - Run one command inside the Yazelix environment

### Workspace Actions

- `yzx popup` - Toggle the managed popup program, usually `lazygit`; the popup keybinding refreshes Yazi sidebar git state when it closes
- `yzx menu --popup` - Open the popup command palette, usually through `Alt+Shift+M`
- `yzx config ui` - Open Yazelix's ratconfig-backed JSONC settings editor, usually through `Alt+Shift+C`
- `yzx sidebar refresh` - Refresh the managed Yazi sidebar file tree and status widgets

### Config and Recovery

- `yzx config [--path]` - Show the active config or print its resolved path
- `yzx config set PATH JSON` - Set a supported config value while preserving comments
- `yzx config unset PATH` - Remove an explicit config value so defaults apply
- `yzx edit config` - Open the main managed Yazelix config file in your editor
- `yzx restart [-s | --skip] [--config FILE] [--with KEY=VALUE]` - Restart Yazelix in a fresh window after config changes
- `yzx doctor [--verbose] [--fix]` - Run health checks and diagnostics

### Updates

- `yzx update` - Show supported update-owner paths
- `yzx update upstream` - Upgrade the active default-profile Yazelix package
- `yzx update home_manager` - Refresh the current Home Manager flake input and print `home-manager switch`

### Status and Extras

- `yzx status [--versions]` - Show current Yazelix status and optional tool versions
- `yzx cursors` - Inspect Ghostty cursor presets, effects, and resolved colors
- `yzx dev inspect_session [--json]` - Inspect the current Yazelix/Zellij tab session snapshot for runtime debugging
- `yzx dev profile [--cold] [--desktop] [--launch] [--clear-cache]` - Profile startup phases under `~/.local/share/yazelix/profiles/startup/`

📖 **[Complete yzx CLI Documentation →](./docs/yzx_cli.md)** - Full examples, diagnostics, profile tools, and maintainer surfaces

## Troubleshooting

🔍 **Quick diagnosis:** `yzx doctor` - Automated health checks and fixes

📖 **[Complete Troubleshooting Guide →](./docs/troubleshooting.md)** - Comprehensive solutions for common issues

## Editor Terminal Integration
Want to use Yazelix tools (Nushell, zoxide, starship, lazygit) inside your editor? Zed, VS Code, and Cursor all work seamlessly with `yzx env`

**Quick Setup:**
1. Open your editor's integrated terminal
2. Run `yzx env` to load all Yazelix tools without the UI in your configured shell
3. Enjoy the full Yazelix environment in place
Need to stay in your editor's existing shell? Run `yzx env --no-shell` instead

For more advanced integration options, see our [Zed + VS Code terminal integration guide](./docs/editor_terminal_integration.md)

## Styling and Themes
Yazelix includes transparency settings and theme configurations for a beautiful terminal experience, with terminal emulator configs that include transparency settings you can comment or uncomment and Helix themes that include transparent options; see [docs/styling.md](./docs/styling.md) for customization details

For Helix themes, you can use transparent themes by editing your Helix config:
```toml
# theme = "base16_transparent"
theme = "term16_dark"  # Recommended transparent theme
```

## Layouts

Yazelix layouts are Zellij layouts with Yazelix-owned pane identity layered on top: a managed `sidebar` pane, a managed `editor` pane, and sidebar-aware swap layouts that can collapse, widen, or refocus panes without losing workspace state

The default left sidebar is a Yazi file tree launched by `yzx sidebar yazi`, and the default right sidebar launches host-installed `codex`. `workspace.left_sidebar.*` and `workspace.right_sidebar.*` control each side pane command, args, and width; `editor.hide_sidebar_on_file_open` can collapse the left sidebar after opening files

The packaged runtime ships one managed sidebar family. `Alt+[` and `Alt+]` are still bound to previous/next layout-family cycling, but with one family they usually leave the visible layout unchanged. Use `Alt+Shift+H/J/K/L` for everyday surface toggles and `Ctrl+y` / `Ctrl+Shift+Y` for sidebar/editor focus

Keep complex custom layouts in Zellij KDL under `configs/zellij/layouts/`; custom sidebar swap families are maintainer-level work because Yazelix family-aware controls only know the built-in sidebar family

See [Layouts](./docs/layouts.md) for layout files, config keys, and customization boundaries

## Keybindings

Yazelix uses Zellij as the workspace layer, so the most important bindings are global workspace bindings rather than editor-local shortcuts; run `yzx keys` inside Yazelix for the live summary, and see [docs/keybindings.md](./docs/keybindings.md) for the full reference

| Keybinding | What It Does |
|------------|--------------|
| `Ctrl+y` | Toggle focus between the managed editor and left sidebar, which defaults to a Yazi file tree |
| `Ctrl+Shift+Y` | Toggle focus between the managed editor and right Codex agent sidebar |
| `Alt+Shift+H` | Show or hide the left sidebar |
| `Alt+r` | Smart reveal/focus key; forwards into the editor when appropriate |
| `Alt+[` / `Alt+]` | Previous/next layout family; with the packaged single family this usually has no visible effect |
| `Alt+m` | Open a new terminal in the current tab workspace root |
| `Alt+Shift+L` | Toggle the managed Codex agent sidebar |
| `Alt+Shift+J` | Toggle the bottom managed popup command, usually `lazygit`, and refresh the Yazi file-tree sidebar git state when the popup keybinding closes it |
| `Alt+Shift+K` | Toggle the top managed popup command, usually `yzx config ui`, Yazelix's ratconfig-backed settings editor |
| `Alt+Shift+M` | Open the `yzx` command palette popup |
| `Alt+Shift+C` | Open the Yazelix config UI popup |
| `Alt+1..9` | Jump directly to tabs 1 through 9 |
| `Alt+w` / `Alt+q` | Move to the next or previous tab |
| `Ctrl+Shift+H` / `Ctrl+Shift+L` | Move the current tab left or right |
| `Ctrl+Shift+J` / `Ctrl+Shift+K` | Move the current pane down or up |
| `Alt+Shift+F` | Toggle pane fullscreen |

Yazi still has its own keymap too: press `~` inside Yazi for its built-in help, remap Yazelix-owned Yazi integration keys with `yazi.keybindings` in `settings.jsonc`, and use the most useful file-tree sidebar flows such as `Enter` to open through the managed editor integration, `Alt+z` to pick a directory with zoxide and retarget the workspace, and `Alt+p` to open the selected directory in a new pane as the current tab workspace root

Helix and Neovim integration is intentionally small: use `Ctrl+y`, `Ctrl+Shift+Y`, and `Alt+Shift+H` for workspace navigation, use `Alt+r` / `yzx reveal` when you want the editor to reveal the current file in the managed Yazi file tree, and see [docs/helix_keybindings.md](./docs/helix_keybindings.md) and [docs/neovim_keybindings.md](./docs/neovim_keybindings.md) for editor-local setup details

## I'm Lost! Too Much Information
Start by learning Zellij on its own, then optionally Yazi, and re-read this README afterwards

## Contributing to Yazelix
See [contributing](./docs/contributing.md)
