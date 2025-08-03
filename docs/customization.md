# Customization Guide

Yazelix is highly customizable! Here are the main ways you can tailor your experience:

- **Configuration File**: Edit `~/.config/yazelix/yazelix.nix` for all core options. See [yazelix_default.nix](../yazelix_default.nix) for a full list and descriptions of every option (shell, editor, terminal, recommended tools, sidebar toggle, debug mode, etc).
- **Terminal Emulator Configs**: For all supported terminals, copy and edit the provided configs:
  - [WezTerm config](../configs/terminal_emulators/wezterm/.wezterm.lua)
  - [Ghostty config](../configs/terminal_emulators/ghostty/config)
  - [Kitty config](../configs/terminal_emulators/kitty/kitty.conf)
  - [Alacritty config](../configs/terminal_emulators/alacritty/alacritty.toml)
  - See [WezTerm docs](https://wezfurlong.org/wezterm/config/files.html) for advanced customization.
- **Zellij Configuration**: Three-layer configuration system for maximum flexibility:
  - **User customizations**: Edit [../configs/zellij/user_config.kdl](../configs/zellij/user_config.kdl) for your personal Zellij settings (themes, keybindings, plugins, etc.)
  - **Yazelix overrides**: [../configs/zellij/yazelix_overrides.kdl](../configs/zellij/yazelix_overrides.kdl) contains Yazelix-specific defaults
  - **Layouts**: Customize layouts in [../configs/zellij/layouts/](../configs/zellij/layouts/)
  - **Dynamic merging**: Configurations automatically merge on startup with your settings taking highest priority
- **Yazi Plugins & Keymaps**: Tweak Yazi behavior in [../configs/yazi/](../configs/yazi/) (see [init.lua](../configs/yazi/init.lua), [keymap.toml](../configs/yazi/keymap.toml), and [plugins/](../configs/yazi/plugins/)).
- **Helix Themes & Keybindings**: Change Helix themes and keybindings in your `~/.config/helix/config.toml`. See [Styling and Themes](./styling.md) and [Keybindings](./keybindings.md).
- **Keybindings**: Yazelix remaps conflicting keybindings and provides discoverable shortcuts. See [keybindings.md](./keybindings.md) for all details.
- **Styling & Transparency**: Adjust terminal and editor appearance. See [styling.md](./styling.md).
- **VSCode/Cursor Integration**: Use Yazelix tools in your editor's integrated terminal. See [vscode_cursor_integration.md](./vscode_cursor_integration.md).
- **Yazelix Collection**: For a full list of integrated tools and links to their documentation, see [yazelix_collection.md](./yazelix_collection.md). 