{
  pkgs,
  src ? ../.,
  nixgl ? null,
  name ? "yazelix-runtime",
  rustCoreHelper ? null,
  runtimeVariant ? "ghostty",
  runtimeToolSources ? { },
  components ? { },
  extraRuntimePackages ? [ ],
  extraRuntimeCommands ? [ "tu" ],
  yaziAssets ? null,
}:

let
  runtimeToolRegistry = import ./runtime_tool_registry.nix {
    inherit pkgs nixgl runtimeVariant runtimeToolSources;
  };
  runtimeComponentRegistry = import ./runtime_component_registry.nix {
    lib = pkgs.lib;
    inherit components;
  };
  cursorsEnabled = runtimeComponentRegistry.manifest.cursors.enabled;
  runtimeDeps = runtimeToolRegistry.runtimePackages ++ extraRuntimePackages;
  runtimeBinDirs = map (pkg: "${pkg}/bin") runtimeDeps;
  escapedRuntimeBinDirs = pkgs.lib.escapeShellArgs runtimeBinDirs;
  exportedRuntimeCommands = runtimeToolRegistry.exportedCommands ++ extraRuntimeCommands;
  escapedExportedRuntimeCommands = pkgs.lib.escapeShellArgs exportedRuntimeCommands;
  yaziAssetsRoot =
    if yaziAssets == null then
      "${src}/configs/yazi"
    else
      "${yaziAssets}/share/yazelix_yazi_assets";
in
pkgs.runCommand name { } ''
  mkdir -p "$out"

  ln -s ${src}/assets "$out/assets"
  ln -s ${src}/config_metadata "$out/config_metadata"
  mkdir -p "$out/configs"
  for config_entry in ${src}/configs/*; do
    config_name="$(basename "$config_entry")"
    if [ "$config_name" = "yazi" ]; then
      continue
    fi
    ln -s "$config_entry" "$out/configs/$config_name"
  done
  mkdir -p "$out/configs/yazi/plugins"
  for yazi_file in README.md yazelix_keymap.toml yazelix_theme.toml yazelix_yazi.toml; do
    ln -s "${src}/configs/yazi/$yazi_file" "$out/configs/yazi/$yazi_file"
  done
  for yazi_plugin in sidebar-state.yazi sidebar-status.yazi zoxide-editor.yazi; do
    ln -s "${src}/configs/yazi/plugins/$yazi_plugin" "$out/configs/yazi/plugins/$yazi_plugin"
  done
  ln -s "${yaziAssetsRoot}/flavors" "$out/configs/yazi/flavors"
  ln -s "${yaziAssetsRoot}/yazelix_starship.toml" "$out/configs/yazi/yazelix_starship.toml"
  for yazi_plugin in auto-layout.yazi git.yazi lazygit.yazi starship.yazi; do
    ln -s "${yaziAssetsRoot}/plugins/$yazi_plugin" "$out/configs/yazi/plugins/$yazi_plugin"
  done
  ln -s ${src}/docs "$out/docs"
  ln -s ${src}/nushell "$out/nushell"
  ln -s ${src}/shells "$out/shells"

  ln -s ${src}/CHANGELOG.md "$out/CHANGELOG.md"
  ln -s ${src}/tombi.toml "$out/tombi.toml"
  ln -s ${src}/yazelix_default.toml "$out/yazelix_default.toml"
  ${pkgs.lib.optionalString cursorsEnabled ''
    ln -s ${src}/yazelix_ghostty_cursors_default.toml "$out/yazelix_ghostty_cursors_default.toml"
  ''}
  printf '%s\n' ${pkgs.lib.escapeShellArg runtimeVariant} > "$out/runtime_variant"
  printf '%s\n' ${pkgs.lib.escapeShellArg runtimeComponentRegistry.manifestJson} > "$out/runtime_components.json"
  printf '%s\n' ${pkgs.lib.escapeShellArg runtimeToolRegistry.manifestJson} > "$out/runtime_tools.json"

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
