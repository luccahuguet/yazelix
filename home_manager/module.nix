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
      type = types.enum [ "wezterm" "ghostty" "kitty" "alacritty" ];
      default = "ghostty";
      description = "Preferred terminal emulator for launch commands";
    };
    
    # Editor configuration (flat structure to match main flake)
    set_editor = mkOption {
      type = types.bool;
      default = true;
      description = "Whether to set EDITOR environment variable";
    };
    override_existing = mkOption {
      type = types.bool;
      default = true;
      description = "Whether to override existing EDITOR if already set";
    };
    editor_command = mkOption {
      type = types.str;
      default = "hx";
      description = "Custom editor command (hx, vim, nvim, etc.)";
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
    
    user_packages = mkOption {
      type = types.listOf types.package;
      default = [];
      description = "Additional packages to install in Yazelix environment";
    };
  };

  config = mkIf cfg.enable {
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
          
          # Editor configuration
          ${if cfg.set_editor then ''
          set_editor = true;
          override_existing = ${if cfg.override_existing then "true" else "false"};
          editor_command = "${cfg.editor_command}";
          '' else ''
          set_editor = false;
          override_existing = false;
          editor_command = "hx";
          ''}
          
          # Debug and display options
          debug_mode = ${if cfg.debug_mode then "true" else "false"};
          skip_welcome_screen = ${if cfg.skip_welcome_screen then "true" else "false"};
          ascii_art_mode = "${cfg.ascii_art_mode}";
          show_macchina_on_welcome = ${if cfg.show_macchina_on_welcome then "true" else "false"};
          
          # Session configuration
          persistent_sessions = ${if cfg.persistent_sessions then "true" else "false"};
          session_name = "${cfg.session_name}";
          
          # User packages
          user_packages = with pkgs; ${nixPackagesToString cfg.user_packages};
        }
      '';
    };
  };
}
