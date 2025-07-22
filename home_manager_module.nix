{
  config,
  lib,
  pkgs,
  ...
}:

let
  cfg = config.programs.yazelix;

  # Convert camelCase options to snake_case for yazelix.nix
  yazelixConfig = {
    recommended_deps = cfg.recommendedDeps;
    yazi_extensions = cfg.yaziExtensions;
    yazi_media = cfg.yaziMedia;
    helix_mode = cfg.helixMode;
    default_shell = cfg.defaultShell;
    extra_shells = cfg.extraShells;
    debug_mode = cfg.debugMode;
    skip_welcome_screen = cfg.skipWelcomeScreen;
    preferred_terminal = cfg.preferredTerminal;
    ascii_art_mode = cfg.asciiArtMode;
    show_macchina_on_welcome = cfg.showMacchinaOnWelcome;
    editor_config = {
      set_editor = cfg.editorConfig.setEditor;
      override_existing = cfg.editorConfig.overrideExisting;
      editor_command = cfg.editorConfig.editorCommand;
    };
    user_packages = cfg.userPackages;
  };

  # Generate yazelix.nix content
  yazelixNixContent = ''
    { pkgs }:
    ${lib.generators.toPretty { } yazelixConfig}
  '';

in
{
  options.programs.yazelix = {
    enable = lib.mkEnableOption "Yazelix integrated terminal environment";

    package = lib.mkOption {
      type = lib.types.package;
      default = pkgs.fetchFromGitHub {
        owner = "luccahuguet";
        repo = "yazelix";
        rev = "main";
        sha256 = lib.fakeSha256; # Users need to update this
      };
      description = "Yazelix package source";
    };

    # Core configuration options
    recommendedDeps = lib.mkOption {
      type = lib.types.bool;
      default = true;
      description = "Include recommended tools like lazygit, mise, atuin, etc.";
    };

    yaziExtensions = lib.mkOption {
      type = lib.types.bool;
      default = true;
      description = "Include Yazi extensions for file previews, archives, etc.";
    };

    yaziMedia = lib.mkOption {
      type = lib.types.bool;
      default = true;
      description = "Include heavy media packages for Yazi (~800MB-1.2GB). Enables video/image previews.";
    };

    helixMode = lib.mkOption {
      type = lib.types.enum [
        "release"
        "source"
      ];
      default = "release";
      description = ''
        Helix build mode:
        - "release": Use latest Helix release from nixpkgs (recommended for first-time users)
        - "source": Use Helix from flake repository (bleeding edge features)
      '';
    };

    # Shell configuration
    defaultShell = lib.mkOption {
      type = lib.types.enum [
        "nu"
        "bash"
        "fish"
        "zsh"
      ];
      default = "nu";
      description = "Default shell for Zellij sessions";
    };

    extraShells = lib.mkOption {
      type = lib.types.listOf (
        lib.types.enum [
          "fish"
          "zsh"
        ]
      );
      default = [ ];
      description = "Extra shells to install beyond nushell and bash";
    };

    # Terminal configuration
    preferredTerminal = lib.mkOption {
      type = lib.types.enum [
        "wezterm"
        "ghostty"
        "kitty"
      ];
      default = "wezterm";
      description = "Preferred terminal emulator for launching Yazelix";
    };

    # Editor configuration
    editorConfig = {
      setEditor = lib.mkOption {
        type = lib.types.bool;
        default = true;
        description = "Whether to set EDITOR environment variable";
      };

      overrideExisting = lib.mkOption {
        type = lib.types.bool;
        default = true;
        description = "Whether to override existing EDITOR if already set";
      };

      editorCommand = lib.mkOption {
        type = lib.types.str;
        default = "hx";
        description = "Editor command to use (hx, vim, nvim, etc.)";
      };
    };

    # UI and behavior options
    asciiArtMode = lib.mkOption {
      type = lib.types.enum [
        "static"
        "animated"
      ];
      default = "animated";
      description = "ASCII art display mode in welcome screen";
    };

    skipWelcomeScreen = lib.mkOption {
      type = lib.types.bool;
      default = false;
      description = "Skip the welcome screen on startup";
    };

    showMacchinaOnWelcome = lib.mkOption {
      type = lib.types.bool;
      default = true;
      description = "Show system information using macchina on welcome screen";
    };

    debugMode = lib.mkOption {
      type = lib.types.bool;
      default = false;
      description = "Enable verbose debug logging";
    };

    # Custom packages
    userPackages = lib.mkOption {
      type = lib.types.listOf lib.types.package;
      default = [ ];
      description = "Custom packages to include in the Yazelix environment";
    };
  };

  config = lib.mkIf cfg.enable {
    # NOTE: We don't manage the yazelix files directly to avoid git conflicts.
    # Users should manage their yazelix installation separately (git clone, nix develop, etc.)

    # Ensure state directory exists with proper permissions
    home.activation.yazelixStateDir = lib.hm.dag.entryAfter [ "writeBoundary" ] ''
      run mkdir -p "$HOME/.local/share/yazelix"/{logs,initializers,cache}
      run chmod 755 "$HOME/.local/share/yazelix"
    '';

    # Set up shell integration
    programs.bash.initExtra =
      lib.mkIf (cfg.defaultShell == "bash" || lib.elem "bash" cfg.extraShells)
        ''
          # Yazelix integration
          if [ -f "$HOME/.config/yazelix/shells/bash/yazelix_bash_config.sh" ]; then
            source "$HOME/.config/yazelix/shells/bash/yazelix_bash_config.sh"
          fi
        '';

    programs.fish.shellInit =
      lib.mkIf (cfg.defaultShell == "fish" || lib.elem "fish" cfg.extraShells)
        ''
          # Yazelix integration
          if test -f "$HOME/.config/yazelix/shells/fish/yazelix_fish_config.fish"
            source "$HOME/.config/yazelix/shells/fish/yazelix_fish_config.fish"
          end
        '';

    programs.zsh.initExtra = lib.mkIf (cfg.defaultShell == "zsh" || lib.elem "zsh" cfg.extraShells) ''
      # Yazelix integration
      if [ -f "$HOME/.config/yazelix/shells/zsh/yazelix_zsh_config.zsh" ]; then
        source "$HOME/.config/yazelix/shells/zsh/yazelix_zsh_config.zsh"
      fi
    '';

    # Nushell integration is more complex due to how home-manager handles it
    programs.nushell = lib.mkIf (cfg.defaultShell == "nu") {
      enable = lib.mkDefault true;
      extraConfig = ''
        # Yazelix integration
        if ("~/.config/yazelix/nushell/config/config.nu" | path exists) {
          source "~/.config/yazelix/nushell/config/config.nu"
        }
      '';
    };
  };
}
