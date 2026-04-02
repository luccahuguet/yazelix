{
  description = "Yazelix flake interface";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs =
    {
      self,
      nixpkgs,
    }:
    let
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];
      forAllSystems = nixpkgs.lib.genAttrs systems;
      mkPkgs = system: nixpkgs.legacyPackages.${system};
      runtimePackage = pkgs: import ./yazelix_runtime_package.nix { inherit pkgs; };
      yazelixPackage = pkgs: import ./yazelix_package.nix { inherit pkgs; };
    in
    {
      packages = forAllSystems (
        system:
        let
          pkgs = mkPkgs system;
          lockedDevenv = import ./locked_devenv_package.nix { inherit pkgs; };
          runtime = runtimePackage pkgs;
          yazelix = yazelixPackage pkgs;
          install = pkgs.writeShellScriptBin "yazelix-install" ''
            set -eu

            if [ -z "''${HOME:-}" ]; then
              echo "Error: HOME is not set." >&2
              exit 1
            fi

            runtime_target="${runtime}"
            runtime_root="$HOME/.local/share/yazelix/runtime"
            runtime_current="$runtime_root/current"
            bin_dir="$HOME/.local/bin"
            yzx_link="$bin_dir/yzx"
            config_root="$HOME/.config/yazelix"
            user_config_dir="$config_root/user_configs"
            main_config="$user_config_dir/yazelix.toml"
            pack_config="$user_config_dir/yazelix_packs.toml"

            ${pkgs.coreutils}/bin/mkdir -p "$runtime_root" "$bin_dir" "$user_config_dir"
            ${pkgs.coreutils}/bin/ln -sfn "$runtime_target" "$runtime_current"
            ${pkgs.coreutils}/bin/ln -sfn "$runtime_current/bin/yzx" "$yzx_link"

            if [ ! -f "$main_config" ]; then
              ${pkgs.coreutils}/bin/cp "$runtime_current/yazelix_default.toml" "$main_config"
            fi

            if [ ! -f "$pack_config" ]; then
              ${pkgs.coreutils}/bin/cp "$runtime_current/yazelix_packs_default.toml" "$pack_config"
            fi

            echo "🔄 Refreshing Yazelix shell hooks..."
            YAZELIX_RUNTIME_DIR="$runtime_current" \
            YAZELIX_DIR="$runtime_current" \
            YAZELIX_STATE_DIR="$HOME/.local/share/yazelix" \
            YAZELIX_LOGS_DIR="$HOME/.local/share/yazelix/logs" \
            ${pkgs.nushell}/bin/nu "$runtime_current/nushell/scripts/setup/environment.nu" --skip-welcome

            echo "🔄 Rebuilding generated Yazelix runtime configs..."
            ${pkgs.coreutils}/bin/rm -rf \
              "$HOME/.local/share/yazelix/configs/yazi" \
              "$HOME/.local/share/yazelix/configs/zellij"
            YAZELIX_RUNTIME_DIR="$runtime_current" \
            YAZELIX_DIR="$runtime_current" \
            YAZELIX_STATE_DIR="$HOME/.local/share/yazelix" \
            YAZELIX_LOGS_DIR="$HOME/.local/share/yazelix/logs" \
            ${pkgs.nushell}/bin/nu "$runtime_current/nushell/scripts/setup/yazi_config_merger.nu" "$runtime_current" --quiet
            YAZELIX_RUNTIME_DIR="$runtime_current" \
            YAZELIX_DIR="$runtime_current" \
            YAZELIX_STATE_DIR="$HOME/.local/share/yazelix" \
            YAZELIX_LOGS_DIR="$HOME/.local/share/yazelix/logs" \
            PATH="${pkgs.zellij}/bin:$PATH" \
            ${pkgs.nushell}/bin/nu -c "use '$runtime_current/nushell/scripts/setup/zellij_config_merger.nu' [generate_merged_zellij_config]; generate_merged_zellij_config '$runtime_current' | ignore"

            echo "✅ Yazelix runtime installed."
            echo "   Runtime: $runtime_current -> $runtime_target"
            echo "   CLI: $yzx_link"
            echo "   Config: $config_root"
            echo
            echo "Next step:"
            echo "  yzx launch"
          '';
        in
        {
          default = runtime;
          locked_devenv = lockedDevenv;
          runtime = runtime;
          yazelix = yazelix;
          install = install;
        }
      );

      apps = forAllSystems (system: {
        install = {
          type = "app";
          program = "${self.packages.${system}.install}/bin/yazelix-install";
        };
      });
    };
}
