# Yazelix v6.4: Nix

## Overview
Yazelix integrates Yazi, Zellij, and Helix, hence the name, get it?

- Zellij orchestrates everything, with Yazi as a sidebar and Helix as the editor
- To hide the sidebar, make your pane fullscreen! (`Ctrl p + f` or `Alt f`)
- Every keybinding from Zellij that conflicts with Helix is remapped [see here](#keybindings)
- When you hit Enter on a file/folder in the "sidebar":
  - If Helix is already open in the topmost pane of the stack (default position in latest Zellij version), it opens that file/folder in a new buffer in Helix
  - If Helix isn’t open, it launches Helix in a new pane for you
  - It always finds a running Helix instance if it exists and is in the top pane of the stacked group (Zellij naturally pushes the Helix pane there, though it may move when deleting or creating panes)
  - It automatically renames the Zellij tab to the file's underlying Git repo or directory name
- Features include:
  - "Reveal file in sidebar" (press `Alt y` in Helix to reveal the file in Yazi, see [Yazelix Custom Keybindings](#yazelix-custom-keybindings))
  - A Yazi plugin to enhance the status bar in the sidebar pane, making it uncluttered, colorful, and showing file permissions
  - A [Git plugin](https://github.com/yazi-rs/plugins/tree/main/git.yazi) showing file changes in the Yazi sidebar
  - Dynamic column updates in Yazi (parent, current, preview) via the [auto-layout plugin](https://github.com/josephschmitt/auto-layout.yazi), perfect for sidebar use
- This project includes config files for Zellij, Yazi, terminal emulators, Nushell scripts, Lua plugins, and a lot of love
- The boot sequence of the Nix version is the following:
  - You open WezTerm -> WezTerm is configured to run `~/.config/yazelix/shell_scripts/start-yazelix.sh` -> the script navigates to the Yazelix directory and runs `nix develop --impure --command zellij ...` -> the flake reads `yazelix.toml`, installs dependencies, generates initializer scripts, configures the environment, and launches Zellij with Nushell as the default shell

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

## Improvements of v6.4 over v6
- **Warning**: After upgrading to Yazelix v6.4, terminate any running Yazi instances and old terminals to prevent conflicts
- Introduces a Nix-based development environment via `flake.nix`, simplifying dependency installation and ensuring consistent versions for Zellij, Yazi, Helix, Nushell, lazygit, Starship, and other tools (recommended installation method)
- Adds [lazygit](https://github.com/jesseduffield/lazygit), a fast, terminal-based Git TUI for managing Git repositories
- Adds [Starship](https://starship.rs), a customizable, fast prompt for Nushell, enhancing the terminal experience with Git status and contextual info
- Allows you to build Helix from source automatically
- Installs and configures dependencies automatically
- The `clip` command from [nuscripts](https://github.com/nushell/nuscripts) is included, allowing you to copy text to the system clipboard directly from Nushell. Use it like `ls src/*.py | get name | to text | clip`.

## Compatibility
- The Nix-based installation currently supports only WezTerm; the Cargo-based installation supports any terminal emulator, including WezTerm and Ghostty (includes a Ghostty config)
- Editor: Helix (for now)
- See the version compatibility table [here](./docs/table_of_versions.md)

## Instructions to Set It Up
Yazelix v6.4 offers two installation pipelines: **Nix-based (recommended)** for a consistent, reproducible environment (requires WezTerm), and **Cargo-based** for users preferring a straightforward Rust-based setup with any terminal emulator.

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
4. Move the wezterm terminal config to ~/.wezterm.lua:
   ```bash
   cp ~/.config/yazelix/terminal_configs/wezterm_nix/.wezterm.lua ~/.wezterm.lua
   ```
5. Done! Now just open wezterm!

5.1. Optional: If you just want to load the tools without entering zellij + yazi, just enter the Nix development environment:
   ```bash
  nix develop --impure ~/.config/yazelix
   ```


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
   - Yazi Extensions (enabled by default in `yazelix.toml`): `ffmpeg`, `p7zip`, `jq`, `poppler`, `fd`, `ripgrep`, `imagemagick` (extend Yazi’s functionality, e.g., media previews, archives, search)
   - Sets environment variables: `YAZI_CONFIG_HOME` (points to `~/.config/yazelix/yazi`), `ZELLIJ_DEFAULT_LAYOUT` (set to `yazelix`), and `EDITOR` (set to `hx`)
   - Configurable in `yazelix.toml`:
     - `build_helix_from_source` (default: `true`): Set to `false` to use the pre-built Helix from `nixpkgs` instead of building from source. Building from source ensures the latest Helix features (e.g., for `Alt y` to reveal files in Yazi) but takes longer. Using `nixpkgs` is faster but may use an older version; check compatibility in `./docs/table_of_versions.md`.
     - `include_optional_deps` (default: `true`): Set to `false` to exclude optional dependencies like `mise` and `lazygit`.
     - `include_yazi_extensions` (default: `true`): Set to `false` to exclude Yazi extension dependencies like `ffmpeg` and `poppler`.

5. Configure WezTerm (required for Nix-based setup):
   - Copy the provided WezTerm config, which launches Yazelix via `start-yazelix.sh`:
     ```bash
     cp ~/.config/yazelix/terminal_configs/wez/.wezterm.lua ~/.wezterm.lua
     ```
6. (Optional) Make Yazelix’s Yazi config your default (plugin-enhanced, width-adjusted):
   - For Nushell users, add to `~/.config/nushell/env.nu` (edit with `config env`):
     ```nushell
     $env.YAZI_CONFIG_HOME = "~/.config/yazelix/yazi"
     ```

### Option 2: Cargo-Based Installation (UNTESTED, you might prefer using the main branch...)
See the detailed [Cargo-based installation guide](./docs/cargo_installation.md) for instructions on installing dependencies with `cargo` and configuring your terminal emulator.

### Initializer Scripts
Yazelix generates Nushell initializer scripts in `~/.config/yazelix/nushell/initializers/` during the Nix environment setup (`nix develop --impure`):
- `mise_init.nu`: Runs `mise activate nu` (only if `include_optional_deps = true` in `yazelix.toml`).
- `starship_init.nu`: Runs `starship init nu`.
- `zoxide_init.nu`: Runs `zoxide init nushell --cmd z`.
These are sourced in `~/.config/yazelix/nushell/config/config.nu` and **regenerated each time you open WezTerm** to reflect the current tool versions. Do not edit these files manually, as they will be overwritten. For custom configurations, use `~/.config/nushell/config.nu` or tool-specific configs (e.g., `~/.config/starship.toml`).

For Cargo-based setups with Bash/Zsh, manually generate equivalent scripts:
```bash
mise activate bash > ~/.config/yazelix/nushell/initializers/mise_init.bash
starship init bash > ~/.config/yazelix/nushell/initializers/starship_init.bash
zoxide init bash --cmd z > ~/.config/yazelix/nushell/initializers/zoxide_init.bash
source ~/.config/yazelix/nushell/initializers/mise_init.bash
source ~/.config/yazelix/nushell/initializers/starship_init.bash
source ~/.config/yazelix/nushell/initializers/zoxide_init.bash
```

**Notes**:
- The Nix-based approach is recommended for its reproducibility and ease of dependency management but requires WezTerm, which runs `start-yazelix.sh` to launch Zellij with the Yazelix layout
- The `--impure` flag in `nix develop` allows access to the HOME environment variable, necessary for config paths
- The Cargo-based approach supports any terminal emulator, offering more flexibility
- Tweak configs to make them yours; this is a starting point
- For extra configuration, see: [WezTerm Docs](https://wezfurlong.org/wezterm/config/files.html) or [Ghostty Docs](https://ghostty.org/docs/config)
- Run `~/.config/yazelix/start-yazelix.sh` to launch Yazelix in Zellij

That’s it! Open issues or PRs if you’d like 😉

## Why Use This Project?
- Easy to configure and personalize
- I daily-drive Yazelix and will always try to improve and maintain it
- Zero-conflict keybindings (no need to lock Zellij) and a powerful Yazi sidebar
- Cool Yazi plugins included out of the box
- Features like `reveal in Yazi` (from Helix) and opening files from Yazi in a Helix buffer
- Enhanced Git integration with `lazygit` and a customizable Starship prompt
- Nix-based setup ensures consistent, reproducible environments

## Troubleshooting
- If it’s not working properly:
  - For Nix users, ensure you're in the Nix shell (`nix develop --impure`) and using WezTerm
  - For Cargo users, verify all required dependencies are installed and up-to-date
  - Check version compatibility [here](./docs/table_of_versions.md)
  - Enable `config.debug_key_events = true` in `~/.wezterm.lua` for detailed logging

## Keybindings
| New Zellij Keybinding | Previous Keybinding | Helix Action that conflicted before | Zellij Action Remapped     |
|-----------------------|---------------------|-------------------------------------|----------------------------|
| Ctrl e                | Ctrl o              | jump_backward                       | SwitchToMode "Session"     |
| Ctrl y                | Ctrl s              | save_selection                      | SwitchToMode "Scroll"      |
| Alt w                 | Alt i               | shrink_selection                    | MoveTab "Left"             |
| Alt q                 | Alt o               | expand_selection                    | MoveTab "Right"            |
| Alt m                 | Alt n               | select_next_sibling                 | NewPane                    |
| Alt 2                 | Ctrl b              | move_page_up                        | SwitchToMode "Tmux"        |

If you find a conflict, please open an issue

## Discoverability of Keybindings
- **Zellij**: Shows all keybindings visually in the status bar—works out of the box
- **Helix**: Similar to Zellij, keybindings are easy to discover
- **Yazi**: Press `~` to see all keybindings and commands (use `Alt f` to fullscreen the pane for a better view)
- **Nushell**:
  - Run `tutor` in Nushell
  - Read the [Nushell Book](https://www.nushell.sh/book/)
  - Use `help commands | find tables` to search, for example, commands related to tables
- **lazygit**: Press `?` to view keybindings
- **Starship**: Customizable prompt; configure in `~/.config/starship.toml` (see [Starship docs](https://starship.rs/config/))

## Yazelix Custom Keybindings
- **Zellij**: `Alt f` toggles pane fullscreen
- **Helix**: `Alt y` reveals the file from the Helix buffer in Yazi, add this to your Helix config:
  ```toml
  [keys.normal]
  A-y = ":sh nu ~/.config/yazelix/nushell/reveal_in_yazi.nu \"%{buffer_name}\""
  ```
  - **Limitation**: Only works for Helix instances opened from Yazi
  - **Requirement**: Build Helix from source until the next release includes command expansions

## Keybinding Tips
- **Zellij**: `Ctrl p` then `r` for a split to the right; `Ctrl p` then `d` for a downward split
- **Yazi**: 
  - `Z`: Use Zoxide (fuzzy find known paths)
  - `z`: Use fzf (fuzzy find unknown paths)
  - `SPACE`: Select files
  - `y`: Yank (copy); `Y`: Unyank (cancel copy)
  - `x`: Cut; `X`: Uncut (cancel cut)
  - `a`: Add a file (`filename.ext`) or folder (`foldername/`)
- **Nushell**:
  - `Ctrl r`: interactive history search
  - `Ctrl o`: open a temporary buffer
- **lazygit**:
  - `c`: Commit changes
  - `p`: Push commits
  - `P`: Pull changes
  - `s`: Stage/unstage files

## Tips
- Add more swap layouts as needed using the KDL files in `layouts`
- I recommend WezTerm for Nix-based setups; Ghostty or WezTerm for Cargo-based setups
- Use `lazygit` for fast Git operations in a Zellij pane
- Customize the Starship prompt in `~/.config/starship.toml` for a personalized experience
- For stability, consider pinning `nixpkgs` to a specific commit in `flake.nix` (e.g., `nixpkgs.url = "github:nixos/nixpkgs/<commit-hash>"`)

## I’m Lost! Too Much Information
Start by learning Zellij on its own, then optionally Yazi, and re-read this README afterwards

## Thanks
- To Yazi, Zellij, Helix, Nushell, lazygit, zoxide, and Starship contributors/maintainers for their amazing projects and sometimes even guidance
- To Yazi’s author for contributing Lua code to make the sidebar status bar look awesome
- Nix rocks
- To [Joseph Schmitt](https://github.com/josephschmitt) for his excellent [auto-layout plugin](https://github.com/josephschmitt/auto-layout.yazi)

## Contributing to Yazelix
See [contributing](./docs/contributing.md)

## Similar Projects
- If you frequently use other terminal editors besides Helix or terminal file managers other than Yazi, check out [zide](https://github.com/josephschmitt/zide)
- If you care about Yazi but don’t care much about Zellij or having a sidebar, you can integrate Yazi and Helix with [one line of config](https://github.com/sxyazi/yazi/pull/2461) (experimental, not working for some people as of March 15, 2025)

## Acknowledgments
- The `clip` command is sourced from the [nuscripts](https://github.com/nushell/nuscripts) repository, licensed under the MIT License.
- 95% of the work (and the idea) of the excellent [auto-layout plugin](https://github.com/josephschmitt/auto-layout.yazi) was made by [Joseph Schmitt](https://github.com/josephschmitt). Later I added some fixes for new versions of Yazi and added logging and some checks
