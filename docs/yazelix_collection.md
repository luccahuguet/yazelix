# Yazelix Collection

Yazelix is built on a focused terminal-workspace stack. This catalog lists the projects, packages, plugins, and helper tools that make up the current Yazelix product surface.

## Integration Levels

**Core workspace integration**: Tools that define the main Yazelix experience and have Yazelix-owned configuration, layout, or command behavior.

**Managed runtime helper**: Tools shipped in the packaged runtime and exposed through `yzx env`, `yzx run`, generated configs, Yazi previews, status widgets, or shell initializers.

**Supported external surface**: Tools or config surfaces Yazelix supports when the user provides them, but which are not always bundled in the active runtime package.

**Maintainer tooling**: Repo-local development and release tools used by maintainers, not part of the normal user workflow.

---

## Package And Distribution Surfaces

- `#default` and `#yazelix` — The same complete Mars-backed Yazelix package and app
- `homeManagerModules.default` — The narrow package-plus-sidecars Home Manager module

Checks and development shells remain maintainer outputs rather than supported product API. Runtime-only packages, package builders, overlays, child-package mirrors, Beads, and the install preflight are not re-exported from the main flake.

## First-Party Child Repositories

Regular Yazelix users do not need to install or wire these repositories separately. The regular Yazelix package already integrates the modules it uses, and the child repositories exist so people can use focused Yazelix subsystems without adopting the whole workspace. `yazelix-screen` and `yazelix-cursors` are also usable outside Zellij.

Maintainer-facing fork status, child-repo ownership tables, README delta rules, and review evidence live in [Fork And Child-Repo Maintenance](./contracts/fork_child_repo_maintenance.md).

