# Yazelix v9

<div align="center">
  <img src="assets/logo.png" alt="Yazelix Logo" width="200"/>
</div>

## Preview
![yazelix_v8_demo](assets/demos/yazelix_v8_demo.gif)

**v8.5 with zjstatus:**
![yazelix_v8_5_example](assets/screenshots/yazelix_v8_5_example.jpeg)

## Overview
Yazelix integrates Yazi, Zellij, and Helix, hence the name, get it?

- **Use your preferred shell**: Bash, Fish, Zsh, or Nushell - Yazelix works with all of them
- Zellij orchestrates everything, with Yazi as a sidebar and your chosen editor (Helix by default)
- To hide the sidebar, make your pane fullscreen! (`Ctrl p + f` or `Alt Shift f`)
- Every keybinding from Zellij that conflicts with Helix is remapped [see here](#keybindings)
- When you hit Enter on a file/folder in the "sidebar":
  - **With Helix**: If Helix is already open in the topmost pane of the stack, it opens that file/folder in a new buffer in Helix. If Helix isn't open, it launches Helix in a new pane for you. It always finds a running Helix instance if it exists and is in the top pane of the stacked group.
  - **With other editors**: Opens the file in a new pane with your configured editor
  - It automatically renames the Zellij tab to the file's underlying Git repo or directory name
- Features include:
  - "Reveal file in sidebar" (press `Alt y` in Helix to reveal the file in Yazi, `Alt y` in Yazi to focus Helix, see [Keybindings](#keybindings))
  - A Yazi plugin to enhance the status bar in the sidebar pane, making it uncluttered, colorful, and showing file permissions
  - A [Git plugin](https://github.com/yazi-rs/plugins/tree/main/git.yazi) showing file changes in the Yazi sidebar
  - Dynamic column updates in Yazi (parent, current, preview) via the [auto-layout plugin](https://github.com/luccahuguet/auto-layout.yazi), perfect for sidebar use
  - **Modular editor support**: Use Helix for full integration features, or any other editor via the `editor_command` setting
- This project includes config files for Zellij, Yazi, terminal emulators, Nushell scripts, Lua plugins, and a lot of love
- See [boot sequence](./docs/boot_sequence.md) for details on how Yazelix starts up

## Vision
- Using the terminal should be easy, beautiful, pratical and reproducible.
- Good defaults over customization. Have both when possible
- Yazelix is always on the edge of project versions
- Yazelix is always evolving, it's a living being
- Yazelix is easy to use
- What is even Yazelix?
- Yazelix lets you say `I use Yazelix btw`
- Boy, do we Nix
- Integration, integration, integration
- Like [Omarchy](https://github.com/olimorris/omarchy) but for your terminal
- Made with love.

## Acknowledgments
See [Yazelix Collection](./docs/yazelix_collection.md) for a full list of all projects, tools, and plugins Yazelix integrates, including links to each project and their homepages.

## Improvements of v9 over v8
- **Flexible layout system**: Sidebar mode remains the default, with optional no-sidebar mode for different workflows:
  - **Sidebar mode** (default): IDE-like workflow with persistent Yazi file navigation (recommended!)
  - **No-sidebar mode**: Available via `enable_sidebar = false`, no yazi sidebar, saves some screen space. Usefull if you use other editors that have a builtin file tree 
- **Pack-based configuration system**: Simplified package management with technology stacks:
  - Enable entire tech stacks with `packs = ["python", "js_ts", "config"]` instead of commenting individual packages
  - 5 curated packs: `python` (ruff, uv, ty), `js_ts` (biome, bun), `rust` (cargo tools), `config` (formatters), `file-management` (utilities)
  - Hybrid approach: use packs for bulk selection, `user_packages` for individual tools
- **Enhanced Zellij layouts**: Added comprehensive layout system with both sidebar and no-sidebar variants:
  - **Sidebar layouts** (default): `basic`, `stacked`, `three_column`, `sidebar_closed` - persistent file navigation
  - **No-sidebar layouts**: `basic`, `stacked`, `two_column` - clean, full-screen workflows
- **New sidebar_closed swap layout**: Dynamic sidebar toggling: use the sidebar_closed swap layout, reach it with `Alt+[` / `Alt+]` for space optimization when needed
- **New zjstatus plugin integration**: Added custom status bar plugin with shell and editor information:
  - **Shell indicator**: Shows current configured shell (e.g., `[shell: nu]`)
  - **Editor indicator**: Shows current configured editor (e.g., `[editor: vim]`)
  - **Clean layout**: `[shell: nu] [editor: vim] YAZELIX` with proper spacing and color coding
  - **Replaces default Zellij status bar** with more informative yazelix-specific display
- **Dynamic Three-Layer Zellij Configuration**: Completely rewritten configuration system with modular, maintainable approach:
  - **Layer 1**: Zellij defaults (fetched dynamically via `zellij setup --dump-config`)
  - **Layer 2**: Yazelix overrides (`yazelix_overrides.kdl`) - Yazelix-specific settings
  - **Layer 3**: User configuration (`user_config.kdl`) - Your personal customizations with **highest priority**
  - **Smart caching**: Only regenerates when source files change for faster startup
  - **XDG-compliant**: Generated config saved to `~/.local/share/yazelix/configs/zellij/`
  - **Comprehensive template**: `user_config.kdl` includes documented examples for themes, keybindings, plugins, and advanced options
  - **Improved maintainability**: Removed old static `config.kdl` system that required manual updates
  - **Better user experience**: Users can now easily customize Zellij by editing a single, well-documented file
  - **Reference documentation**: See [configs/zellij/example_generated_config.kdl](./configs/zellij/example_generated_config.kdl) for the complete default Zellij configuration with all available keybindings and options
- **Bidirectional Alt+y navigation**: Enhanced file manager and editor integration with seamless navigation:
  - **From Helix**: `Alt+y` reveals current file in Yazi sidebar (existing functionality)
  - **From Yazi**: `Alt+y` focuses and moves Helix pane to top (new functionality)
  - **Consistent behavior**: Uses same intelligent Helix detection logic as file opening system
  - **Smart pane management**: Automatically moves found Helix pane to top of stack for better workflow
- **Alt+p directory opening**: New Yazi keybinding for instant workspace expansion:
  - **Quick pane creation**: `Alt+p` in Yazi opens selected directory in new Zellij pane
  - **Smart file handling**: For files, opens parent directory; for directories, opens the directory itself
  - **Proper shell environment**: New panes start with correctly configured Nushell in target directory
- **Enhanced startup robustness**: Improved Nix detection with automatic environment setup, reliable terminal integration across all emulators, and graceful error handling with clear diagnostics
- **Health Check System (`yzx doctor`)**: Comprehensive diagnostic tool that automatically detects and fixes common issues including Helix runtime conflicts, environment variable problems, configuration validation, and system health monitoring. Supports `--verbose` and `--fix` flags for detailed output and automatic issue resolution.
- **Atuin shell history integration**: Added atuin to the automatic initializer system for enhanced shell history with search, sync, and statistics across all supported shells
- **Professional logo and desktop integration**: High-quality Yazelix logo with crisp multi-size icons and automatic desktop entry setup for all desktop environments (GNOME, KDE, XFCE, COSMIC, etc.)
- **CLI-only environment mode (`yzx env`)**: New command for loading Yazelix tools without the UI interface:
  - **Quick access**: `yzx env` loads all Yazelix tools (helix, yazi, lazygit, etc.) in your current terminal
  - **No interface overhead**: Skips welcome screen and Zellij launch, giving you direct access to tools
  - **Clean messaging**: Shows environment status and available commands without interruption
  - **Perfect for scripts**: Ideal for automation, VS Code integration, or when you just need the tools


## Compatibility
- **Platform**: Works on any Linux distribution. Likely works on macOS as well (untested)
- **Terminal**: WezTerm, Ghostty, Kitty, or Alacritty
- **Editor**: Any editor, but Helix has first-class support (reveal in sidebar, open buffer in running instance, etc). Configure other editors via `editor_command` setting in `yazelix.nix`
- **Shell**: Bash, Fish, Zsh, or Nushell - use whichever you prefer
- See the version compatibility table [here](./docs/version_table.md) (generated dynamically!)

## Installation

📖 **[Complete Installation Guide →](./docs/installation.md)** - Detailed step-by-step setup instructions

**Quick Overview**: Yazelix uses Nix for reproducible, reliable installations that guarantee everyone gets the exact same tool versions. You don't need to learn Nix - just install it once and forget it exists!

## Quick Setup

1. **Install Nix**: `curl --proto '=https' --tlsv1.2 -sSf -L https://install.determinate.systems/nix | sh -s -- install`
2. **Clone Yazelix**: `git clone https://github.com/luccahuguet/yazelix ~/.config/yazelix`  
3. **Copy terminal config** (optional): See [Step 5 in installation guide](./docs/installation.md#step-5-set-up-yazelix-to-auto-launch-in-your-terminal)
4. **Launch**: Open your terminal or run `nu ~/.config/yazelix/nushell/scripts/core/start_yazelix.nu`

### Helix Integration
For Helix-Yazi integration, add this to your Helix config (`~/.config/helix/config.toml`):

```toml
[keys.normal]
# Yazelix sidebar integration - reveal current file in Yazi sidebar
A-y = ":sh nu ~/.config/yazelix/nushell/scripts/integrations/reveal_in_yazi.nu \"%{buffer_name}\""
```

📖 **[Complete Helix Keybindings Guide →](./docs/keybindings.md)** - Recommended keybindings for enhanced editing experience

## Version Check
Check installed tool versions: `nu nushell/scripts/utils/version_info.nu`

## Helix Pane Detection Logic

When opening files from Yazi, Yazelix will:
- Check the topmost pane and the next two below for a zellij pane named `editor` (which will be the Helix pane).
- If Helix is found, it is moved to the top and reused; if not, a new Helix pane is opened.
- This is need because sometimes when opening a new zellij pane in the pane stack, or deleting one, the editor pane will move around. Most of the times it will move down twice! So the workaround works.

## Version History & Changelog

For a detailed history of all major Yazelix version bumps and changelogs, see [Version History](./docs/history.md).

## Customization & Configuration

Yazelix uses a **layered configuration system** that safely merges your personal settings with Yazelix defaults:

- **Core settings**: Edit `~/.config/yazelix/yazelix.nix` for shell, editor, terminal, and package preferences
- **Tool customization**: Add personal overrides in `configs/yazi/personal/` or `configs/zellij/personal/` directories 
- **Your configs persist** across Yazelix updates without git conflicts
- **Intelligent merging**: TOML sections merge properly, avoiding duplicate keys and conflicts

📖 **[Complete Customization Guide →](./docs/customization.md)** - Detailed instructions for customizing every tool

### Editor Configuration

📝 **[Editor Configuration Guide →](./docs/editor_configuration.md)** - Complete guide for configuring editors

**Quick setup:**
- **Default (recommended)**: `editor_command = null` - Uses yazelix's Helix, no conflicts
- **System Helix**: `editor_command = "hx"` - Requires matching `helix_runtime_path` 
- **Other editors**: `editor_command = "nvim"` - Basic integration, loses Helix features

### Alternative: CLI-Only Mode
To use Yazelix tools without starting the full interface (no sidebar, no zellij):
```bash
nix develop --impure ~/.config/yazelix
```
This gives you access to all tools (helix, yazi, lazygit, etc.) in your current terminal with your preferred shell. The tools are available on-demand without the automatic Zellij workspace.

### Packages & Customization

**What Gets Installed:**
- **Essential tools**: [Yazi](https://github.com/sxyazi/yazi) (file manager), [Zellij](https://github.com/zellij-org/zellij) (terminal multiplexer), [Helix](https://helix-editor.com) (editor), shells (bash/nushell, plus your preferred shell), [fzf](https://github.com/junegunn/fzf), [zoxide](https://github.com/ajeetdsouza/zoxide), [Starship](https://starship.rs)
- **Recommended tools** (enabled by default): [lazygit](https://github.com/jesseduffield/lazygit) (or `lg`), [mise](https://github.com/jdx/mise), [cargo-update](https://github.com/nabijaczleweli/cargo-update), [ouch](https://github.com/ouch-org/ouch), [atuin](https://github.com/atuinsh/atuin) (shell history manager), etc

- **Yazi extensions** (enabled by default): `p7zip`, `jq`, `poppler`, `fd`, `ripgrep` (for archives, search, document previews)
- **Yazi media extensions** (enabled by default): `ffmpeg`, `imagemagick` (for media previews - ~800MB-1.2GB)
- **Environment setup**: Proper paths, variables, and shell configurations

**Customize Your Installation:**
If you followed [step 3 in the installation guide](./docs/installation.md#step-3-configure-your-installation-optional), you already have your `~/.config/yazelix/yazelix.nix` config file ready! You can modify it anytime and restart Yazelix to apply changes. See [yazelix_default.nix](./yazelix_default.nix) for all available options and their descriptions.

**Terminal Emulator Selection:**
- **Ghostty** (default): Modern, fast terminal written in Zig with great performance
- **WezTerm**: Better image preview support in Yazi, recommended if you need media previews
- **Kitty**: Fast, feature-rich, GPU-accelerated terminal
- **Alacritty**: Fast, GPU-accelerated terminal written in Rust
- Configure your preference in `yazelix.nix` with `preferred_terminal = "terminal_name"` (options: wezterm, ghostty, kitty, alacritty)

[See the full Customization Guide here.](./docs/customization.md)

---

## Home Manager Integration

Yazelix includes optional Home Manager support for declarative configuration management. See [home_manager/README.md](home_manager/README.md) for setup instructions.

## Notes
- The `--impure` flag in `nix develop` allows access to the HOME environment variable, necessary for config paths
- Tweak configs to make them yours; this is just a starting point! 
- For extra configuration, see: [WezTerm Docs](https://wezfurlong.org/wezterm/config/files.html)
- Add more swap layouts as needed using the KDL files in `configs/zellij/layouts`
- Use `lazygit`, it's great

## Why Use This Project?
- Easy to configure and personalize
- I daily-drive Yazelix and will always try to improve and maintain it
- Zero-conflict keybindings (no need to lock Zellij) and a powerful Yazi sidebar
- Cool Yazi plugins included out of the box
- Features like `reveal in Yazi` (from Helix) and opening files from Yazi in your configured editor
- Enhanced Git integration with `lazygit` and a customizable Starship prompt
- Nix-based setup ensures consistent, declarative, reproducible environments


## When should you not use yazelix?
- If you hate having fun
- If you suffer from a severe case of nix-allergy

## Initializer Scripts
Yazelix auto-generates initialization scripts for Starship, Zoxide, Mise, and Carapace for your configured default shell, regenerated every startup. See [docs/initializer_scripts.md](./docs/initializer_scripts.md) for details.

## yzx Command Line Interface

🔧 **Complete CLI Reference:** `yzx help` - Shell-agnostic command suite

📖 **[Complete yzx CLI Documentation →](./docs/yzx_cli.md)** - Comprehensive command reference and usage guide

**Quick Commands:**
- `yzx env` - Load Yazelix tools without UI (CLI-only mode)
- `yzx doctor [--verbose] [--fix]` - Health checks and diagnostics  
- `yzx launch` - Launch Yazelix in new terminal window
- `yzx start` - Start Yazelix in current terminal
- `yzx info` - Show system information and current settings
- `yzx versions` - Display all tool versions

## Troubleshooting

🔍 **Quick diagnosis:** `yzx doctor` - Automated health checks and fixes

📖 **[Complete Troubleshooting Guide →](./docs/troubleshooting.md)** - Comprehensive solutions for common issues

## VS Code and Cursor Integration
Want to use Yazelix tools (Nushell, zoxide, starship, lazygit) in your VS Code or Cursor integrated terminal? Now it's even easier with `yzx env`!

**Quick Setup:**
1. Open VS Code/Cursor integrated terminal
2. Run `yzx env` to load all Yazelix tools without the UI
3. Enjoy full Yazelix environment in your editor

For more advanced integration options, see our [VS Code/Cursor integration guide](./docs/vscode_cursor_integration.md).

## Styling and Themes
Yazelix includes transparency settings and theme configurations for a beautiful terminal experience. The terminal emulator configs include transparency settings you can comment/uncomment, and Helix comes with transparent theme options. See [docs/styling.md](./docs/styling.md) for customization details.

For Helix themes, you can use transparent themes by editing your Helix config:
```toml
# theme = "base16_transparent"
theme = "term16_dark"  # Recommended transparent theme
```

## Layouts
Yazelix includes adaptive layouts that organize your workspace. Use `three_column` for Claude Code and AI tools, and more. See [docs/layouts.md](./docs/layouts.md) for details and customization.

## Keybindings
Keybindings are discoverable in each tool (e.g., `~` in Yazi, `?` in lazygit). See [docs/keybindings.md](./docs/keybindings.md) for full details, custom keybindings, and usage tips.


## I'm Lost! Too Much Information
Start by learning Zellij on its own, then optionally Yazi, and re-read this README afterwards

## Contributing to Yazelix
See [contributing](./docs/contributing.md)
