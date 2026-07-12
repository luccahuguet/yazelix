# Yazelix Home Manager Configuration Example
# Shows all available options with sensible defaults
#
# For complete option reference, see:
#   - home_manager/module.nix (complete option definitions)
#   - config.toml (generated canonical settings file)

{ config, pkgs, ... }:

{
  programs.yazelix = {
    enable = true;
    manage_config = true; # Opt into declarative Home Manager ownership of config.toml for this example
    terminal = "mars"; # Mars is the packaged terminal; configure host terminals to run `yzx enter`

    # Semantic config.toml overrides
    shell_program = "zsh";
    editor_command = "hx";
    agent_command = "auto";
    welcome_enabled = true;
    welcome_style = "static";
    bar_widgets = [
      "editor"
      "shell"
      "term"
      "codex_usage"
      "cpu"
      "ram"
    ];
    popups.zenith = {
      command = "zenith";
      keybinding = "Alt Shift I";
      keep_alive = true;
    };

    # Complete native Mars config
    config.mars.text = ''
      [mars.appearance]
      preset = "dark"
    '';

    # Guarded native Zellij preferences
    config.zellij.text = ''
      scroll_buffer_size 5000
      mouse_mode true
    '';

  };

  # Optional: install Nushell as your normal interactive shell outside Yazelix.
  # home.packages = with pkgs; [
  #   nushell
  # ];
}
