# Yazelix Home Manager Configuration Example
# Shows all available options with sensible defaults
#
# For complete option reference, see:
#   - home_manager/module.nix (complete option definitions)
#   - settings.jsonc (generated canonical settings file)

{ config, pkgs, ... }:

{
  programs.yazelix = {
    enable = true;
    manage_config = true; # Opt into declarative Home Manager ownership of settings.jsonc for this example
    runtime_variant = "ghostty"; # Optional: "ghostty", "wezterm", or Linux-only "ratty"

    # Shell entry
    default_shell = "zsh";

    # Terminal preference
    terminals = [
      "ghostty"
      "wezterm"
    ];
    terminal_config_mode = "yazelix"; # Optional: "yazelix" or "user"
    # Ghostty cursor presets and effects live in ~/.config/yazelix_ghostty_cursors/settings.jsonc
    transparency = "medium"; # Optional: "none".."super_high"

    # Editor configuration
    # editor_command = null;       # Optional: Use Yazelix's Helix (recommended)
    editor_command = "hx"; # Optional: Use the packaged Helix command from the Yazelix runtime
    # editor_command = "nvim";     # Optional: Use other editor (loses Helix features)
    # helix_external = {
    #   binary = "/home/user/helix/target/release/hx";
    #   runtime_path = "/home/user/helix/runtime";
    # }; # Optional: custom Helix fork binary/runtime pair
    helix_steel_plugins = {
      enabled = [
        "recentf"
        "splash"
        "spacemacs_theme"
      ];
      extra = [
        # {
        #   id = "hello_yazelix";
        #   source = "hello_yazelix.scm"; # Below ~/.config/yazelix/helix/steel_plugins
        #   public_commands = [ "hello-yazelix" ];
        # }
      ];
    };
    # yazi_command = "/path/to/custom/yazi";            # Optional: managed Yazi binary override
    # yazi_ya_command = "/path/to/custom/ya";           # Optional: managed Yazi CLI override

    # Development-friendly settings
    debug_mode = true; # Enable verbose logging
    skip_welcome_screen = false; # Show welcome screen
    welcome_style = "static"; # Static Yazelix logo for faster startup
    game_of_life_cell_style = "full_block"; # Optional: "full_block" or "dotted"
    show_macchina_on_welcome = true;

    # Zellij customization
    hide_sidebar_on_file_open = false;
    disable_zellij_tips = true;
    zellij_rounded_corners = true;
    zellij_theme = "default"; # Optional: any built-in theme name

    # Yazi customization
    yazi_plugins = [
      "git"
      "starship"
    ];
    yazi_theme = "tokyo-night"; # Optional: flavor name or "random-dark"
    yazi_sort_by = "alphabetical";
  };

  # Optional: install Nushell as your normal interactive shell outside Yazelix.
  # home.packages = with pkgs; [
  #   nushell
  # ];
}
