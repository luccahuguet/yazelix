# Yazelix Home Manager Configuration Example
# Shows all available options with sensible defaults
#
# For complete option reference, see:
#   - home_manager/module.nix (complete option definitions)
#   - yazelix_default.toml (TOML structure with detailed comments)

{ config, pkgs, ... }:

{
  programs.yazelix = {
    enable = true;
    manage_config = true; # Opt into declarative Home Manager ownership of yazelix.toml for this example
    runtime_variant = "ghostty"; # Optional: "ghostty" or "wezterm"

    # Shell entry
    default_shell = "zsh";

    # Terminal preference
    terminals = [
      "ghostty"
      "wezterm"
    ];
    terminal_config_mode = "yazelix"; # Optional: "yazelix" or "user"
    ghostty_trail_color = "random"; # Optional: Ghostty color palette rerolled for each Yazelix Ghostty window
    ghostty_trail_effect = "random"; # Optional: "tail", "warp", "sweep", "random", or null
    ghostty_mode_effect = "random"; # Optional: "ripple", "sonic_boom", "rectangle_boom", "ripple_rectangle", "random", or null
    ghostty_trail_glow = "medium"; # Optional: "none", "low", "medium", or "high"
    transparency = "medium"; # Optional: "none".."super_high"

    # Editor configuration
    # editor_command = null;       # Optional: Use Yazelix's Helix (recommended)
    editor_command = "hx"; # Optional: Use system Helix from PATH
    # editor_command = "nvim";     # Optional: Use other editor (loses Helix features)
    # helix_runtime_path = "/home/user/helix/runtime";  # Optional: only for custom/nonstandard Helix runtimes
    # yazi_command = "/path/to/custom/yazi";            # Optional: managed Yazi binary override
    # yazi_ya_command = "/path/to/custom/ya";           # Optional: managed Yazi CLI override

    # Development-friendly settings
    debug_mode = true; # Enable verbose logging
    skip_welcome_screen = false; # Show welcome screen
    welcome_style = "static"; # Static Yazelix logo for faster startup
    game_of_life_cell_style = "full_block"; # Optional: "full_block" or "dotted"
    show_macchina_on_welcome = true;

    # Zellij customization
    initial_sidebar_state = "open";
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

    # Persistent sessions for long-running work
    persistent_sessions = true;
    session_name = "main-dev";
    enable_atuin = true;
  };

  # Optional: install Nushell as your normal interactive shell outside Yazelix.
  # home.packages = with pkgs; [
  #   nushell
  # ];
}
