{ pkgs, src ? ../. , name ? "yazelix-runtime" }:

let
  lockedDevenv = import ./locked_devenv_package.nix { inherit pkgs src; };
in
pkgs.runCommand name { } ''
  mkdir -p "$out"

  ln -s ${src}/assets "$out/assets"
  ln -s ${src}/config_metadata "$out/config_metadata"
  ln -s ${src}/configs "$out/configs"
  ln -s ${src}/docs "$out/docs"
  ln -s ${src}/nushell "$out/nushell"
  ln -s ${src}/rust_plugins "$out/rust_plugins"
  ln -s ${src}/shells "$out/shells"

  ln -s ${src}/CHANGELOG.md "$out/CHANGELOG.md"
  ln -s ${src}/.taplo.toml "$out/.taplo.toml"
  ln -s ${src}/devenv.lock "$out/devenv.lock"
  ln -s ${src}/devenv.nix "$out/devenv.nix"
  ln -s ${src}/devenv.yaml "$out/devenv.yaml"
  ln -s ${src}/yazelix_default.toml "$out/yazelix_default.toml"
  ln -s ${src}/yazelix_packs_default.toml "$out/yazelix_packs_default.toml"

  mkdir -p "$out/bin"
  ln -s ${lockedDevenv}/bin/devenv "$out/bin/devenv"
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
