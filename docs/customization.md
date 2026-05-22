# Customization Guide

Start with this model:

- `~/.config/yazelix/settings.jsonc` is the main workspace settings file for shell, editor, terminal, Zellij, Yazi, popup, status, and layout behavior
- `~/.config/yazelix_ghostty_cursors/settings.jsonc` owns standalone Ghostty cursor presets and shader settings
- Generated runtime state lives under `~/.local/share/yazelix`; edit the config inputs, not generated runtime files
- Home Manager installs can own the settings files declaratively; the config UI and status surfaces show read-only ownership when that applies

The sections below cover the override surfaces that sit around that main model.

- **Configuration File**: On first launch, Yazelix creates the main settings and Ghostty cursor settings from shipped defaults. Old mutable `yazelix.toml`, old `cursors.toml`, old `user_configs/` paths, and older embedded cursor settings blocks raise a clear error instead of being rewritten automatically.
  - Run `yzx config ui`, Yazelix's ratconfig-backed JSONC settings editor, to browse and edit settings, defaults, stale-field diagnostics, Home Manager/read-only ownership, and managed sidecar status
  - Use `yzx config set PATH JSON` and `yzx config unset PATH` for safe comment-preserving edits to supported settings and cursor fields
  - Yazelix snapshots the main config for each new window. Live popup, menu, sidebar, reveal, and editor-launch commands keep using that window snapshot, so config edits apply to the next Yazelix window or after `yzx restart`
  - For temporary changes, use repeatable `--with KEY=VALUE` on `yzx launch`, `yzx enter`, or `yzx restart`; Yazelix writes an ephemeral settings snapshot and does not mutate your config file
  - `yzx status --json` and `yzx inspect --json` include `session_config_snapshot` with the active snapshot path, source config, and readable snapshot errors
- **Terminal Configurations**:
  - **Bundled terminals** (yazelix-ghostty, etc.): Configs generated dynamically from your yazelix settings
    - **Ghostty cursor shaders**: Edit `~/.config/yazelix_ghostty_cursors/settings.jsonc` to choose the cursor trail, enabled cursor list, global effects, duration, glow, and Kitty fallback toggle. `yzx cursors` shows the active settings path and resolved preset colors. `settings.trail = "random"` picks from `enabled_cursors`, `settings.trail = "none"` disables the Ghostty palette shader, and `settings.kitty_enable_cursor = false` disables Kitty's simple fallback trail. Cursor definitions use `family = "mono"` for one base color with a derived accent, `family = "split"` for two colors split by `divider = "vertical" | "horizontal"` with `transition = "soft" | "hard"`, or `family = "curated_template"` for hand-tuned shaders.
    - **Standalone Yazelix Ghostty cursors**: Build or install `.#yazelix_ghostty_cursors` to get the generated Yazelix Ghostty cursor shaders without launching Yazelix. Run `yzc init`, edit `~/.config/yazelix_ghostty_cursors/settings.jsonc`, then run `yzc generate ghostty` to write `~/.config/yazelix_ghostty_cursors/ghostty.conf` and generated shaders. Add `config-file = ~/.config/yazelix_ghostty_cursors/ghostty.conf` to Ghostty. `nix run .#yzc -- --help` exposes the same CLI as a flake app, and `.#ghostty_cursor_shaders` remains available as a compatibility package attribute for the same output.
    - **Transparency**: Configure `transparency = "none"`, `"low"`, `"medium"`, or `"high"`
    - **Yazelix-specific terminal overrides**: Add personal terminal-native settings under `~/.config/yazelix/`
      - `terminal_ghostty.conf`
      - `terminal_kitty.conf`
      - `terminal_alacritty.toml`
      - `terminal_foot.ini`
      Yazelix owns startup/integration-critical behavior; these override files are for terminal-local preferences such as theme, fonts, opacity, padding, and cursor style.
    - **Config ownership switch**: `terminal.config_mode = "yazelix"` keeps using Yazelix-managed configs; `"user"` loads the terminal's native config file instead and fails if it is missing
    - **No manual copying required** - generated automatically when launching Yazelix
  - **Reference configs** (generated snapshot): `configs/terminal_emulators/`
    - Snapshots match the generated configs under `~/.local/share/yazelix/configs/terminal_emulators/`
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
- **Helix Themes & Keybindings**: For Yazelix-managed Helix sessions, change Helix themes and keybindings in `~/.config/yazelix/helix.toml`. If you want to start from an existing personal Helix config, run `yzx import helix`. See [Styling and Themes](./styling.md) and [Keybindings](./keybindings.md).
- **Managed Shell Hooks**: Add Yazelix-only shell customizations under `~/.config/yazelix/` instead of personal dotfiles. Supported hook files are `shell_bash.sh`, `shell_zsh.zsh`, `shell_fish.fish`, and `shell_nu.nu`, and they are sourced at the end of the corresponding managed Yazelix shell startup.
- **Keybindings**: Yazelix remaps conflicting keybindings and provides discoverable shortcuts. See [keybindings.md](./keybindings.md) for all details.
- **Styling & Transparency**: Adjust terminal and editor appearance. See [styling.md](./styling.md).
- **Editor Terminal Integration**: Use Yazelix tools in Zed, VS Code, or Cursor integrated terminals. See [editor_terminal_integration.md](./editor_terminal_integration.md).
- **Standalone Screen Animations**: Build or run `.#yzs` to preview the Yazelix screen animation engines outside Zellij and outside a Yazelix session. It supports boids, Mandelbrot, and Game of Life styles and exits on keypress.
- **Yazelix Collection**: For a full list of integrated tools and links to their documentation, see [yazelix_collection.md](./yazelix_collection.md). 
