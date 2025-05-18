{
  description = "Nix shell for Yazelix";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system: let
      pkgs = import nixpkgs { inherit system; };
    in {
      devShells.default = pkgs.mkShell {
        buildInputs = with pkgs; [
          zellij
          helix
          nushell
          yazi
          zoxide
          cargo-update
          cargo-binstall
          ffmpeg
          p7zip
          jq
          poppler
          fd
          ripgrep
          fzf
          imagemagick
          lazygit
        ];

        shellHook = ''
          # Set up Zellij config directory
          export ZELLIJ_CONFIG_DIR="$HOME/.config/yazelix/zellij"
          mkdir -p "$ZELLIJ_CONFIG_DIR" || { echo "Error: Failed to create ZELLIJ_CONFIG_DIR"; exit 1; }
          if [ -d "$PWD/config/zellij" ]; then
            cp -r "$PWD/config/zellij/." "$ZELLIJ_CONFIG_DIR/" || { echo "Error: Failed to copy Zellij configs"; exit 1; }
          fi

          # Set up Yazi config
          export YAZI_CONFIG_HOME="$PWD/config/yazi"
          mkdir -p "$HOME/.local/state/yazi" || { echo "Error: Failed to create Yazi state directory"; exit 1; }

          # Ensure Nushell config directory
          export XDG_CONFIG_HOME="$HOME/.config"
          mkdir -p "$HOME/.config/nushell" || { echo "Error: Failed to create Nushell config directory"; exit 1; }

          # Check if ~/.config/yazelix/nushell/config.nu exists
          if [ ! -f "$HOME/.config/yazelix/nushell/config.nu" ]; then
            echo "Warning: ~/.config/yazelix/nushell/config.nu not found. Creating empty file."
            mkdir -p "$HOME/.config/yazelix/nushell"
            echo "# Custom Yazelix Nushell config" > "$HOME/.config/yazelix/nushell/config.nu"
          fi

          # Manage ~/.config/nushell/config.nu
          if [ -f "$HOME/.config/nushell/config.nu" ]; then
            # Check if source command already exists
            if ! grep -q "source ~/.config/yazelix/nushell/config.nu" "$HOME/.config/nushell/config.nu"; then
              echo "source ~/.config/yazelix/nushell/config.nu" >> "$HOME/.config/nushell/config.nu"
            fi
          else
            # Create config.nu with source command
            echo "# Nushell config file" > "$HOME/.config/nushell/config.nu"
            echo "source ~/.config/yazelix/nushell/config.nu" >> "$HOME/.config/nushell/config.nu"
          fi

          # Set up Helix runtime
          export HELIX_RUNTIME="$PWD/config/helix/runtime"

          # Set editor
          export EDITOR=hx

          # Set Zellij default layout
          export ZELLIJ_DEFAULT_LAYOUT=yazelix

          # Print welcome message
          echo "Yazelix environment ready! "
        '';
      };
    });
}
