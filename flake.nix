# flake.nix
{
  description = "Nix shell for Yazelix";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    helix.url = "github:helix-editor/helix";
    nixgl.url = "github:guibou/nixGL";
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
      helix,
      nixgl,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        mesaCompatibilityOverlay = final: prev:
          if prev ? mesa then { mesa = prev.mesa // { drivers = prev.mesa; }; } else { };

        pkgs = import nixpkgs {
          inherit system;
          overlays = [ mesaCompatibilityOverlay ];
        };

        # Platform detection - nixGL is Linux-only (macOS uses native Metal API)
        isLinux = pkgs.stdenv.isLinux;

        # Apply nixGL overlay only on Linux (for GPU acceleration on non-NixOS systems)
        pkgsWithNixGL = if isLinux then
          import nixpkgs {
            inherit system;
            overlays = [
              mesaCompatibilityOverlay
              nixgl.overlay
            ];
          }
        else
          pkgs;

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
              enable_sidebar = false;
              packs = [ ];
              user_packages = [ ];
              editor_command = "hx";
              helix_runtime_path = null;
            };

        # Variables to control recommended, Yazi extension, Helix source, default shell, and debug mode
        recommendedDepsEnabled = config.recommended_deps or true;
        atuinEnabled = config.enable_atuin or false;
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
        yazelixExtraTerminals = config.extra_terminals or [ ];
        yazelixDebugMode = config.debug_mode or false; # Read debug_mode, default to false
        yazelixSkipWelcomeScreen = config.skip_welcome_screen or false; # Read skip_welcome_screen, default to false
        yazelixPreferredTerminal = config.preferred_terminal or "ghostty"; # Default to ghostty (Homebrew on macOS, Nix on Linux)
        yazelixTerminalConfigMode = config.terminal_config_mode or "yazelix"; # Default to Yazelix-managed configs
        yazelixAsciiArtMode = config.ascii_art_mode or "static"; # Read ascii_art_mode, default to static

        # Editor configuration
        # When editor_command is null, use yazelix's Helix to ensure binary/runtime compatibility
        # When set to a string, use that command (user must ensure runtime compatibility)
        editorCommand = if (config.editor_command or null) == null 
                       then "${helixPackage}/bin/hx"
                       else config.editor_command;
        
        # Helix runtime path configuration
        helixRuntimePath = config.helix_runtime_path or null;

        # Sidebar configuration
        yazelixEnableSidebar = config.enable_sidebar or true;
        yazelixLayoutName =
          if (builtins.hasAttr "zellij_layout_override" config && config.zellij_layout_override != "")
          then config.zellij_layout_override
          else if yazelixEnableSidebar then "yzx_side" else "yzx_no_side";


        # Helix package selection
        helixFromSource = helix.packages.${system}.default;
        helixPackage = if buildHelixFromSource then helixFromSource else pkgs.helix;

        # Simplified approach - no vendor-specific nixGL for now

        # Simplified terminal wrappers without nixGL complexity

        # Ghostty wrapper - Linux-only (nixpkgs Ghostty is Linux-only)
        ghosttyWrapper = if isLinux then
          pkgs.writeShellScriptBin "yazelix-ghostty" (
            if isLinux then ''
              MODE="''${YAZELIX_TERMINAL_CONFIG_MODE:-${yazelixTerminalConfigMode}}"
              MODE="''${MODE:-auto}"
              USER_CONF="$HOME/.config/ghostty/config"
              YZ_CONF="$HOME/.local/share/yazelix/configs/terminal_emulators/ghostty/config"
              CONF="$YZ_CONF"
              if [ "$MODE" = "user" ] || [ "$MODE" = "auto" ]; then
                if [ -f "$USER_CONF" ]; then CONF="$USER_CONF"; fi
              fi
              exec ${pkgsWithNixGL.nixgl.nixGLIntel}/bin/nixGLIntel ${pkgs.ghostty}/bin/ghostty \
                --config-file="$CONF" \
                --class="com.yazelix.Yazelix" \
                --x11-instance-name="yazelix" \
                --title="Yazelix - Ghostty" "$@"
            '' else ''
              MODE="''${YAZELIX_TERMINAL_CONFIG_MODE:-${yazelixTerminalConfigMode}}"
              MODE="''${MODE:-auto}"
              USER_CONF="$HOME/.config/ghostty/config"
              YZ_CONF="$HOME/.local/share/yazelix/configs/terminal_emulators/ghostty/config"
              CONF="$YZ_CONF"
              if [ "$MODE" = "user" ] || [ "$MODE" = "auto" ]; then
                if [ -f "$USER_CONF" ]; then CONF="$USER_CONF"; fi
              fi
              exec ${pkgs.ghostty}/bin/ghostty \
                --config-file="$CONF" \
                --class="com.yazelix.Yazelix" \
                --title="Yazelix - Ghostty" "$@"
            ''
          )
        else null;

        # Kitty wrapper - with nixGL on Linux, native on macOS
        kittyWrapper = if (yazelixPreferredTerminal == "kitty" || builtins.elem "kitty" yazelixExtraTerminals) then
          pkgs.writeShellScriptBin "yazelix-kitty" (
            if isLinux then ''
              MODE="''${YAZELIX_TERMINAL_CONFIG_MODE:-${yazelixTerminalConfigMode}}"
              MODE="''${MODE:-auto}"
              USER_CONF="$HOME/.config/kitty/kitty.conf"
              YZ_CONF="$HOME/.local/share/yazelix/configs/terminal_emulators/kitty/kitty.conf"
              CONF="$YZ_CONF"
              if [ "$MODE" = "user" ] || [ "$MODE" = "auto" ]; then
                if [ -f "$USER_CONF" ]; then CONF="$USER_CONF"; fi
              fi
              exec ${pkgsWithNixGL.nixgl.nixGLIntel}/bin/nixGLIntel ${pkgs.kitty}/bin/kitty \
                --config="$CONF" \
                --class="com.yazelix.Yazelix" \
                --title="Yazelix - Kitty" "$@"
            '' else ''
              MODE="''${YAZELIX_TERMINAL_CONFIG_MODE:-${yazelixTerminalConfigMode}}"
              MODE="''${MODE:-auto}"
              USER_CONF="$HOME/.config/kitty/kitty.conf"
              YZ_CONF="$HOME/.local/share/yazelix/configs/terminal_emulators/kitty/kitty.conf"
              CONF="$YZ_CONF"
              if [ "$MODE" = "user" ] || [ "$MODE" = "auto" ]; then
                if [ -f "$USER_CONF" ]; then CONF="$USER_CONF"; fi
              fi
              exec ${pkgs.kitty}/bin/kitty \
                --config="$CONF" \
                --class="com.yazelix.Yazelix" \
                --title="Yazelix - Kitty" "$@"
            ''
          )
        else null;

        # WezTerm wrapper - with nixGL on Linux, native on macOS
        weztermWrapper = if (yazelixPreferredTerminal == "wezterm" || builtins.elem "wezterm" yazelixExtraTerminals) then
          pkgs.writeShellScriptBin "yazelix-wezterm" (
            if isLinux then ''
              MODE="''${YAZELIX_TERMINAL_CONFIG_MODE:-${yazelixTerminalConfigMode}}"
              MODE="''${MODE:-auto}"
              USER_CONF_MAIN="$HOME/.wezterm.lua"
              USER_CONF_ALT="$HOME/.config/wezterm/wezterm.lua"
              if [ -f "$USER_CONF_MAIN" ]; then USER_CONF="$USER_CONF_MAIN"; else USER_CONF="$USER_CONF_ALT"; fi
              YZ_CONF="$HOME/.local/share/yazelix/configs/terminal_emulators/wezterm/.wezterm.lua"
              CONF="$YZ_CONF"
              if [ "$MODE" = "user" ] || [ "$MODE" = "auto" ]; then
                if [ -f "$USER_CONF" ]; then CONF="$USER_CONF"; fi
              fi
              exec ${pkgsWithNixGL.nixgl.nixGLIntel}/bin/nixGLIntel ${pkgs.wezterm}/bin/wezterm \
                --config-file="$CONF" \
                --config 'window_decorations="NONE"' \
                --config enable_tab_bar=false \
                start --class=com.yazelix.Yazelix "$@"
            '' else ''
              MODE="''${YAZELIX_TERMINAL_CONFIG_MODE:-${yazelixTerminalConfigMode}}"
              MODE="''${MODE:-auto}"
              USER_CONF_MAIN="$HOME/.wezterm.lua"
              USER_CONF_ALT="$HOME/.config/wezterm/wezterm.lua"
              if [ -f "$USER_CONF_MAIN" ]; then USER_CONF="$USER_CONF_MAIN"; else USER_CONF="$USER_CONF_ALT"; fi
              YZ_CONF="$HOME/.local/share/yazelix/configs/terminal_emulators/wezterm/.wezterm.lua"
              CONF="$YZ_CONF"
              if [ "$MODE" = "user" ] || [ "$MODE" = "auto" ]; then
                if [ -f "$USER_CONF" ]; then CONF="$USER_CONF"; fi
              fi
              exec ${pkgs.wezterm}/bin/wezterm \
                --config-file="$CONF" \
                --config 'window_decorations="NONE"' \
                --config enable_tab_bar=false \
                start --class=com.yazelix.Yazelix "$@"
            ''
          )
        else null;

        # Alacritty wrapper - with nixGL on Linux, native on macOS
        alacrittyWrapper = if (yazelixPreferredTerminal == "alacritty" || builtins.elem "alacritty" yazelixExtraTerminals) then
          pkgs.writeShellScriptBin "yazelix-alacritty" (
            if isLinux then ''
              MODE="''${YAZELIX_TERMINAL_CONFIG_MODE:-${yazelixTerminalConfigMode}}"
              MODE="''${MODE:-auto}"
              USER_CONF="$HOME/.config/alacritty/alacritty.toml"
              YZ_CONF="$HOME/.local/share/yazelix/configs/terminal_emulators/alacritty/alacritty.toml"
              CONF="$YZ_CONF"
              if [ "$MODE" = "user" ] || [ "$MODE" = "auto" ]; then
                if [ -f "$USER_CONF" ]; then CONF="$USER_CONF"; fi
              fi
              exec ${pkgsWithNixGL.nixgl.nixGLIntel}/bin/nixGLIntel ${pkgs.alacritty}/bin/alacritty \
                --config-file="$CONF" \
                --class="com.yazelix.Yazelix" \
                --title="Yazelix - Alacritty" "$@"
            '' else ''
              MODE="''${YAZELIX_TERMINAL_CONFIG_MODE:-${yazelixTerminalConfigMode}}"
              MODE="''${MODE:-auto}"
              USER_CONF="$HOME/.config/alacritty/alacritty.toml"
              YZ_CONF="$HOME/.local/share/yazelix/configs/terminal_emulators/alacritty/alacritty.toml"
              CONF="$YZ_CONF"
              if [ "$MODE" = "user" ] || [ "$MODE" = "auto" ]; then
                if [ -f "$USER_CONF" ]; then CONF="$USER_CONF"; fi
              fi
              exec ${pkgs.alacritty}/bin/alacritty \
                --config-file="$CONF" \
                --class="com.yazelix.Yazelix" \
                --title="Yazelix - Alacritty" "$@"
            ''
          )
        else null;

        # Foot wrapper - Linux-only (Wayland terminal, not available on macOS)
        footWrapper = if (yazelixPreferredTerminal == "foot" || builtins.elem "foot" yazelixExtraTerminals) && isLinux then
          pkgs.writeShellScriptBin "yazelix-foot" (
            if isLinux then ''
              MODE="''${YAZELIX_TERMINAL_CONFIG_MODE:-${yazelixTerminalConfigMode}}"
              MODE="''${MODE:-auto}"
              USER_CONF="$HOME/.config/foot/foot.ini"
              YZ_CONF="$HOME/.local/share/yazelix/configs/terminal_emulators/foot/foot.ini"
              CONF="$YZ_CONF"
              if [ "$MODE" = "user" ] || [ "$MODE" = "auto" ]; then
                if [ -f "$USER_CONF" ]; then CONF="$USER_CONF"; fi
              fi
              exec ${pkgsWithNixGL.nixgl.nixGLIntel}/bin/nixGLIntel ${pkgs.foot}/bin/foot \
                --config="$CONF" \
                --app-id="com.yazelix.Yazelix" "$@"
            '' else ''
              MODE="''${YAZELIX_TERMINAL_CONFIG_MODE:-${yazelixTerminalConfigMode}}"
              MODE="''${MODE:-auto}"
              USER_CONF="$HOME/.config/foot/foot.ini"
              YZ_CONF="$HOME/.local/share/yazelix/configs/terminal_emulators/foot/foot.ini"
              CONF="$YZ_CONF"
              if [ "$MODE" = "user" ] || [ "$MODE" = "auto" ]; then
                if [ -f "$USER_CONF" ]; then CONF="$USER_CONF"; fi
              fi
              exec ${pkgs.foot}/bin/foot \
                --config="$CONF" \
                --app-id="com.yazelix.Yazelix" "$@"
            ''
          )
        else null;

        # Desktop launcher script for yazelix - force rebuild with timestamp
        yazelixDesktopLauncher = if true then
          pkgs.writeShellScriptBin "yazelix-desktop-launcher" ''
            # Updated launcher - should use yazelix environment
            cd ~/.config/yazelix
            export YAZELIX_DIR="$HOME/.config/yazelix"
            exec nix develop --impure --command nu "$YAZELIX_DIR/nushell/scripts/core/launch_yazelix.nu"
          ''
        else null;

        # Desktop entry for yazelix with logo
        yazelixDesktopEntry = if true then
          pkgs.makeDesktopItem {
            name = "com.yazelix.Yazelix";
            exec = "${yazelixDesktopLauncher}/bin/yazelix-desktop-launcher";
            icon = "yazelix"; # Generic name, we'll copy logo separately
            desktopName = "Yazelix";
            comment = "Yazi + Zellij + Helix integrated terminal environment";
            categories = [ "Development" ];
            startupWMClass = "com.yazelix.Yazelix";
          }
        else null;

        # Essential dependencies (required for core Yazelix functionality)
        # Note: Only nu and bash are always included; fish/zsh are conditional
        essentialDeps = with pkgs; [
          zellij # Terminal multiplexer for managing panes and layouts
          helixPackage # Helix editor, either built from source or from nixpkgs
          yazi # Fast terminal file manager with sidebar integration
          nushell # Modern shell with structured data support (follow nixpkgs)
          fzf # Fuzzy finder for quick file and command navigation
          zoxide # Smart directory jumper for efficient navigation
          starship # Customizable shell prompt with Git status
          bashInteractive # Interactive Bash shell
          macchina # Modern, fast system info fetch tool (Rust, maintained)
          mise # Tool version manager - pre-configured in Yazelix shell initializers
        ] ++ (if isLinux then [
          libnotify # Provides notify-send for desktop notifications (used by Nushell clip command, Linux-only)
        ] else []) ++ (if isLinux then [
          # Desktop integration (Linux-only - .desktop files are FreeDesktop standard)
          yazelixDesktopLauncher # Desktop launcher script
          yazelixDesktopEntry # Desktop entry with logo
        ] else []) ++ (if isLinux then [
          # nixGL for GPU acceleration on non-NixOS Linux systems (not needed on macOS)
          pkgsWithNixGL.nixgl.nixGLIntel # For Intel/Mesa GPU acceleration
        ] else []) ++ (if isLinux then [
          # Ghostty terminal with GPU acceleration support (Linux-only in nixpkgs)
          ghosttyWrapper # Ghostty with nixGL wrapper
          ghostty # Base ghostty package
        ] else []) ++ (if (yazelixPreferredTerminal == "kitty" || builtins.elem "kitty" yazelixExtraTerminals) then [
          # Kitty terminal with GPU acceleration support
          kittyWrapper # Kitty with nixGL wrapper
          kitty # Base kitty package
          nerd-fonts.fira-code # Required fonts for Kitty config
          nerd-fonts.symbols-only # Required symbols for Kitty config
        ] else []) ++ (if (yazelixPreferredTerminal == "wezterm" || builtins.elem "wezterm" yazelixExtraTerminals) then [
          # WezTerm terminal with GPU acceleration support
          weztermWrapper # WezTerm with nixGL wrapper
          wezterm # Base wezterm package
        ] else []) ++ (if (yazelixPreferredTerminal == "alacritty" || builtins.elem "alacritty" yazelixExtraTerminals) then [
          # Alacritty terminal with GPU acceleration support
          alacrittyWrapper # Alacritty with nixGL wrapper
          alacritty # Base alacritty package
          nerd-fonts.fira-code # Preferred Nerd Font (matches README)
          nerd-fonts.symbols-only # Symbols fallback for extra glyphs
        ] else []) ++ (if isLinux && (yazelixPreferredTerminal == "foot" || builtins.elem "foot" yazelixExtraTerminals) then [
          footWrapper
          foot
        ] else []);

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
          lazygit # Terminal-based Git TUI for managing repositories
          atuin # Shell history manager with sync and search capabilities
          carapace # Command-line completion tool for multiple shells
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

        # Pack definitions - technology stacks for easy bulk installation
        packDefinitions = {
          python = with pkgs; [
            ruff # Fast Python linter and code formatter
            uv # Ultra-fast Python package installer and resolver
            ty # Extremely fast Python type checker from Astral
            python3Packages.ipython # Enhanced interactive Python REPL with autocomplete and syntax highlighting
          ];
          js_ts = with pkgs; [
            biome # Formats JS, TS, JSON, CSS, and lints JS/TS
            bun # Fast all-in-one JavaScript runtime, bundler, test runner, and package manager
          ];
          rust = with pkgs; [
            cargo-update # Updates Rust crates for project maintenance
            cargo-binstall # Faster installation of Rust tools
            cargo-edit # Add, remove, and upgrade dependencies from the command line (cargo add/rm)
            cargo-watch # Auto-recompile on file changes
            cargo-audit # Audit dependencies for security vulnerabilities
            cargo-nextest # Next-generation test runner with better output and parallelism
          ];
          config = with pkgs; [
            taplo # TOML formatter and language server for configuration files
            nixfmt-rfc-style # Official Nix code formatter following RFC style guidelines
            mpls # Markdown Preview Language Server with live browser preview
          ];
          file-management = with pkgs; [
            ouch # Compression tool for handling archives
            erdtree # Modern tree command with file size display
            serpl # Command-line tool for search and replace operations
          ];
          git = with pkgs; [
            onefetch # Git repository summary with statistics and language breakdown
            gh # GitHub CLI for repository management and PR workflows
            delta # Syntax-highlighting pager for git diffs
            gitleaks # Scan git repos for secrets and credentials
            jujutsu # Modern version control system with powerful conflict resolution
            prek # Prettier git commit logs and history viewer
          ];
          nix = with pkgs; [
            nil # Nix language server for IDE features
            nixd # Alternative Nix language server with advanced features
            nixfmt-rfc-style # Official Nix code formatter following RFC style guidelines
          ];
        };

        # Resolve packs to packages
        selectedPacks = config.packs or [];
        packPackages = builtins.concatLists (
          map (packName: 
            if builtins.hasAttr packName packDefinitions 
            then packDefinitions.${packName}
            else throw "Unknown pack '${packName}'. Available packs: ${builtins.concatStringsSep ", " (builtins.attrNames packDefinitions)}"
          ) selectedPacks
        );

        # Combine dependencies based on config
        allDeps =
          essentialDeps
          ++ extraShellDeps
          ++ (if recommendedDepsEnabled then recommendedDeps else [ ])
          ++ (if yaziExtensionsEnabled then yaziExtensionsDeps else [ ])
          ++ (if yaziMediaEnabled then yaziMediaDeps else [ ])
          ++ packPackages
          ++ (config.user_packages or [ ]);

      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = allDeps;

          shellHook = ''
            # Export essential environment variables
            export YAZELIX_DIR="$HOME/.config/yazelix"
            export IN_YAZELIX_SHELL="true"
            export YAZELIX_DEBUG_MODE="${if yazelixDebugMode then "true" else "false"}"
            export ZELLIJ_DEFAULT_LAYOUT="${yazelixLayoutName}"
            export YAZELIX_DEFAULT_SHELL="${yazelixDefaultShell}"
            export YAZELIX_ENABLE_SIDEBAR="${if yazelixEnableSidebar then "true" else "false"}"
            export YAZI_CONFIG_HOME="$HOME/.local/share/yazelix/configs/yazi"
            export YAZELIX_HELIX_MODE="${helixMode}"
            export YAZELIX_PREFERRED_TERMINAL="${yazelixPreferredTerminal}"
            export YAZELIX_TERMINAL_CONFIG_MODE="${yazelixTerminalConfigMode}"
            export YAZELIX_ASCII_ART_MODE="${yazelixAsciiArtMode}"

            # Set HELIX_RUNTIME - use custom path if specified, otherwise use Nix package runtime
            ${if helixRuntimePath != null then 
              ''export HELIX_RUNTIME="${helixRuntimePath}"'' 
            else 
              ''export HELIX_RUNTIME="${helixPackage}/lib/runtime"''
            }

            # Set EDITOR environment variable to configured command
            export EDITOR="${editorCommand}"
            if [ "$YAZELIX_ENV_ONLY" != "true" ]; then
              echo "📝 Set EDITOR to: ${editorCommand}"
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
              "${if atuinEnabled then "true" else "false"}" \
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
