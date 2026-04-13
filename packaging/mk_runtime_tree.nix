{ pkgs, src ? ../., nixgl ? null, name ? "yazelix-runtime" }:

let
  runtimeDeps = import ./runtime_deps.nix { inherit pkgs nixgl; };
  runtimeBinDirs = map (pkg: "${pkg}/bin") runtimeDeps;
  escapedRuntimeBinDirs = pkgs.lib.escapeShellArgs runtimeBinDirs;
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
  ln -s ${src}/yazelix_default.toml "$out/yazelix_default.toml"

  mkdir -p "$out/bin"
  for bin_dir in ${escapedRuntimeBinDirs}; do
    if [ -d "$bin_dir" ]; then
      for entry in "$bin_dir"/*; do
        [ -e "$entry" ] || continue
        ln -sfn "$entry" "$out/bin/$(basename "$entry")"
      done
    fi
  done
  cat > "$out/bin/yzx" <<EOF
#!/bin/sh
PATH="${pkgs.nushell}/bin:\$PATH"
YAZELIX_INVOKED_YZX_PATH="\$0"
export YAZELIX_INVOKED_YZX_PATH
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
