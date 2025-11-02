{ config, lib, pkgs, ... }:

with lib;

let
  cfg = config.programs.yazelix;

  boolToToml = value: if value then "true" else "false";

  escapeString = value:
    let
      safe = lib.replaceStrings [ "\"" "\\" ] [ "\\\"" "\\\\" ] value;
    in "\"${safe}\"";

  listToToml = values:
    if values == [] then "[]"
    else "[ " + (concatStringsSep ", " (map escapeString values)) + " ]";

  packagesToToml = packages:
    let
      names = map (pkg: pkg.pname or pkg.name or "unknown") packages;
    in listToToml names;

in {
  options.programs.yazelix = {
    enable = mkEnableOption "Yazelix terminal environment";
    
    # Configuration options (mirrors yazelix_default.toml structure)
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
      type = types.enum [ "blaze" "snow" "cosmic" "ocean" "forest" "sunset" "neon" "party" "eclipse" "dusk" "orchid" "reef" "inferno" "random" "none" ];
      default = "random";
      description = ''
        Cursor trail preset.
        Supported by all terminal emulators: "none"
        Supported by Ghostty: "blaze", "snow", "cosmic", "ocean", "forest", "sunset", "neon", "party", "eclipse", "dusk", "orchid", "reef", "inferno", "random"
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

        - null (default): Use yazelix's Nix-provided Helix - full integration
        - "nvim": Use Neovim - first-class support with full integration
        - "hx": Use system Helix from PATH (requires matching helix_runtime_path)
        - Other editors: "vim", "nano", "emacs", etc. (basic integration only)
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
      default = true;
      description = "Disable Zellij tips popup on startup for cleaner launches";
    };

    zellij_rounded_corners = mkOption {
      type = types.bool;
      default = true;
      description = "Enable rounded corners for Zellij pane frames";
    };

    debug_mode = mkOption {
      type = types.bool;
      default = false;
      description = "Enable verbose debug logging";
    };
    
    skip_welcome_screen = mkOption {
      type = types.bool;
      default = true;
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
    
    language_packs = mkOption {
      type = types.listOf (types.enum [ "python" "ts" "rust" "go" "kotlin" "gleam" "nix" ]);
      default = [];
      description = "Language packs - complete toolchains for programming languages";
    };

    tool_packs = mkOption {
      type = types.listOf (types.enum [ "config" "file-management" "git" ]);
      default = [];
      description = "Tool packs - general-purpose development tools";
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

    # Generate yazelix.toml configuration file
    xdg.configFile."yazelix/yazelix.toml" = {
      text = let
        editorCommand = if cfg.editor_command != null then cfg.editor_command else "";
        helixRuntimeLine =
          if cfg.helix_runtime_path != null then
            [ "runtime_path = ${escapeString cfg.helix_runtime_path}" ]
          else
            [];
      in lib.concatStringsSep "\n" (
        [
          "# Generated by the Yazelix Home Manager module."
          "# Edit your Home Manager configuration instead of this file."
          ""
          "[core]"
          "recommended_deps = ${boolToToml cfg.recommended_deps}"
          "yazi_extensions = ${boolToToml cfg.yazi_extensions}"
          "yazi_media = ${boolToToml cfg.yazi_media}"
          "debug_mode = ${boolToToml cfg.debug_mode}"
          "skip_welcome_screen = ${boolToToml cfg.skip_welcome_screen}"
          "show_macchina_on_welcome = ${boolToToml cfg.show_macchina_on_welcome}"
          ""
          "[helix]"
          "mode = ${escapeString cfg.helix_mode}"
        ] ++ helixRuntimeLine ++ [
          ""
          "[editor]"
          "command = ${escapeString editorCommand}"
          "enable_sidebar = ${boolToToml cfg.enable_sidebar}"
          ""
          "[shell]"
          "default_shell = ${escapeString cfg.default_shell}"
          "extra_shells = ${listToToml cfg.extra_shells}"
          "enable_atuin = ${boolToToml cfg.enable_atuin}"
          ""
          "[terminal]"
          "preferred_terminal = ${escapeString cfg.preferred_terminal}"
          "extra_terminals = ${listToToml cfg.extra_terminals}"
          "config_mode = ${escapeString cfg.terminal_config_mode}"
          "cursor_trail = ${escapeString cfg.cursor_trail}"
          "transparency = ${escapeString cfg.transparency}"
          ""
          "[zellij]"
          "disable_tips = ${boolToToml cfg.disable_zellij_tips}"
          "rounded_corners = ${boolToToml cfg.zellij_rounded_corners}"
          "persistent_sessions = ${boolToToml cfg.persistent_sessions}"
          "session_name = ${escapeString cfg.session_name}"
          ""
          "[ascii]"
          "mode = ${escapeString cfg.ascii_art_mode}"
          ""
          "[packs]"
          "language = ${listToToml cfg.language_packs}"
          "tools = ${listToToml cfg.tool_packs}"
          "user_packages = ${packagesToToml cfg.user_packages}"
          ""
        ]
      ) + "\n";
    };
  };
}
