{ pkgs }:

pkgs.runCommand "yazelix-runtime" { } ''
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
  ln -s ${pkgs.nushell}/bin/nu "$out/bin/nu"
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
''
