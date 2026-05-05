# Yazelix Collection

Yazelix is built on a focused terminal-workspace stack. This catalog lists the projects, packages, plugins, and helper tools that make up the current Yazelix product surface.

## Integration Levels

**Core workspace integration**: Tools that define the main Yazelix experience and have Yazelix-owned configuration, layout, or command behavior.

**Managed runtime helper**: Tools shipped in the packaged runtime and exposed through `yzx env`, `yzx run`, generated configs, Yazi previews, status widgets, or shell initializers.

**Supported external surface**: Tools or config surfaces Yazelix supports when the user provides them, but which are not always bundled in the selected runtime variant.

**Maintainer tooling**: Repo-local development and release tools used by maintainers, not part of the normal user workflow.

---

## Package And Distribution Surfaces

- `#yazelix` — The default flake package and app, backed by the Ghostty runtime variant
- `#yazelix_ghostty` — Explicit Ghostty runtime package, equivalent to the default packaged terminal variant
- `#yazelix_wezterm` — Explicit WezTerm runtime package for users who prefer WezTerm behavior, especially around image preview support
- `#runtime`, `#runtime_ghostty`, `#runtime_wezterm` — Runtime-only package outputs used by the wrapper packages and validation surfaces
- `#yazelix_agent_tools` and `#runtime_agent_tools` — Opt-in runtime variants that add agent usage helpers such as `tokenusage`
- `#yazelix_screen` — Standalone terminal animation package for the Yazelix screen engines outside Zellij and outside a full Yazelix session
- `#ghostty_cursor_shaders` — Standalone Ghostty cursor shader package with generated GLSL files and example Ghostty config snippets
- `homeManagerModules.yazelix` — The Home Manager module for declarative installs, with `runtime_variant = "ghostty"` by default and `"wezterm"` available explicitly

## Core Workspace Stack

