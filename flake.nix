# flake.nix
{
  description = "Nix shell for Yazelix";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    helix.url = "github:helix-editor/helix";
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
      helix,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs { inherit system; };

        # Read configuration from yazelix.nix
        homeDir = builtins.getEnv "HOME";
        configFile =
          if homeDir != "" then
            "${homeDir}/.config/yazelix/yazelix.nix"
          else
            throw "HOME environment variable is unset or empty";
        defaultConfigFile =
          if homeDir != "" then
            "${homeDir}/.config/yazelix/yazelix_default.nix"
          else
            throw "HOME environment variable is unset or empty";
        config =
          if builtins.pathExists configFile then
            import configFile { inherit pkgs; }
          else if builtins.pathExists defaultConfigFile then
            import defaultConfigFile { inherit pkgs; }
          else
            {
              recommended_deps = true;
              yazi_extensions = true;
              yazi_media = true;
              helix_mode = "release";
              default_shell = "nu";
              extra_shells = [ ];
              debug_mode = false;
              skip_welcome_screen = false;
              user_packages = [ ];
              editor_config = {
                set_editor = true;
                override_existing = true;
                editor_command = "hx";
              };
            };

        # Variables to control recommended, Yazi extension, Helix source, default shell, and debug mode
        recommendedDepsEnabled = config.recommended_deps or true;
        yaziExtensionsEnabled = config.yazi_extensions or true;
        yaziMediaEnabled = config.yazi_media or true;
        # Helix build mode: "release" or "source"
        helixMode = config.helix_mode or "release";
        useNixpkgsHelix = helixMode == "release";
        useSourceHelix = helixMode == "source";
        # Build from source for source mode
        buildHelixFromSource = useSourceHelix;
        yazelixDefaultShell = config.default_shell or "nu";
        yazelixExtraShells = config.extra_shells or [ ];
        yazelixDebugMode = config.debug_mode or false; # Read debug_mode, default to false
        yazelixSkipWelcomeScreen = config.skip_welcome_screen or false; # Read skip_welcome_screen, default to false
        yazelixPreferredTerminal = config.preferred_terminal or "wezterm"; # Read preferred_terminal, default to wezterm
        yazelixAsciiArtMode = config.ascii_art_mode or "animated"; # Read ascii_art_mode, default to animated

        # Editor configuration
        editorConfig =
          config.editor_config or {
            set_editor = true;
            override_existing = true;
            editor_command = "hx";
          };

        # Helix package selection
        helixFromSource = helix.packages.${system}.default;
        helixPackage = if buildHelixFromSource then helixFromSource else pkgs.helix;

        # Essential dependencies (required for core Yazelix functionality)
        # Note: Only nu and bash are always included; fish/zsh are conditional
        essentialDeps = with pkgs; [
          zellij # Terminal multiplexer for managing panes and layouts
          helixPackage # Helix editor, either built from source or from nixpkgs
          yazi # Fast terminal file manager with sidebar integration
          nushell # Modern shell with structured data support
          fzf # Fuzzy finder for quick file and command navigation
          zoxide # Smart directory jumper for efficient navigation
          starship # Customizable shell prompt with Git status
          bashInteractive # Interactive Bash shell
          macchina # Modern, fast system info fetch tool (Rust, maintained)
        ];

        # Extra shell dependencies (fish/zsh only when needed)
        extraShellDeps =
          with pkgs;
          (
            if (yazelixDefaultShell == "fish" || builtins.elem "fish" yazelixExtraShells) then [ fish ] else [ ]
          )
          ++ (
            if (yazelixDefaultShell == "zsh" || builtins.elem "zsh" yazelixExtraShells) then [ zsh ] else [ ]
          );

        # Recommended dependencies (enhance functionality but not Yazi-specific)
        recommendedDeps = with pkgs; [
          cargo-update # Updates Rust crates for project maintenance
          cargo-binstall # Faster installation of Rust tools
          lazygit # Terminal-based Git TUI for managing repositories
          mise # Tool version manager for consistent environments
          ouch # Compression tool for handling archives
          libnotify # Provides notify-send for desktop notifications (used by Nushell clip command)
          carapace # Command-line completion tool for multiple shells
          serpl # Command-line tool for search and replace
          biome # formats JS, TS, JSON, CSS, and lints js/ts
          markdown-oxide # Personal Knowledge Management System (PKMS) that works with text editors through LSP
          vhs # Create terminal showcases and tutorials with code
        ];

        # Yazi extension dependencies (enhance Yazi functionality, lightweight)
        yaziExtensionsDeps = with pkgs; [
          p7zip # Archive utility for handling compressed files
          jq # JSON processor for parsing and formatting in Yazi plugins
          fd # Fast file finder for efficient search in Yazi
          ripgrep # High-performance search tool for file content
          poppler # PDF rendering for document previews in Yazi
        ];

        # Heavy media packages (WARNING: ~800MB-1.2GB total)
        yaziMediaDeps = with pkgs; [
          ffmpeg # Multimedia processing for media previews (~400-600MB)
          imagemagick # Image processing for thumbnails (~200-300MB)
        ];

        # Combine dependencies based on config
        allDeps =
          essentialDeps
          ++ extraShellDeps
          ++ (if recommendedDepsEnabled then recommendedDeps else [ ])
          ++ (if yaziExtensionsEnabled then yaziExtensionsDeps else [ ])
          ++ (if yaziMediaEnabled then yaziMediaDeps else [ ])
          ++ (config.user_packages or [ ]);

      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = allDeps;

          shellHook = ''
            # Export essential environment variables
            export YAZELIX_DIR="$HOME/.config/yazelix"
            export YAZELIX_DEBUG_MODE="${if yazelixDebugMode then "true" else "false"}"
            export ZELLIJ_DEFAULT_LAYOUT=yazelix
            export YAZELIX_DEFAULT_SHELL="${yazelixDefaultShell}"
            export YAZI_CONFIG_HOME="$YAZELIX_DIR/configs/yazi"
            export YAZELIX_HELIX_MODE="${helixMode}"
            export YAZELIX_PREFERRED_TERMINAL="${yazelixPreferredTerminal}"
            export YAZELIX_ASCII_ART_MODE="${yazelixAsciiArtMode}"

            # Set HELIX_RUNTIME for both modes - both use hx from PATH
            export HELIX_RUNTIME="${helixPackage}/share/helix/runtime"

            # Set EDITOR environment variable based on configuration
            if [ "${if editorConfig.set_editor then "true" else "false"}" = "true" ]; then
              if [ -z "$EDITOR" ] || [ "${
                if editorConfig.override_existing then "true" else "false"
              }" = "true" ]; then
                export EDITOR="${editorConfig.editor_command}"
                echo "📝 Set EDITOR to: ${editorConfig.editor_command}"
              else
                echo "📝 Keeping existing EDITOR='$EDITOR' (override_existing=false)"
              fi
            else
              echo "📝 Skipping EDITOR setup (set_editor=false)"
            fi

            # Disable Nix warning about Git directory
            export NIX_CONFIG="warn-dirty = false"

            # Auto-copy config file if it doesn't exist
            if [ ! -f "$YAZELIX_DIR/yazelix.nix" ] && [ -f "$YAZELIX_DIR/yazelix_default.nix" ]; then
              cp "$YAZELIX_DIR/yazelix_default.nix" "$YAZELIX_DIR/yazelix.nix"
              echo "Created yazelix.nix from template. Customize it for your needs!"
            fi

            # Run main environment setup script
            nu "$YAZELIX_DIR/nushell/scripts/setup/environment.nu" \
              "$YAZELIX_DIR" \
              "${if recommendedDepsEnabled then "true" else "false"}" \
              "${if buildHelixFromSource then "true" else "false"}" \
              "${yazelixDefaultShell}" \
              "${if yazelixDebugMode then "true" else "false"}" \
              "${
                if yazelixExtraShells == [ ] then "NONE" else builtins.concatStringsSep "," yazelixExtraShells
              }" \
              "${if yazelixSkipWelcomeScreen then "true" else "false"}" \
              "${helixMode}" \
              "${yazelixAsciiArtMode}" \
              "${if config.show_macchina_on_welcome or false then "true" else "false"}"
          '';
        };
      }
    );
}
