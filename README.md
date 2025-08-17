# Yazelix v8: Lots of polish, support for any editor, home-manager config, better zellij tab navigation, persistent sessions and more!

## Preview
![yazelix_v8_demo](assets/demos/yazelix_v8_demo.gif)

**Latest v8.5 with zjstatus:**
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
- Like [Omakub](https://github.com/basecamp/omakub) but for your terminal
- Made with love.

## Acknowledgments
See [Yazelix Collection](./docs/yazelix_collection.md) for a full list of all projects, tools, and plugins Yazelix integrates, including links to each project and their homepages.

## Improvements of v8.5 over v8
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


## Compatibility
- **Platform**: Works on any Linux distribution. Likely works on macOS as well (untested)
- **Terminal**: WezTerm, Ghostty, Kitty, or Alacritty
- **Editor**: Any editor, but Helix has first-class support (reveal in sidebar, open buffer in running instance, etc). Configure other editors via `editor_command` setting in `yazelix.nix`
- **Shell**: Bash, Fish, Zsh, or Nushell - use whichever you prefer
- See the version compatibility table [here](./docs/version_table.md) (generated dynamically!)

## Instructions to Set It Up

**What is Nix?** Nix is just a package manager that ensures reproducible, reliable software installations. Think of it like a super-powered version of `apt`, `brew`, or `chocolatey` that:
- Never breaks your system (installs are isolated)
- Allows multiple versions of the same software
- Makes it easy to share exact development environments
- Can completely uninstall without leaving traces

**Why does Yazelix use Nix?** It guarantees that everyone gets the exact same versions of tools (Yazi, Zellij, Helix, etc.) that work perfectly together, regardless of your operating system or existing software.

**Important**: You don't need to learn Nix or Nushell to use Yazelix! Nix just installs the tools and yazelix uses nushell internally, and you can use your preferred shell (bash, fish, zsh, or nushell) for your daily work. You can install nix and nushell once, and forget they ever existed

### Prerequisites
- **Nushell** - Required to run yazelix, used internally (but you can use any of our supported shells)
  - See installation instructions: https://www.nushell.sh/book/installation.html
- **Supported terminal emulators** (choose your favorite!):
  - **WezTerm** 
    - Modern, fast, written in Rust
    - Instructions here: https://wezfurlong.org/wezterm/installation.html
  - **Ghostty** 
    - Modern, fast, written in Zig, newer
    - Instructions here: https://ghostty.org/download
    - **Note**: Due to a [Zellij/Yazi/Ghostty interaction](https://github.com/zellij-org/zellij/issues/2814#issuecomment-2965117327), image previews in Yazi may not display properly, for now. If this is a problem for you, use WezTerm instead
  - **Kitty**
    - Fast, feature-rich, GPU-accelerated terminal
    - Instructions here: https://sw.kovidgoyal.net/kitty/binary/
  - **Alacritty**
    - Fast, GPU-accelerated terminal written in Rust
    - Instructions here: https://github.com/alacritty/alacritty/blob/master/INSTALL.md

### Step-by-Step Installation

#### 1. Install Nix Package Manager
We use the **Determinate Systems Nix Installer** - it's reliable, fast, and includes modern features out of the box:

```bash
curl --proto '=https' --tlsv1.2 -sSf -L https://install.determinate.systems/nix | sh -s -- install
```

**What this does:**
- Installs Nix with flakes: just follow the instructions
- Sets up proper file permissions and system integration
- Provides a reliable uninstaller if you ever want to remove Nix
- Verify it with `nix --version`

#### 2. Download Yazelix
Clone the Yazelix repository to your system:
```bash
git clone https://github.com/luccahuguet/yazelix ~/.config/yazelix
```

#### 3. Configure Your Installation (Optional)
**Before installing dependencies**, create and customize your configuration to control what gets downloaded (else, yazelix will create a config for you based on yazelix_default.nix):

```bash
# Create your personal config from the template
cp ~/.config/yazelix/yazelix_default.nix ~/.config/yazelix/yazelix.nix

# Edit the configuration to suit your needs
# Use your preferred editor (hx, vim, etc.)
hx ~/.config/yazelix/yazelix.nix
```

**📦 Dependency Groups & Size Estimates:**

| Group | Size | Default | Description |
|-------|------|---------|-------------|
| **✅ Essential Tools** | ~225MB | Always included | Core Yazelix functionality |
| **🔧 Recommended Tools** | ~350MB | Enabled | Productivity enhancers |
| **🗂️ Yazi Extensions** | ~125MB | Enabled | File preview & archive support |
| **🎬 Yazi Media** | ~1GB | Disabled | Heavy media processing |

**💡 Installation Options:**
- **Minimal install**: ~225MB (essential only)
- **Standard install**: ~700MB (default config)
- **Full install**: ~1.7GB (all groups enabled)

📋 For detailed package breakdowns and configuration strategies, see **[Package Sizes Documentation](./docs/package_sizes.md)**
- **Custom shells**: Set `default_shell` to your preference (`"nu"`, `"bash"`, `"fish"`, `"zsh"`)
- **Terminal preference**: Set `preferred_terminal` (`"ghostty"`, `"wezterm"`, `"kitty"`, `"alacritty"`)
- **Editor choice**: Configure your editor (see Editor Configuration section below)

#### 4. Install Fonts (Required for Kitty and Alacritty)
If you're using Kitty or Alacritty, install Nerd Fonts for proper icon display using modern Nix commands:

**Option A: Using nix profile (recommended - modern replacement for nix-env):**
```bash
nix profile add nixpkgs#nerd-fonts.fira-code nixpkgs#nerd-fonts.symbols-only
```

**Option B: Using Home Manager (if you use Home Manager for system configuration):**
Add to your Home Manager configuration:
```nix
home.packages = with pkgs; [
  nerd-fonts.fira-code
  nerd-fonts.symbols-only
];
```

**Fallback: Legacy nix-env (if modern methods don't work):**
```bash
nix-env -iA nixpkgs.nerd-fonts.fira-code nixpkgs.nerd-fonts.symbols-only
```

**Note**: WezTerm and Ghostty have better font fallback and don't require this step.

#### 5. Set Up Yazelix to Auto-Launch in Your Terminal

**Option A: Automatic Launch (Recommended for most users)**  
Copy the appropriate terminal config to automatically start Yazelix:

**For WezTerm:**
```bash
cp ~/.config/yazelix/configs/terminal_emulators/wezterm/.wezterm.lua ~/.wezterm.lua
```

**For Ghostty:**
```bash
cp ~/.config/yazelix/configs/terminal_emulators/ghostty/config ~/.config/ghostty/config
```

**For Kitty:**
```bash
cp ~/.config/yazelix/configs/terminal_emulators/kitty/kitty.conf ~/.config/kitty/kitty.conf
```

**For Alacritty:**
```bash
cp ~/.config/yazelix/configs/terminal_emulators/alacritty/alacritty.toml ~/.config/alacritty/alacritty.toml
```

**Result**: Every time you open your terminal, it will automatically launch Yazelix. You won't need to run any commands.

---

**Option B: Manual Launch (For users who don't want to modify terminal configs)**

If you prefer to keep your existing terminal configuration unchanged, just run Yazelix once and it will automatically set up the `yzx` command for you:

```bash
nu ~/.config/yazelix/nushell/scripts/core/start_yazelix.nu
```

This will automatically configure your shell and then you can use:
- `yzx launch` (opens Yazelix in a new terminal window)  
- `yzx start` (starts Yazelix in current terminal)
- `yzx help` (see all available commands)

#### 6. Using Yazelix
**Option A users**: Simply open your terminal! Yazelix will automatically launch with the full environment.  
**Option B users**: Use `yzx launch` or `yzx start` to launch Yazelix when needed.

**First Run**: The first time you launch Yazelix, it will install all dependencies (Zellij, Yazi, Helix, etc.). This may take several minutes, but subsequent launches will be instant.

**Quick start tips:**
- Use `alt hjkl` to switch between Zellij panes and tabs
- Press `Enter` in Yazi to open files in your configured editor
- Use `yzx help` to see all available management commands
- Use `Alt+Shift+f` to toggle fullscreen on the current pane

#### 7. (Optional but Recommended) Configure Helix Keybindings for Yazelix Integration
To enable full Helix-Yazi integration, add the following to your Helix config (usually `~/.config/helix/config.toml`):

```toml
[keys.normal]
# Yazelix sidebar integration - reveal current file in Yazi sidebar
A-y = ":sh nu ~/.config/yazelix/nushell/scripts/integrations/reveal_in_yazi.nu \"%{buffer_name}\""
```
- **Note:** Only works for Helix instances opened from Yazi.

**Additional Recommended Helix Keybindings:**
Add these keybindings for improved editing experience:

```toml
[keys.normal]
# Vim-like paragraph navigation
"{" = "goto_prev_paragraph"
"}" = "goto_next_paragraph"
# Extend selection up by line
X = "extend_line_up"
# Print the current line's git blame information to the statusline
space.B = ":echo %sh{git blame -L %{cursor_line},+1 %{buffer_name}}"
# Reload config and buffer
A-r = [":config-reload", ":reload"]
# Toggle hidden files in file picker
space.H = ":toggle-option file-picker.hidden"
# Yank diagnostic information
C-y = ":yank-diagnostic"
# Move line up with Ctrl+k
C-k = [
  "extend_to_line_bounds",
  "delete_selection",
  "move_line_up",
  "paste_before",
]
# Move line down with Ctrl+j
C-j = ["extend_to_line_bounds", "delete_selection", "paste_after"]
# Navigate down and go to first non-whitespace
ret = ["move_line_down", "goto_first_nonwhitespace"]
# Navigate up and go to first non-whitespace
A-ret = ["move_line_up", "goto_first_nonwhitespace"]
# Open languages.toml config
tab.l = ":o ~/.config/helix/languages.toml"
# Open Helix config
tab.c = ":config-open"
g.e = "goto_file_end"
```

See [docs/keybindings.md](./docs/keybindings.md) for complete details and usage tips.

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
If you followed [step 3](#3-configure-your-installation-optional), you already have your `~/.config/yazelix/yazelix.nix` config file ready! You can modify it anytime and restart Yazelix to apply changes. See [yazelix_default.nix](./yazelix_default.nix) for all available options and their descriptions.

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

## Troubleshooting

📖 **[Complete Troubleshooting Guide →](./docs/troubleshooting.md)** - Comprehensive solutions for common issues

## VS Code and Cursor Integration
Want to use Yazelix tools (Nushell, zoxide, starship, lazygit) in your VS Code or Cursor integrated terminal? See our [VS Code/Cursor integration guide](./docs/vscode_cursor_integration.md) for step-by-step setup instructions that give you the full Yazelix environment in your editor's terminal.

## Styling and Themes
Yazelix includes transparency settings and theme configurations for a beautiful terminal experience. The WezTerm config includes transparency settings (`window_background_opacity = 0.9`), and Helix comes with transparent theme options. See [docs/styling.md](./docs/styling.md) for customization details.

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
