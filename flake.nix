{
  description = "Nix shell for Yazelix";

  inputs = {
    # Nixpkgs for package management
    nixpkgs.url = "github:nixos/nixpkgs/nixos-24.11";
    # Yazi Flake for version 25.4.8
    yazi = {
      url = "github:sxyazi/yazi/v25.4.8";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    # Helix Flake for source build (commit 0efa8207)
    helix = {
      url = "github:helix-editor/helix/0efa8207";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    # Nushell Flake for version 0.103.0
    nushell = {
      url = "github:nushell/nushell/0.103.0";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    # Flake-utils for multi-system support
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, yazi, helix, nushell, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system: let
      pkgs = import nixpkgs { inherit system; };
    in {
      devShells.default = pkgs.mkShell {
        # Dependencies for Yazelix
        buildInputs = with pkgs; [
          zellij # Version ~0.42.1
          helix.packages.${pkgs.system}.helix # Version 25.01.1 (commit 0efa8207)
          nushell.packages.${pkgs.system}.default # Version 0.103.0
          yazi.packages.${pkgs.system}.default # Version 25.4.8
          zoxide # Version ~0.9.7
          cargo-update
          cargo-binstall
          wezterm # Version ~20240203-110809-5046fc22
          # Yazi dependencies
          ffmpeg
          p7zip
          jq
          poppler
          fd
          ripgrep
          fzf
          imagemagick
        ];

        # Environment variables to point to config files in the repo
        shellHook = ''
          export ZELLIJ_CONFIG_DIR=$PWD/config/zellij
          export YAZI_CONFIG_HOME=$PWD/config/yazi
          export HELIX_RUNTIME=$PWD/config/helix/runtime
          export NU_CONFIG_DIR=$PWD/config/nushell
          export EDITOR=helix
          export ZELLIJ_DEFAULT_LAYOUT=yazelix
          export WEZTERM_CONFIG_FILE=$PWD/terminal_configs/wez/.wezterm.lua

          # Create log directories
          mkdir -p $HOME/.config/yazelix/logs
          mkdir -p $HOME/.local/state/yazi

          # Alias for convenience
          alias yazelix='zellij -l yazelix'

          echo "Yazelix environment ready! Run 'zellij -l yazelix' to start."
        '';
      };
    });
}
