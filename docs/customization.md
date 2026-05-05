# Customization Guide

Yazelix is highly customizable! Here are the main ways you can tailor your experience:

- **Configuration File**: Edit `~/.config/yazelix/settings.jsonc` for core options and Ghostty cursor settings. On first launch, Yazelix creates it from the shipped defaults; old mutable `yazelix.toml` and `cursors.toml` files auto-migrate when safe and block with a clear error when ownership is ambiguous.
  - Run `yzx config ui` to browse settings, defaults, stale-field diagnostics, Home Manager/read-only ownership, and managed sidecar status without writing files
  - Yazelix snapshots the main config for each new window. Live popup, menu, sidebar, reveal, and editor-launch commands keep using that window snapshot, so config edits apply to the next Yazelix window or after `yzx restart`
  - For temporary changes, use repeatable `--with KEY=VALUE` on `yzx launch`, `yzx enter`, or `yzx restart`; Yazelix writes an ephemeral settings snapshot and does not mutate your config file
  - `yzx status --json` and `yzx inspect --json` include `session_config_snapshot` with the active snapshot path, source config, and readable snapshot errors
- **Terminal Configurations**:
  - **Bundled terminals** (yazelix-ghostty, etc.): Configs generated dynamically from your yazelix settings
    - **Ghostty cursor shaders**: Edit the `cursors` object in `~/.config/yazelix/settings.jsonc` to choose the cursor trail, enabled cursor list, global effects, duration, glow, and Kitty fallback toggle. `yzx cursors` shows the active settings path and resolved preset colors. `cursors.settings.trail = "random"` picks from `cursors.enabled_cursors`, `cursors.settings.trail = "none"` disables the Ghostty palette shader, and `cursors.settings.kitty_enable_cursor = false` disables Kitty's simple fallback trail. Cursor definitions use `family = "mono"` for one base color with a derived accent, `family = "split"` for two colors split by `divider = "vertical" | "horizontal"` with `transition = "soft" | "hard"`, or `family = "curated_template"` for hand-tuned shaders.
    - **Standalone Ghostty cursor shaders**: Build or install `.#ghostty_cursor_shaders` to get the generated Yazelix Ghostty cursor shaders without launching Yazelix. The package output includes complete GLSL files plus `share/yazelix/ghostty_cursor_shaders/examples/ghostty_blaze_tail.conf` with `custom-shader` lines for your Ghostty config.
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
- **Zellij Configuration**: Yazelix-managed user config plus generated runtime overlays:
  - **Quick start**: Edit `~/.config/yazelix/zellij.kdl`
  - **Full guide**: [Zellij Configuration Documentation](./zellij-configuration.md)
  - **Three layers**: Your Yazelix-managed Zellij config or Zellij defaults + Yazelix dynamic settings + Yazelix enforced settings
  - **Managed input boundary**: Yazelix reads `zellij.kdl` in managed mode and regenerates the merged runtime config on startup
- **Yazi Configuration**: Git-conflict-free two-layer configuration system:
  - **Quick start**: create only the Yazi override files you need under `~/.config/yazelix/`
    - `yazi.toml`
    - `yazi_keymap.toml`
    - `yazi_init.lua`
  - **Full guide**: [Yazi Configuration Documentation](./yazi-configuration.md)
  - **Two layers**: Yazelix defaults + your personal overrides (highest priority)
  - **TOML merging**: Intelligent section merging prevents duplicate keys, personal configs are git ignored
- **Helix Themes & Keybindings**: For Yazelix-managed Helix sessions, change Helix themes and keybindings in `~/.config/yazelix/helix.toml`. If you want to start from an existing personal Helix config, run `yzx import helix`. See [Styling and Themes](./styling.md) and [Keybindings](./keybindings.md).
- **Managed Shell Hooks**: Add Yazelix-only shell customizations under `~/.config/yazelix/` instead of personal dotfiles. Supported hook files are `shell_bash.sh`, `shell_zsh.zsh`, `shell_fish.fish`, and `shell_nu.nu`, and they are sourced at the end of the corresponding managed Yazelix shell startup.
- **Keybindings**: Yazelix remaps conflicting keybindings and provides discoverable shortcuts. See [keybindings.md](./keybindings.md) for all details.
- **Styling & Transparency**: Adjust terminal and editor appearance. See [styling.md](./styling.md).
- **Editor Terminal Integration**: Use Yazelix tools in Zed, VS Code, or Cursor integrated terminals. See [editor_terminal_integration.md](./editor_terminal_integration.md).
- **Standalone Screen Animations**: Build or run `.#yazelix_screen` to preview the Yazelix screen animation engines outside Zellij and outside a Yazelix session. It supports boids, Mandelbrot, and Game of Life styles and exits on keypress.
- **Yazelix Collection**: For a full list of integrated tools and links to their documentation, see [yazelix_collection.md](./yazelix_collection.md). 
