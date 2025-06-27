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
            YAZELIX_DEBUG_MODE_SHELL="${if yazelixDebugMode then "true" else "false"}"
            YAZELIX_LOG_DIR="$HOME/.config/yazelix/logs"
            mkdir -p "$YAZELIX_LOG_DIR" # Ensure log directory exists

            # Auto-trim old shellhook logs (keep only the 10 most recent)
            find "$YAZELIX_LOG_DIR" -name "shellhook_*.log" -type f -print0 | \
              xargs -0 ls -t | tail -n +11 | xargs -r rm -f 2>/dev/null || true

            YAZELIX_SHELLHOOK_LOG_FILE="$YAZELIX_LOG_DIR/shellhook_$(date +%Y%m%d_%H%M%S).log"

            # Logging functions
            _log_to_file_and_stdout() {
              local message="$1"
              echo "$message" >> "$YAZELIX_SHELLHOOK_LOG_FILE"
              # Conditionally echo to stderr for interactive `nix develop` sessions or for WARN/key INFO
              # Avoid echoing "already sourced" messages to stdout unless in debug mode.
              if [ "$YAZELIX_DEBUG_MODE_SHELL" = "true" ] || [[ "$message" == *"[WARN]"* ]] || \
                 ([[ "$message" == *"[INFO]"* ]] && ! [[ "$message" == *"[INFO] Yazelix Bash config (with standard comment) already sourced"* || \
                                                         "$message" == *"[INFO] Yazelix Nushell config (with standard comment) already sourced"* ]]); then
                echo "$message" >&2
              fi
            }
            log_msg() { _log_to_file_and_stdout "[$(date +'%T')] $1"; }
            debug_msg() { if [ "$YAZELIX_DEBUG_MODE_SHELL" = "true" ]; then _log_to_file_and_stdout "[$(date +'%T') DEBUG] $1"; fi; }
            warn_msg() { _log_to_file_and_stdout "[$(date +'%T') WARN] $1"; }
            info_msg() { _log_to_file_and_stdout "[$(date +'%T') INFO] $1"; } # Key info messages

            log_msg "--- Yazelix Flake shellHook Started (Logging to: $YAZELIX_SHELLHOOK_LOG_FILE) ---"

            # Auto-copy config file if it doesn't exist
            YAZELIX_CONFIG_FILE="$HOME/.config/yazelix/yazelix.nix"
            YAZELIX_DEFAULT_CONFIG="$HOME/.config/yazelix/yazelix_default.nix"
            if [ ! -f "$YAZELIX_CONFIG_FILE" ] && [ -f "$YAZELIX_DEFAULT_CONFIG" ]; then
              debug_msg "Creating user config file from template: $YAZELIX_CONFIG_FILE"
              cp "$YAZELIX_DEFAULT_CONFIG" "$YAZELIX_CONFIG_FILE" || warn_msg "Failed to copy config template"
              info_msg "Created yazelix.nix from template. Customize it for your needs!"
            fi

            debug_msg "Using HOME=$HOME"
            # Nix variable `configFile` is used here for logging its value from Nix context
            debug_msg "Nix 'configFile' variable value: ${configFile}"
            debug_msg "include_optional_deps: ${if includeOptionalDeps then "true" else "false"}"
            debug_msg "include_yazi_extensions: ${if includeYaziExtensions then "true" else "false"}"
            debug_msg "include_yazi_media: ${if includeYaziMedia then "true" else "false"}"
            debug_msg "build_helix_from_source: ${if buildHelixFromSource then "true" else "false"}"
            debug_msg "default_shell: ${yazelixDefaultShell}"
            debug_msg "debug_mode active: $YAZELIX_DEBUG_MODE_SHELL"
            debug_msg ""

            # --- Shell Initializers (Universal Generator) ---
            debug_msg "Generating shell initializers for all supported shells..."
            nu "$HOME/.config/yazelix/nushell/scripts/generate-shell-initializers.nu" "$HOME/.config/yazelix" ${
              if includeOptionalDeps then "true" else "false"
            } 2>>"$YAZELIX_SHELLHOOK_LOG_FILE" || warn_msg "Failed to generate shell initializers"
            debug_msg "Shell initializers generation complete."
            debug_msg ""

            # --- Ensure ~/.bashrc sources the PERSISTED Yazelix Bash config ---
            debug_msg "Setting up Bash configuration sourcing..."
            PERSISTED_YAZELIX_BASH_CONFIG_FILE="$HOME/.config/yazelix/bash/yazelix_bash_config.sh"
            BASHRC_FILE="$HOME/.bashrc"
            YAZELIX_BASH_COMMENT_LINE="# Source Yazelix Bash configuration (added by Yazelix)"
            YAZELIX_BASH_SOURCE_LINE="source \"$PERSISTED_YAZELIX_BASH_CONFIG_FILE\""

            if [ ! -f "$PERSISTED_YAZELIX_BASH_CONFIG_FILE" ]; then
              warn_msg "Persisted Yazelix Bash config not found at $PERSISTED_YAZELIX_BASH_CONFIG_FILE. Please ensure it exists in your Yazelix project."
            else
              touch "$BASHRC_FILE" || warn_msg "Failed to touch $BASHRC_FILE"
              if ! grep -qF -- "$YAZELIX_BASH_COMMENT_LINE" "$BASHRC_FILE"; then
                debug_msg "Yazelix Bash sourcing not found in $BASHRC_FILE. Adding it."
                {
                  echo "" # Add a newline for separation
                  echo "$YAZELIX_BASH_COMMENT_LINE"
                  echo "$YAZELIX_BASH_SOURCE_LINE"
                } >> "$BASHRC_FILE"
                info_msg "Added Yazelix Bash config source to $BASHRC_FILE. You might need to source it manually: source $BASHRC_FILE"
              else
                info_msg "Yazelix Bash config (with standard comment) already sourced in $BASHRC_FILE."
              fi
            fi
            debug_msg "Bash configuration sourcing setup complete."
            debug_msg ""

            # --- Yazi Setup ---
            debug_msg "Setting up Yazi..."
            export YAZI_CONFIG_HOME="$HOME/.config/yazelix/yazi"
            info_msg "YAZI_CONFIG_HOME set to: $YAZI_CONFIG_HOME"
            debug_msg "Yazi setup complete."
            debug_msg ""

            # --- Nushell Setup ---
            debug_msg "Setting up Nushell configuration sourcing..."
            NUSHELL_USER_CONFIG_FILE="$HOME/.config/nushell/config.nu"
            YAZELIX_NUSHELL_CONFIG_TO_SOURCE="$HOME/.config/yazelix/nushell/config/config.nu"
            YAZELIX_NUSHELL_COMMENT_LINE="# Source Yazelix Nushell configuration (added by Yazelix)"
            YAZELIX_NUSHELL_SOURCE_LINE="source \"$YAZELIX_NUSHELL_CONFIG_TO_SOURCE\""

            mkdir -p "$(dirname "$NUSHELL_USER_CONFIG_FILE")" || warn_msg "Could not create Nushell config directory: $(dirname "$NUSHELL_USER_CONFIG_FILE")"
            if [ ! -f "$NUSHELL_USER_CONFIG_FILE" ]; then
              debug_msg "$NUSHELL_USER_CONFIG_FILE not found. Creating it."
              echo "# Nushell user configuration (created by Yazelix setup)" > "$NUSHELL_USER_CONFIG_FILE"
              info_msg "Created new $NUSHELL_USER_CONFIG_FILE"
            fi
            if ! grep -qF -- "$YAZELIX_NUSHELL_COMMENT_LINE" "$NUSHELL_USER_CONFIG_FILE"; then
              debug_msg "Yazelix Nushell sourcing not found in $NUSHELL_USER_CONFIG_FILE. Adding it."
              {
                echo "" # Add a newline for separation
                echo "$YAZELIX_NUSHELL_COMMENT_LINE"
                echo "$YAZELIX_NUSHELL_SOURCE_LINE"
              } >> "$NUSHELL_USER_CONFIG_FILE"
              info_msg "Added Yazelix Nushell config source to $NUSHELL_USER_CONFIG_FILE"
            else
              info_msg "Yazelix Nushell config (with standard comment) already sourced in $NUSHELL_USER_CONFIG_FILE."
            fi
            debug_msg "Nushell configuration sourcing setup complete."
            debug_msg ""

            # --- Fish Setup ---
            debug_msg "Setting up Fish configuration sourcing..."
            FISH_USER_CONFIG_FILE="$HOME/.config/fish/config.fish"
            YAZELIX_FISH_CONFIG_TO_SOURCE="$HOME/.config/yazelix/fish/yazelix_fish_config.fish"
            YAZELIX_FISH_COMMENT_LINE="# Source Yazelix Fish configuration (added by Yazelix)"
            YAZELIX_FISH_SOURCE_LINE="source \"$YAZELIX_FISH_CONFIG_TO_SOURCE\""

            if [ -f "$YAZELIX_FISH_CONFIG_TO_SOURCE" ]; then
              mkdir -p "$(dirname "$FISH_USER_CONFIG_FILE")" || warn_msg "Could not create Fish config directory"
              touch "$FISH_USER_CONFIG_FILE" || warn_msg "Failed to touch $FISH_USER_CONFIG_FILE"
              if ! grep -qF -- "$YAZELIX_FISH_COMMENT_LINE" "$FISH_USER_CONFIG_FILE"; then
                debug_msg "Adding Yazelix Fish sourcing to $FISH_USER_CONFIG_FILE"
                {
                  echo ""
                  echo "$YAZELIX_FISH_COMMENT_LINE"
                  echo "$YAZELIX_FISH_SOURCE_LINE"
                } >> "$FISH_USER_CONFIG_FILE"
                info_msg "Added Yazelix Fish config source to $FISH_USER_CONFIG_FILE"
              else
                info_msg "Yazelix Fish config already sourced in $FISH_USER_CONFIG_FILE"
              fi
            else
              debug_msg "Fish config not found, skipping Fish setup"
            fi
            debug_msg "Fish configuration sourcing setup complete."
            debug_msg ""

            # --- Helix Setup ---
            debug_msg "Setting up Helix..."
            # Set EDITOR to the available Helix binary (helix first, then hx as fallback)
            if command -v helix >/dev/null 2>&1; then
              export EDITOR=helix
              debug_msg "Found 'helix' binary, setting EDITOR=helix"
            elif command -v hx >/dev/null 2>&1; then
              export EDITOR=hx
              debug_msg "Found 'hx' binary, setting EDITOR=hx"
            else
              export EDITOR=hx  # Default fallback
              warn_msg "Neither 'helix' nor 'hx' binary found, defaulting to EDITOR=hx"
            fi
            info_msg "EDITOR set to: $EDITOR"
            debug_msg "Helix setup complete."
            debug_msg ""

            # --- Set executable permissions ---
            debug_msg "Setting executable permissions for shell scripts..."
            chmod +x "$HOME/.config/yazelix/bash/launch-yazelix.sh" || warn_msg "Could not set executable permissions for launch-yazelix.sh"
            chmod +x "$HOME/.config/yazelix/bash/start-yazelix.sh" || warn_msg "Could not set executable permissions for start-yazelix.sh"
            debug_msg "Executable permissions setup complete."
            debug_msg ""

            # --- Display configuration status ---
            # This shell variable holds the path determined by Nix, used for display and shell checks
            CONFIG_FILE_PATH_FOR_SHELL="${configFile}"
            log_msg "--- Yazelix Configuration Status ---"
            log_msg "  Config file path: $CONFIG_FILE_PATH_FOR_SHELL"
            if [ -f "$CONFIG_FILE_PATH_FOR_SHELL" ]; then # This check uses the shell variable
              log_msg "  Config file found at $CONFIG_FILE_PATH_FOR_SHELL"
            else
              log_msg "  Config file not found at $CONFIG_FILE_PATH_FOR_SHELL, using defaults"
            fi
            log_msg "  include_optional_deps: ${if includeOptionalDeps then "true" else "false"}"
            log_msg "  include_yazi_extensions: ${if includeYaziExtensions then "true" else "false"}"
            log_msg "  include_yazi_media: ${if includeYaziMedia then "true" else "false"}"
            log_msg "  build_helix_from_source: ${if buildHelixFromSource then "true" else "false"}"
            log_msg "  default_shell: ${yazelixDefaultShell}"
            log_msg "  debug_mode: ${if yazelixDebugMode then "true" else "false"}"
            log_msg "------------------------------------"
            debug_msg ""

            # --- Final Configuration ---
            debug_msg "Setting final environment variables..."
            export ZELLIJ_DEFAULT_LAYOUT=yazelix
            export YAZELIX_DEFAULT_SHELL="${yazelixDefaultShell}" # Export for start-yazelix.sh
            info_msg "ZELLIJ_DEFAULT_LAYOUT set to: $ZELLIJ_DEFAULT_LAYOUT"
            info_msg "YAZELIX_DEFAULT_SHELL set to: $YAZELIX_DEFAULT_SHELL"
            debug_msg ""

            log_msg "--- Yazelix Flake shellHook Finished ---"
            # This final message will always go to stdout/stderr for interactive shells
            echo "Yazelix environment ready! Use 'z' for smart directory navigation. ShellHook logs in $YAZELIX_SHELLHOOK_LOG_FILE"
          '';
        };
      }
    );
}
