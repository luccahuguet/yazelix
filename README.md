# Yazelix v7.5: Nix installs and configures everything for you!

## Preview
![yazelix_v7_demo](assets/demos/yazelix_v7_demo.gif)

## Overview
Yazelix integrates Yazi, Zellij, and Helix, hence the name, get it?

- **Use your preferred shell**: Bash, Fish, Zsh, or Nushell - Yazelix works with all of them
- Zellij orchestrates everything, with Yazi as a sidebar and your chosen editor (Helix by default)
- To hide the sidebar, make your pane fullscreen! (`Ctrl p + f` or `Alt f`)
- Every keybinding from Zellij that conflicts with Helix is remapped [see here](#keybindings)
- When you hit Enter on a file/folder in the "sidebar":
  - **With Helix**: If Helix is already open in the topmost pane of the stack, it opens that file/folder in a new buffer in Helix. If Helix isn't open, it launches Helix in a new pane for you. It always finds a running Helix instance if it exists and is in the top pane of the stacked group.
  - **With other editors**: Opens the file in a new pane with your configured editor
  - It automatically renames the Zellij tab to the file's underlying Git repo or directory name
- Features include:
  - "Reveal file in sidebar" (press `Alt y` in Helix to reveal the file in Yazi, see [Keybindings](#keybindings))
  - A Yazi plugin to enhance the status bar in the sidebar pane, making it uncluttered, colorful, and showing file permissions
  - A [Git plugin](https://github.com/yazi-rs/plugins/tree/main/git.yazi) showing file changes in the Yazi sidebar
  - Dynamic column updates in Yazi (parent, current, preview) via the [auto-layout plugin](https://github.com/josephschmitt/auto-layout.yazi), perfect for sidebar use
  - **Modular editor support**: Use Helix for full integration features, or any other editor via the `editor_command` setting
- This project includes config files for Zellij, Yazi, terminal emulators, Nushell scripts, Lua plugins, and a lot of love
- See [boot sequence](./docs/boot_sequence.md) for details on how Yazelix starts up

## Vision
- Using the terminal should be easy, beautiful, pratical and reproducible.
- Yazelix is always on the edge of project versions
- Yazelix is always evolving, it's a living being
- Yazelix is easy to use and crazy at the same time (what really is this project?)
- Yazelix lets you say `I use Yazelix btw` (careful saying that, you might accidentally scare an innocent Arch user)
- Boy, do we Nix
- Integration, integration, integration
- Like [Omakub](https://github.com/basecamp/omakub) but for your terminal

## Acknowledgments
See [Project Credits](./docs/project_credits.md) for a full list of all projects, tools, and plugins Yazelix integrates, including links to each project and their homepages.

## Improvements of v7.5 over v7
- **Modular Editor Support**: Complete rewrite of file opening logic to support any editor while preserving full Helix integration. Now you can use Vim, Nano, Emacs, or any editor via the `editor_command` setting in `yazelix.nix` - Helix users get all advanced features (open in same buffer, reveal in sidebar, etc), while other editors get basic Zellij integration (new panes, tab renaming)
- **Big File/Folder Project-Wide Refactoring**: Complete reorganization of the codebase structure for better maintainability and organization
- **Yazelix Config Validation**: Added validation system to warn users of invalid configuration options in `yazelix.nix`
- **Configurable Editor Environment Variables**: New config options to set EDITOR environment variable when empty, override existing EDITOR, and add custom editor commands
- **Configurable Welcome ASCII Art**: You can now choose between animated or static ASCII art in the welcome screen using the new `ascii_art_mode` option in your `yazelix.nix` config. Set `ascii_art_mode = "animated"` (default) or `ascii_art_mode = "static"` for a non-animated welcome.
- **Shell-agnostic `yzx` command**: Introduces a unified `yzx` command that works across all supported shells (bash, fish, zsh, nushell) with full subcommand support. No more shell-specific limitations - use `yzx help`, `yzx get_config`, `yzx versions`, etc. from any shell!
- **Seamless Yazelix restart**: `yzx restart` launches a new Yazelix instance before killing the old one, providing a smooth and reliable restart experience.
- **Project Credits page**: Yazelix now includes a dedicated credits page (`docs/project_credits.md`) listing all integrated tools and inspirations.
- **Added macchina to welcome screen**: Added a system info summary using macchina (neofetch alternative) to the welcome screen. It can be disabled in the config.
- **Dynamic Config Validation**: Yazelix now uses a dynamic config validator that checks your config against yazelix_default.nix every time Yazelix starts. It warns about unknown fields, missing fields, and invalid values for key options (like default_shell, helix_mode, preferred_terminal, ascii_art_mode). No more silent config errors!
- **Improved Helix Pane Detection**: Yazelix now checks the topmost pane and the next two below for a Zellij pane named `editor` (the Helix pane) when opening files from Yazi, reusing it if found, or opening a new one if not. See [Helix Pane Detection Logic](#helix-pane-detection-logic) for details.
- **Ergonomic Tab Navigation**: Added browser-like tab navigation in Zellij:
  - `Alt+number` to jump directly to tabs 1-9
  - `Alt+w/q` to walk (focus) next/previous tab
  - `Alt+Shift+H/L` to move tabs left/right
  - Cleaned up legacy/conflicting keybindings for a more user-friendly experience
- **Persistent Sessions Configuration**: Added support for persistent Zellij sessions with flexible configuration parsing. Configure `persistent_sessions = true` and `session_name = "your_session"` in `yazelix.nix` to reuse the same session across restarts
- **Full version history and project evolution is now documented in detail (see Version History & Changelog below)**

## Version History & Changelog

For a detailed history of all major Yazelix version bumps and changelogs, see [Version History](./docs/history.md).

## Helix Pane Detection Logic

When opening files from Yazi, Yazelix will:
- Check the topmost pane and the next two below for a zellij pane named `editor` (which will be the Helix pane).
- If Helix is found, it is moved to the top and reused; if not, a new Helix pane is opened.
- This is need because sometimes when opening a new zellij pane in the pane stack, or deleting one, the editor pane will move around. Most of the times it will move down twice! So the workaround works.

## Compatibility
- **Terminal**: WezTerm, Ghostty, Kitty, or Alacritty
- **Editor**: Any editor, but Helix has first-class support (reveal in sidebar, open buffer in running instance, etc). Configure other editors via `editor_command` setting in `yazelix.nix`
- **Shell**: Bash, Fish, Zsh, or Nushell - use whichever you prefer
- See the version compatibility table [here](./docs/version_table.md) (generated dynamically!)

## Version Check
Check installed tool versions: `nu nushell/scripts/utils/version_info.nu`

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
- **Editor choice**: Change `editor_command` from `"hx"` to `"vim"`, `"nvim"`, etc. if you prefer

#### 4. Install Fonts (Required for Kitty and Alacritty)
If you're using Kitty or Alacritty, install Nerd Fonts for proper icon display:

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
- Use `Alt+f` to toggle fullscreen on the current pane

#### 7. (Optional but Recommended) Configure Helix Keybinding for Yazelix Integration
To enable the "reveal file in Yazi sidebar" feature from within Helix (press `Alt-y` in normal mode), add the following to your Helix config (usually `~/.config/helix/config.toml`):

```toml
[keys.normal]
A-y = ":sh nu ~/.config/yazelix/nushell/scripts/integrations/reveal_in_yazi.nu \"%{buffer_name}\""
```
- This lets you quickly reveal the current file in the Yazi sidebar from Helix.
- See [docs/keybindings.md](./docs/keybindings.md) for more details and tips.
- **Limitation:** Only works for Helix instances opened from Yazi.

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

## Notes
- The `--impure` flag in `nix develop` allows access to the HOME environment variable, necessary for config paths
- Tweak configs to make them yours; this is just a starting point! 
- For extra configuration, see: [WezTerm Docs](https://wezfurlong.org/wezterm/config/files.html)
- Add more swap layouts as needed using the KDL files in `layouts`
- Use `lazygit`
- **Steel Support**: Patchy and Steel support was temporarily removed due to rapid codebase growth causing integration conflicts. Pre-release testing didn't catch all edge cases. A polished, stable Steel integration will be readded soon.

## Why Use This Project?
- Easy to configure and personalize
- I daily-drive Yazelix and will always try to improve and maintain it
- Zero-conflict keybindings (no need to lock Zellij) and a powerful Yazi sidebar
- Cool Yazi plugins included out of the box
- Features like `reveal in Yazi` (from Helix) and opening files from Yazi in your configured editor
- Enhanced Git integration with `lazygit` and a customizable Starship prompt
- Nix-based setup ensures consistent, declarative, reproducible environments

## Initializer Scripts
See [docs/initializer_scripts.md](./docs/initializer_scripts.md) for details on how Yazelix generates and uses initializer scripts for Nushell and Bash/Zsh.

## Troubleshooting
For setup issues, version compatibility, and debugging:

- See [Version Table](./docs/version_table.md) for compatibility information
- For general issues, check the logs in the `logs/` directory or enable debug mode in `yazelix.nix`
- **Naming Convention**: If you encounter "file not found" errors, ensure your terminal configs reference the updated script names (using underscores instead of hyphens, e.g., `start_yazelix.nu` instead of `start-yazelix.nu`)
- **Script Reorganization**: If you encounter "file not found" errors after updating Yazelix, your terminal configs may need updating. The scripts have been reorganized into subdirectories:
  - **WezTerm**: Update `~/.wezterm.lua` to use `~/.config/yazelix/nushell/scripts/core/start_yazelix.nu`
  - **Ghostty**: Update `~/.config/ghostty/config` to use `~/.config/yazelix/nushell/scripts/core/start_yazelix.nu`

## VS Code and Cursor Integration
Want to use Yazelix tools (Nushell, zoxide, starship, lazygit) in your VS Code or Cursor integrated terminal? See our [VS Code/Cursor integration guide](./docs/vscode_cursor_integration.md) for step-by-step setup instructions that give you the full Yazelix environment in your editor's terminal.

## Styling and Themes
Yazelix includes transparency settings and theme configurations for a beautiful terminal experience. The WezTerm config includes transparency settings (`window_background_opacity = 0.9`), and Helix comes with transparent theme options. See [docs/styling.md](./docs/styling.md) for customization details.

For Helix themes, you can use transparent themes by editing your Helix config:
```toml
# theme = "base16_transparent"
theme = "term16_dark"  # Recommended transparent theme
```

## Keybindings
Keybindings are discoverable in each tool (e.g., `~` in Yazi, `?` in lazygit). See [docs/keybindings.md](./docs/keybindings.md) for full details, custom keybindings, and usage tips.

## I'm Lost! Too Much Information
Start by learning Zellij on its own, then optionally Yazi, and re-read this README afterwards

## Contributing to Yazelix
See [contributing](./docs/contributing.md)

## Similar Projects
- If you care about Yazi but don't care much about Zellij or having a sidebar, you can integrate Yazi and Helix with [one line of config](https://github.com/sxyazi/yazi/pull/2461) (experimental, not working for some people as of March 15, 2025) 