- [Zellij](https://github.com/zellij-org/zellij) — Terminal multiplexer that owns Yazelix panes, tabs, layouts, keybindings, and session context. Yazelix ships generated Zellij layouts and runtime overlays for the managed workspace.
- [Yazi](https://github.com/sxyazi/yazi) — File manager used as the default Yazelix sidebar and file-tree surface. Yazelix manages Yazi config, plugins, themes, reveal/open flows, and workspace retargeting.
- [Helix](https://helix-editor.com) — Default modal editor. Yazelix integrates Helix with the managed editor pane, Yazi reveal/open flows, and generated editor config.
- [Neovim](https://neovim.io/) — First-class supported editor alternative. Yazelix supports managed editor-pane targeting and reveal/open workflows for Neovim as well.
- [Nushell](https://www.nushell.sh) — Default shell and the remaining shell/UI core. The packaged runtime provides `nu`, the generated shell initializers, `yzx env`, and the small shell floor that still belongs in Nushell.
- [zjstatus](https://github.com/dj95/zjstatus) — Zellij status plugin used for the Yazelix top bar, tab labels, widget tray, custom text, CPU/RAM widgets, and optional agent usage widgets.
- [Yazelix pane orchestrator](../rust_plugins/zellij_pane_orchestrator) — First-party Zellij plugin that owns managed pane identity, editor/sidebar handoff, transient panes, status-cache facts, and workspace state.
- [yazelix_screen](../rust_core/yazelix_screen) — First-party Rust animation engine used by welcome/screen styles such as logo, boids, Mandelbrot, and Game of Life.

## Terminal Emulators

- [Ghostty](https://ghostty.org/) — Default packaged terminal runtime. Yazelix uses Ghostty for the first-party cursor trail and mode-change shader experience.
- [WezTerm](https://wezfurlong.org/wezterm/) — Packaged alternate runtime through `#yazelix_wezterm` and `runtime_variant = "wezterm"`, useful for users who prefer WezTerm terminal behavior and image-preview compatibility.
- [Kitty](https://sw.kovidgoyal.net/kitty/) — Supported PATH-provided terminal choice. Yazelix can generate Kitty config and launch Kitty when it is available on the host.
- [Alacritty](https://github.com/alacritty/alacritty) — Supported PATH-provided terminal choice with generated Yazelix config.
- [Foot](https://codeberg.org/dnkl/foot) — Supported Linux PATH-provided terminal choice with generated Yazelix config.
- [ghostty-cursor-shaders](https://github.com/sahaj-b/ghostty-cursor-shaders) — Upstream inspiration for the Yazelix-managed Ghostty cursor shader system. Yazelix vendors/adapts the shader direction through `settings.jsonc` cursor settings, generated config, and the standalone `#ghostty_cursor_shaders` package.

## Editors And Shells

- [Helix](https://helix-editor.com) — Default editor and strongest editor integration target.
- [Neovim](https://neovim.io/) — First-class editor alternative.
- [Vim](https://www.vim.org/), [Kakoune](https://kakoune.org/), and other terminal editors — Supported through `[editor].command` when they run inside a terminal and can accept Yazelix file-open flows.
- [Bash](https://www.gnu.org/software/bash/) — Runtime shell option and compatibility shell.
- [Fish](https://fishshell.com/) — Runtime shell option with generated initializer support.
- [Zsh](https://www.zsh.org/) — Runtime shell option with generated initializer support.

## Runtime Helper Tools

- [fzf](https://github.com/junegunn/fzf) — Fuzzy finder used by Yazi and shell navigation flows.
- [zoxide](https://github.com/ajeetdsouza/zoxide) — Directory jumper used from the shell, Yazi's native `Z` flow, Yazelix's `Alt+z` direct-open Yazi flow, and `yzx warp`.
- [starship](https://starship.rs) — Prompt engine configured for the managed shells and surfaced inside Yazi through `starship.yazi`.
- [lazygit](https://github.com/jesseduffield/lazygit) — Default managed popup command, normally toggled with `Alt+t`.
- [carapace](https://github.com/rsteube/carapace-bin) — Cross-shell completion engine used by generated shell initializers.
- [mise](https://mise.jdx.dev/) — Runtime/version manager integration loaded by the generated shell initializers.
- [macchina](https://github.com/Macchina-CLI/macchina) — System information helper used by the optional welcome-screen machine summary.
- [tombi](https://tombi-toml.github.io/tombi/) — TOML formatter/linter shipped with Yazelix for managed TOML tooling.
- [jq](https://github.com/jqlang/jq) — JSON helper used by bundled runtime flows and Yazi plugins.
- [fd](https://github.com/sharkdp/fd) — Fast file search helper used by Yazi and runtime tooling.
- [ripgrep](https://github.com/BurntSushi/ripgrep) — Fast text search helper used by Yazi and runtime tooling.
- [p7zip](https://github.com/p7zip-project/p7zip) — Archive preview/extract helper for Yazi.
- [poppler](https://poppler.freedesktop.org/) — PDF preview helper for Yazi.
- [resvg](https://github.com/linebender/resvg) — SVG rendering helper in the packaged runtime.
- [Nix](https://nixos.org/) — Package/runtime owner for Yazelix installs, updates, and Home Manager integration.
- [nixGL](https://github.com/guibou/nixGL) — Optional Linux GL wrapper used by packaged terminal runtime paths when needed.
- [xclip](https://github.com/astrand/xclip), [wl-clipboard](https://github.com/bugaevc/wl-clipboard), and [xsel](https://github.com/kfish/xsel) — Linux clipboard helpers available to Zellij and the managed runtime.
- [tokenusage](https://github.com/hanbu97/tokenusage) — Optional agent-usage helper included by `#yazelix_agent_tools` and Home Manager `agent_usage_programs = [ "tokenusage" ]`.

## Yazi Plugins And Extensions

Plugin catalog: https://github.com/yazi-rs/plugins

- [auto-layout.yazi](https://github.com/luccahuguet/auto-layout.yazi) — Maintained Yazelix fork of the adaptive Yazi column-layout plugin, core to the sidebar fit.
- [sidebar-status.yazi](../configs/yazi/plugins/sidebar-status.yazi/main.lua) — Yazelix-only plugin that removes space-hungry status content so Yazi fits cleanly as a sidebar.
- [sidebar-state.yazi](../configs/yazi/plugins/sidebar-state.yazi/main.lua) — Yazelix-only plugin support for sidebar state and workspace coordination.
- [zoxide-editor.yazi](../configs/yazi/plugins/zoxide-editor.yazi/main.lua) — Yazelix-only plugin behind `Alt+z`; it reuses Zoxide selection and sends the chosen directory straight to the managed editor/workspace.
- [git.yazi](https://github.com/yazi-rs/plugins/tree/main/git.yazi) — Vendored upstream plugin with Yazelix patching for git status in the Yazi file tree.
- [starship.yazi](https://github.com/Rolv-Apneseth/starship.yazi) — Vendored upstream plugin that displays Starship context in Yazi.
- [lazygit.yazi](https://github.com/Lil-Dank/lazygit.yazi) — Vendored upstream plugin that launches lazygit from Yazi.
- [Yazi flavors](../configs/yazi/flavors) — Bundled Yazi theme/flavor catalog available through the Yazelix-managed Yazi config surface.

## User Configuration Surfaces

- [`settings.jsonc`](../config_metadata/yazelix_settings.schema.json) — Canonical semantic settings surface under `~/.config/yazelix/settings.jsonc`, including Ghostty cursor presets and effects
- [Yazi configuration](./yazi-configuration.md) — Personal Yazi config overlays under `~/.config/yazelix/`
- [Zellij configuration](./zellij-configuration.md) — Managed Zellij user config under `~/.config/yazelix/zellij.kdl`
- [Terminal overrides](./terminal_emulators.md) — Terminal-native override files for Ghostty, Kitty, and Alacritty
- [Managed shell hooks](../shells/zsh/README.md) — Yazelix-only shell hook files for Bash, Zsh, Fish, and Nushell.

## Runtime Boundary

The current Yazelix line is a fixed packaged runtime, not a user-extensible package-manager graph.

That means:

- there is no `yazelix_packs.toml` sidecar in the current runtime surface
- there is no public `yzx packs` or `yazelix packs` workflow
- `user_packages` and runtime-local `devenv` are outside the supported current surface
- helper tools listed above are part of the shipped runtime or an explicit flake/Home Manager variant
- alternative terminals outside Ghostty/WezTerm are supported when the user provides them on `PATH`

## Maintainer Tooling

Repo maintenance uses a broader maintainer toolchain than the end-user runtime surface.

- [gh](https://cli.github.com/) — GitHub CLI for issues, PRs, and repo workflow.
- [prek](https://github.com/j178/prek) — Fast local pre-commit hook runner used with the root [`.pre-commit-config.yaml`](../.pre-commit-config.yaml).
- [Beads](https://github.com/steveyegge/beads) — Local issue tracker exposed through `bd` for planning, Beads/GitHub lifecycle sync, and durable agent context.
- [nu-lint](https://github.com/nushell/nu-lint) — Optional Nushell linter for explicit maintainer lint runs.
- [fenix](https://github.com/nix-community/fenix) — Rust toolchain input used by the flake packages and maintainer shell.

---

Thank you to the maintainers of these projects and to the open source community that makes Yazelix possible.
