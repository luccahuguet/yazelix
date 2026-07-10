# Customization Guide

Start with this model:

- `~/.config/yazelix/settings.jsonc` is the main workspace settings file for shell, editor, Zellij, Yazi, popup, status, and layout behavior
- `~/.config/yazelix/mars/config.toml` is the optional complete native Mars config
- `~/.config/yazelix_cursors/settings.jsonc` owns Yazelix cursor presets and shader settings
- Generated runtime state lives under `~/.local/share/yazelix`; edit the config inputs, not generated runtime files
- Home Manager installs can own the settings files declaratively; the config UI and status surfaces show read-only ownership when that applies

The sections below cover the override surfaces that sit around that main model.

- **Configuration File**: On first launch, Yazelix creates the main settings and cursor settings from shipped defaults. Old mutable `yazelix.toml`, old `cursors.toml`, old `user_configs/` paths, and older embedded cursor settings blocks raise a clear error instead of being rewritten automatically.
  - Run `yzx config ui` to edit JSONC settings and browse or patch the native Mars TOML document through Ratconfig
  - Use `yzx config set PATH JSON` and `yzx config unset PATH` for safe comment-preserving edits to supported settings and cursor fields
  - Yazelix snapshots the main config for each new window. Live popup, menu, sidebar, reveal, and editor-launch commands keep using that window snapshot, so config edits apply to the next Yazelix window or after `yzx restart`
  - For temporary changes, use repeatable `--with KEY=VALUE` on `yzx launch`, `yzx enter`, or `yzx restart`; Yazelix writes an ephemeral settings snapshot and does not mutate your config file
  - `yzx status --json` and `yzx inspect --json` include `session_config_snapshot` with the active snapshot path, source config, and readable snapshot errors
- **Terminal Configurations**:
  - **Mars**: The default `#yazelix` package, `#yazelix_mars`, and `programs.yazelix.terminal = "mars"` launch Mars with `~/.config/yazelix/mars/config.toml` when that complete file exists, otherwise with the packaged complete config. There is no generated or merged Mars config.
  - **Ghostty**: The most tested mature host-terminal path, with a strong macOS story. Configure Ghostty to start `yzx enter`; run `yzx cursors ghostty setup` when you want Yazelix cursor shaders in a user-owned Ghostty config.
  - **Other terminals**: Rio, WezTerm, Kitty, Foot, Ratty, Alacritty, and other capable emulators work by starting Yazelix with `yzx enter`; their native terminal config stays host-owned
    - **Yazelix cursor shaders**: Edit `~/.config/yazelix_cursors/settings.jsonc` to choose the cursor trail, enabled cursor list, global effects, duration, glow, and Kitty fallback toggle. `yzx cursors` shows the active settings path and resolved preset colors. `settings.trail = "random"` picks from `enabled_cursors`, `settings.trail = "none"` disables the Ghostty-compatible palette shader, and `settings.kitty_enable_cursor = false` disables Kitty's simple fallback trail. Cursor definitions use `family = "mono"` for one base color with a derived accent or `family = "split"` for two colors split by `divider = "vertical" | "horizontal"` with `transition = "soft" | "hard"`.
    - **Scrollback ownership**: Configure Mars scrollback natively in `mars/config.toml`; Zellij separately owns pane history inside Yazelix
    - **Appearance mode**: `appearance.mode` controls remaining Yazelix-owned generated themes. Mars appearance belongs to `[mars.appearance]` in the native Mars config
    - **Transparency**: Set `window.opacity` and `window.opacity-cells` directly in `mars/config.toml`
    - **Cursor**: Configure Mars `[yazelix.cursor]` directly. The separate `~/.config/yazelix_cursors/settings.jsonc` registry continues to own standalone Ghostty cursor generation
    - **Home Manager**: Set exactly one of `programs.yazelix.config.mars.text` or `programs.yazelix.config.mars.source` to install the complete file declaratively
- **Visible managed stubs**: Yazelix creates lightweight README or hook stubs under `~/.config/yazelix/` when a managed surface becomes relevant. It does not create behavior-owning Zellij or Helix config files automatically, so native fallback and `yzx import` discovery keep working until you choose those managed surfaces.
- **Native config status**: Yazelix treats native tool configs as user-owned unless you explicitly import them or select a supported native read-only mode. The shared status words are `managed_default`, `managed_override`, `imported_override`, `native_read_only`, `native_available`, `native_required_missing`, `home_manager_read_only`, and `generated_runtime`.
- **Zellij Configuration**: `settings.jsonc` for Yazelix-owned behavior plus generated runtime overlays and an advanced native sidecar:
  - **Quick start**: edit `settings.jsonc` for keybindings, popup commands, widgets, and layout settings
  - **Advanced native settings**: edit `~/.config/yazelix/zellij.kdl` for Zellij settings Yazelix does not render
  - **Full guide**: [Zellij Configuration Documentation](./zellij-configuration.md)
  - **Managed input boundary**: Yazelix rejects `keybinds` blocks in managed `zellij.kdl` and regenerates the merged runtime config on startup
- **Yazi Configuration**: Git-conflict-free two-layer configuration system:
  - **Quick start**: create only the Yazi override files you need under `~/.config/yazelix/yazi/`
    - `yazi.toml`
    - `keymap.toml`
    - `init.lua`
    - `plugins/`
  - **Full guide**: [Yazi Configuration Documentation](./yazi-configuration.md)
  - **Two layers**: Yazelix defaults + your personal overrides (highest priority)
  - **TOML merging**: Intelligent section merging prevents duplicate keys, personal configs are git ignored
- **Helix Themes & Keybindings**: For Yazelix-managed Helix sessions, change Helix themes and keybindings in `~/.config/yazelix/helix/config.toml`, and place custom theme TOML files under `~/.config/yazelix/helix/themes/`. Native `~/.config/helix/themes/` is for plain Helix outside Yazelix, and old `~/.config/yazelix/user_conf/helix/themes/` files are unsupported legacy state. If you want to start from an existing personal Helix config, run `yzx import helix`. See [Styling and Themes](./styling.md) and [Keybindings](./keybindings.md).
- **Managed Shell Hooks**: Add Yazelix-only shell customizations under `~/.config/yazelix/` instead of personal dotfiles. Supported managed-startup hook files are `shell_bash.sh`, `shell_zsh.zsh`, `shell_fish.fish`, and `shell_nu.nu`, and they are sourced at the end of the corresponding managed Yazelix shell startup. `shell_xonsh.xsh` is host-owned; `shell.default_shell = "xonsh"` requires host `xonsh` on `PATH`, and native xonsh startup should source the hook from `~/.xonshrc` or `~/.config/xonsh/rc.xsh`.
- **Keybindings**: Yazelix remaps conflicting keybindings and provides discoverable shortcuts. See [keybindings.md](./keybindings.md) for all details.
- **Styling & Transparency**: Adjust terminal and editor appearance. See [styling.md](./styling.md).
- **Editor Terminal Integration**: Use Yazelix tools in Zed, VS Code, or Cursor integrated terminals. See [editor_terminal_integration.md](./editor_terminal_integration.md).
- **Standalone Screen Animations**: Build or run `.#yzs` to preview the Yazelix screen animation engines outside Zellij and outside a Yazelix session. It supports boids, Mandelbrot, and Game of Life styles and exits on keypress.
- **Yazelix Collection**: For a full list of integrated tools and links to their documentation, see [yazelix_collection.md](./yazelix_collection.md). 
