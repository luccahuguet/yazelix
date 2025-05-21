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
               };

      # Variables to control optional, Yazi extension, and Helix source dependencies
      includeOptionalDeps = config.include_optional_deps or true;
      includeYaziExtensions = config.include_yazi_extensions or true;
      buildHelixFromSource = config.build_helix_from_source or true;

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
          # Log HOME for debugging
          echo "Using HOME=$HOME"

          # Create initializers directory
          mkdir -p "$HOME/.config/yazelix/nushell/initializers" || echo "Warning: Could not create initializers directory"

          # Generate initializer scripts
          ${if includeOptionalDeps then ''
            mise activate nu > "$HOME/.config/yazelix/nushell/initializers/mise_init.nu" 2>/dev/null || echo "Warning: Failed to generate mise_init.nu"
          '' else ''
            echo "mise initialization skipped (include_optional_deps=false)"
            touch "$HOME/.config/yazelix/nushell/initializers/mise_init.nu"
          ''}
          starship init nu > "$HOME/.config/yazelix/nushell/initializers/starship_init.nu" 2>/dev/null || echo "Warning: Failed to generate starship_init.nu"
          zoxide init nushell --cmd z > "$HOME/.config/yazelix/nushell/initializers/zoxide_init.nu" 2>/dev/null || echo "Warning: Failed to generate zoxide_init.nu"

          # Yazi Setup
          export YAZI_CONFIG_HOME="$HOME/.config/yazelix/yazi"

          # Nushell Setup
          mkdir -p "$HOME/.config/nushell" || echo "Warning: Could not create Nushell config directory"
          if [ ! -f "$HOME/.config/nushell/config.nu" ]; then
            echo "# Nushell user configuration" > "$HOME/.config/nushell/config.nu"
            echo "Created new $HOME/.config/nushell/config.nu"
          fi
          if ! grep -q "source.*yazelix/nushell/config/config.nu" "$HOME/.config/nushell/config.nu"; then
            echo "# Source Yazelix Nushell configuration" >> "$HOME/.config/nushell/config.nu"
            echo "source $HOME/.config/yazelix/nushell/config/config.nu" >> "$HOME/.config/nushell/config.nu"
            echo "Added Yazelix config source to $HOME/.config/nushell/config.nu"
          fi

          # Helix Setup
          export EDITOR=hx

          # Set executable permissions for launch-yazelix.sh
          chmod +x "$HOME/.config/yazelix/shell_scripts/launch-yazelix.sh" || echo "Warning: Could not set executable permissions for launch-yazelix.sh"

          # Display configuration status
          echo "Yazelix configuration:"
          echo "  Config file path: ${configFile}"
          if [ -f "${configFile}" ]; then
            echo "  Config file found at ${configFile}"
          else
            echo "  Config file not found at ${configFile}, using defaults"
          fi
          echo "  include_optional_deps: ${if includeOptionalDeps then "true" else "false"}"
          echo "  include_yazi_extensions: ${if includeYaziExtensions then "true" else "false"}"
          echo "  build_helix_from_source: ${if buildHelixFromSource then "true" else "false"}"

          # Final Configuration
          export ZELLIJ_DEFAULT_LAYOUT=yazelix
          echo "Yazelix environment ready! Use 'z' for smart directory navigation."
        '';
      };
    });
}
