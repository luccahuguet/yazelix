# flake.nix
{
  description = "Nix shell for Yazelix";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    helix.url = "github:helix-editor/helix";
    patchy.url = "github:nik-rev/patchy/v1.3.0";
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
      helix,
      patchy,
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
              helix_mode = "default";
              patchy_helix_config = {
                pull_requests = [
                  "12309"
                  "8908"
                  "13197"
                  "11700"
                  "11497"
                  "13133"
                ];
                patches = [ ];
                pin_commits = true;
              };
              default_shell = "nu";
              extra_shells = [ ];
              debug_mode = false;
              skip_welcome_screen = false;
              user_packages = [ ];
            };

        # Variables to control optional, Yazi extension, Helix source, patchy, default shell, and debug mode
        includeOptionalDeps = config.include_optional_deps or true;
        includeYaziExtensions = config.include_yazi_extensions or true;
        includeYaziMedia = config.include_yazi_media or true;
        # Helix build mode: "default", "source", "patchy", or "steel"
        helixMode = config.helix_mode or "default";
        useNixpkgsHelix = helixMode == "default";
        useSourceHelix = helixMode == "source";
        usePatchyHelix = helixMode == "patchy";
        useSteelHelix = helixMode == "steel";
        patchyHelixConfig =
          config.patchy_helix_config or {
            pull_requests = [
              "12309"
              "8908"
              "13197"
              "11700"
              "11497"
              "13133"
            ];
            patches = [ ];
            pin_commits = true;
          };
        # Build from source for all non-default modes
        buildHelixFromSource = useSourceHelix || usePatchyHelix || useSteelHelix;
        yazelixDefaultShell = config.default_shell or "nu";
        yazelixExtraShells = config.extra_shells or [ ];
        yazelixDebugMode = config.debug_mode or false; # Read debug_mode, default to false
        yazelixSkipWelcomeScreen = config.skip_welcome_screen or false; # Read skip_welcome_screen, default to false

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

        # Optional dependencies (enhance functionality but not Yazi-specific)
        optionalDeps =
          with pkgs;
          [
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
          ]
          ++ (if usePatchyHelix then [ patchy.packages.${system}.default ] else [ ]);

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

        # Steel plugin system dependencies (only for steel mode)
        steelDeps = with pkgs; [
          steel # Steel scheme interpreter for helix scripting
          # TODO: Add when available in nixpkgs:
          # steel-language-server # Steel LSP server
          # forge # Steel package manager
        ];

        # Combine dependencies based on config
        allDeps =
          essentialDeps
          ++ extraShellDeps
          ++ (if includeOptionalDeps then optionalDeps else [ ])
          ++ (if includeYaziExtensions then yaziExtensionsDeps else [ ])
          ++ (if includeYaziMedia then yaziMediaDeps else [ ])
          ++ (if useSteelHelix then steelDeps else [ ])
          ++ (config.user_packages or [ ]);

        # Helper variables for argument handling
        patchyPRsArg =
          let
            prs = patchyHelixConfig.pull_requests or [ ];
          in
          if prs == [ ] then "NONE" else builtins.concatStringsSep "," prs;

        patchyPatchesArg =
          let
            patches = patchyHelixConfig.patches or [ ];
          in
          if patches == [ ] then "NONE" else builtins.concatStringsSep "," patches;

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
            export YAZELIX_HELIX_MODE="${helixMode}"


            # Set helix path based on mode
            if [ "${helixMode}" = "source" ] || [ "${helixMode}" = "patchy" ] || [ "${helixMode}" = "steel" ]; then
              if [ -f "$YAZELIX_DIR/helix_patchy/target/release/hx" ]; then
                export YAZELIX_PATCHY_HX="$YAZELIX_DIR/helix_patchy/target/release/hx"
                export EDITOR="$YAZELIX_PATCHY_HX"
              fi
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
              "${if includeOptionalDeps then "true" else "false"}" \
              "${if buildHelixFromSource then "true" else "false"}" \
              "${yazelixDefaultShell}" \
              "${if yazelixDebugMode then "true" else "false"}" \
              "${
                if yazelixExtraShells == [ ] then "NONE" else builtins.concatStringsSep "," yazelixExtraShells
              }" \
              "${if yazelixSkipWelcomeScreen then "true" else "false"}" \
              "${helixMode}" \
              "${patchyPRsArg}" \
              "${patchyPatchesArg}" \
              "${if (patchyHelixConfig.pin_commits or true) then "true" else "false"}"
          '';
        };
      }
    );
}
