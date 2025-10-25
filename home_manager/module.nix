{ config, lib, pkgs, ... }:

with lib;

let
  cfg = config.programs.yazelix;
  
  # Helper function to convert package list to Nix expression string
  nixPackagesToString = packages: 
    if packages == [] then "[]"
    else "[" + (concatStringsSep " " (map (pkg: pkg.pname or "«unknown»") packages)) + "]";

in {
  options.programs.yazelix = {
    enable = mkEnableOption "Yazelix terminal environment";
    
    # Configuration options (mirrors yazelix_default.nix structure)
    recommended_deps = mkOption {
      type = types.bool;
      default = true;
      description = "Install recommended productivity tools (~350MB)";
    };
    
    yazi_extensions = mkOption {
      type = types.bool;
      default = true;
      description = "Install Yazi file preview extensions (~125MB)";
    };
    
    yazi_media = mkOption {
      type = types.bool;
      default = false;
      description = "Install Yazi media processing tools (~1GB)";
    };
    
    helix_mode = mkOption {
      type = types.enum [ "release" "source" ];
      default = "release";
      description = "Helix build mode: release (nixpkgs) or source (flake)";
    };
    
    default_shell = mkOption {
      type = types.enum [ "nu" "bash" "fish" "zsh" ];
      default = "nu";
      description = "Default shell for Zellij sessions";
    };
    
    extra_shells = mkOption {
      type = types.listOf (types.enum [ "fish" "zsh" ]);
      default = [];
      description = "Additional shells to install beyond nu/bash";
    };

    preferred_terminal = mkOption {
      type = types.enum [ "wezterm" "ghostty" "kitty" "alacritty" "foot" ];
      default = "ghostty";
      description = "Preferred terminal emulator for launch commands";
    };

    terminal_config_mode = mkOption {
      type = types.enum [ "auto" "user" "yazelix" ];
      default = "yazelix";
      description = ''
        How Yazelix selects terminal configs:
        - "yazelix": use Yazelix-managed configs in ~/.local/share/yazelix (default)
        - "auto": prefer user configs if present, otherwise Yazelix configs
        - "user": always use user configs (e.g., ~/.config/ghostty/config)
      '';
    };

    extra_terminals = mkOption {
      type = types.listOf (types.enum [ "wezterm" "kitty" "alacritty" "foot" ]);
      default = [];
      description = "Additional terminal emulators to install beyond Ghostty";
    };

    cursor_trail = mkOption {
      type = types.enum [ "blaze" "snow" "cosmic" "ocean" "forest" "sunset" "neon" "party" "prism" "orchid" "reef" "random" "none" ];
      default = "random";
      description = ''
        Cursor trail preset.
        Supported by all terminal emulators: "none"
        Supported by Ghostty: "blaze", "snow", "cosmic", "ocean", "forest", "sunset", "neon", "party", "prism", "orchid", "reef", "random"
        Supported by Ghostty and Kitty: "snow"
        "random" chooses a different Ghostty trail each generation (excluding "none" and "party")
      '';
    };

    transparency = mkOption {
      type = types.enum [ "none" "low" "medium" "high" ];
      default = "low";
      description = ''
        Terminal transparency level for all terminals.

        - "none": No transparency (opacity = 1.0)
        - "low": Light transparency (opacity = 0.95)
        - "medium": Medium transparency (opacity = 0.9)
        - "high": High transparency (opacity = 0.8)
      '';
    };
    
    # Editor configuration
    editor_command = mkOption {
      type = types.nullOr types.str;
      default = null;
      description = ''
        Editor command - yazelix will always set this as EDITOR.
        
        - null (default): Use yazelix's Nix-provided Helix to avoid runtime conflicts
        - "hx": Use system Helix from PATH (requires matching helix_runtime_path)
        - Other editors: "vim", "nvim", "nano", etc. (loses Helix-specific features)
      '';
    };
    
    helix_runtime_path = mkOption {
      type = types.nullOr types.str;
      default = null;
      description = ''
        Custom Helix runtime path - only set this if editor_command points to a custom Helix build.
        
        IMPORTANT: The runtime MUST match your Helix binary version to avoid startup errors.
        Example: "/home/user/helix/runtime" for a custom Helix build in ~/helix
      '';
    };
    
    enable_sidebar = mkOption {
      type = types.bool;
      default = true;
      description = "Enable or disable the Yazi sidebar";
    };

    disable_zellij_tips = mkOption {
      type = types.bool;
      default = false;
      description = "Disable Zellij tips popup on startup for cleaner launches";
    };
    
    debug_mode = mkOption {
      type = types.bool;
      default = false;
      description = "Enable verbose debug logging";
    };
    
    skip_welcome_screen = mkOption {
      type = types.bool;
      default = false;
      description = "Skip the welcome screen on startup";
    };
    
    ascii_art_mode = mkOption {
      type = types.enum [ "static" "animated" ];
      default = "static";
      description = "ASCII art display mode";
    };
    
    show_macchina_on_welcome = mkOption {
      type = types.bool;
      default = true;
      description = "Show macchina system info on welcome screen";
    };
    
    persistent_sessions = mkOption {
      type = types.bool;
      default = false;
      description = "Enable persistent Zellij sessions";
    };
    
    session_name = mkOption {
      type = types.str;
      default = "yazelix";
      description = "Session name for persistent sessions";
    };
    
    packs = mkOption {
      type = types.listOf (types.enum [ "python" "js_ts" "rust" "config" "file-management" ]);
      default = [];
      description = "Package packs to enable entire technology stacks";
    };

    enable_atuin = mkOption {
      type = types.bool;
      default = false;
      description = "Enable Atuin shell history integration (disabled by default).";
    };
    
    user_packages = mkOption {
      type = types.listOf types.package;
      default = [];
      description = "Additional packages to install in Yazelix environment";
    };
  };

  config = mkIf cfg.enable {
    # Desktop integration - copy yazelix assets
    xdg.configFile."yazelix/assets/logo.png".source = ../assets/logo.png;
    xdg.configFile."yazelix/assets/icons/48x48/yazelix.png".source = ../assets/icons/48x48/yazelix.png;
    xdg.configFile."yazelix/assets/icons/64x64/yazelix.png".source = ../assets/icons/64x64/yazelix.png;
    xdg.configFile."yazelix/assets/icons/128x128/yazelix.png".source = ../assets/icons/128x128/yazelix.png;
    xdg.configFile."yazelix/assets/icons/256x256/yazelix.png".source = ../assets/icons/256x256/yazelix.png;
    xdg.configFile."yazelix/docs/desktop_icon_setup.md".source = ../docs/desktop_icon_setup.md;

    # Desktop entry for application launcher
    xdg.desktopEntries.yazelix = {
      name = "Yazelix";
      comment = "Yazi + Zellij + Helix integrated terminal environment";
      exec = "${config.xdg.configHome}/yazelix/nushell/scripts/core/desktop_launcher.nu";
      icon = "yazelix";
      categories = [ "Development" ];
      type = "Application";
      startupWMClass = "com.yazelix.Yazelix";
    };

    # Generate yazelix.nix configuration file
    xdg.configFile."yazelix/yazelix.nix" = {
      text = ''
        { pkgs }:
        {
          # Dependency groups
          recommended_deps = ${if cfg.recommended_deps then "true" else "false"};
          yazi_extensions = ${if cfg.yazi_extensions then "true" else "false"};
          yazi_media = ${if cfg.yazi_media then "true" else "false"};
          
          # Helix configuration
          helix_mode = "${cfg.helix_mode}";
          
          # Shell configuration
          default_shell = "${cfg.default_shell}";
          extra_shells = ${builtins.toJSON cfg.extra_shells};

          # Terminal configuration
          preferred_terminal = "${cfg.preferred_terminal}";
          terminal_config_mode = "${cfg.terminal_config_mode}";
          enable_atuin = ${if cfg.enable_atuin then "true" else "false"};
          extra_terminals = ${builtins.toJSON cfg.extra_terminals};
          cursor_trail = "${cfg.cursor_trail}";
          transparency = "${cfg.transparency}";
          
          # Editor configuration
          editor_command = ${if cfg.editor_command != null then ''"${cfg.editor_command}"'' else "null"};
          helix_runtime_path = ${if cfg.helix_runtime_path != null then ''"${cfg.helix_runtime_path}"'' else "null"};
          
          # UI configuration
          enable_sidebar = ${if cfg.enable_sidebar then "true" else "false"};
          disable_zellij_tips = ${if cfg.disable_zellij_tips then "true" else "false"};

          # Debug and display options
          debug_mode = ${if cfg.debug_mode then "true" else "false"};
          skip_welcome_screen = ${if cfg.skip_welcome_screen then "true" else "false"};
          ascii_art_mode = "${cfg.ascii_art_mode}";
          show_macchina_on_welcome = ${if cfg.show_macchina_on_welcome then "true" else "false"};
          
          # Session configuration
          persistent_sessions = ${if cfg.persistent_sessions then "true" else "false"};
          session_name = "${cfg.session_name}";
          
          # Package packs
          packs = ${builtins.toJSON cfg.packs};
          
          # User packages
          user_packages = with pkgs; ${nixPackagesToString cfg.user_packages};
        }
      '';
    };
  };
}
