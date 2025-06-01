# flake.nix
{
  description = "Nix shell for Yazelix";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    helix.url = "github:helix-editor/helix";
  };

  outputs = { self, nixpkgs, flake-utils, helix, ... }:
    flake-utils.lib.eachDefaultSystem (system: let
      pkgs = import nixpkgs { inherit system; };

      # Read configuration from yazelix.toml
      homeDir = builtins.getEnv "HOME";
      configFile = if homeDir != "" then "${homeDir}/.config/yazelix/yazelix.toml"
                   else throw "HOME environment variable is unset or empty";
      config = if builtins.pathExists configFile
               then builtins.fromTOML (builtins.readFile configFile)
               else {
                 include_optional_deps = true;
                 include_yazi_extensions = true;
                 build_helix_from_source = true;
                 default_shell = "nu";
                 debug_mode = false; # Default for debug_mode
               };

      # Variables to control optional, Yazi extension, Helix source, default shell, and debug mode
      includeOptionalDeps = config.include_optional_deps or true;
      includeYaziExtensions = config.include_yazi_extensions or true;
      buildHelixFromSource = config.build_helix_from_source or true;
      yazelixDefaultShell = config.default_shell or "nu";
      yazelixDebugMode = config.debug_mode or false; # Read debug_mode, default to false

      # Helix package selection
      helixFromSource = helix.packages.${system}.default;
      helixPackage = if buildHelixFromSource then helixFromSource else pkgs.helix;

      # Essential dependencies (required for core Yazelix functionality)
      essentialDeps = with pkgs; [
        zellij        # Terminal multiplexer for managing panes and layouts
        helixPackage  # Helix editor, either built from source or from nixpkgs
        yazi          # Fast terminal file manager with sidebar integration
        nushell       # Modern shell with structured data support
        fzf           # Fuzzy finder for quick file and command navigation
        zoxide        # Smart directory jumper for efficient navigation
        starship      # Customizable shell prompt with Git status
        bashInteractive # Interactive Bash shell
      ];

      # Optional dependencies (enhance functionality but not Yazi-specific)
      optionalDeps = with pkgs; [
        cargo-update  # Updates Rust crates for project maintenance
        cargo-binstall # Faster installation of Rust tools
        lazygit       # Terminal-based Git TUI for managing repositories
        mise          # Tool version manager for consistent environments
        ouch          # Compression tool for handling archives
        libnotify     # Provides notify-send for desktop notifications (used by Nushell clip command)
      ];

      # Yazi extension dependencies (enhance Yazi functionality, e.g., previews, archives)
      yaziExtensionsDeps = with pkgs; [
        ffmpeg        # Multimedia processing for media previews in Yazi
        p7zip         # Archive utility for handling compressed files
        jq            # JSON processor for parsing and formatting in Yazi plugins
        fd            # Fast file finder for efficient search in Yazi
        ripgrep       # High-performance search tool for file content
        poppler       # PDF rendering for document previews in Yazi
        imagemagick   # Image processing for thumbnail generation in Yazi
      ];

      # Combine dependencies based on config
      allDeps = essentialDeps ++ (if includeOptionalDeps then optionalDeps else []) ++ (if includeYaziExtensions then yaziExtensionsDeps else []);

    in {
      devShells.default = pkgs.mkShell {
        buildInputs = allDeps;

        shellHook = ''
          # Convert Nix boolean to shell string for easier conditional
          YAZELIX_DEBUG_MODE_SHELL="${if yazelixDebugMode then "true" else "false"}"

          if [ "$YAZELIX_DEBUG_MODE_SHELL" = "true" ]; then
            echo "--- Yazelix Flake shellHook Started (DEBUG MODE) ---"
            echo "[DEBUG] Using HOME=$HOME"
            echo "[DEBUG] Resolved Yazelix config file: ${configFile}"
            echo "[DEBUG] include_optional_deps: ${if includeOptionalDeps then "true" else "false"}"
            echo "[DEBUG] include_yazi_extensions: ${if includeYaziExtensions then "true" else "false"}"
            echo "[DEBUG] build_helix_from_source: ${if buildHelixFromSource then "true" else "false"}"
            echo "[DEBUG] default_shell: ${yazelixDefaultShell}"
            echo "[DEBUG] debug_mode active: $YAZELIX_DEBUG_MODE_SHELL"
            echo ""
          else
            echo "--- Yazelix Flake shellHook Started ---"
          fi

          # --- Nushell Initializers ---
          if [ "$YAZELIX_DEBUG_MODE_SHELL" = "true" ]; then echo "[DEBUG] Setting up Nushell initializers..."; fi
          NUSHELL_INITIALIZERS_DIR="$HOME/.config/yazelix/nushell/initializers"
          if [ "$YAZELIX_DEBUG_MODE_SHELL" = "true" ]; then echo "[DEBUG] Nushell initializers directory: $NUSHELL_INITIALIZERS_DIR"; fi
          mkdir -p "$NUSHELL_INITIALIZERS_DIR" || echo "[WARN] Could not create Nushell initializers directory: $NUSHELL_INITIALIZERS_DIR"

          ${if includeOptionalDeps then ''
            if [ "$YAZELIX_DEBUG_MODE_SHELL" = "true" ]; then echo "[DEBUG] Generating mise_init.nu (include_optional_deps=true)"; fi
            mise activate nu > "$NUSHELL_INITIALIZERS_DIR/mise_init.nu" 2>/dev/null || echo "[WARN] Failed to generate mise_init.nu"
          '' else ''
            if [ "$YAZELIX_DEBUG_MODE_SHELL" = "true" ]; then echo "[DEBUG] Skipping mise Nushell initialization (include_optional_deps=false)"; fi
            touch "$NUSHELL_INITIALIZERS_DIR/mise_init.nu" || echo "[WARN] Failed to touch empty mise_init.nu"
          ''}

          if [ "$YAZELIX_DEBUG_MODE_SHELL" = "true" ]; then echo "[DEBUG] Generating starship_init.nu"; fi
          starship init nu > "$NUSHELL_INITIALIZERS_DIR/starship_init.nu" 2>/dev/null || echo "[WARN] Failed to generate starship_init.nu"

          if [ "$YAZELIX_DEBUG_MODE_SHELL" = "true" ]; then echo "[DEBUG] Generating zoxide_init.nu"; fi
          zoxide init nushell --cmd z > "$NUSHELL_INITIALIZERS_DIR/zoxide_init.nu" 2>/dev/null || echo "[WARN] Failed to generate zoxide_init.nu"
          if [ "$YAZELIX_DEBUG_MODE_SHELL" = "true" ]; then echo "[DEBUG] Nushell initializers setup complete."; echo ""; fi

          # --- Bash Initializers ---
          if [ "$YAZELIX_DEBUG_MODE_SHELL" = "true" ]; then echo "[DEBUG] Setting up Bash initializers..."; fi
          BASH_INITIALIZERS_DIR="$HOME/.config/yazelix/bash/initializers"
          if [ "$YAZELIX_DEBUG_MODE_SHELL" = "true" ]; then echo "[DEBUG] Bash initializers directory: $BASH_INITIALIZERS_DIR"; fi
          mkdir -p "$BASH_INITIALIZERS_DIR" || echo "[WARN] Could not create Bash initializers directory: $BASH_INITIALIZERS_DIR"

          if [ "$YAZELIX_DEBUG_MODE_SHELL" = "true" ]; then echo "[DEBUG] Generating starship_init.sh for Bash"; fi
          starship init bash > "$BASH_INITIALIZERS_DIR/starship_init.sh" 2>/dev/null || echo "[WARN] Failed to generate starship_init.sh for Bash"

          if [ "$YAZELIX_DEBUG_MODE_SHELL" = "true" ]; then echo "[DEBUG] Generating zoxide_init.sh for Bash"; fi
          zoxide init bash --cmd z > "$BASH_INITIALIZERS_DIR/zoxide_init.sh" 2>/dev/null || echo "[WARN] Failed to generate zoxide_init.sh for Bash"

          ${if includeOptionalDeps then ''
            if [ "$YAZELIX_DEBUG_MODE_SHELL" = "true" ]; then echo "[DEBUG] Generating mise_init.sh for Bash (include_optional_deps=true)"; fi
            mise activate bash > "$BASH_INITIALIZERS_DIR/mise_init.sh" 2>/dev/null || echo "[WARN] Failed to generate mise_init.sh for Bash"
          '' else ''
            if [ "$YAZELIX_DEBUG_MODE_SHELL" = "true" ]; then echo "[DEBUG] Skipping mise Bash initialization (include_optional_deps=false)"; fi
            touch "$BASH_INITIALIZERS_DIR/mise_init.sh" || echo "[WARN] Failed to touch empty mise_init.sh"
          ''}
          if [ "$YAZELIX_DEBUG_MODE_SHELL" = "true" ]; then echo "[DEBUG] Bash initializers setup complete."; echo ""; fi

          # --- Bash Configuration Sourcing ---
          if [ "$YAZELIX_DEBUG_MODE_SHELL" = "true" ]; then echo "[DEBUG] Setting up Bash configuration sourcing..."; fi
          PERSISTED_YAZELIX_BASH_CONFIG_FILE="$HOME/.config/yazelix/bash/yazelix_bash_config.sh"
          BASHRC_FILE="$HOME/.bashrc"
          YAZELIX_BASH_COMMENT_LINE="# Source Yazelix Bash configuration (added by Yazelix)"
          YAZELIX_BASH_SOURCE_LINE="source \"$PERSISTED_YAZELIX_BASH_CONFIG_FILE\""
          if [ "$YAZELIX_DEBUG_MODE_SHELL" = "true" ]; then
            echo "[DEBUG] Persisted Yazelix Bash config: $PERSISTED_YAZELIX_BASH_CONFIG_FILE"
            echo "[DEBUG] User .bashrc file: $BASHRC_FILE"
          fi

          if [ ! -f "$PERSISTED_YAZELIX_BASH_CONFIG_FILE" ]; then
            echo "[WARN] Persisted Yazelix Bash config not found at $PERSISTED_YAZELIX_BASH_CONFIG_FILE. Please ensure it exists."
          else
            if [ "$YAZELIX_DEBUG_MODE_SHELL" = "true" ]; then echo "[DEBUG] Persisted Yazelix Bash config found. Checking $BASHRC_FILE..."; fi
            touch "$BASHRC_FILE" || echo "[WARN] Failed to touch $BASHRC_FILE"
            if ! grep -qF -- "$YAZELIX_BASH_COMMENT_LINE" "$BASHRC_FILE"; then
              if [ "$YAZELIX_DEBUG_MODE_SHELL" = "true" ]; then echo "[DEBUG] Yazelix Bash sourcing not found in $BASHRC_FILE. Adding it."; fi
              {
                echo ""
                echo "$YAZELIX_BASH_COMMENT_LINE"
                echo "$YAZELIX_BASH_SOURCE_LINE"
              } >> "$BASHRC_FILE"
              echo "[INFO] Added Yazelix Bash config source to $BASHRC_FILE. You might need to source it manually: source $BASHRC_FILE"
            else
              if [ "$YAZELIX_DEBUG_MODE_SHELL" = "true" ]; then echo "[INFO] Yazelix Bash config (with standard comment) already sourced in $BASHRC_FILE."; fi
            fi
          fi
          if [ "$YAZELIX_DEBUG_MODE_SHELL" = "true" ]; then echo "[DEBUG] Bash configuration sourcing setup complete."; echo ""; fi

          # --- Yazi Setup ---
          if [ "$YAZELIX_DEBUG_MODE_SHELL" = "true" ]; then echo "[DEBUG] Setting up Yazi..."; fi
          export YAZI_CONFIG_HOME="$HOME/.config/yazelix/yazi"
          if [ "$YAZELIX_DEBUG_MODE_SHELL" = "true" ]; then echo "[INFO] YAZI_CONFIG_HOME set to: $YAZI_CONFIG_HOME"; fi
          if [ "$YAZELIX_DEBUG_MODE_SHELL" = "true" ]; then echo "[DEBUG] Yazi setup complete."; echo ""; fi

          # --- Nushell Configuration Sourcing ---
          if [ "$YAZELIX_DEBUG_MODE_SHELL" = "true" ]; then echo "[DEBUG] Setting up Nushell configuration sourcing..."; fi
          NUSHELL_USER_CONFIG_FILE="$HOME/.config/nushell/config.nu"
          YAZELIX_NUSHELL_CONFIG_TO_SOURCE="$HOME/.config/yazelix/nushell/config/config.nu"
          YAZELIX_NUSHELL_COMMENT_LINE="# Source Yazelix Nushell configuration (added by Yazelix)"
          YAZELIX_NUSHELL_SOURCE_LINE="source \"$YAZELIX_NUSHELL_CONFIG_TO_SOURCE\""
          if [ "$YAZELIX_DEBUG_MODE_SHELL" = "true" ]; then
            echo "[DEBUG] User Nushell config file: $NUSHELL_USER_CONFIG_FILE"
            echo "[DEBUG] Yazelix Nushell config to source: $YAZELIX_NUSHELL_CONFIG_TO_SOURCE"
          fi

          mkdir -p "$(dirname "$NUSHELL_USER_CONFIG_FILE")" || echo "[WARN] Could not create Nushell config directory: $(dirname "$NUSHELL_USER_CONFIG_FILE")"
          if [ ! -f "$NUSHELL_USER_CONFIG_FILE" ]; then
            if [ "$YAZELIX_DEBUG_MODE_SHELL" = "true" ]; then echo "[DEBUG] $NUSHELL_USER_CONFIG_FILE not found. Creating it."; fi
            echo "# Nushell user configuration (created by Yazelix setup)" > "$NUSHELL_USER_CONFIG_FILE"
            echo "[INFO] Created new $NUSHELL_USER_CONFIG_FILE"
          fi

          if ! grep -qF -- "$YAZELIX_NUSHELL_COMMENT_LINE" "$NUSHELL_USER_CONFIG_FILE"; then
            if [ "$YAZELIX_DEBUG_MODE_SHELL" = "true" ]; then echo "[DEBUG] Yazelix Nushell sourcing not found in $NUSHELL_USER_CONFIG_FILE. Adding it."; fi
            {
              echo ""
              echo "$YAZELIX_NUSHELL_COMMENT_LINE"
              echo "$YAZELIX_NUSHELL_SOURCE_LINE"
            } >> "$NUSHELL_USER_CONFIG_FILE"
            echo "[INFO] Added Yazelix Nushell config source to $NUSHELL_USER_CONFIG_FILE"
          else
            if [ "$YAZELIX_DEBUG_MODE_SHELL" = "true" ]; then echo "[INFO] Yazelix Nushell config (with standard comment) already sourced in $NUSHELL_USER_CONFIG_FILE."; fi
          fi
          if [ "$YAZELIX_DEBUG_MODE_SHELL" = "true" ]; then echo "[DEBUG] Nushell configuration sourcing setup complete."; echo ""; fi

          # --- Helix Setup ---
          if [ "$YAZELIX_DEBUG_MODE_SHELL" = "true" ]; then echo "[DEBUG] Setting up Helix..."; fi
          export EDITOR=hx
          if [ "$YAZELIX_DEBUG_MODE_SHELL" = "true" ]; then echo "[INFO] EDITOR set to: $EDITOR"; fi
          if [ "$YAZELIX_DEBUG_MODE_SHELL" = "true" ]; then echo "[DEBUG] Helix setup complete."; echo ""; fi

          # --- Set executable permissions ---
          if [ "$YAZELIX_DEBUG_MODE_SHELL" = "true" ]; then echo "[DEBUG] Setting executable permissions for shell scripts..."; fi
          LAUNCH_SCRIPT="$HOME/.config/yazelix/shell_scripts/launch-yazelix.sh"
          START_SCRIPT="$HOME/.config/yazelix/shell_scripts/start-yazelix.sh"
          chmod +x "$LAUNCH_SCRIPT" || echo "[WARN] Could not set executable permissions for $LAUNCH_SCRIPT"
          chmod +x "$START_SCRIPT" || echo "[WARN] Could not set executable permissions for $START_SCRIPT"
          if [ "$YAZELIX_DEBUG_MODE_SHELL" = "true" ]; then echo "[DEBUG] Executable permissions setup complete."; echo ""; fi

          # --- Display configuration status ---
          echo "--- Yazelix Configuration Status ---"
          CONFIG_FILE_PATH_FOR_SHELL="${configFile}"
          echo "  Config file path (from Nix): $CONFIG_FILE_PATH_FOR_SHELL"
          if [ -f "$CONFIG_FILE_PATH_FOR_SHELL" ]; then # This check uses the shell variable
            echo "  Config file found at $CONFIG_FILE_PATH_FOR_SHELL"
          else
            echo "  Config file NOT FOUND at $CONFIG_FILE_PATH_FOR_SHELL, using defaults."
          fi
          echo "  include_optional_deps: ${if includeOptionalDeps then "true" else "false"}"
          echo "  include_yazi_extensions: ${if includeYaziExtensions then "true" else "false"}"
          echo "  build_helix_from_source: ${if buildHelixFromSource then "true" else "false"}"
          echo "  default_shell: ${yazelixDefaultShell}"
          echo "  debug_mode: ${if yazelixDebugMode then "true" else "false"}" # Display debug_mode status
          echo "------------------------------------"
          if [ "$YAZELIX_DEBUG_MODE_SHELL" = "true" ]; then echo ""; fi

          # --- Final Configuration ---
          if [ "$YAZELIX_DEBUG_MODE_SHELL" = "true" ]; then echo "[DEBUG] Setting final environment variables..."; fi
          export ZELLIJ_DEFAULT_LAYOUT=yazelix
          export YAZELIX_DEFAULT_SHELL="${yazelixDefaultShell}"
          if [ "$YAZELIX_DEBUG_MODE_SHELL" = "true" ]; then
            echo "[INFO] ZELLIJ_DEFAULT_LAYOUT set to: $ZELLIJ_DEFAULT_LAYOUT"
            echo "[INFO] YAZELIX_DEFAULT_SHELL set to: $YAZELIX_DEFAULT_SHELL"
            echo ""
          fi

          if [ "$YAZELIX_DEBUG_MODE_SHELL" = "true" ]; then
            echo "--- Yazelix Flake shellHook Finished (DEBUG MODE) ---"
          else
            echo "--- Yazelix Flake shellHook Finished ---"
          fi
          echo "Yazelix environment ready! Use 'z' for smart directory navigation."
        '';
      };
    });
}
