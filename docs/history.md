# Yazelix Version History & Changelog

> **Note:** Older entries reference the legacy `yazelix.nix` configuration. Yazelix now uses `yazelix.toml`, but historical notes retain the original terminology for accuracy.

## The start

Kintsugi is the japanese art of repairing broken pottery by mending the areas of breakage with a paste dusted or mixed with powdered gold, silver, or platinum. It represents the embracing of the flawed or imperfect, highlighting cracks and repairs as events in the life of an object, rather than allowing its service to end at the time of its damage or breakage.

The origin of Yazelix certainly relates to that, in the sense that I deeply missed having a sidebar/filetree in helix. Yazelix started as a response to a [Reddit interaction](https://www.reddit.com/r/HelixEditor/comments/1bgsauh/comment/kyfrvh2/?utm_source=share&utm_medium=web3x&utm_name=web3xcss&utm_term=1&utm_content=share_button), where I saw the opportunity of doing something similar to the OP but using zellij as a multiplexer instead of kitty, since I was already using zellij.

Zellij is superb, but using Helix with it created another set of problems, namely keybinding conflicts. So I fixed them also, using yazelix, instead of giving up.

And then one day I got tired of having to install all my tools whenever I distrohopped/changed OS, so I nixified Yazelix, reducing the number of dependencies to near zero.

And somewhere along the way I got tired of configuring starship, mise, zoxide too, same as above, so yazelix auto-configures this for me.

And I wondered if I could add support for more shells, and terminal emulators, so I did that. Someone wanted to use fish. Now they have it.

And I wondered if I could have a welcome screen with some art in it, and some info from the yazelix build process, etc, so I built it.

And then I needed a better tab bar, and more customization, so I replaced the tab bar with zjstatus, a great zellij plugin. 

And on and on.

Now using yazelix feels extremely good to me, and I get the feeling the project is getting some traction, with a constant rhythm of github stars growth, and getting known by more people. And if others find joy in using yazelix, that's enough for me. 

Desire is pain, and yazelix came from desire, from imagining something that did not exist the way I wanted. A nagging in my mind, asking "what if". What if I could do this? And that? I have this vision, is it feasible? Can I do this in a more elegant way? Or rather, let me try it and see what happens.

Building through yazelix feels like painting a work of art to me, although I have bad actual artistic skills. The pleasure of bringing idea to reality, and crafting something that feels smart. Few things come close to it.

Much like a broken cup, that got mended with gold into something superior, that's what yazelix is to me, and hopefully not only to me.

There's much to be done yet. Infinite possibilities. But some rather closer than far. I'd love to have you join me on this ride.


## Major Version Descriptions

- **v11**: Blazingly fast launches, instant config reloads, Home Manager parity, and zero friction
- **v10**: Launch interface consolidation, IDE integration, project-oriented workflow, and enhanced UX!
- **v9**: zjstatus, yzx env, yzx doctor, no_sidebar mode, packs, new logo, desktop entry, and more
- **v8**: Lots of polish, support for any editor, home-manager config, better zellij tab navigation, persistent sessions and more!
- **v7**: Nix installs and configures everything for you! ([announcement](#v7-nix-installs-and-configures-everything-for-you))
- **v6**: Reveal, Integrate, Automate: Smarter sidebar, Git status, and seamless file opening.
- **v5**: The POWER of yazi PLUGINS!
- **v4**: A true sidebar opens files in a helix buffer! ([announcement](https://x.com/luccahuguet/status/1842689462968766791))
- **v3**: Helix with a File Tree! Now with helix-friendly keybindings, and monorepo! ([announcement](https://www.reddit.com/r/HelixEditor/comments/1doefzt/yazelix_v3_helix_with_a_file_tree_now_with/))
- **v2**: Yazi-Helix File Tree v2, now with a Closeable Sidebar! (the name 'Yazelix' did not exist yet; [announcement](https://www.reddit.com/r/HelixEditor/comments/1d6nkxs/yazihelix_file_tree_v2_now_with_a_closeable/))
- **v1**: My first Zellij/Yazi/Helix/Nushell setup, inspired by a Reddit interaction, with no integration and a lot of hacks ([announcement](https://www.reddit.com/r/HelixEditor/comments/1d59br3/file_tree_setup_using_yazi_zellij_helix_and/))

## v11: Devenv launch workflow, TOML-first config, way faster cold starts, Neovim parity, a built-in performance toolkit, and richer pack presets

### Changes from v10 to v11

- **Devenv-Based Launch Workflow** – Yazelix now runs through `devenv shell --impure`; devenv’s SQLite cache automatically detects config changes so cold launches from a desktop entry or `yzx launch` drop from ~4s to ~0.5s, and you only pay a longer rebuild when you actually edit `yazelix.toml`. Yazelix is now Blazingly fast! 
- **TOML Configuration Format** – `yazelix.toml` is the single source of truth (auto-created on first launch) with clear legacy warnings when an old `yazelix.nix` is detected.
- **First-Class Neovim Support** – Neovim retains feature parity with Helix (reveal in sidebar, same-instance opening, pane detection).
- **Performance Benchmarking** – `yzx bench` measures terminal launch performance with statistical analysis.
- **Launch Profiling** – `yzx profile` pinpoints environment setup bottlenecks and profiles cold vs warm startup paths.
- **Enhanced UI Controls** – Configurable Zellij options: `disable_zellij_tips` (default: true), `zellij_rounded_corners` (default: true).
- **Streamlined Startup** – Welcome screen disabled by default for faster launches (info still logged).
- **Sweep Testing Framework** – Matrix testing for all shell/terminal/feature combinations (`yzx sweep shells|terminals|all`).
- **Terminal Detection** – Proper terminal identification via `YAZELIX_TERMINAL` environment variable.
- **Conditional Shell Hooks** – Shell hooks load Yazelix tooling only inside managed shells, preventing surprises in regular terminals.
- **Yazi Directory Sync** – Opening files from Yazi moves the sidebar to the file’s parent directory so the view stays in sync with editor context.
- **Simplified Clipboard** – Replaced the custom clipboard module with Nushell’s standard library implementation and added the `clp` helper command. `clp` just calls `clip copy`
- **Comprehensive Pack System** – 10 curated technology packs organized into language_packs and tool_packs:
  - **Language Packs (7)**: Python, TypeScript, Rust, Go, Kotlin, Gleam, Nix – complete toolchains with LSP, formatters, linters, and dev tools.
  - **Tool Packs (3)**: Git (onefetch, gh, delta, gitleaks, jj, prek), Config (taplo, mpls), File Management (ouch, erdtree, serpl).
  - **Enhanced Packs**: Expanded Rust pack (6 tools), improved TypeScript pack with oxlint and typescript-language-server, Python pack with ipython.

## v10: Launch interface consolidation, IDE integration, project-oriented workflow, and enhanced UX!

### Changes from v9 to v10
- zjstatus polish: swap layout widget (colored, hide‑if‑empty), colored shell/editor, smarter overlength hiding with tabs prioritized, improved inactive/active tab variants (fullscreen/sync/floating), and cleaner status without tab separators.
- CLI: new `yzx why` command for a concise "elevator pitch".
- **Launch interface consolidation**: Simplified the previous dual-command flow into a single `yzx launch` command with intuitive flags:
  - `yzx launch` (default): new terminal in current directory (project-oriented)
  - `yzx launch --here`: start in current terminal
  - `yzx launch --path DIR`: launch in specific directory
  - `yzx launch --home`: launch in home directory
  - All flags are composable for maximum flexibility while maintaining a clean interface
- **IDE integration enhancements**: Added quiet mode to `yzx env` for seamless IDE terminal integration:
  - Suppresses verbose output when `YAZELIX_ENV_ONLY=true`
  - Clean, minimal output suitable for IDE/editor terminals (Zed, DataGrip, etc.)
  - Eliminates ASCII art and chatty setup messages in quiet mode
- **Enhanced yzx env functionality**: Improved shell handling and user experience:
  - Now launches configured shell by default (more intuitive than keeping current shell)
  - Added `--no-shell` flag for users who want to keep their current shell
  - Aligned bash and nushell implementations for consistent cross-shell behavior
- **Project-oriented workspace behavior**: All launch commands now start in current working directory by default, creating a more intuitive project-focused workflow that eliminates the need to navigate from home directory
- Docs: POSIX/XDG paths doc + README link; SSH/Remote section in README.
- Desktop integration: Fixed window branding to show "Yazelix" in taskbars/docks, with proper StartupWMClass for desktop entry association.
- Terminal bundling: Added bundled terminal emulators (Ghostty, WezTerm, Kitty, Alacritty) with automatic nixGL GPU acceleration. Only downloads your preferred terminal, with Ghostty always included as fallback.
- Ghostty cursor trails: New presets (blaze, snow, cosmic, ocean, forest, sunset, neon, party, eclipse, dusk, orchid, reef, inferno) plus a `random` option that rotates between colorful presets (excluding `none` and `party`). Configure via `cursor_trail` in `yazelix.nix`. Kitty supports `snow`; WezTerm/Alacritty do not support trails.
- Shell initializers: Aggregate per‑shell initializer (`yazelix_init.*`) to source only what exists. Essential tools (starship, zoxide) are robust; optional tools (mise, carapace, atuin) are omitted when disabled or missing. No more parse‑time source errors.
- Terminal configs: New `terminal_config_mode` with three modes:
  - `yazelix` (default): use Yazelix‑managed configs under `~/.local/share/yazelix/configs` (user configs untouched)
  - `auto`: prefer user configs if present, else Yazelix configs
  - `user`: always use user configs
  Wrappers and `yzx launch` honor this consistently (including via desktop entry) and pass the mode via env.
- Config location: All generated terminal configs moved to XDG state dir `~/.local/share/yazelix/configs/terminal_emulators`.
- Atuin toggle: `enable_atuin` added (default: false). Nushell uses its native history by default; Atuin can be enabled explicitly.

## v9: zjstatus, `yzx env`, `yzx doctor`, no_sidebar mode, pack-based system, new logo, desktop entry and more!

### Changes from v8 to v9

- Flexible layout system: Sidebar mode remains the default, with optional no-sidebar mode for different workflows:
  - Sidebar mode (default): IDE-like workflow with persistent Yazi file navigation (recommended!)
  - No-sidebar mode: Available via `enable_sidebar = false`, no Yazi sidebar, saves some screen space. Useful if you use other editors that have a built-in file tree
- Pack-based configuration system: Simplified package management with technology stacks:
  - Enable entire tech stacks with `packs = ["python", "js_ts", "config"]` instead of commenting individual packages
  - 5 curated packs: `python` (ruff, uv, ty), `js_ts` (biome, bun), `rust` (cargo tools), `config` (formatters), `file-management` (utilities)
  - Hybrid approach: use packs for bulk selection, `user_packages` for individual tools
- Enhanced Zellij layouts: Added comprehensive layout system with both sidebar and no-sidebar variants:
  - Sidebar layouts (default): `basic`, `stacked`, `three_column`, `sidebar_closed` - persistent file navigation
  - No-sidebar layouts: `basic`, `stacked`, `two_column` - clean, full-screen workflows
- New sidebar_closed swap layout: Dynamic sidebar toggling: use the sidebar_closed swap layout, reach it with `Alt+[` / `Alt+]` for space optimization when needed
- New zjstatus plugin integration: Added custom status bar plugin with shell and editor information:
  - Shows your configured shell and editor: `[shell: nu] [editor: hx] YAZELIX` with proper spacing and color coding
  - Replaces default Zellij status bar with a more informative Yazelix-specific display
- Dynamic Three-Layer Zellij Configuration: Completely rewritten configuration system with modular, maintainable approach:
  - Layer 1: Zellij defaults (fetched dynamically via `zellij setup --dump-config`)
  - Layer 2: Yazelix overrides (`yazelix_overrides.kdl`) - Yazelix-specific settings
  - Layer 3: User configuration (`user_config.kdl`) - Your personal customizations with highest priority
  - Smart caching: Only regenerates when source files change for faster startup
  - XDG-compliant: Generated config saved to `~/.local/share/yazelix/configs/zellij/`
  - Comprehensive template: `user_config.kdl` includes documented examples for themes, keybindings, plugins, and advanced options
  - Improved maintainability: Removed old static `config.kdl` system that required manual updates
  - Better user experience: Users can now easily customize Zellij by editing a single, well-documented file
  - Reference documentation: See `configs/zellij/example_generated_config.kdl` for the complete default Zellij configuration
- Alt+p directory opening: New Yazi keybinding for instant workspace expansion:
  - Quick pane creation: `Alt+p` in Yazi opens selected directory in new Zellij pane
  - Smart file handling: For files, opens parent directory; for directories, opens the directory itself
  - Proper shell environment: New panes start with correctly configured Nushell in target directory
- Enhanced startup robustness: Improved Nix detection with automatic environment setup, terminal integration, and clear diagnostics
- Health Check System (`yzx doctor`): Diagnostic tool that detects and fixes common issues (Helix runtime conflicts, env vars, config validation, etc.). Supports `--verbose` and `--fix`.
- Atuin shell history integration: Added atuin to the automatic initializer system
- Project logo and desktop integration: Logo with icons and automatic desktop entry setup
- CLI-only environment mode (`yzx env`): Load Yazelix tools without the UI interface
  - Quick access: `yzx env` loads all tools (helix, yazi, lazygit, etc.) in your configured shell
  - No interface overhead: Skips welcome screen and Zellij launch
  - Clean messaging: Shows environment status and available commands
  - Perfect for scripts: Ideal for automation, VS Code integration, or when you just need the tools
- New sponsor button

## v8: Lots of polish, support for any editor, home-manager config, better zellij tab navigation, persistent sessions and more!

### Changes from v7 to v8

- **Home Manager Integration**: Optional declarative configuration management via Home Manager module
- **Modular Editor Support**: Complete rewrite of file opening logic to support any editor while preserving full Helix integration. Now you can use Vim, Nano, Emacs, or any editor via the `editor_command` setting in `yazelix.nix` - Helix users get all advanced features (open in same buffer, reveal in sidebar, etc), while other editors get basic Zellij integration (new panes, tab renaming)
- **Big File/Folder Project-Wide Refactoring**: Complete reorganization of the codebase structure for better maintainability and organization
- **Yazelix Config Validation**: Added validation system to warn users of invalid configuration options in `yazelix.nix`
- **Configurable Editor Environment Variables**: New config options to set EDITOR environment variable when empty, override existing EDITOR, and add custom editor commands
- **Configurable Welcome ASCII Art**: You can now choose between animated or static ASCII art in the welcome screen using the new `ascii_art_mode` option in your `yazelix.nix` config. Set `ascii_art_mode = "animated"` (default) or `ascii_art_mode = "static"` for a non-animated welcome.
- **Shell-agnostic `yzx` command**: Introduces a unified `yzx` command that works across all supported shells (bash, fish, zsh, nushell) with full subcommand support. No more shell-specific limitations - use `yzx help`, `yzx get_config`, `yzx versions`, etc. from any shell!
- **Seamless Yazelix restart**: `yzx restart` launches a new Yazelix instance before killing the old one, providing a smooth and reliable restart experience.
- **Yazelix Collection page**: Yazelix now includes a dedicated collection page (`docs/yazelix_collection.md`) listing all integrated tools and inspirations.
- **Added macchina to welcome screen**: Added a system info summary using macchina (neofetch alternative) to the welcome screen. It can be disabled in the config.
- **Dynamic Config Validation**: Yazelix now uses a dynamic config validator that checks your config against yazelix_default.nix every time Yazelix starts. It warns about unknown fields, missing fields, and invalid values for key options (like default_shell, helix_mode, preferred_terminal, ascii_art_mode). No more silent config errors!
- **Improved Helix Pane Detection**: Yazelix now checks the topmost pane and the next two below for a Zellij pane named `editor` (the Helix pane) when opening files from Yazi, reusing it if found, or opening a new one if not. See [Helix Pane Detection Logic](../README.md#helix-pane-detection-logic) for details.
- **Ergonomic Tab Navigation**: Added browser-like tab navigation in Zellij:
  - `Alt+number` to jump directly to tabs 1-9
  - `Alt+w/q` to walk (focus) next/previous tab
  - `Alt+Shift+H/L` to move tabs left/right
  - Cleaned up legacy/conflicting keybindings for a more user-friendly experience
- **Persistent Sessions Configuration**: Added support for persistent Zellij sessions with flexible configuration parsing. Configure `persistent_sessions = true` and `session_name = "your_session"` in `yazelix.nix` to reuse the same session across restarts
- **Full version history and project evolution is now documented in detail, right here!**

## v7: Nix installs and configures everything for you!

### Changes from v6 to v7

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
- Adds `launch_yazelix.nu` script to launch your preferred terminal with the Yazelix-specific config. The `yzx` alias is automatically available in your shell once the Yazelix shell configurations are sourced.
- The `clip` command from [nuscripts](https://github.com/nushell/nuscripts) is included, allowing you to copy text to the system clipboard directly from Nushell. Use it like `ls src/*.py | get name | to text | clip` or `open data.csv | clip`, etc
- Introduces dynamic Zellij configuration generation on demand using `nushell/scripts/setup/generate_zellij_config.nu`, which combines Zellij's default settings with Yazelix-specific overrides from `configs/zellij/yazelix_overrides.kdl`, making it easy to stay up-to-date with Zellij defaults while preserving custom settings
- Allows for declaration user-defined git-ignored nix packages directly in yazelix.nix
- Improves the "reveal file in sidebar" feature by using Yazi's `reveal` command to automatically highlight and select the specific file, eliminating manual searching in directories with many files
- Introduces dynamic version table generation using `nu nushell/scripts/utils/version_info.nu`

## v6: Reveal, Integrate, Automate: Smarter sidebar, Git status, and seamless file opening

### Changes from v5 to v6

- **Git Plugin for Yazi**: Added a plugin to the Yazi sidebar that shows file changes, improving Git integration.
- **Reveal-in-Yazi Command**: Introduced a command (Alt-y in Helix) to reveal the current file in Yazi, implemented via Nushell and Yazi's emit-to command.
  - *Limitation*: Only works for Helix instances opened from Yazi.
- **Improved File Opening Logic**: When opening a file from Yazi, Yazelix now always finds a running Helix instance if it exists and is in the correct Zellij pane.
- **Enhanced Tab Naming**: When opening a file from Yazi, the Zellij tab is automatically renamed to the underlying Git repo or directory name.
- **Detailed Logging**: Added detailed logging for Nushell scripts and improved logging instructions for Zellij/Yazi.
- **Robustness and Polish**: Codebase is more robust and features are more polished, especially the "open from Yazi" workflow.
- **Config Recommendation**: Recommended making Yazelix’s Yazi config the default, with environment variable setup instructions for Nushell users. 

## v5: The POWER of yazi PLUGINS!

### Changes from v4 to v5

- **Keybinding Robustness:** Fixed keybindings not working after Zellij session resurrection (#40).
- **Dynamic Tab Naming:** When opening a file from Yazi, the Zellij tab is renamed to the directory of the file you opened (#42).
- **Tab Movement:** Re-added the option to move tabs left/right, but only in tab mode.
- **Improved Pane Stacking:** Focuses on stacking the single pane by default for a more intuitive layout (#10).
- **Better Helix Detection:** Improved logic for detecting whether Helix is running, making file opening more reliable.
- **Config Updates:** Updated Zellij config to v0.41.0, including new plugin manager and configuration plugin support.
- **Yazi Plugin Integration:** Yazi's author contributed Lua code to make the status bar look awesome in the sidebar.
- **General Robustness:** Multiple README and config updates for clarity, troubleshooting, and improved user experience.

## v4: A true sidebar opens files in a helix buffer!

### Changes from v3 to v4

- **Rounded Corners**: The UI now features rounded corners for a more modern look.
- **Sidebar Integration**: When you hit enter on a file or folder in Yazi, if Helix is open in a pane next to Yazi, it will open in a Helix buffer.
    - All it took was some shell scripting magic...
    - It will also change your working directory, so when you press `SPACE f` you open the picker in the correct folder.
- **Improved New-tab Layout**: New panes are now just Yazi in a 100% width pane, working like a picker.
    - You just open a file or folder from Yazi and it goes to its proper place as a sidebar to the right.
- **Fullscreen Panes**: Added a dedicated keybinding (`alt f`) to make panes fullscreen.
- **Repo Renamed**: The repo was previously called `zellij` for easy cloning, but now it's properly named `yazelix`.
    - The project's name is Yazelix, not Zellij. The repo name now matches the project.
    - See the updated setup instructions in the documentation.
- **Nushell Dependency**: Nushell is now a dependency (technically not an improvement for everyone, but it is for me!).
- **open_file Script Rewritten in Nushell**:
    - Now works with files with spaces in the filename.
    - More sensitive to detecting Helix on the next pane (previously, it would sometimes not detect Helix and open a new instance instead of a new buffer).
    - Changes directory into the folder of the file being opened, or into the folder itself if you clicked on a folder.
    - I much prefer writing Nushell over Bash for this logic.

## v3: Helix with a File Tree! Now with helix-friendly keybindings, and monorepo!

### Changes from v2 to v3

- **Monorepo**: Before, the yazi config files were in a separate repo, now it's all integrated here! Monorepo ftw. (Thanks to Zykino from Zellij's discord for that tip!)
- **Yazi Status-bar**: Yazi's maintainer (what an honor!) added an init.lua file that makes the status-bar in yazi look really good in the small width it has.
- **Project Naming**: The project finally got a name: Yazelix. It simply had no name before and that was a mistake.
- **Keybinding Remaps**: Remapped 6 keybindings from Zellij to avoid conflicts with Helix.
    - Use `alt m` for new panes and the rest of the remaps are in Zellij's status-bar.
    - This is configured in the `layouts/yazelix.kdl` file, if you want to change something.

## v2: Yazi-Helix File Tree v2, now with a Closeable Sidebar! (the name 'Yazelix' did not exist yet)

### Changes from v1 to v2

- **Sidebar Control**: Now you can open and close the sidebar.
- **Simplified Dependencies**: No more nushell dependency. Nushell is a beautiful table-centric cross-platform shell written in Rust, but the way I used it was an ugly hack.
- **Simpler Layout Files**: The KDL files are more streamlined.
- **Removes zjstatus Plugin**: The plugin had to be downloaded and configured, while adding nothing game-changing, and I had no ideia how to bundle it in yazelix
- **Status-bar is Back, baby!**: Life without it isn't easy. The status-bar (help bar) makes the setup much more user-friendly. 

## v1: File Tree Setup using Yazi, Zellij, Helix, and Nushell ([announcement](https://www.reddit.com/r/HelixEditor/comments/1d59br3/file_tree_setup_using_yazi_zellij_helix_and/))

Yazelix started as a response to a [Reddit interaction](https://www.reddit.com/r/HelixEditor/comments/1bgsauh/comment/kyfrvh2/?utm_source=share&utm_medium=web3x&utm_name=web3xcss&utm_term=1&utm_content=share_button), where I shared my Zellij-based setup after being asked for details about integrating Yazi, Helix, Zellij, and Nushell.

My original setup was a Zellij layout using Yazi, Helix, and Nushell:
- **Yazi** ran in a small (20% width) pane in every new Zellij tab, providing file explorer functionality.
- **Helix**: Every file I selected in Yazi opened in a new pane within the same Zellij tab.
- **Layout**: I predefined it with two vertical panes beside Yazi. If more than three panes were opened, the leftmost pane stacked to save space.
- **Nushell**: I used Nushell to call Helix, so it could load my environment variables (a 'gambiara' workaround).
- **zjstatus**: I used zjstatus as a better tab-bar plugin, which made the Zellij layout file long but worth it for improved tab management.
- **Config files**: [Yazi Config Files](https://github.com/luccahuguet/yazi-files), [Zellij Config Files](https://github.com/luccahuguet/zellij-files)
- **Inspiration**: I was inspired by a post using Yazi and Kitty, but my version was simpler to implement and used Zellij.

This version had no integration, required Nushell for Helix launching, and was a bit hacky, but it was a practical and shareable starting point for what would become Yazelix.
