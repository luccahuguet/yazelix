# Yazelix v7.8: Nix installs and configures everything for you!

## Preview
![yazelix_v7_demo](assets/demos/yazelix_v7_demo.gif)

## Overview
Yazelix integrates Yazi, Zellij, and Helix, hence the name, get it?

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

## Improvements of v7.9 over v7
- **Modular Editor Support**: Complete rewrite of file opening logic to support any editor while preserving full Helix integration. Now you can use Vim, Nano, Emacs, or any editor via the `editor_command` setting in `yazelix.nix` - Helix users get all advanced features (open in same buffer, reveal in sidebar, etc), while other editors get basic Zellij integration (new panes, tab renaming)
- **Big File/Folder Project-Wide Refactoring**: Complete reorganization of the codebase structure for better maintainability and organization
- **YZX Command Polish**: Enhanced the `yzx` command with improved functionality and user experience
- **Yazelix Config Validation**: Added validation system to warn users of invalid configuration options in `yazelix.nix`
- **Configurable Editor Environment Variables**: New config options to set EDITOR environment variable when empty, override existing EDITOR, and add custom editor commands
- **Configurable Welcome ASCII Art**: You can now choose between animated or static ASCII art in the welcome screen using the new `ascii_art_mode` option in your `yazelix.nix` config. Set `ascii_art_mode = "animated"` (default) or `ascii_art_mode = "static"` for a non-animated welcome.
- **Shell-agnostic `yzx` command**: Introduces a unified `yzx` command that works across all supported shells (bash, fish, zsh, nushell) with full subcommand support. No more shell-specific limitations - use `yzx help`, `yzx get_config`, `yzx versions`, etc. from any shell!
- **Seamless Yazelix restart**: `yzx restart` launches a new Yazelix instance before killing the old one, providing a smooth and reliable restart experience.
- **Project Credits page**: Yazelix now includes a dedicated credits page (`docs/project_credits.md`) listing all integrated tools and inspirations.
- **Added macchina to welcome screen**: Added a system info summary using macchina (neofetch alternative) to the welcome screen. It can be disabled in the config.
- **Dynamic Config Validation**: Yazelix now uses a dynamic config validator that checks your config against yazelix_default.nix every time Yazelix starts. It warns about unknown fields, missing fields, and invalid values for key options (like default_shell, helix_mode, preferred_terminal, ascii_art_mode). No more silent config errors!
- **Zellij Tab Movement Shortcuts**: Added new keybindings in Zellij: `Alt+Shift+H` to move the current tab left, and `Alt+Shift+L` to move the current tab right. This makes tab management much faster and more intuitive.

