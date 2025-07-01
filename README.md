# Yazelix v7: Nix installs and configures everything for you!

## Overview
Yazelix integrates Yazi, Zellij, and Helix, hence the name, get it?

- Zellij orchestrates everything, with Yazi as a sidebar and Helix as the editor
- To hide the sidebar, make your pane fullscreen! (`Ctrl p + f` or `Alt f`)
- Every keybinding from Zellij that conflicts with Helix is remapped [see here](#keybindings)
- When you hit Enter on a file/folder in the "sidebar":
  - If Helix is already open in the topmost pane of the stack (default position in latest Zellij version), it opens that file/folder in a new buffer in Helix
  - If Helix isn't open, it launches Helix in a new pane for you
  - It always finds a running Helix instance if it exists and is in the top pane of the stacked group (Zellij naturally pushes the Helix pane there, though it may move when deleting or creating panes)
  - It automatically renames the Zellij tab to the file's underlying Git repo or directory name
- Features include:
  - "Reveal file in sidebar" (press `Alt y` in Helix to reveal the file in Yazi, see [Keybindings](#keybindings))
  - A Yazi plugin to enhance the status bar in the sidebar pane, making it uncluttered, colorful, and showing file permissions
  - A [Git plugin](https://github.com/yazi-rs/plugins/tree/main/git.yazi) showing file changes in the Yazi sidebar
  - Dynamic column updates in Yazi (parent, current, preview) via the [auto-layout plugin](https://github.com/josephschmitt/auto-layout.yazi), perfect for sidebar use
- This project includes config files for Zellij, Yazi, terminal emulators, Nushell scripts, Lua plugins, and a lot of love
- See [boot sequence](./docs/boot_sequence.md) for details on how Yazelix starts up

## Vision
- Using the terminal should be easy, beautiful, pratical and reproducible.
- Yazelix is always on the edge of project versions (do you like living on the edge, you know, dangerously?)
- Yazelix is always evolving, it's a living being
- Yazelix is easy to use and crazy at the same time (what really is this project?)
- Yazelix lets you say `I use Yazelix btw` (careful saying that, you might accidentally scare an innocent Arch user)
- Boy, do we Nix
- Integration, integration, integration
- Like [Omakub](https://github.com/basecamp/omakub) but for your terminal

## Preview
![yazelix_v7_demo](assets/demos/yazelix-v7-demo.gif)

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
- Adds `launch-yazelix.sh` script to streamline setup by launching WezTerm with the Yazelix-specific config and automatically adding `yazelix` and `yzx` aliases to your shell configuration (e.g., `~/.bashrc` or `~/.zshrc`) and Nushell config, eliminating manual configuration steps, if you want to! See more details [here](./docs/terminal_setup.md)
- The `clip` command from [nuscripts](https://github.com/nushell/nuscripts) is included, allowing you to copy text to the system clipboard directly from Nushell. Use it like `ls src/*.py | get name | to text | clip` or `open data.csv | clip`, etc
- Introduces dynamic Zellij configuration generation on demand using `nushell/scripts/setup/generate-zellij-config.nu`, which combines Zellij's default settings with Yazelix-specific overrides from `zellij/yazelix-overrides.kdl`, making it easy to stay up-to-date with Zellij defaults while preserving custom settings
- Allows for declaration user-defined git-ignored nix packages directly in yazelix.nix
- Improves the "reveal file in sidebar" feature by using Yazi's `reveal` command to automatically highlight and select the specific file, eliminating manual searching in directories with many files
- Introduces dynamic version table generation using `nu nushell/scripts/utils/version-info.nu`


## Compatibility
- Terminal: Ghostty or WezTerm
- Editor: Helix (`hx` binary provided by Nix)
- Shell: Nushell, Bash, Fish, Zsh
- See the version compatibility table [here](./docs/version_table.md) (generated dynamically!)

## Version Check
Check installed tool versions: `nu nushell/scripts/utils/version-info.nu`

## Instructions to Set It Up

**Firstly, what is Nix?** Nix is a powerful package manager that ensures reproducible, reliable software installations. Think of it like a super-powered version of `apt`, `brew`, or `chocolatey` that:
- Never breaks your system (installs are isolated): High reproducibility
- Allows multiple versions of the same software
- Makes it easy to share exact development environments
- Can completely uninstall without leaving traces

**Nix allows yazelix to let you take _full_ control of your shell**

**Why does Yazelix use Nix?** It guarantees that everyone gets the exact same versions of tools (Yazi, Zellij, Helix, etc.) that work perfectly together, regardless of your operating system or existing software.

### Prerequisites
- **Supported terminal emulator** (choose your favorite! Or both?):
  - **Ghostty** 
    - Modern, fast, written in Zig, newer
    - Instructions here: https://ghostty.org/download
    - **Note**: Due to a [Zellij/Yazi/Ghostty interaction](https://github.com/zellij-org/zellij/issues/2814#issuecomment-2965117327), image previews in Yazi may not display properly, for now
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
cp ~/.config/yazelix/terminal_configs/ghostty/config ~/.config/ghostty/config
```

**For WezTerm:**
```bash
cp ~/.config/yazelix/terminal_configs/wezterm/.wezterm.lua ~/.wezterm.lua
```

**Result**: Every time you open your terminal, it will automatically launch Yazelix. You won't need to run any commands.

**Alternative**: See [Terminal Setup Guide](./docs/terminal_setup.md) to be able to launch Yazelix from your terminal (e.g., `yazelix` or `yzx`)

#### 4. Using Yazelix
Simply open your terminal (Ghostty or WezTerm)! Yazelix will automatically launch with the full environment.
Might take a while to install the dependencies, but only on the first run

**Quick start tips:**
- Use `alt hjkl` to switch between Zellij panes and tabs
- Press `Enter` in Yazi to open files in Helix

### Alternative: CLI-Only Mode
To use Yazelix tools without starting the full interface (no sidebar, no zellij):
```bash
nix develop --impure ~/.config/yazelix
```
This gives you access to all tools (helix, yazi, lazygit, etc.) in your current terminal, that includes yazi and zellij, but they'll open on demand, not on their own.

### Packages & Customization

**What Gets Installed:**
- **Required tools**: [Yazi](https://github.com/sxyazi/yazi) (file manager), [Zellij](https://github.com/zellij-org/zellij) (terminal multiplexer), [Helix](https://helix-editor.com) (editor), [Nushell](https://www.nushell.sh/book/installation.html) (shell), [fzf](https://github.com/junegunn/fzf), [zoxide](https://github.com/ajeetdsouza/zoxide), [Starship](https://starship.rs)
- **Optional tools** (enabled by default): [lazygit](https://github.com/jesseduffield/lazygit) (or `lg`), [mise](https://github.com/jdxcode/mise), [cargo-update](https://github.com/nabijaczleweli/cargo-update), [ouch](https://github.com/ouch-org/ouch), etc

- **Yazi extensions** (enabled by default): `p7zip`, `jq`, `poppler`, `fd`, `ripgrep` (for archives, search, document previews)
- **Yazi media extensions** (enabled by default): `ffmpeg`, `imagemagick` (for media previews - ~800MB-1.2GB)
- **Environment setup**: Proper paths, variables, and shell configurations

**Customize Your Installation:**
Edit `~/.config/yazelix/yazelix.nix` (auto-created from template on first run). See [yazelix_default.nix](./yazelix_default.nix) for all available options and their descriptions.

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
- Features like `reveal in Yazi` (from Helix) and opening files from Yazi in a Helix buffer
- Enhanced Git integration with `lazygit` and a customizable Starship prompt
- Nix-based setup ensures consistent, reproducible environments

## Initializer Scripts
See [docs/initializer_scripts.md](./docs/initializer_scripts.md) for details on how Yazelix generates and uses initializer scripts for Nushell and Bash/Zsh.

## Troubleshooting
For setup issues, version compatibility, and debugging:

- Review the [Terminal Setup Guide](./docs/terminal_setup.md) for terminal configuration problems
- See [Version Table](./docs/version_table.md) for compatibility information
- For general issues, check the logs in the `logs/` directory or enable debug mode in `yazelix.nix`

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

Yazelix is built on the shoulders of giants. We're grateful to the essential projects that make this integration possible:

**Core Tools:**
- **Yazi**
- **Zellij**
- **Helix**
- **Nushell**
- **Nix**

**Utilities:**
- **Starship**
- **lazygit**
- **fzf**
- **zoxide**

**Terminal Emulators:**
- **WezTerm**
- **Ghostty**

**One of a Kind Contributions:**
- **nuscripts** - Source of the `clip` command (MIT License)
- **auto-layout.yazi** - 95% of the work by Joseph Schmitt

Thank you to all maintainers and the open source community. 
