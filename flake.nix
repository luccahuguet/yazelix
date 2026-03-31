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
      devenvLock = builtins.fromJSON (builtins.readFile ./devenv.lock);
      devenvNode = devenvLock.nodes.devenv.locked;
      pinnedDevenvInstallable = "github:${devenvNode.owner}/${devenvNode.repo}/${devenvNode.rev}#devenv";
      mkPkgs = system: nixpkgs.legacyPackages.${system};
    in
    {
      packages = forAllSystems (
        system:
        let
          pkgs = mkPkgs system;
          runtime = pkgs.runCommand "yazelix-runtime" { } ''
            mkdir -p "$out"

            ln -s ${./assets} "$out/assets"
            ln -s ${./config_metadata} "$out/config_metadata"
            ln -s ${./configs} "$out/configs"
            ln -s ${./docs} "$out/docs"
            ln -s ${./nushell} "$out/nushell"
            ln -s ${./rust_plugins} "$out/rust_plugins"
            ln -s ${./shells} "$out/shells"

            ln -s ${./CHANGELOG.md} "$out/CHANGELOG.md"
            ln -s ${./devenv.lock} "$out/devenv.lock"
            ln -s ${./devenv.nix} "$out/devenv.nix"
            ln -s ${./devenv.yaml} "$out/devenv.yaml"
            ln -s ${./yazelix_default.toml} "$out/yazelix_default.toml"
            ln -s ${./yazelix_packs_default.toml} "$out/yazelix_packs_default.toml"

            mkdir -p "$out/bin"
            cat > "$out/bin/yzx" <<EOF
#!/bin/sh
PATH="${pkgs.nushell}/bin:\$PATH"
SCRIPT_PATH="\$0"
if [ -L "\$SCRIPT_PATH" ]; then
  LINK_TARGET="\$(readlink "\$SCRIPT_PATH")"
  case "\$LINK_TARGET" in
    /*) SCRIPT_PATH="\$LINK_TARGET" ;;
    *) SCRIPT_PATH="\$(dirname "\$SCRIPT_PATH")/\$LINK_TARGET" ;;
  esac
fi
exec "\$(dirname "\$SCRIPT_PATH")/../shells/posix/yzx_cli.sh" "\$@"
EOF
            chmod +x "$out/bin/yzx"
          '';
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
            skip_devenv="''${YAZELIX_INSTALL_SKIP_DEVENV:-0}"

            ${pkgs.coreutils}/bin/mkdir -p "$runtime_root" "$bin_dir" "$user_config_dir"
            ${pkgs.coreutils}/bin/ln -sfn "$runtime_target" "$runtime_current"
            ${pkgs.coreutils}/bin/ln -sfn "$runtime_current/bin/yzx" "$yzx_link"

            if [ ! -f "$main_config" ]; then
              ${pkgs.coreutils}/bin/cp "$runtime_current/yazelix_default.toml" "$main_config"
            fi

            if [ ! -f "$pack_config" ]; then
              ${pkgs.coreutils}/bin/cp "$runtime_current/yazelix_packs_default.toml" "$pack_config"
            fi

            profile_json="$(${pkgs.coreutils}/bin/mktemp)"
            if [ "$skip_devenv" = "1" ]; then
              echo "ℹ️ Skipping devenv installation because YAZELIX_INSTALL_SKIP_DEVENV=1."
            elif nix profile list --json > "$profile_json" 2>/dev/null && ${pkgs.jq}/bin/jq -e '.elements | has("devenv")' "$profile_json" >/dev/null 2>&1; then
              echo "ℹ️ devenv already present in your Nix profile."
            else
              echo "🔄 Installing Yazelix-pinned devenv CLI..."
              nix profile install "${pinnedDevenvInstallable}"
              echo "✅ devenv CLI installed."
            fi
            ${pkgs.coreutils}/bin/rm -f "$profile_json"

            echo "🔄 Refreshing Yazelix shell hooks..."
            YAZELIX_RUNTIME_DIR="$runtime_current" \
            YAZELIX_DIR="$runtime_current" \
            YAZELIX_STATE_DIR="$HOME/.local/share/yazelix" \
            YAZELIX_LOGS_DIR="$HOME/.local/share/yazelix/logs" \
            ${pkgs.nushell}/bin/nu "$runtime_current/nushell/scripts/setup/environment.nu" --skip-welcome

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
          runtime = runtime;
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