## Improvements of v7 over v6
- **Warning**: After upgrading to Yazelix v7, terminate any running zellij sessions and old terminals to prevent conflicts
- Introduces a Nix-based development environment via `flake.nix`, simplifying dependency installation and ensuring consistent versions for Zellij, Yazi, Helix, Nushell, lazygit, Starship, and other tools
- Introduces `yazelix.nix` configuration file for customizing dependencies, shells, and build options!
- Adds [lazygit](https://github.com/jesseduffield/lazygit), a fast, terminal-based Git TUI for managing Git repositories
- Adds [Starship](https://starship.rs), a customizable, fast prompt for Nushell, enhancing the terminal experience with Git status and contextual info
- Adds [markdown-oxide](https://oxide.md/index), a Personal Knowledge Management System (PKMS) that works with your favorite text editor through LSP, inspired by and compatible with Obsidian
- Allows you to build Helix from source automatically
- Installs and configures dependencies automatically
- Introduces (optional) yazelix welcome screen with helpful tips and better error handling during environment setup
- Adds terminal transparency settings because we reaaally believe in transparency
- Adds `launch_yazelix.nu` script to launch your preferred terminal with the Yazelix-specific config. The `yazelix` and `yzx` aliases are automatically available in your shell once the Yazelix shell configurations are sourced.
- The `clip` command from [nuscripts](https://github.com/nushell/nuscripts) is included, allowing you to copy text to the system clipboard directly from Nushell. Use it like `ls src/*.py | get name | to text | clip` or `open data.csv | clip`, etc
- Introduces dynamic Zellij configuration generation on demand using `nushell/scripts/setup/generate_zellij_config.nu`, which combines Zellij's default settings with Yazelix-specific overrides from `configs/zellij/yazelix_overrides.kdl`, making it easy to stay up-to-date with Zellij defaults while preserving custom settings
- Allows for declaration user-defined git-ignored nix packages directly in yazelix.nix
- Improves the "reveal file in sidebar" feature by using Yazi's `reveal` command to automatically highlight and select the specific file, eliminating manual searching in directories with many files
- Introduces dynamic version table generation using `nu nushell/scripts/utils/version_info.nu`


## Compatibility
- Terminal: Ghostty or WezTerm
- Editor: Any editor, but hx is has first class support (`reaveal in sidebar, open buffer in running hx instance, etc). Configure other editors via `editor_command` setting in `yazelix.nix`
- Shell: Nushell, Bash, Fish, Zsh
- See the version compatibility table [here](./docs/version_table.md) (generated dynamically!)

## Version Check
Check installed tool versions: `nu nushell/scripts/utils/version_info.nu`

## Instructions to Set It Up

**Firstly, what is Nix?** Nix is a powerful package manager that ensures reproducible, reliable software installations. Think of it like a super-powered version of `apt`, `brew`, or `chocolatey` that:
- Never breaks your system (installs are isolated): High reproducibility
- Allows multiple versions of the same software
- Makes it easy to share exact development environments
- Can completely uninstall without leaving traces

**Nix allows yazelix to let you take _full_ control of your shell**

**Why does Yazelix use Nix?** It guarantees that everyone gets the exact same versions of tools (Yazi, Zellij, Helix, etc.) that work perfectly together, regardless of your operating system or existing software.

### Prerequisites
- **Nushell** - Required to boot Yazelix
  - See installation instructions: https://www.nushell.sh/book/installation.html
- **Supported terminal emulators** (choose your favorite! Or both?):
  - **Ghostty** 
    - Modern, fast, written in Zig, newer
    - Instructions here: https://ghostty.org/download
    - **Note**: Due to a [Zellij/Yazi/Ghostty interaction](https://github.com/zellij-org/zellij/issues/2814#issuecomment-2965117327), image previews in Yazi may not display properly, for now. If this is a problem for you, use WezTerm instead
  - **WezTerm** 
    - Modern, fast, written in Rust
    - Instructions here: https://wezfurlong.org/wezterm/installation.html

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

#### 3. Set Up Yazelix to Auto-Launch in Your Terminal
Copy the appropriate terminal config to automatically start Yazelix:

**For Ghostty:**
```bash
cp ~/.config/yazelix/configs/terminal_emulators/ghostty/config ~/.config/ghostty/config
```

**For WezTerm:**
```bash
cp ~/.config/yazelix/configs/terminal_emulators/wezterm/.wezterm.lua ~/.wezterm.lua
```

**Result**: Every time you open your terminal, it will automatically launch Yazelix. You won't need to run any commands.

**Alternative**: Use `yazelix` or `yzx` commands from any terminal to launch Yazelix (aliases are automatically available when shell configs are sourced). The `yzx` command also provides additional subcommands like `yzx help`, `yzx get_config`, `yzx versions`, etc. for managing your Yazelix installation.

#### 4. Using Yazelix
Simply open your terminal (Ghostty or WezTerm)! Yazelix will automatically launch with the full environment.

**First Run**: The first time you open your terminal, Yazelix will install all dependencies (Zellij, Yazi, Helix, etc.). This may take several minutes, but subsequent launches will be instant.

**Quick start tips:**
- Use `alt hjkl` to switch between Zellij panes and tabs
- Press `Enter` in Yazi to open files in your configured editor
- Use `yazelix` or `yzx` commands from any terminal to launch Yazelix manually
- Use `yzx help` to see all available management commands

#### 5. (Optional but Recommended) Configure Helix Keybinding for Yazelix Integration
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
This gives you access to all tools (helix, yazi, lazygit, etc.) in your current terminal, that includes yazi and zellij, but they'll open on demand, not on their own.

### Packages & Customization

**What Gets Installed:**
- **Essential tools**: [Yazi](https://github.com/sxyazi/yazi) (file manager), [Zellij](https://github.com/zellij-org/zellij) (terminal multiplexer), [Helix](https://helix-editor.com) (editor), [Nushell](https://www.nushell.sh/book/installation.html) (shell), [fzf](https://github.com/junegunn/fzf), [zoxide](https://github.com/ajeetdsouza/zoxide), [Starship](https://starship.rs)
- **Recommended tools** (enabled by default): [lazygit](https://github.com/jesseduffield/lazygit) (or `lg`), [mise](https://github.com/jdx/mise), [cargo-update](https://github.com/nabijaczleweli/cargo-update), [ouch](https://github.com/ouch-org/ouch), [atuin](https://github.com/atuinsh/atuin) (shell history manager), etc

- **Yazi extensions** (enabled by default): `p7zip`, `jq`, `poppler`, `fd`, `ripgrep` (for archives, search, document previews)
- **Yazi media extensions** (enabled by default): `ffmpeg`, `imagemagick` (for media previews - ~800MB-1.2GB)
- **Environment setup**: Proper paths, variables, and shell configurations

**Customize Your Installation:**
Edit `~/.config/yazelix/yazelix.nix` (auto-created from template on first run). See [yazelix_default.nix](./yazelix_default.nix) for all available options and their descriptions.

**Terminal Emulator Selection:**
- **WezTerm** (default): Better image preview support in Yazi, recommended for most users
- **Ghostty**: Modern, fast terminal written in Zig
- Configure your preference in `yazelix.nix` with `preferred_terminal = "wezterm"` or `preferred_terminal = "ghostty"`

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
- Nix-based setup ensures consistent, reproducible environments

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
- If you frequently use other terminal editors besides Helix or terminal file managers other than Yazi, check out [zide](https://github.com/josephschmitt/zide)
- If you care about Yazi but don't care much about Zellij or having a sidebar, you can integrate Yazi and Helix with [one line of config](https://github.com/sxyazi/yazi/pull/2461) (experimental, not working for some people as of March 15, 2025)

## Acknowledgments

See [Project Credits](./docs/project_credits.md) for a full list of all projects, tools, and plugins Yazelix integrates, including links to each project and their homepages. 
