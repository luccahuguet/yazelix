# Project Credits

Yazelix is built on the shoulders of giants. Here are the projects, tools, and plugins that Yazelix integrates with or is inspired by, organized to match the Yazelix configuration structure. Each entry links to the project's homepage or repository and includes a description of its role in Yazelix.

---

## Essential Tools
- [Yazi](https://github.com/sxyazi/yazi) — A blazing-fast, modern terminal file manager with Vim-like keybindings, preview support, and extensibility. Yazi is the sidebar and file navigation backbone of Yazelix.
- [Zellij](https://github.com/zellij-org/zellij) — A powerful terminal multiplexer that manages panes, layouts, and tabs. Zellij orchestrates the Yazelix workspace, allowing seamless integration between file manager, editor, and shell.
- [Helix](https://helix-editor.com) — A modal text editor inspired by Kakoune and Neovim, featuring fast performance, tree-sitter syntax highlighting, and LSP support. Helix is the default editor for Yazelix, enabling advanced workflows and sidebar integration.
- [Nushell](https://www.nushell.sh) — A modern shell that treats data as structured tables, making scripting and configuration more robust. Nushell is the default shell for Yazelix and powers its configuration and scripting.
- [fzf](https://github.com/junegunn/fzf) — A general-purpose command-line fuzzy finder. Used in Yazelix for quick file and directory navigation, especially within Yazi and Nushell.
- [zoxide](https://github.com/ajeetdsouza/zoxide) — A smarter cd command, tracking your most-used directories for instant navigation. Integrated into Yazi and Nushell for fast directory switching.
- [starship](https://starship.rs) — A minimal, blazing-fast, and customizable prompt for any shell. Provides Yazelix with a beautiful, informative, and context-aware shell prompt.
- [bash](https://www.gnu.org/software/bash/) — The GNU Bourne Again Shell, included for compatibility and as a fallback shell option.
- [macchina](https://github.com/Macchina-CLI/macchina) — A fast, customizable system information fetch tool. Used to display system info on the Yazelix welcome screen.
- [libnotify](https://github.com/GNOME/libnotify) — Provides desktop notifications from the command line. Used for visual feedback in some Yazelix scripts.

## Extra Shells
- [Fish](https://fishshell.com/) — The Friendly Interactive Shell. Fish offers user-friendly features, autosuggestions, and syntax highlighting. Yazelix can install and integrate with Fish if selected in the configuration.
- [Zsh](https://www.zsh.org/) — The Z Shell. Zsh is a powerful, highly customizable shell with advanced scripting capabilities. Yazelix can install and integrate with Zsh if selected in the configuration.

## Recommended Tools
- [lazygit](https://github.com/jesseduffield/lazygit) — A simple terminal UI for git commands, making version control fast and intuitive. Yazelix includes lazygit for easy git management.
- [atuin](https://github.com/atuinsh/atuin) — A shell history manager with sync and search capabilities. Enhances command recall and productivity in Yazelix.
- [carapace](https://github.com/rsteube/carapace-bin) — A cross-shell command-line completion engine. Improves tab completion in supported shells.
- [markdown-oxide](https://oxide.md/index) — A personal knowledge management system (PKMS) that works with text editors through LSP. Included for advanced note-taking and documentation workflows.

## Yazi Extensions
- [p7zip](https://github.com/p7zip-project/p7zip) — A port of the 7-Zip archiver. Enables archive extraction and compression in Yazi.
- [jq](https://github.com/jqlang/jq) — A lightweight and flexible command-line JSON processor. Used by Yazi plugins for parsing and manipulating JSON data.
- [fd](https://github.com/sharkdp/fd) — A simple, fast, and user-friendly alternative to find. Powers fast file search in Yazi.
- [ripgrep](https://github.com/BurntSushi/ripgrep) — A line-oriented search tool that recursively searches your current directory for a regex pattern. Used for fast text search in Yazi.
- [poppler](https://poppler.freedesktop.org/) — A PDF rendering library. Enables PDF previews in Yazi.

## Yazi Media Extensions
- [ffmpeg](https://ffmpeg.org/) — A complete, cross-platform solution to record, convert, and stream audio and video. Used for media previews in Yazi.
- [ImageMagick](https://imagemagick.org/) — A software suite to create, edit, compose, or convert bitmap images. Enables image previews and thumbnails in Yazi.

## Terminal Emulators
- [WezTerm](https://wezfurlong.org/wezterm/) — A GPU-accelerated terminal emulator and multiplexer written in Rust. Yazelix supports WezTerm for its advanced features, performance, and modern design.
- [Ghostty](https://ghostty.org/) — A fast, modern terminal emulator written in Zig. Yazelix supports Ghostty as an equally excellent choice, offering speed and a modern feature set.
- [Kitty](https://sw.kovidgoyal.net/kitty/) — A fast, feature-rich, GPU-accelerated terminal emulator. Yazelix supports Kitty for its performance, modern features, and excellent font rendering.
- [Alacritty](https://github.com/alacritty/alacritty) — A fast, GPU-accelerated terminal emulator written in Rust. Yazelix supports Alacritty for its simplicity, speed, and cross-platform support.

## Editor Integration
- [Helix](https://helix-editor.com) — The default modal text editor for Yazelix, with deep integration for sidebar and buffer management.
- [vim](https://www.vim.org/) / [neovim](https://neovim.io/) / [kakoune](https://kakoune.org/) / etc / **any terminal editor**: Yazelix is designed to let you set your preferred terminal editor via the `editor_command` configuration option. You can use any editor that launches from the terminal and Yazelix will integrate with your chosen editor for file opening from yazi and from the terminal.


## Yazi Plugins & Extensions
- [auto-layout.yazi](https://github.com/josephschmitt/auto-layout.yazi) — A Yazi plugin that dynamically adjusts the column layout for optimal sidebar usage. Core to the Yazelix sidebar experience.
- [git.yazi](https://github.com/yazi-rs/plugins/tree/main/git.yazi) — A plugin that shows git status and changes directly in the Yazi sidebar, improving project awareness.
- [sidebar_status.yazi](https://github.com/sxyazi/yazi-plugins) — Enhances the Yazi sidebar with additional status information and visual cues.

## Notable Contributions & One-of-a-Kind Integrations
- [nuscripts](https://github.com/nushell/nuscripts) — A collection of Nushell scripts, including the `clip` command for copying to the system clipboard. Used in Yazelix for clipboard integration.
- [auto-layout.yazi](https://github.com/josephschmitt/auto-layout.yazi) — Special thanks to Joseph Schmitt for the auto-layout plugin, which is essential to the Yazelix sidebar experience.

## User Packages
Yazelix allows you to add your own Nix packages via the `user_packages` config option in `yazelix.nix`. The default configuration includes examples organized by "packs":

### Package Management Pack
- [cargo-update](https://github.com/nabijaczleweli/cargo-update) — Updates Rust crates for project maintenance
- [cargo-binstall](https://github.com/cargo-bins/cargo-binstall) — Faster installation of Rust tools
- [mise](https://github.com/jdx/mise) — Tool version manager for consistent environments

### JavaScript/TypeScript Pack
- [biome](https://biomejs.dev/) — Formats JS, TS, JSON, CSS, and lints JS/TS
- [bun](https://bun.sh/) — Fast all-in-one JavaScript runtime, bundler, test runner, and package manager

### Python Pack
- [ruff](https://github.com/astral-sh/ruff) — Fast Python linter and code formatter
- [uv](https://github.com/astral-sh/uv) — Ultra-fast Python package installer and resolver
- [ty](https://github.com/astral-sh/ruff) — Extremely fast Python type checker from Astral

### File Management Pack
- [ouch](https://github.com/ouch-org/ouch) — Compression tool for handling archives
- [erdtree](https://github.com/solidiquis/erdtree) — Modern tree command with file size display
- [serpl](https://github.com/serpl/serpl) — Command-line tool for search and replace operations

### Configuration Pack
- [taplo](https://github.com/tamasfe/taplo) — TOML toolkit for formatting, validation, and language server support

These packages are commented out by default in `yazelix_default.nix` - uncomment the ones you want to use!

---

Thank you to all maintainers and the open source community for making Yazelix possible! 