{
  description = "Nix shell for Yazelix";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system: let
      pkgs = import nixpkgs { inherit system; };

      # Read configuration from yazelix.toml
      homeDir = builtins.getEnv "HOME";
      configFile = if homeDir != "" then "${homeDir}/.config/yazelix/yazelix.toml"
                   else throw "HOME environment variable is unset or empty";
      config = if builtins.pathExists configFile
               then builtins.fromTOML (builtins.readFile configFile)
               else { include_optional_deps = true; include_yazi_extensions = true; };

      # Variables to control optional and Yazi extension dependencies
      includeOptionalDeps = config.include_optional_deps or true;
      includeYaziExtensions = config.include_yazi_extensions or true;

      # Essential dependencies (required for core Yazelix functionality)
      essentialDeps = with pkgs; [
        zellij
        helix
        yazi
        nushell
        fzf
        zoxide
        starship
      ];

      # Optional dependencies (enhance functionality but not Yazi-specific)
      optionalDeps = with pkgs; [
        cargo-update
        cargo-binstall
        lazygit
        mise
        ouch
      ];

      # Yazi extension dependencies (enhance Yazi functionality, e.g., previews, archives)
      yaziExtensionsDeps = with pkgs; [
        ffmpeg
        p7zip
        jq
        fd
        ripgrep
        poppler
        imagemagick
      ];

      # Combine dependencies based on config
      allDeps = essentialDeps ++ (if includeOptionalDeps then optionalDeps else []) ++ (if includeYaziExtensions then yaziExtensionsDeps else []);
    in {
      devShells.default = pkgs.mkShell {
        buildInputs = allDeps;

        shellHook = ''
          # Log HOME for debugging
          echo "Using HOME=$HOME"

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

          # Starship Setup
          export STARSHIP_SHELL=nu

          # Helix Setup
          export EDITOR=hx

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

          # Final Configuration
          export ZELLIJ_DEFAULT_LAYOUT=yazelix
          echo "Yazelix environment ready! Use 'z' for smart directory navigation."
        '';
      };
    });
}
