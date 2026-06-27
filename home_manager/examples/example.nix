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
    terminal = "mars"; # Mars is the packaged terminal; configure host terminals to run `yzx enter`

    # Shell entry
    default_shell = "zsh";

    # Terminal behavior for Mars
    terminal_config_mode = "yazelix"; # Optional: "yazelix" or "user"
    # Cursor presets and effects live in ~/.config/yazelix_cursors/settings.jsonc
    transparency = "medium"; # Optional: "none".."super_high"

    # Editor configuration
    # editor_command = null;       # Optional: Use Yazelix's Helix (recommended)
    editor_command = "hx"; # Optional: Use the packaged Helix command from the Yazelix runtime
    # editor_command = "nvim";     # Optional: Use other editor (loses Helix features)
    # helix_external = {
    #   binary = "/home/user/helix/target/release/hx";
    #   runtime_path = "/home/user/helix/runtime";
    # }; # Optional: Yazelix-compatible Helix fork binary/runtime pair
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
    custom_popups = [
      {
        id = "zenith";
        command = [ "zenith" ];
        keybindings = [ "Alt Shift I" ];
        keep_alive = true;
      }
      # {
      #   id = "btop";
      #   command = [ "btop" ];
      #   keybindings = [ "Alt Shift Y" ];
      #   keep_alive = true;
      # }
    ];

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
