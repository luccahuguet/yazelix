# Yazelix Collection

Yazelix is built on the shoulders of giants. Here are the projects, tools, and plugins that Yazelix integrates with or is inspired by, organized to match the Yazelix configuration structure. Each entry links to the project's homepage or repository and includes a description of its role in Yazelix.

## Integration Levels

**Deep Integration** (🚀 deep-integration): Essential tools like Yazi, Zellij, and Helix have custom configurations, keybindings, and scripts that make them work seamlessly together.

**Pre-configured** (🔧 auto-configured): Tools with custom Yazelix configurations, shell initializers, or special setup.

**Fixed Runtime Helpers**: Useful tools shipped in the trimmed v15 runtime. These are part of the default runtime/tooling surface, not a user-managed pack graph.

---

## Essential Tools
- [Yazi](https://github.com/sxyazi/yazi) — A blazing-fast, modern terminal file manager with Vim-like keybindings, preview support, and extensibility. Yazi is the sidebar and file navigation backbone of Yazelix. 🚀 deep-integration
- [Zellij](https://github.com/zellij-org/zellij) — A powerful terminal multiplexer that manages panes, layouts, and tabs. Zellij orchestrates the Yazelix workspace, allowing seamless integration between file manager, editor, and shell. 🚀 deep-integration
- [zjstatus](https://github.com/dj95/zjstatus) — A configurable status bar plugin for Zellij. Yazelix uses zjstatus to display shell and editor information with custom formatting: `[shell: nu] [editor: hx] YAZELIX`. 🚀 deep-integration
- [Helix](https://helix-editor.com) — A modal text editor inspired by Kakoune and Neovim, featuring fast performance, tree-sitter syntax highlighting, and LSP support. Helix is the default editor for Yazelix, enabling advanced workflows and sidebar integration. 🚀 deep-integration
- [Nushell](https://www.nushell.sh) — A modern shell that treats data as structured tables, making scripting and configuration more robust. Nushell is the default shell for Yazelix and powers its configuration and scripting. (default shell)
- [fzf](https://github.com/junegunn/fzf) — A general-purpose command-line fuzzy finder. Used in Yazelix for quick file and directory navigation. Press `z` in Yazi or `fzf` from terminal.
- [zoxide](https://github.com/ajeetdsouza/zoxide) — A smarter cd command, tracking your most-used directories for instant navigation. Use `z` from the terminal, `Z` inside Yazi for the native jump flow, or `Alt+z` inside Yazi for Yazelix's direct-open jump into the managed editor/workspace. 🔧 auto-configured
- [starship](https://starship.rs) — A minimal, blazing-fast, and customizable prompt for any shell. Provides Yazelix with a beautiful, informative, and context-aware shell prompt. 🔧 auto-configured
- [bash](https://www.gnu.org/software/bash/) — The GNU Bourne Again Shell, included for compatibility and as a fallback shell option.
- [macchina](https://github.com/Macchina-CLI/macchina) — A fast, customizable system information fetch tool. Used to display system info on the Yazelix welcome screen.
- [libnotify](https://github.com/GNOME/libnotify) — Provides desktop notifications from the command line. Used for visual feedback in some Yazelix scripts.

## Extra Shells
- [Fish](https://fishshell.com/) — The Friendly Interactive Shell. Fish offers user-friendly features, autosuggestions, and syntax highlighting. Yazelix can install and integrate with Fish if selected in the configuration.
- [Zsh](https://www.zsh.org/) — The Z Shell. Zsh is a powerful, highly customizable shell with advanced scripting capabilities. Yazelix can install and integrate with Zsh if selected in the configuration.

## Runtime Helper Tools
- [lazygit](https://github.com/jesseduffield/lazygit) — A simple terminal UI for git commands, making version control fast and intuitive. Yazelix includes lazygit for easy git management.
- [carapace](https://github.com/rsteube/carapace-bin) — A cross-shell command-line completion engine. Improves tab completion in supported shells. 🔧 auto-configured
- [mise](https://mise.jdx.dev/) — Runtime/version manager integrations used by the generated shell initializers. 🔧 auto-configured
- [jq](https://github.com/jqlang/jq) — JSON processing used by bundled helper flows and Yazi plugins.
- [fd](https://github.com/sharkdp/fd) — Fast file search used by Yazi and helper scripts.
- [ripgrep](https://github.com/BurntSushi/ripgrep) — Fast text search used by Yazi and one-shot runtime tooling.
- [p7zip](https://github.com/p7zip-project/p7zip) — Archive support for Yazi preview/extract flows.
- [poppler](https://poppler.freedesktop.org/) — PDF preview support in Yazi.

## Terminal Emulators
- [WezTerm](https://wezfurlong.org/wezterm/) — A GPU-accelerated terminal emulator and multiplexer written in Rust. Yazelix supports WezTerm for its advanced features, performance, and modern design.
- [Ghostty](https://ghostty.org/) — A fast, modern terminal emulator written in Zig. Yazelix supports Ghostty as an equally excellent choice, offering speed and a modern feature set.
- [ghostty-cursor-shaders](https://github.com/sahaj-b/ghostty-cursor-shaders) — Vendored and adapted inside Yazelix to power Ghostty cursor trails and mode-change effects through generated config and Yazelix-managed palette/effect selection. 🔧 auto-configured
- [Kitty](https://sw.kovidgoyal.net/kitty/) — A fast, feature-rich, GPU-accelerated terminal emulator. Yazelix supports Kitty for its performance, modern features, and excellent font rendering.
- [Alacritty](https://github.com/alacritty/alacritty) — A fast, GPU-accelerated terminal emulator written in Rust. Yazelix supports Alacritty for its simplicity, speed, and cross-platform support.
- [foot](https://codeberg.org/dnkl/foot) — A minimalist Wayland terminal that stays lightweight while still supporting modern features like ligatures and Sixel graphics. Under evaluation for Yazelix once profiling confirms the benefits.

## Editor Integration
- [Helix](https://helix-editor.com) — The default modal text editor for Yazelix, with deep integration for sidebar and buffer management. 🚀 deep-integration
- [vim](https://www.vim.org/) / [neovim](https://neovim.io/) / [kakoune](https://kakoune.org/) / etc / **any terminal editor**: Yazelix is designed to let you set your preferred terminal editor via `[editor].command` in `yazelix.toml`. You can use any editor that launches from the terminal and Yazelix will integrate with your chosen editor for file opening from Yazi and from the terminal.


## Yazi Plugins & Extensions
Plugin catalog: https://github.com/yazi-rs/plugins
- [auto-layout.yazi](https://github.com/luccahuguet/auto-layout.yazi) — A Yazi plugin that dynamically adjusts the column layout for optimal sidebar usage. Core to the Yazelix sidebar experience. This is a maintained fork of Joseph Schmitt's [original implementation](https://github.com/josephschmitt/auto-layout.yazi) (unmaintained).
- [sidebar-status.yazi](../configs/yazi/plugins/sidebar-status.yazi/main.lua) — Removes a space-hungry status item so Yazi fits cleanly as a sidebar. Yazelix-only plugin.
- [zoxide-editor.yazi](../configs/yazi/plugins/zoxide-editor.yazi/main.lua) — Bundled Yazelix plugin that reuses Zoxide's interactive jump flow but sends the chosen directory straight into the managed editor/workspace instead of leaving you one level too deep inside Yazi.
- [git.yazi](https://github.com/yazi-rs/plugins/tree/main/git.yazi) — A plugin that shows git status and changes directly in the Yazi sidebar, improving project awareness.
- [starship.yazi](https://github.com/Rolv-Apneseth/starship.yazi) — Displays the Starship prompt in Yazi's header, showing contextual information like git branch, virtual environments, and project details.
- [lazygit.yazi](https://github.com/Lil-Dank/lazygit.yazi) — Launch lazygit directly from Yazi with a keybinding, providing seamless git workflow integration.

## Nushell scripts
- [nuscripts](https://github.com/nushell/nuscripts) — A collection of Nushell scripts, including the `clip` command for copying to the system clipboard. Used in Yazelix for clipboard integration. 🔧 auto-configured

## Runtime Surface

The trimmed v15 branch no longer treats Yazelix as a user-extensible pack graph or package manager. The packaged runtime ships a fixed tool stack, and user configuration focuses on workspace/layout/editor/shell/terminal behavior rather than package composition.

That means:
- there is no `yazelix_packs.toml` sidecar in the current v15 line
- there is no public `yzx packs` / `yazelix packs` workflow in the current v15 line
- the runtime helper tools listed above are part of the shipped runtime surface

## Maintainer Tooling

Repo maintenance still uses a broader maintainer toolchain than the end-user runtime surface. Common maintainer tools around this repo include:
- [gh](https://cli.github.com/) — GitHub CLI for issues, PRs, and repo workflow
- [prek](https://github.com/j178/prek) — Fast local pre-commit hook runner
- [beads](https://github.com/steveyegge/beads) — Beads issue tracker (`bd`) for local planning and GitHub issue contract work
- [nu-lint](https://github.com/nushell/nu-lint) — Optional Nushell linter for explicit maintainer lint runs

Use the repo's root [`../.pre-commit-config.yaml`](../.pre-commit-config.yaml) with `prek install`, then run `prek run --all-files` when you want the fast maintainer checks on demand.

---

Thank you to all maintainers and the open source community for making Yazelix possible! 
