# Yazelix v7: Nix installs everything for you!

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
- The boot sequence of the Nix version is the following:
  - You run `yazelix` or `yzx` (or `~/.config/yazelix/bash/launch-yazelix.sh`) -> The `launch-yazelix.sh` script automatically adds `yazelix` and `yzx` aliases to your shell configuration (e.g., `~/.bashrc` or `~/.zshrc`) and launches WezTerm with the Yazelix-specific configuration.
  - WezTerm, as configured by `~/.config/yazelix/terminal_configs/wezterm_nix/.wezterm.lua`, then executes the `~/.config/yazelix/bash/start-yazelix.sh` script.
  - The `start-yazelix.sh` script navigates to the Yazelix project directory and runs `nix develop --impure --command ...`.
  - Inside the `nix develop` environment:
    - The `flake.nix` reads `~/.config/yazelix/yazelix.toml` to determine configurations, including the `default_shell` (which defaults to `nu` but can be set to `bash`).
    - Dependencies are installed.
    - The `shellHook` generates initializer scripts for Bash and Nushell, and exports the chosen default shell as an environment variable (`YAZELIX_DEFAULT_SHELL`).
    - Finally, Zellij is launched using the `YAZELIX_DEFAULT_SHELL` to set its default shell (e.g., `zellij --default-shell nu`).

## Vision
- Yazelix is always on the edge of project versions (do you like living on the edge, you know, dangerously?)
- Yazelix is always evolving, it's a living being
- Yazelix is easy to use and crazy at the same time (what really is this project?)
- Yazelix lets you say `I use Yazelix btw` (careful saying that, you might accidentally scare an innocent Arch user)
- Boy, do we Nix
- Integration, integration, integration

## Preview
![yazelix_v6_demo](assets/reveal_fullscreen_stacked.gif)
v6 demo

