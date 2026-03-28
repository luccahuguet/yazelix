# Customization Guide

Yazelix is highly customizable! Here are the main ways you can tailor your experience:

- **Configuration File**: Edit `~/.config/yazelix/yazelix.toml` for all core options. See [yazelix_default.toml](../yazelix_default.toml) for a full list and descriptions of every option (shell, editor, terminal, recommended tools, sidebar toggle, debug mode, etc).
- **Terminal Configurations**:
  - **Bundled terminals** (yazelix-ghostty, etc.): Configs generated dynamically from your yazelix settings
    - **Ghostty cursor shaders**: Use `ghostty_trail_color` for the palette, `ghostty_trail_effect` for cursor-movement trails, and `ghostty_mode_effect` for mode-change pulses like Neovim normal/insert transitions. `ghostty_trail_color = "none"` disables the Yazelix palette shader and Kitty fallback trail; the others default to `random`. Helix does not support every trail effect yet; Neovim currently has the best support.
    - **Transparency**: Configure `transparency = "none"`, `"low"`, `"medium"`, or `"high"`
    - **Yazelix-specific terminal overrides**: For Ghostty, Kitty, and Alacritty, add personal terminal-native settings under `~/.config/yazelix/terminal_overrides/`
      - `ghostty`
      - `kitty.conf`
      - `alacritty.toml`
      Yazelix owns startup/integration-critical behavior; these override files are for terminal-local preferences such as theme, fonts, opacity, padding, and cursor style.
    - **Config ownership switch**: `terminal.config_mode = "yazelix"` keeps using Yazelix-managed configs; `"user"` loads the terminal's native config file instead and fails if it is missing
    - **No manual copying required** - generated automatically when launching Yazelix
  - **Reference configs** (generated snapshot): `configs/terminal_emulators/`
    - Refresh snapshots: `yzx dev sync_terminal_configs` (uses `yazelix_default.toml`)
    - Snapshots match the generated configs under `~/.local/share/yazelix/configs/terminal_emulators/`
- **Zellij Configuration**: Git-conflict-free three-layer configuration system:
  - **Quick start**: 
    ```bash
    cp ~/.config/yazelix/configs/zellij/user/user_config.kdl ~/.config/yazelix/configs/zellij/personal/user_config.kdl
    ```
    Then edit the personal copy
  - **Full guide**: [Zellij Configuration Documentation](./zellij-configuration.md)
  - **Three layers**: Zellij defaults + Yazelix overrides + your personal settings (highest priority)
  - **Smart merging**: Configurations automatically merge on startup, personal configs are git ignored
- **Yazi Configuration**: Git-conflict-free two-layer configuration system:
  - **Quick start**: 
    ```bash
    cp -r configs/yazi/user configs/yazi/personal
    ```
    Then edit personal configs
  - **Full guide**: [Yazi Configuration Documentation](./yazi-configuration.md)
  - **Two layers**: Yazelix defaults + your personal overrides (highest priority)
  - **TOML merging**: Intelligent section merging prevents duplicate keys, personal configs are git ignored
- **Helix Themes & Keybindings**: Change Helix themes and keybindings in your `~/.config/helix/config.toml`. See [Styling and Themes](./styling.md) and [Keybindings](./keybindings.md).
- **Keybindings**: Yazelix remaps conflicting keybindings and provides discoverable shortcuts. See [keybindings.md](./keybindings.md) for all details.
- **Styling & Transparency**: Adjust terminal and editor appearance. See [styling.md](./styling.md).
- **Editor Terminal Integration**: Use Yazelix tools in Zed, VS Code, or Cursor integrated terminals. See [editor_terminal_integration.md](./editor_terminal_integration.md).
- **Yazelix Collection**: For a full list of integrated tools and links to their documentation, see [yazelix_collection.md](./yazelix_collection.md). 
