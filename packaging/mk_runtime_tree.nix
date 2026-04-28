{
  pkgs,
  src ? ../.,
  nixgl ? null,
  name ? "yazelix-runtime",
  rustCoreHelper ? null,
  runtimeVariant ? "ghostty",
}:

let
  runtimeDeps = import ./runtime_deps.nix { inherit pkgs nixgl runtimeVariant; };
  runtimeBinDirs = map (pkg: "${pkg}/bin") runtimeDeps;
  escapedRuntimeBinDirs = pkgs.lib.escapeShellArgs runtimeBinDirs;
  exportedRuntimeCommands = [
    "nu"
    "bash"
    "fish"
    "zsh"
    "zellij"
    "ghostty"
    "wezterm"
    "hx"
    "helix"
    "nvim"
    "neovim"
    "yazi"
    "ya"
    "fzf"
    "zoxide"
    "starship"
    "lazygit"
    "lg"
    "carapace"
    "macchina"
    "mise"
    "tombi"
    "git"
    "jq"
    "fd"
    "rg"
    "7z"
    "7za"
    "7zr"
    "pdfinfo"
    "pdftotext"
    "pdftoppm"
    "pdftocairo"
    "resvg"
  ];
  escapedExportedRuntimeCommands = pkgs.lib.escapeShellArgs exportedRuntimeCommands;
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
  ln -s ${src}/tombi.toml "$out/tombi.toml"
  ln -s ${src}/yazelix_default.toml "$out/yazelix_default.toml"
  ln -s ${src}/yazelix_cursors_default.toml "$out/yazelix_cursors_default.toml"
  printf '%s\n' ${pkgs.lib.escapeShellArg runtimeVariant} > "$out/runtime_variant"

  mkdir -p "$out/libexec"
  for bin_dir in ${escapedRuntimeBinDirs}; do
    if [ -d "$bin_dir" ]; then
      for entry in "$bin_dir"/*; do
        [ -e "$entry" ] || continue
        ln -sfn "$entry" "$out/libexec/$(basename "$entry")"
      done
    fi
  done
  ${pkgs.lib.optionalString (rustCoreHelper != null) ''
    ln -sfn "${rustCoreHelper}/bin/yzx" "$out/libexec/yzx"
    ln -sfn "${rustCoreHelper}/bin/yzx_core" "$out/libexec/yzx_core"
    ln -sfn "${rustCoreHelper}/bin/yzx_control" "$out/libexec/yzx_control"
  ''}

  mkdir -p "$out/toolbin"
  for command_name in ${escapedExportedRuntimeCommands}; do
    if [ -e "$out/libexec/$command_name" ]; then
      ln -sfn "$out/libexec/$command_name" "$out/toolbin/$command_name"
    fi
  done

  mkdir -p "$out/bin"
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