## Improvements of v7 over v6
- **Warning**: After upgrading to Yazelix v7, terminate any running zellij sessions and old terminals to prevent conflicts
- Introduces a Nix-based development environment via `flake.nix`, simplifying dependency installation and ensuring consistent versions for Zellij, Yazi, Helix, Nushell, lazygit, Starship, and other tools (recommended installation method)
- Adds [lazygit](https://github.com/jesseduffield/lazygit), a fast, terminal-based Git TUI for managing Git repositories
- Adds [Starship](https://starship.rs), a customizable, fast prompt for Nushell, enhancing the terminal experience with Git status and contextual info
- Adds [markdown-oxide](https://oxide.md/index), a Personal Knowledge Management System (PKMS) that works with your favorite text editor through LSP, inspired by and compatible with Obsidian
- Allows you to build Helix from source automatically
- Installs and configures dependencies automatically
- The `clip` command from [nuscripts](https://github.com/nushell/nuscripts) is included, allowing you to copy text to the system clipboard directly from Nushell. Use it like `ls src/*.py | get name | to text | clip`.
- Adds `launch-yazelix.sh` script to streamline setup by launching WezTerm with the Yazelix-specific config and automatically adding `yazelix` and `yzx` aliases to your shell configuration (e.g., `~/.bashrc` or `~/.zshrc`) and Nushell config, eliminating manual configuration steps
- Introduces dynamic Zellij configuration generation on demand using `nushell/scripts/generate-zellij-config.nu`, which combines Zellij's default settings with Yazelix-specific overrides from `zellij/yazelix-overrides.kdl`, making it easy to stay up-to-date with Zellij defaults while preserving custom settings

## Compatibility
- The Nix-based installation currently supports only WezTerm; the Cargo-based installation supports any terminal emulator, including WezTerm and Ghostty (includes a Ghostty config)
- Editor: Helix (for now)
- See the version compatibility table [here](./docs/table_of_versions.md)

## Instructions to Set It Up
Yazelix v7 offers two installation pipelines: **Nix-based (recommended)** for a consistent, reproducible environment (requires WezTerm), and **Cargo-based** for users preferring a straightforward Rust-based setup with any terminal emulator.

### Option 1: Nix-Based Installation (Recommended)
1. Install Nix (Single-User):
   - On Linux/macOS, run the following command to install Nix in single-user mode:
     ```bash
     curl --proto '=https' --tlsv1.2 -sSf -L https://install.determinate.systems/nix | sh -s -- install
     ```
   - Follow the prompts to complete installation. This sets up Nix for the current user only, which is simpler and suits most Yazelix users.
   - Multi-user installations (using `--daemon`) may work but are untested with Yazelix. If you need multi-user, see the [Nix installation docs](https://nixos.org/manual/nix/stable/installation/multi-user.html) and ensure `/nix` is accessible. For single-user, ensure `~/.nix-profile` is in your PATH.
2. Enable Nix flakes:
   - Create or edit `~/.config/nix/nix.conf` to enable experimental features:
     ```bash
     mkdir -p ~/.config/nix
     echo "experimental-features = nix-command flakes" >> ~/.config/nix/nix.conf
     ```
3. Clone this repo into your `~/.config` directory:
   ```bash
   git clone https://github.com/luccahuguet/yazelix ~/.config/yazelix
   ```
4. Launch Yazelix with this command:
     ```bash
     chmod +x ~/.config/yazelix/bash/launch-yazelix.sh
     ~/.config/yazelix/bash/launch-yazelix.sh
     ```
   - This launches WezTerm with the Yazelix-specific configuration, starting Zellij with the Yazelix layout. The script automatically adds `yazelix` and `yzx` aliases to your shell configuration (e.g., `~/.bashrc` or `~/.zshrc`). To use the aliases in the current session, run `source ~/.bashrc` (or `source ~/.zshrc` if using Zsh).
   - After sourcing, you can launch Yazelix by running `yazelix` or `yzx` in your terminal.
5. Optional: Enter the Nix development environment without Zellij:
   - To load the Yazelix tools without starting Zellij, run:
     ```bash
     nix develop --impure ~/.config/yazelix
     ```
   - In this environment, `yazelix` and `yzx` aliases are also available in Nushell.

   This installs and configures:
   - Required:
     - [Yazi](https://github.com/sxyazi/yazi) (file manager and CLI)
     - [Zellij](https://github.com/zellij-org/zellij) (terminal multiplexer)
     - [Helix](https://helix-editor.com) (editor, built from source by default)
     - [Nushell](https://www.nushell.sh/book/installation.html) (shell)
     - [fzf](https://github.com/junegunn/fzf) (fuzzy finder for Yazi)
     - [zoxide](https://github.com/ajeetdsouza/zoxide) (smart directory navigation)
     - [Starship](https://starship.rs) (customizable prompt)
   - Optional (enabled by default in `yazelix.toml`): [cargo-update](https://github.com/nabijaczleweli/cargo-update) (updates Rust crates), [cargo-binstall](https://github.com/cargo-bins/cargo-binstall) (faster Rust tool installation), [lazygit](https://github.com/jesseduffield/lazygit) (Git TUI), [mise](https://github.com/jdxcode/mise) (tool version manager), [ouch](https://github.com/ouch-org/ouch) (compression tool)
   - Yazi Extensions (enabled by default in `yazelix.toml`): `ffmpeg`, `p7zip`, `jq`, `poppler`, `fd`, `ripgrep`, `imagemagick` (extend Yazi's functionality, e.g., media previews, archives, search)
   - Sets environment variables: `YAZI_CONFIG_HOME` (points to `~/.config/yazelix/yazi`), `ZELLIJ_DEFAULT_LAYOUT` (set to `yazelix`), and `EDITOR` (set to `hx`)
   - Configurable in `~/.config/yazelix/yazelix.toml`:
     - `build_helix_from_source` (default: `true`): Set to `false` to use the pre-built Helix from `nixpkgs` instead of building from source. Building from source ensures the latest Helix features (e.g., for `Alt y` to reveal files in Yazi) but takes longer. Using `nixpkgs` is faster but may use an older version; check compatibility in `./docs/table_of_versions.md`.
     - `include_optional_deps` (default: `true`): Set to `false` to exclude optional dependencies like `mise` and `lazygit`.
     - `include_yazi_extensions` (default: `true`): Set to `false` to exclude Yazi extension dependencies like `ffmpeg` and `poppler`.
     - `default_shell` (default: `"nu"`): Sets the default shell for Zellij when Yazelix starts.
       - Accepted values: `"nu"` (for Nushell) or `"bash"`.
       - If this option is omitted from `yazelix.toml`, it defaults to `"nu"`.
       - Both Nushell and Bash are always installed by the Nix environment and available for use, regardless of this setting. This option only controls the default shell Zellij launches into.

6. (Optional) Make Yazelix's Yazi config your default (plugin-enhanced, width-adjusted):
   - For Nushell users, add to `~/.config/nushell/env.nu` (edit with `config env`):
     ```nushell
     $env.YAZI_CONFIG_HOME = "~/.config/yazelix/yazi"
     ```

### Option 2: Cargo-Based Installation (UNTESTED, you might prefer using the main branch...)
See the detailed [Cargo-based installation guide](./docs/cargo_installation.md) for instructions on installing dependencies with `cargo` and configuring your terminal emulator.

## Notes
- The Nix-based approach is recommended for its reproducibility and ease of dependency management. It requires WezTerm, which is configured (via `~/.config/yazelix/terminal_configs/wezterm_nix/.wezterm.lua`) to run the `~/.config/yazelix/bash/start-yazelix.sh` script upon launch. The `launch-yazelix.sh` script initiates this process. The `start-yazelix.sh` script then sets up the Nix environment and starts Zellij.
- The `--impure` flag in `nix develop` allows access to the HOME environment variable, necessary for config paths.
- The Cargo-based approach supports any terminal emulator, offering more flexibility.
- Tweak configs to make them yours; this is a starting point.
- For extra configuration, see: [WezTerm Docs](https://wezfurlong.org/wezterm/config/files.html) or [Ghostty Docs](https://ghostty.org/docs/config).
- Run `~/.config/yazelix/bash/launch-yazelix.sh` to launch Yazelix in Zellij.

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
See [docs/troubleshooting.md](./docs/troubleshooting.md) for help with setup issues, version compatibility, and debugging, including important notes for upgrading to v7.

## VS Code and Cursor Integration
Want to use Yazelix tools (Nushell, zoxide, starship, lazygit) in your VS Code or Cursor integrated terminal? See our [VS Code/Cursor integration guide](./docs/vscode_cursor_integration.md) for step-by-step setup instructions that give you the full Yazelix environment in your editor's terminal.

## Keybindings
Keybindings are discoverable in each tool (e.g., `~` in Yazi, `?` in lazygit). See [docs/keybindings.md](./docs/keybindings.md) for full details, custom keybindings, and usage tips.

## Tips
- Add more swap layouts as needed using the KDL files in `layouts`
- Use `lazygit`

## I'm Lost! Too Much Information
Start by learning Zellij on its own, then optionally Yazi, and re-read this README afterwards

## Contributing to Yazelix
See [contributing](./docs/contributing.md)

## Similar Projects
- If you frequently use other terminal editors besides Helix or terminal file managers other than Yazi, check out [zide](https://github.com/josephschmitt/zide)
- If you care about Yazi but don't care much about Zellij or having a sidebar, you can integrate Yazi and Helix with [one line of config](https://github.com/sxyazi/yazi/pull/2461) (experimental, not working for some people as of March 15, 2025)

## Acknowledgments
- The `clip` command is sourced from the [nuscripts](https://github.com/nushell/nuscripts) repository, licensed under the MIT License.
- 95% of the work (and the idea) of the excellent [auto-layout plugin](https://github.com/josephschmitt/auto-layout.yazi) was made by [Joseph Schmitt](https://github.com/josephschmitt). Later I added some fixes for new versions of Yazi and added logging and some checks