- [yazelix-screen](https://github.com/luccahuguet/yazelix-screen) — Standalone terminal animation engine consumed by Yazelix welcome/screen rendering; its owning flake exposes `#yzs`.
- [mars](https://github.com/luccahuguet/mars) — Rust terminal fork focused on Yazelix stack compatibility, Kitty-protocol work, and agent-driven development workflows; consumed inside the complete Yazelix package.
- [yazelix-cursors](https://github.com/luccahuguet/yazelix-cursors) — Standalone cursor preset, Ghostty-compatible shader, and `yzc` CLI repository consumed by Yazelix cursor settings; its owning flake exposes `#yzc`.
- [yazelix-helix](https://github.com/luccahuguet/yazelix-helix) — Standalone-usable Steel-enabled Helix fork with `--config-dir`, Yazelix bridge hooks behind explicit runtime flags, and packaged reusable Steel plugin defaults consumed by Yazelix managed Helix sessions.
- [yazelix-zellij-bar](https://github.com/luccahuguet/yazelix-zellij-bar) — Standalone Zellij/zjstatus bar preset consumed by Yazelix tab/status rendering and available from its owning flake.
- [yazelix-zellij-pane-orchestrator](https://github.com/luccahuguet/yazelix-zellij-pane-orchestrator) — First-party Zellij plugin wasm that owns managed pane identity, editor/sidebar handoff, focus actions, and layout-family commands and is available from its owning flake.
- [ratconfig](https://github.com/luccahuguet/ratconfig) — Reusable Ratatui config editor crate with TOML and JSONC adapters, consumed by Yazelix config UI while Yazelix keeps settings schema, Home Manager ownership, validation, and activation timing in this repo.
- [yazelix-zellij-popup](https://github.com/luccahuguet/yazelix-zellij-popup) — Standalone Zellij popup plugin for plain-Zellij users; its owning flake exposes `#yzpp`, and regular Yazelix sessions package the same wasm for popup, menu, and config UI panes.
- [yazelix-zellij-config-pack](https://github.com/luccahuguet/yazelix-zellij-config-pack) — Standalone renderer/config-pack package available from its owning flake.
- [yazelix-yazi-assets](https://github.com/luccahuguet/yazelix-yazi-assets) — Standalone Yazi flavor and reusable plugin asset pack consumed by Yazelix Yazi runtime generation and available from its owning flake.
- [yazelix-zellij](https://github.com/luccahuguet/yazelix-zellij) — Temporary product integration fork consumed by the Mars Kitty-passthrough runtime path so upstream Yazi image previews can use Kitty graphics through Zellij; this fork should be dropped and archived once upstream Zellij supports the required Kitty graphics path directly enough for Yazelix to return to upstream Zellij.

## Core Workspace Stack

- [Zellij](https://github.com/zellij-org/zellij) — Terminal multiplexer that owns Yazelix panes, tabs, layouts, keybindings, and session context. Yazelix ships generated Zellij layouts and runtime overlays for the managed workspace.
- [Yazi](https://github.com/sxyazi/yazi) — File manager used as the default Yazelix sidebar and file-tree surface. Yazelix manages Yazi config, plugins, themes, reveal/open flows, and workspace retargeting.
- [Helix](https://helix-editor.com) — Default modal editor. Yazelix integrates Helix with the managed editor pane, Yazi reveal/open flows, and generated editor config.
- [Neovim](https://neovim.io/) — First-class supported editor alternative. Yazelix supports managed editor-pane targeting and reveal/open workflows for Neovim as well.
- [Nushell](https://www.nushell.sh) — Default shell and the remaining shell/UI core. The packaged runtime provides `nu`, the generated shell initializers, `yzx env`, and the small shell floor that still belongs in Nushell.
- [zjstatus](https://github.com/dj95/zjstatus) — Zellij status plugin used for the Yazelix top bar, tab labels, widget tray, custom text, CPU/RAM widgets, and optional agent usage widgets.
- [Yazelix Zellij pane orchestrator](https://github.com/luccahuguet/yazelix-zellij-pane-orchestrator) — First-party Zellij plugin that owns managed editor/sidebar identity, editor/sidebar handoff, status-cache facts, screen-saver launch, and workspace state.
- [yazelix-screen](https://github.com/luccahuguet/yazelix-screen) — First-party Rust animation engine used by welcome/screen styles such as logo, boids, Mandelbrot, and Game of Life.

## Terminal Emulators

- [Mars](https://github.com/luccahuguet/mars) — Packaged terminal runtime. Mars is a Rust terminal fork focused on Yazelix stack compatibility, optional Kitty protocol growth, and agent-driven development workflows.
- [Ghostty](https://ghostty.org/), [Rio](https://github.com/raphamorim/rio), [WezTerm](https://wezfurlong.org/wezterm/), [Foot](https://codeberg.org/dnkl/foot), [Ratty](https://github.com/orhun/ratty), [Kitty](https://sw.kovidgoyal.net/kitty/), and other capable terminals — Supported host-terminal entrypoints; configure them to run `yzx enter`
- [ghostty-cursor-shaders](https://github.com/sahaj-b/ghostty-cursor-shaders) — Upstream inspiration for the Yazelix-managed Ghostty-compatible cursor shader system. Yazelix adapts the shader direction through the child-owned cursor registry and package.

## Editors And Shells

- [Helix](https://helix-editor.com) — Default editor and strongest editor integration target.
- [Neovim](https://neovim.io/) — First-class editor alternative.
- [Vim](https://www.vim.org/), [Kakoune](https://kakoune.org/), and other terminal editors — Supported through `[editor].command` when they run inside a terminal and can accept Yazelix file-open flows.
- [Bash](https://www.gnu.org/software/bash/) — Runtime shell option and compatibility shell.
- [Fish](https://fishshell.com/) — Runtime shell option with generated initializer support.
- [Zsh](https://www.zsh.org/) — Runtime shell option with generated initializer support.

## Runtime Helper Tools

- [fzf](https://github.com/junegunn/fzf) — Fuzzy finder used by Yazi and shell navigation flows.
- [zoxide](https://github.com/ajeetdsouza/zoxide) — Directory jumper used from the shell, Yazi's native `Z` flow, and Yazelix's `Alt+z` direct-open Yazi flow.
- [starship](https://starship.rs) — Prompt engine configured for the managed shells and surfaced inside Yazi through `starship.yazi`.
- [lazygit](https://github.com/jesseduffield/lazygit) — Default managed popup command, normally toggled with `Alt+Shift+J`.
- [Zenith](https://github.com/bvaisvil/zenith) — Bundled process information viewer for the managed process popup, normally toggled with `Alt+Shift+I`.
- [bottom](https://github.com/ClementTsang/bottom) and [SysWatch](https://github.com/matthart1983/syswatch) — Good process monitor alternatives for users who prefer a custom popup command.
- [Steel](https://github.com/mattwparas/steel) — Scheme runtime and authoring tools for Yazelix-managed Helix Steel plugins.
- [carapace](https://github.com/rsteube/carapace-bin) — Cross-shell completion engine used by generated shell initializers.
- [mise](https://mise.jdx.dev/) — Host-managed runtime/version manager integration loaded by generated shell initializers when `mise` is on `PATH`.
- [macchina](https://github.com/Macchina-CLI/macchina) — System information helper used by the optional welcome-screen machine summary.
- [tombi](https://tombi-toml.github.io/tombi/) — Optional host-managed TOML formatter/linter integration.
- [jq](https://github.com/jqlang/jq) — JSON helper used by bundled runtime flows and Yazi plugins.
- [fd](https://github.com/sharkdp/fd) — Fast file search helper used by Yazi and runtime tooling.
- [ripgrep](https://github.com/BurntSushi/ripgrep) — Fast text search helper used by Yazi and runtime tooling.
- [p7zip](https://github.com/p7zip-project/p7zip) — Archive preview/extract helper for Yazi.
- [poppler](https://poppler.freedesktop.org/) — PDF preview helper for Yazi.
- [resvg](https://github.com/linebender/resvg) — SVG rendering helper in the packaged runtime.
- [Nix](https://nixos.org/) — Package/runtime owner for Yazelix installs, updates, and Home Manager integration.
- [nixGL](https://github.com/guibou/nixGL) — Optional Linux GL wrapper used by packaged terminal runtime paths when needed.
- [xclip](https://github.com/astrand/xclip), [wl-clipboard](https://github.com/bugaevc/wl-clipboard), and [xsel](https://github.com/kfish/xsel) — Linux clipboard helpers available to Zellij and the managed runtime.
- [tokenusage](https://github.com/hanbu97/tokenusage) — Agent-usage helper included in the packaged runtime for the default Codex and Claude status widgets.

## Yazi Plugins And Extensions

Plugin catalog: https://github.com/yazi-rs/plugins

- [auto-layout.yazi](https://github.com/luccahuguet/yazelix-yazi-assets/tree/main/plugins/auto-layout.yazi) — Maintained Yazelix fork of the adaptive Yazi column-layout plugin, core to the sidebar fit.
- [sidebar-status.yazi](../configs/yazi/plugins/sidebar-status.yazi/main.lua) — Yazelix-only plugin that removes space-hungry status content so Yazi fits cleanly as a sidebar.
- [sidebar-state.yazi](../configs/yazi/plugins/sidebar-state.yazi/main.lua) — Yazelix-only plugin support for sidebar state and workspace coordination.
- [zoxide-editor.yazi](../configs/yazi/plugins/zoxide-editor.yazi/main.lua) — Yazelix-only plugin behind `Alt+z`; it reuses Zoxide selection and sends the chosen directory straight to the managed editor/workspace.
- [git.yazi](https://github.com/luccahuguet/yazelix-yazi-assets/tree/main/plugins/git.yazi) — Vendored upstream plugin with Yazelix patching for git status in the Yazi file tree.
- [starship.yazi](https://github.com/luccahuguet/yazelix-yazi-assets/tree/main/plugins/starship.yazi) — Vendored upstream plugin that displays Starship context in Yazi.
- [lazygit.yazi](https://github.com/luccahuguet/yazelix-yazi-assets/tree/main/plugins/lazygit.yazi) — Vendored upstream plugin that launches lazygit from Yazi.
- [Yazi flavors](https://github.com/luccahuguet/yazelix-yazi-assets/tree/main/flavors) — Bundled Yazi theme/flavor catalog available through the Yazelix-managed Yazi config surface.

## User Configuration Surfaces

- [`config.toml`](../config_metadata/yazelix_settings.schema.json) — Canonical semantic settings inventory; main settings live under `~/.config/yazelix/config.toml` and cursor presets live under `~/.config/yazelix/cursors.toml`
- [Yazi configuration](./yazi-configuration.md) — Personal Yazi config overlays under `~/.config/yazelix/yazi/`
- [Zellij configuration](./zellij-configuration.md) — `config.toml` for Yazelix-owned behavior plus nested native preference and plugin sidecars
- [Terminal emulators](./terminal_emulators.md) — Mars as the deepest integrated packaged terminal, Ghostty as the most tested mature host-terminal path, and Rio, WezTerm, Kitty, Foot, Ratty, Alacritty, and other capable emulators through `yzx enter`
- [Managed shell hooks](./customization.md) — Yazelix-only shell hook files for Bash, Zsh, Fish, Nushell, and host-owned xonsh initializers, with managed paths listed in [POSIX/XDG Paths](./posix_xdg.md).

## Runtime Boundary

The current Yazelix line is a fixed packaged runtime, not a user-extensible package-manager graph.

That means:

- there is no `yazelix_packs.toml` sidecar in the current runtime surface
- there is no public `yzx packs` or `yazelix packs` workflow
- `user_packages` and runtime-local `devenv` are outside the supported current surface
- helper tools listed above are part of the shipped runtime or an explicit flake/Home Manager variant
- non-Mars terminals are supported through `yzx enter` when the user provides the terminal

## Maintainer Tooling

Repo maintenance uses a broader maintainer toolchain than the end-user runtime surface.

- [gh](https://cli.github.com/) — GitHub CLI for issues, PRs, and repo workflow.
- [prek](https://github.com/j178/prek) — Fast local pre-commit hook runner used with the root [`.pre-commit-config.yaml`](../.pre-commit-config.yaml).
- [Beads Rust](https://github.com/Dicklesworthstone/beads_rust) — Local issue tracker exposed through `br` for planning, Beads/GitHub lifecycle sync, and durable agent context.
- [nu-lint](https://github.com/nushell/nu-lint) — Optional Nushell linter for explicit maintainer lint runs.
- [fenix](https://github.com/nix-community/fenix) — Rust toolchain input used by the flake packages and maintainer shell.

---

Thank you to the maintainers of these projects and to the open source community that makes Yazelix possible.
