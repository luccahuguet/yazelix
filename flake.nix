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
          starship
          mise # Added Mise to buildInputs
        ];

        shellHook = ''
          # Logging Setup
          mkdir -p "$HOME/.config/yazelix/logs" || echo "Warning: Could not create logs directory at $HOME/.config/yazelix/logs"
          LOG_FILE="$HOME/.config/yazelix/logs/setup-yazelix.log"
          echo "=== Yazelix setup log: $(date) ===" >> "$LOG_FILE"
          exec > >(tee -a "$LOG_FILE") 2>&1

          # Zellij Setup
          export ZELLIJ_CONFIG_DIR="$HOME/.config/yazelix/zellij"
          mkdir -p "$ZELLIJ_CONFIG_DIR" || { echo "Error: Failed to create ZELLIJ_CONFIG_DIR"; exit 1; }
          if [ -d "$PWD/config/zellij" ]; then
            cp -r "$PWD/config/zellij/." "$ZELLIJ_CONFIG_DIR/" || { echo "Error: Failed to copy Zellij configs"; exit 1; }
          fi

          # Yazi Setup
          export YAZI_CONFIG_HOME="$PWD/yazi"

          # Nushell Setup
          export XDG_CONFIG_HOME="$HOME/.config"
          mkdir -p "$HOME/.config/nushell" || echo "Warning: Could not create Nushell config directory; it may already exist or be managed elsewhere."
          mkdir -p "$HOME/.config/yazelix/nushell"
          if [ ! -f "$HOME/.config/yazelix/nushell/config.nu" ]; then
            echo "Warning: ~/.config/yazelix/nushell/config.nu not found. Creating minimal file."
            echo "# Yazelix Nushell config" > "$HOME/.config/yazelix/nushell/config.nu"
          fi
          if [ -f "$HOME/.config/nushell/config.nu" ]; then
            if [ ! -f "$HOME/.config/nushell/config.nu.bak" ]; then
              cp "$HOME/.config/nushell/config.nu" "$HOME/.config/nushell/config.nu.bak"
              echo "Backed up existing ~/.config/nushell/config.nu to ~/.config/nushell/config.nu.bak"
            fi
            if ! grep -q "source ~/.config/yazelix/nushell/config.nu" "$HOME/.config/nushell/config.nu"; then
              echo "# Source Yazelix Nushell config for Starship, Zoxide, and Mise integration" >> "$HOME/.config/nushell/config.nu"
              echo "source ~/.config/yazelix/nushell/config.nu" >> "$HOME/.config/nushell/config.nu"
              echo "Added Yazelix config source to ~/.config/nushell/config.nu"
            fi
          else
            echo "# Nushell config file" > "$HOME/.config/nushell/config.nu"
            echo "# Source Yazelix Nushell config for Starship, Zoxide, and Mise integration" >> "$HOME/.config/nushell/config.nu"
            echo "source ~/.config/yazelix/nushell/config.nu" >> "$HOME/.config/nushell/config.nu"
            echo "Created new ~/.config/nushell/config.nu with Yazelix config source"
          fi

          # Starship Setup
          export STARSHIP_SHELL=nu
          echo "# Starship initialization for Nushell" > "$HOME/.config/yazelix/nushell/starship_init.nu"
          starship init nu >> "$HOME/.config/yazelix/nushell/starship_init.nu"
          if ! grep -q "source ~/.config/yazelix/nushell/starship_init.nu" "$HOME/.config/yazelix/nushell/config.nu"; then
            echo "source ~/.config/yazelix/nushell/starship_init.nu" >> "$HOME/.config/yazelix/nushell/config.nu"
          fi
          if [ -f "$HOME/.config/yazelix/nushell/starship.nu" ]; then
            rm "$HOME/.config/yazelix/nushell/starship.nu"
          fi

          # Zoxide Setup
          echo "# Zoxide initialization for Nushell" > "$HOME/.config/yazelix/nushell/zoxide_init.nu"
          zoxide init nushell >> "$HOME/.config/yazelix/nushell/zoxide_init.nu"
          if ! grep -q "source ~/.config/yazelix/nushell/zox ide_init.nu" "$HOME/.config/yazelix/nushell/config.nu"; then
            echo "source ~/.config/yazelix/nushell/zoxide_init.nu" >> "$HOME/.config/yazelix/nushell/config.nu"
          fi

          # Mise Setup
          if [ -f "$HOME/.config/yazelix/nushell/mise_init.nu" ]; then
            if ! grep -q "source ~/.config/yazelix/nushell/mise_init.nu" "$HOME/.config/yazelix/nushell/config.nu"; then
              echo "source ~/.config/yazelix/nushell/mise_init.nu" >> "$HOME/.config/yazelix/nushell/config.nu"
              echo "Added Mise initialization to ~/.config/yazelix/nushell/config.nu"
            fi
          else
            echo "Warning: ~/.config/yazelix/nushell/mise_init.nu not found. Mise will not be activated."
          fi

          # Helix Setup
          # export HELIX_RUNTIME="$PWD/config/helix/runtime"
          export EDITOR=hx

          # Final Configuration
          export ZELLIJ_DEFAULT_LAYOUT=yazelix
          echo "Yazelix environment ready! Use 'z' for smart directory navigation and 'mise' for runtime management."
        '';
      };
    });
}
