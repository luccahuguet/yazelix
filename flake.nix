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
              include_optional_deps = true;
              include_yazi_extensions = true;
              include_yazi_media = true;
              build_helix_from_source = true;
              default_shell = "nu";
              debug_mode = false;
              user_packages = [ ];
            };

        # Variables to control optional, Yazi extension, Helix source, default shell, and debug mode
        includeOptionalDeps = config.include_optional_deps or true;
        includeYaziExtensions = config.include_yazi_extensions or true;
        includeYaziMedia = config.include_yazi_media or true;
        buildHelixFromSource = config.build_helix_from_source or true;
        yazelixDefaultShell = config.default_shell or "nu";
        yazelixDebugMode = config.debug_mode or false; # Read debug_mode, default to false

        # Helix package selection
        helixFromSource = helix.packages.${system}.default;
        helixPackage = if buildHelixFromSource then helixFromSource else pkgs.helix;

        # Essential dependencies (required for core Yazelix functionality)
        essentialDeps = with pkgs; [
          zellij # Terminal multiplexer for managing panes and layouts
          helixPackage # Helix editor, either built from source or from nixpkgs
          yazi # Fast terminal file manager with sidebar integration
          nushell # Modern shell with structured data support
          fish # Fish shell for users who prefer it
          fzf # Fuzzy finder for quick file and command navigation
          zoxide # Smart directory jumper for efficient navigation
          starship # Customizable shell prompt with Git status
          bashInteractive # Interactive Bash shell
        ];

        # Optional dependencies (enhance functionality but not Yazi-specific)
        optionalDeps = with pkgs; [
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
          ++ (if includeOptionalDeps then optionalDeps else [ ])
          ++ (if includeYaziExtensions then yaziExtensionsDeps else [ ])
          ++ (if includeYaziMedia then yaziMediaDeps else [ ])
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
            export YAZI_CONFIG_HOME="$YAZELIX_DIR/yazi"

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
              "${if includeOptionalDeps then "true" else "false"}" \
              "${if buildHelixFromSource then "true" else "false"}" \
              "${yazelixDefaultShell}" \
              "${if yazelixDebugMode then "true" else "false"}"
          '';
        };
      }
    );
}
