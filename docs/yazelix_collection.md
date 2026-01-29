# Yazelix Collection

Yazelix is built on the shoulders of giants. Here are the projects, tools, and plugins that Yazelix integrates with or is inspired by, organized to match the Yazelix configuration structure. Each entry links to the project's homepage or repository and includes a description of its role in Yazelix.

## Integration Levels

**Deep Integration** (🚀 deep-integration): Essential tools like Yazi, Zellij, and Helix have custom configurations, keybindings, and scripts that make them work seamlessly together.

**Pre-configured** (🔧 auto-configured): Tools with custom Yazelix configurations, shell initializers, or special setup.

**Curated Recommendations**: High-quality tools included in `yazelix_default.toml` as optional packages. These can be easily enabled/disabled by updating your config - **Yazelix doesn't have special integration with most of these projects**. They're just excellent tools we recommend!

---

## Essential Tools
- [Yazi](https://github.com/sxyazi/yazi) — A blazing-fast, modern terminal file manager with Vim-like keybindings, preview support, and extensibility. Yazi is the sidebar and file navigation backbone of Yazelix. 🚀 deep-integration
- [Zellij](https://github.com/zellij-org/zellij) — A powerful terminal multiplexer that manages panes, layouts, and tabs. Zellij orchestrates the Yazelix workspace, allowing seamless integration between file manager, editor, and shell. 🚀 deep-integration
- [zjstatus](https://github.com/dj95/zjstatus) — A configurable status bar plugin for Zellij. Yazelix uses zjstatus to display shell and editor information with custom formatting: `[shell: nu] [editor: hx] YAZELIX`. 🚀 deep-integration
- [Helix](https://helix-editor.com) — A modal text editor inspired by Kakoune and Neovim, featuring fast performance, tree-sitter syntax highlighting, and LSP support. Helix is the default editor for Yazelix, enabling advanced workflows and sidebar integration. 🚀 deep-integration
- [Nushell](https://www.nushell.sh) — A modern shell that treats data as structured tables, making scripting and configuration more robust. Nushell is the default shell for Yazelix and powers its configuration and scripting. (default shell)
- [fzf](https://github.com/junegunn/fzf) — A general-purpose command-line fuzzy finder. Used in Yazelix for quick file and directory navigation. Press `z` in Yazi or `fzf` from terminal.
- [zoxide](https://github.com/ajeetdsouza/zoxide) — A smarter cd command, tracking your most-used directories for instant navigation. Press `Z` in Yazi or `z` from terminal. 🔧 auto-configured
- [starship](https://starship.rs) — A minimal, blazing-fast, and customizable prompt for any shell. Provides Yazelix with a beautiful, informative, and context-aware shell prompt. 🔧 auto-configured
- [bash](https://www.gnu.org/software/bash/) — The GNU Bourne Again Shell, included for compatibility and as a fallback shell option.
- [macchina](https://github.com/Macchina-CLI/macchina) — A fast, customizable system information fetch tool. Used to display system info on the Yazelix welcome screen.
- [libnotify](https://github.com/GNOME/libnotify) — Provides desktop notifications from the command line. Used for visual feedback in some Yazelix scripts.

## Extra Shells
- [Fish](https://fishshell.com/) — The Friendly Interactive Shell. Fish offers user-friendly features, autosuggestions, and syntax highlighting. Yazelix can install and integrate with Fish if selected in the configuration.
- [Zsh](https://www.zsh.org/) — The Z Shell. Zsh is a powerful, highly customizable shell with advanced scripting capabilities. Yazelix can install and integrate with Zsh if selected in the configuration.

## Recommended Tools
- [lazygit](https://github.com/jesseduffield/lazygit) — A simple terminal UI for git commands, making version control fast and intuitive. Yazelix includes lazygit for easy git management.
- [atuin](https://github.com/atuinsh/atuin) — A shell history manager with sync and search capabilities. Enhances command recall and productivity in Yazelix. 🔧 auto-configured
- [carapace](https://github.com/rsteube/carapace-bin) — A cross-shell command-line completion engine. Improves tab completion in supported shells. 🔧 auto-configured
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
- [foot](https://codeberg.org/dnkl/foot) — A minimalist Wayland terminal that stays lightweight while still supporting modern features like ligatures and Sixel graphics. Under evaluation for Yazelix once profiling confirms the benefits.

## Editor Integration
- [Helix](https://helix-editor.com) — The default modal text editor for Yazelix, with deep integration for sidebar and buffer management. 🚀 deep-integration
- [vim](https://www.vim.org/) / [neovim](https://neovim.io/) / [kakoune](https://kakoune.org/) / etc / **any terminal editor**: Yazelix is designed to let you set your preferred terminal editor via the `editor_command` configuration option. You can use any editor that launches from the terminal and Yazelix will integrate with your chosen editor for file opening from yazi and from the terminal.


## Yazi Plugins & Extensions
Plugin catalog: https://github.com/yazi-rs/plugins
- [auto-layout.yazi](https://github.com/luccahuguet/auto-layout.yazi) — A Yazi plugin that dynamically adjusts the column layout for optimal sidebar usage. Core to the Yazelix sidebar experience. This is a maintained fork of Joseph Schmitt's [original implementation](https://github.com/josephschmitt/auto-layout.yazi) (unmaintained).
- [sidebar-status.yazi](../configs/yazi/plugins/sidebar-status.yazi/main.lua) — Removes a space-hungry status item so Yazi fits cleanly as a sidebar. Yazelix-only plugin.
- [git.yazi](https://github.com/yazi-rs/plugins/tree/main/git.yazi) — A plugin that shows git status and changes directly in the Yazi sidebar, improving project awareness.
- [starship.yazi](https://github.com/Rolv-Apneseth/starship.yazi) — Displays the Starship prompt in Yazi's header, showing contextual information like git branch, virtual environments, and project details.
- [lazygit.yazi](https://github.com/Lil-Dank/lazygit.yazi) — Launch lazygit directly from Yazi with a keybinding, providing seamless git workflow integration.

## Nushell scripts
- [nuscripts](https://github.com/nushell/nuscripts) — A collection of Nushell scripts, including the `clip` command for copying to the system clipboard. Used in Yazelix for clipboard integration. 🔧 auto-configured

## User Packages

Yazelix offers two ways to add packages:

**Pack declarations**: Define packs in `[packs.declarations]` and enable them via `packs.enabled`:
```toml
[packs]
enabled = ["python", "git"]
user_packages = ["docker", "kubectl", "gleam"]

[packs.declarations]
python = [
  "ruff",
  "uv",
  "ty",
  "python3Packages.ipython",
]
git = [
  "onefetch",
  "gh",
  "delta",
  "gitleaks",
  "jujutsu",
  "prek",
]
```

**Individual packages**: Add specific tools via `user_packages` in `yazelix.toml`:
```toml
[packs]
user_packages = ["docker", "kubectl", "gleam"]
```

## Example Pack Declarations

Complete toolchains you can declare:

### Python Pack (`python`)
- [ruff](https://github.com/astral-sh/ruff) — Fast Python linter and code formatter
- [uv](https://github.com/astral-sh/uv) — Ultra-fast Python package installer and resolver
- [ty](https://github.com/astral-sh/ty) — Extremely fast Python type checker from Astral
- [ipython](https://ipython.org/) — Enhanced interactive Python REPL with autocomplete, syntax highlighting, and magic commands

### TypeScript Pack (`ts`)
- [typescript-language-server](https://github.com/typescript-language-server/typescript-language-server) — TypeScript language server for IDE features and LSP support
- [biome](https://biomejs.dev/) — Formats JS, TS, JSON, CSS, and lints JS/TS
- [oxlint](https://oxc-project.github.io/) — Extremely fast TypeScript/JavaScript linter from the oxc project
- [bun](https://bun.sh/) — Fast all-in-one JavaScript runtime, bundler, test runner, and package manager

### Rust Pack (`rust`)
- [cargo-edit](https://github.com/killercup/cargo-edit) — Add, remove, and upgrade dependencies from the command line (`cargo add`, `cargo rm`)
- [cargo-watch](https://github.com/watchexec/cargo-watch) — Auto-recompile and re-run on file changes
- [cargo-audit](https://github.com/rustsec/rustsec/tree/main/cargo-audit) — Audit dependencies for security vulnerabilities

### Rust Extra Pack (`rust_extra`)
- [cargo-update](https://github.com/nabijaczleweli/cargo-update) — Updates Rust crates for project maintenance
- [cargo-binstall](https://github.com/cargo-bins/cargo-binstall) — Faster installation of Rust tools
- [cargo-nextest](https://github.com/nextest-rs/nextest) — Next-generation test runner with better output and parallelism

### Go Pack (`go`)
- [gopls](https://github.com/golang/tools/tree/master/gopls) — Official Go language server for IDE features and LSP support
- [golangci-lint](https://github.com/golangci/golangci-lint) — Fast, comprehensive Go linter aggregator running multiple linters in parallel

### Go Extra Pack (`go_extra`)
- [delve](https://github.com/go-delve/delve) — Powerful debugger for Go with breakpoints, variable inspection, and more
- [air](https://github.com/cosmtrek/air) — Live reload for Go development with hot reloading on file changes
- [govulncheck](https://golang.org/x/vuln/cmd/govulncheck) — Official Go vulnerability scanner from the Go security team

### Kotlin Pack (`kotlin`)
- [kotlin-language-server](https://github.com/fwcd/kotlin-language-server) — Language server for IDE features and LSP support
- [ktlint](https://github.com/pinterest/ktlint) — Linter and formatter with automatic code style fixing
- [detekt](https://github.com/detekt/detekt) — Static code analysis tool for code quality and smell detection
- [gradle](https://gradle.org/) — Build automation tool for Kotlin/JVM projects

### Nix Pack (`nix`)
- [nil](https://github.com/oxalica/nil) — Nix language server for IDE features (LSP support for Helix, VSCode, etc.)
- [nixd](https://github.com/nix-community/nixd) — Alternative Nix language server with advanced features and diagnostics
- [nixfmt](https://github.com/NixOS/nixfmt) — Official Nix code formatter

## Tool Packs

General-purpose development tools:

### Configuration Pack (`config`)
- [taplo](https://github.com/tamasfe/taplo) — TOML formatter and language server for configuration files (included by default)
- [mpls](https://github.com/mhersson/mpls) — Markdown Preview Language Server with live browser preview and Mermaid/PlantUML support
- [yaml-language-server](https://github.com/redhat-developer/yaml-language-server) — Language Server for YAML files

### File Management Pack (`file-management`)
- [ouch](https://github.com/ouch-org/ouch) — Compression tool for handling archives
- [erdtree](https://github.com/solidiquis/erdtree) — Modern tree command with file size display
- [serpl](https://github.com/serpl/serpl) — Command-line tool for search and replace operations

### Git Pack (`git`)
- [onefetch](https://github.com/o2sh/onefetch) — Git repository summary with statistics and language breakdown
- [gh](https://cli.github.com/) — GitHub CLI for repository management and PR workflows
- [prek](https://github.com/piotrek-szczygiel/prek) — Prettier git commit logs and history viewer

### Jujutsu Pack (`jj`)
- [jujutsu](https://github.com/martinvonz/jj) — Modern version control system with powerful conflict resolution (command: `jj`)
- [lazyjj](https://github.com/Cretezy/lazyjj) — LazyGit-style TUI for jj
- [jjui](https://github.com/idursun/jjui) — TUI for Jujutsu VCS

### AI Pack (`ai`)
- [gemini-cli](https://github.com/google-gemini/gemini-cli) — Gemini CLI for chat and automation
- [codex](https://github.com/openai/codex) — Codex CLI for agentic coding workflows
- [opencode](https://github.com/opencode-ai/opencode) — OpenCode CLI for code assistance

### Unfree Pack (`unfree`)
- [claude-code](https://github.com/anthropics/claude-code) — Claude Code CLI (unfree; enabled via the `unfree` pack)

**Usage**: Enable packs in `yazelix.toml` by listing them in `packs.enabled` and defining them in `packs.declarations`, or add individual tools via `user_packages` for fine-grained control.

---

Thank you to all maintainers and the open source community for making Yazelix possible! 
