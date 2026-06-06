{
  pkgs,
  src ? ../.,
  nixgl ? null,
  name ? "yazelix-runtime",
  rustCoreHelper ? null,
  runtimeVariant ? "ghostty",
  runtimeToolSources ? { },
  runtimeIdentity ? { },
  components ? { },
  extraRuntimePackages ? [ ],
  extraRuntimeCommands ? [ "tu" ],
  screenAssets,
  yaziAssets ? null,
  yazelixTerminalPackage ? null,
  zellijPluginArtifacts ? { },
  enableZellijKittyPassthrough ? false,
}:

let
  runtimeToolRegistry = import ./runtime_tool_registry.nix {
    inherit pkgs nixgl runtimeVariant runtimeToolSources yazelixTerminalPackage;
  };
  runtimeComponentRegistry = import ./runtime_component_registry.nix {
    lib = pkgs.lib;
    inherit components;
  };
  cursorsEnabled = runtimeComponentRegistry.manifest.cursors.enabled;
  runtimeDeps = runtimeToolRegistry.runtimePackages ++ extraRuntimePackages;
  runtimeIdentityJson = builtins.toJSON (
    {
      schema_version = 1;
      runtime_variant = runtimeVariant;
    } // runtimeIdentity // runtimeToolRegistry.terminalPackageRuntimeIdentity
  );
  runtimeBinDirs = map (pkg: "${pkg}/bin") runtimeDeps;
  escapedRuntimeBinDirs = pkgs.lib.escapeShellArgs runtimeBinDirs;
  exportedRuntimeCommands = runtimeToolRegistry.exportedCommands ++ extraRuntimeCommands;
  escapedExportedRuntimeCommands = pkgs.lib.escapeShellArgs exportedRuntimeCommands;
  yaziAssetsRoot =
    if yaziAssets == null then
      "${src}/configs/yazi"
    else
      "${yaziAssets}/share/yazelix_yazi_assets";
  paneOrchestratorWasm = zellijPluginArtifacts.pane_orchestrator or null;
  yzppWasm = zellijPluginArtifacts.yzpp or null;
  requirePluginArtifact =
    name: value:
    if value == null then
      throw "Missing first-party Zellij plugin package artifact `${name}`"
    else
      value;
  requireScreenAssets =
    if screenAssets == null then
      throw "Missing yazelix-screen package for child-owned screen assets"
    else
      screenAssets;
in
pkgs.runCommand name { } ''
  mkdir -p "$out"

  mkdir -p "$out/assets"
  ln -s ${src}/assets/icons "$out/assets/icons"
  mkdir -p "$out/assets/third_party"
  if [ -e "${requireScreenAssets}/share/yazelix_screen/ascii_magician_1mposter.gif" ]; then
    ln -s "${requireScreenAssets}/share/yazelix_screen/ascii_magician_1mposter.gif" \
      "$out/assets/third_party/ascii_magician_1mposter.gif"
  fi
  if [ -e "${requireScreenAssets}/share/yazelix_screen/ascii_magician_1mposter_frames" ]; then
    ln -s "${requireScreenAssets}/share/yazelix_screen/ascii_magician_1mposter_frames" \
      "$out/assets/third_party/ascii_magician_1mposter_frames"
  fi
  ln -s ${src}/config_metadata "$out/config_metadata"
  mkdir -p "$out/configs"
  for config_entry in ${src}/configs/*; do
    config_name="$(basename "$config_entry")"
    if [ "$config_name" = "yazi" ] || [ "$config_name" = "zellij" ]; then
      continue
    fi
    ln -s "$config_entry" "$out/configs/$config_name"
  done
  mkdir -p "$out/configs/zellij/plugins"
  for zellij_entry in ${src}/configs/zellij/*; do
    zellij_name="$(basename "$zellij_entry")"
    if [ "$zellij_name" = "plugins" ]; then
      continue
    fi
    ln -s "$zellij_entry" "$out/configs/zellij/$zellij_name"
  done
  ln -s "${requirePluginArtifact "pane_orchestrator" paneOrchestratorWasm}" "$out/configs/zellij/plugins/yazelix_pane_orchestrator.wasm"
  ln -s "${requirePluginArtifact "yzpp" yzppWasm}" "$out/configs/zellij/plugins/yzpp.wasm"
  ln -s "${src}/configs/zellij/plugins/zjstatus.wasm" "$out/configs/zellij/plugins/zjstatus.wasm"

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
  mkdir -p "$out/docs"
  ln -s ${src}/docs/upgrade_notes.toml "$out/docs/upgrade_notes.toml"
  ln -s ${src}/nushell "$out/nushell"
  ln -s ${src}/shells "$out/shells"

  ln -s ${src}/CHANGELOG.md "$out/CHANGELOG.md"
  ln -s ${src}/settings_default.jsonc "$out/settings_default.jsonc"
  ${pkgs.lib.optionalString cursorsEnabled ''
    ln -s ${src}/yazelix_ghostty_cursors_default.toml "$out/yazelix_ghostty_cursors_default.toml"
  ''}
  printf '%s\n' ${pkgs.lib.escapeShellArg runtimeVariant} > "$out/runtime_variant"
  printf '%s\n' ${pkgs.lib.escapeShellArg runtimeIdentityJson} > "$out/runtime_identity.json"
  printf '%s\n' ${pkgs.lib.escapeShellArg runtimeComponentRegistry.manifestJson} > "$out/runtime_components.json"
  printf '%s\n' ${pkgs.lib.escapeShellArg runtimeToolRegistry.manifestJson} > "$out/runtime_tools.json"
  ${pkgs.lib.optionalString enableZellijKittyPassthrough ''
    mkdir -p "$out/runtime_features"
    touch "$out/runtime_features/zellij_kitty_passthrough"
  ''}
  ${pkgs.lib.optionalString (runtimeVariant == "yzxterm" && yazelixTerminalPackage != null) ''
    mkdir -p "$out/share"
    ln -sfn "${yazelixTerminalPackage}/share/yazelix-terminal" \
      "$out/share/yazelix-terminal"
  ''}

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
  if [ -x "$out/shells/posix/yazelix_hx.sh" ] && [ -e "$out/libexec/hx" ]; then
    ln -sfn "$out/shells/posix/yazelix_hx.sh" "$out/toolbin/hx"
    if [ -e "$out/toolbin/helix" ]; then
      ln -sfn "$out/shells/posix/yazelix_hx.sh" "$out/toolbin/helix"
    fi
  fi

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
