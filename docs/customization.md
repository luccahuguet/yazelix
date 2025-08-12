# Customization Guide

Yazelix is highly customizable! Here are the main ways you can tailor your experience:

- **Configuration File**: Edit `~/.config/yazelix/yazelix.nix` for all core options. See [yazelix_default.nix](../yazelix_default.nix) for a full list and descriptions of every option (shell, editor, terminal, recommended tools, sidebar toggle, debug mode, etc).
- **Terminal Emulator Configs**: For all supported terminals, copy and edit the provided configs:
  - [WezTerm config](../configs/terminal_emulators/wezterm/.wezterm.lua)
  - [Ghostty config](../configs/terminal_emulators/ghostty/config)
  - [Kitty config](../configs/terminal_emulators/kitty/kitty.conf)
  - [Alacritty config](../configs/terminal_emulators/alacritty/alacritty.toml)
  - See [WezTerm docs](https://wezfurlong.org/wezterm/config/files.html) for advanced customization.
- **Zellij Configuration**: Git-conflict-free three-layer configuration system:
  - **Quick start**: `cp ~/.config/yazelix/configs/zellij/user/user_config.kdl ~/.config/yazelix/configs/zellij/personal/user_config.kdl` then edit the personal copy
  - **Full guide**: [Zellij Configuration Documentation](./zellij-configuration.md)
  - **Three layers**: Zellij defaults + Yazelix overrides + your personal settings (highest priority)
  - **Smart merging**: Configurations automatically merge on startup, personal configs are git ignored
- **Yazi Configuration**: Git-conflict-free two-layer configuration system:
  - **Quick start**: `cp -r configs/yazi/user configs/yazi/personal` then edit personal configs  
  - **Full guide**: [Yazi Configuration Documentation](./yazi-configuration.md)
  - **Two layers**: Yazelix defaults + your personal overrides (highest priority)
  - **TOML merging**: Intelligent section merging prevents duplicate keys, personal configs are git ignored
- **Helix Themes & Keybindings**: Change Helix themes and keybindings in your `~/.config/helix/config.toml`. See [Styling and Themes](./styling.md) and [Keybindings](./keybindings.md).
- **Keybindings**: Yazelix remaps conflicting keybindings and provides discoverable shortcuts. See [keybindings.md](./keybindings.md) for all details.
- **Styling & Transparency**: Adjust terminal and editor appearance. See [styling.md](./styling.md).
- **VSCode/Cursor Integration**: Use Yazelix tools in your editor's integrated terminal. See [vscode_cursor_integration.md](./vscode_cursor_integration.md).
- **Yazelix Collection**: For a full list of integrated tools and links to their documentation, see [yazelix_collection.md](./yazelix_collection.md). 