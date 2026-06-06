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
  yaziAssets ? null,
  yazelixCursorsPackage ? null,
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
  cursorPackageContract =
    if !cursorsEnabled then
      null
    else if yazelixCursorsPackage == null then
      throw "Missing yazelix_cursors package for cursor runtime assets"
    else if !(builtins.hasAttr "yazelixCursorPackageContract" yazelixCursorsPackage) then
      throw "yazelix_cursors package is missing yazelixCursorPackageContract passthru metadata"
    else
      yazelixCursorsPackage.yazelixCursorPackageContract;
  cursorShaderRoot =
    if !cursorsEnabled then
      null
    else if cursorPackageContract.schemaVersion != 1 then
      throw "Unsupported yazelix_cursors package contract schema"
    else if cursorPackageContract.packageName != "yazelix-cursors" then
      throw "Unexpected yazelix_cursors package contract packageName"
    else if cursorPackageContract.shaderRoot != "share/yazelix/yazelix_cursors/shaders" then
      throw "Unexpected yazelix_cursors shaderRoot package contract"
    else if !(builtins.elem "yzxterm" cursorPackageContract.requiredTargets) then
      throw "yazelix_cursors package contract does not declare the yzxterm cursor target"
    else if !(builtins.elem "build_shaders.nu" cursorPackageContract.forbiddenShaderFiles) then
      throw "yazelix_cursors package contract does not forbid stale build_shaders.nu shader assets"
    else
      "${yazelixCursorsPackage}/${cursorPackageContract.shaderRoot}";
  paneOrchestratorWasm = zellijPluginArtifacts.pane_orchestrator or null;
  zjstatusWasm = zellijPluginArtifacts.zjstatus or null;
  yzppWasm = zellijPluginArtifacts.yzpp or null;
  requirePluginArtifact =
    name: value:
    if value == null then
      throw "Missing first-party Zellij plugin package artifact `${name}`"
    else
      value;
in
pkgs.runCommand name { } ''
  mkdir -p "$out"

  mkdir -p "$out/assets"
  ln -s ${src}/assets/icons "$out/assets/icons"
  ln -s ${src}/config_metadata "$out/config_metadata"
  mkdir -p "$out/configs"
  for config_entry in ${src}/configs/*; do
    config_name="$(basename "$config_entry")"
    if [ "$config_name" = "yazi" ] || [ "$config_name" = "zellij" ] || [ "$config_name" = "terminal_emulators" ]; then
      continue
    fi
    ln -s "$config_entry" "$out/configs/$config_name"
  done
  mkdir -p "$out/configs/terminal_emulators"
  for terminal_entry in ${src}/configs/terminal_emulators/*; do
    terminal_name="$(basename "$terminal_entry")"
    if [ "$terminal_name" = "ghostty" ]; then
      mkdir -p "$out/configs/terminal_emulators/ghostty"
      ln -s "$terminal_entry/config" "$out/configs/terminal_emulators/ghostty/config"
      ${pkgs.lib.optionalString cursorsEnabled ''
        test -s "${cursorShaderRoot}/cursor_trail_common.glsl"
        test -s "${cursorShaderRoot}/variants/reef.glsl"
        test -s "${cursorShaderRoot}/upstream_effects/ripple_rectangle_cursor.glsl"
        test -s "${cursorShaderRoot}/generated_effects/tail.glsl"
        test ! -e "${cursorShaderRoot}/build_shaders.nu"
        ln -s "${cursorShaderRoot}" "$out/configs/terminal_emulators/ghostty/shaders"
      ''}
    else
      ln -s "$terminal_entry" "$out/configs/terminal_emulators/$terminal_name"
    fi
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
  ln -s "${requirePluginArtifact "zjstatus" zjstatusWasm}" "$out/configs/zellij/plugins/zjstatus.wasm"
  ln -s "${requirePluginArtifact "yzpp" yzppWasm}" "$out/configs/zellij/plugins/yzpp.wasm"

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
