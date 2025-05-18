# Yazelix v7: The power of Yazi plugins and Nushell scripting!

## Overview
Yazelix integrates Yazi, Zellij, and Helix, hence the name, get it?

- Zellij orchestrates everything, with Yazi as a sidebar and Helix as the editor
- To hide the sidebar, just make your pane fullscreen! (`Ctrl p + f` or `Alt f`)
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

# Vision
- Yazelix is always on the edge of project versions (do you like living on the edge, you know, dangerously?)
- Yazelix is always evolving, it's a living being
- Yazelix is easy to use and crazy at the same time (what really is this project?)
- Yazelix enables you to say `I use yazelix btw` (careful saying that, you might accidentaly scare an innocent arch user)
- Yazelix eats glass and laughs, says it wasnt crunchy enough
- Boy, do we nix
- Integration, integration, integration

## Preview
![yazelix_v6_demo](assets/reveal_fullscreen_stacked.gif)
v6 demo

## Improvements of v7 over v6
- **Warning**: After upgrading to Yazelix v7, terminate any running Yazi instances and old terminals to prevent conflicts
- Introduces a Nix-based development environment via `flake.nix`, simplifying dependency installation and ensuring consistent versions for Zellij, Yazi, Helix, Nushell, lazygit, Starship, and other tools (recommended installation method)
- Adds [lazygit](https://github.com/jesseduffield/lazygit), a fast, terminal-based Git TUI for managing Git repositories
- Adds [Starship](https://starship.rs), a customizable, fast prompt for Nushell, enhancing the terminal experience with Git status and contextual info

## Compatibility
- The Nix-based installation currently supports only WezTerm; the Cargo-based installation supports any terminal emulator, including WezTerm and Ghostty (includes a Ghostty config; I use Ghostty as my daily driver)
- Editor: Helix (for now)
- See the version compatibility table [here](./docs/table_of_versions.md)

## Instructions to Set It Up
Yazelix v7 offers two installation pipelines: **Nix-based (recommended)** for a consistent, reproducible environment (requires WezTerm), and **Cargo-based** for users preferring a straightforward Rust-based setup with any terminal emulator.

### Option 1: Nix-Based Installation (Recommended)
1. Install Nix:
   - On Linux/macOS, run:
     ```bash
     curl --proto '=https' --tlsv1.2 -sSf -L https://install.determinate.systems/nix | sh -s -- install
     ```
   - Follow the prompts to complete installation
   - Verify:
     ```bash
     nix --version
     ```
2. Enable Nix flakes:
   - Create or edit `~/.config/nix/nix.conf`:
     ```bash
     mkdir -p ~/.config/nix
     echo "experimental-features = nix-command flakes" >> ~/.config/nix/nix.conf
     ```
3. Clone this repo into your `~/.config` directory:
   ```bash
   git clone https://github.com/luccahuguet/yazelix ~/.config/yazelix
   ```
4. Enter the Nix development environment:
   ```bash
   cd ~/.config/yazelix
   nix develop
   ```
   This installs and configures:
   - Required: [Yazi-fm and Yazi-cli](https://github.com/sxyazi/yazi), [Zellij](https://github.com/zellij-org/zellij), [Helix](https://helix-editor.com), [Nushell](https://www.nushell.sh/book/installation.html), [fzf](https://github.com/junegunn/fzf) (for fuzzy finding in Yazi), [cargo-update](https://github.com/nabijaczleweli/cargo-update) (for updating Rust crates), [cargo-binstall](https://github.com/cargo-bins/cargo-binstall) (for faster Rust tool installation)
   - Optional: [Zoxide](https://github.com/ajeetdsouza/zoxide) (smart directory navigation), [lazygit](https://github.com/jesseduffield/lazygit) (Git TUI), [Starship](https://starship.rs) (customizable prompt), `ffmpeg`, `p7zip`, `jq`, `poppler`, `fd`, `ripgrep`, `imagemagick` (extend Yazi's functionality, e.g., media previews, search, archives)
5. Configure WezTerm (required for Nix-based setup):
   ```bash
   cp ~/.config/yazelix/terminal_configs/wez/.wezterm.lua ~/.wezterm.lua
   ```
6. (Optional) Make Yazelix’s Yazi config your default (plugin-enhanced, width-adjusted):
   - For Nushell users, add to `~/.config/nushell/env.nu` (edit with `config env`):
     ```nushell
     $env.YAZI_CONFIG_HOME = "~/.config/yazelix/yazi"
     ```

### Option 2: Cargo-Based Installation
See the detailed [Cargo-based installation guide](./docs/cargo_installation.md) for instructions on installing dependencies with `cargo` and configuring your terminal emulator.

**Notes**:
- The Nix-based approach is recommended for its reproducibility and ease of dependency management but currently requires WezTerm
- The Cargo-based approach supports any terminal emulator, offering more flexibility
- Tweak configs to make them yours, this is a starting point
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
  - For Nix users, ensure you're in the Nix shell (`nix develop`) and using WezTerm
  - For Cargo users, verify all required dependencies are installed and up-to-date
  - Check version compatibility [here](./docs/table_of_versions.md)

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
  - Run `tutor` on a Nushell
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

## I’m Lost! Too Much Information
Start by learning Zellij on its own, then optionally Yazi, and re-read this README afterwards

## Thanks
- To Yazi, Zellij, Helix, Nushell, lazygit, and Starship contributors/maintainers for their amazing projects and guidance
- To Yazi’s author for contributing Lua code to make the sidebar status bar look awesome
- To [Joseph Schmitt](https://github.com/josephschmitt) for his excellent [auto-layout plugin](https://github.com/josephschmitt/auto-layout.yazi)

## Contributing to Yazelix
See [contributing](./contributing.md)

## Similar Projects
- If you frequently use other terminal editors besides Helix or terminal file managers other than Yazi, check out [zide](https://github.com/josephschmitt/zide)
- If you care about Yazi but don’t care much about Zellij or having a sidebar, you can integrate Yazi and Helix with [one line of config](https://github.com/sxyazi/yazi/pull/2461) (experimental, not working for some people as of March 15, 2025)
