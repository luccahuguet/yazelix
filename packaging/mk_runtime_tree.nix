{
  pkgs,
  src ? ../.,
  nixgl ? null,
  name ? "yazelix-runtime",
  rustCoreHelper ? null,
  runtimeVariant ? "mars",
  runtimeToolSources ? { },
  runtimeIdentity ? { },
  components ? { },
  extraRuntimePackages ? [ ],
  extraRuntimeCommands ? [ "tu" ],
  yaziAssets ? null,
  yazelixHelixPackage ? null,
  yazelixCursorsPackage ? null,
  marsTerminalPackage ? null,
  zellijPluginArtifacts ? { },
  enableZellijKittyPassthrough ? false,
}:

let
  runtimeToolRegistry = import ./runtime_tool_registry.nix {
    inherit pkgs nixgl runtimeVariant runtimeToolSources marsTerminalPackage;
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
  yaziAssetsRuntimeToolManifest =
    if yaziAssets == null then
      { }
    else
      {
        ccboard = {
          source = "bundled";
          commands = [ "ccboard" ];
          required_commands = [ "ccboard" ];
          hostable = false;
          disableable = false;
          notes = [
            "Packaged by the yazi-assets child package under runtime_tools/ccboard."
            "Mission Control launches this tool through libexec/ccboard."
          ];
        };
        codedb = {
          source = "bundled";
          commands = [
            "codedb"
            "nu_plugin_codedb"
          ];
          required_commands = [
            "codedb"
            "nu_plugin_codedb"
          ];
          hostable = false;
          disableable = false;
          notes = [
            "Packaged by the yazi-assets child package under runtime_tools/codedb."
          ];
        };
      };
  runtimeToolManifest = runtimeToolRegistry.manifest // yaziAssetsRuntimeToolManifest;
  runtimeToolManifestJson = builtins.toJSON runtimeToolManifest;
  requiredSteelPluginIds = [
    "recentf"
    "splash"
    "spacemacs_theme"
    "keymaps"
    "labelled_buffers"
  ];
  expectedHelixSteelPluginRoot = "share/yazelix_helix/steel_plugins";
  helixPackageContract =
    if yazelixHelixPackage == null then
      throw "Missing yazelix_helix package for Helix Steel plugin defaults"
    else if !(builtins.hasAttr "yazelixHelixPackageContract" yazelixHelixPackage) then
      throw "yazelix_helix package is missing yazelixHelixPackageContract passthru metadata"
    else
      yazelixHelixPackage.yazelixHelixPackageContract;
  helixSteelPluginRoot =
    if !(builtins.hasAttr "schemaVersion" helixPackageContract) || helixPackageContract.schemaVersion != 1 then
      throw "Unsupported yazelix_helix package contract schema"
    else if !(builtins.hasAttr "packageName" helixPackageContract) || helixPackageContract.packageName != "yazelix-helix" then
      throw "Unexpected yazelix_helix package contract packageName"
    else if !(builtins.hasAttr "steelPluginRoot" helixPackageContract) || helixPackageContract.steelPluginRoot != expectedHelixSteelPluginRoot then
      throw "Unexpected yazelix_helix steelPluginRoot package contract"
    else if !(builtins.hasAttr "pluginIds" helixPackageContract) then
      throw "yazelix_helix package contract does not declare pluginIds"
    else if !(pkgs.lib.all (pluginId: builtins.elem pluginId helixPackageContract.pluginIds) requiredSteelPluginIds) then
      throw "yazelix_helix package contract does not declare all required Steel plugin ids"
    else
      "${yazelixHelixPackage}/${helixPackageContract.steelPluginRoot}";
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
    else if !(builtins.elem "mars" cursorPackageContract.requiredTargets) then
      throw "yazelix_cursors package contract does not declare the mars cursor target"
    else if !(builtins.elem "build_shaders.nu" cursorPackageContract.forbiddenShaderFiles) then
      throw "yazelix_cursors package contract does not forbid stale build_shaders.nu shader assets"
    else if !(builtins.hasAttr "requiredShaderFiles" cursorPackageContract) then
      throw "yazelix_cursors package contract does not declare requiredShaderFiles"
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
  runtimeInputLinks =
    [
      {
        source = "${src}/assets/icons";
        target = "assets/icons";
      }
      {
        source = "${src}/docs/upgrade_notes.toml";
        target = "docs/upgrade_notes.toml";
      }
      {
        source = "${src}/nushell";
        target = "nushell";
      }
      {
        source = "${src}/shells";
        target = "shells";
      }
      {
        source = "${src}/CHANGELOG.md";
        target = "CHANGELOG.md";
      }
      {
        source = "${src}/settings_default.jsonc";
        target = "settings_default.jsonc";
      }
    ]
    ++ pkgs.lib.optional cursorsEnabled {
      source = "${src}/yazelix_cursors_default.toml";
      target = "yazelix_cursors_default.toml";
    };
  renderRuntimeInputLink =
    { source, target }:
    ''
      link_runtime_input ${pkgs.lib.escapeShellArg source} ${pkgs.lib.escapeShellArg target}
    '';
  renderedRuntimeInputLinks = pkgs.lib.concatMapStrings renderRuntimeInputLink runtimeInputLinks;
in
pkgs.runCommand name { } ''
  mkdir -p "$out"

  link_runtime_input() {
    source_path="$1"
    target_path="$out/$2"
    mkdir -p "$(dirname "$target_path")"
    ln -s "$source_path" "$target_path"
  }

  replace_runtime_link() {
    source_path="$1"
    target_path="$out/$2"
    mkdir -p "$(dirname "$target_path")"
    ln -sfn "$source_path" "$target_path"
  }

  ${renderedRuntimeInputLinks}
  mkdir -p "$out/config_metadata"
  for metadata_entry in ${src}/config_metadata/*; do
    metadata_name="$(basename "$metadata_entry")"
    link_runtime_input "$metadata_entry" "config_metadata/$metadata_name"
  done
  if [ -d "${yaziAssetsRoot}/config_metadata" ]; then
    for metadata_entry in ${yaziAssetsRoot}/config_metadata/*; do
      metadata_name="$(basename "$metadata_entry")"
      link_runtime_input "$metadata_entry" "config_metadata/$metadata_name"
    done
  fi
  mkdir -p "$out/configs"
  for config_entry in ${src}/configs/*; do
    config_name="$(basename "$config_entry")"
    if [ "$config_name" = "helix" ] || [ "$config_name" = "yazi" ] || [ "$config_name" = "zellij" ] || [ "$config_name" = "terminal_emulators" ]; then
      continue
    fi
    link_runtime_input "$config_entry" "configs/$config_name"
  done
  mkdir -p "$out/configs/helix"
  for helix_entry in ${src}/configs/helix/*; do
    helix_name="$(basename "$helix_entry")"
    if [ "$helix_name" = "steel_plugins" ]; then
      continue
    fi
    link_runtime_input "$helix_entry" "configs/helix/$helix_name"
  done
  link_runtime_input "${helixSteelPluginRoot}" "configs/helix/steel_plugins"
  mkdir -p "$out/configs/terminal_emulators"
  terminal_config_root="${src}/configs/terminal_emulators/${runtimeVariant}"
  if [ -d "$terminal_config_root" ]; then
    link_runtime_input "$terminal_config_root" "configs/terminal_emulators/${runtimeVariant}"
  fi
  ${pkgs.lib.optionalString cursorsEnabled ''
    ${pkgs.lib.concatMapStringsSep "\n    " (shaderFile: ''test -s "${cursorShaderRoot}/${shaderFile}"'') cursorPackageContract.requiredShaderFiles}
    test ! -e "${cursorShaderRoot}/build_shaders.nu"
    link_runtime_input "${cursorShaderRoot}" "configs/terminal_emulators/ghostty/shaders"
  ''}
  mkdir -p "$out/configs/zellij/plugins"
  for zellij_entry in ${src}/configs/zellij/*; do
    zellij_name="$(basename "$zellij_entry")"
    if [ "$zellij_name" = "plugins" ]; then
      continue
    fi
    link_runtime_input "$zellij_entry" "configs/zellij/$zellij_name"
  done
  link_runtime_input "${requirePluginArtifact "pane_orchestrator" paneOrchestratorWasm}" "configs/zellij/plugins/yazelix_pane_orchestrator.wasm"
  link_runtime_input "${requirePluginArtifact "zjstatus" zjstatusWasm}" "configs/zellij/plugins/zjstatus.wasm"
  link_runtime_input "${requirePluginArtifact "yzpp" yzppWasm}" "configs/zellij/plugins/yzpp.wasm"

  mkdir -p "$out/configs/yazi/plugins"
  for yazi_file in README.md; do
    link_runtime_input "${src}/configs/yazi/$yazi_file" "configs/yazi/$yazi_file"
  done
  for yazi_plugin in sidebar-state.yazi sidebar-status.yazi zoxide-editor.yazi; do
    link_runtime_input "${src}/configs/yazi/plugins/$yazi_plugin" "configs/yazi/plugins/$yazi_plugin"
  done
  link_runtime_input "${yaziAssetsRoot}/flavors" "configs/yazi/flavors"
  link_runtime_input "${yaziAssetsRoot}/yazelix_starship.toml" "configs/yazi/yazelix_starship.toml"
  if [ -d "${yaziAssetsRoot}/runtime_tools" ]; then
    link_runtime_input "${yaziAssetsRoot}/runtime_tools" "runtime_tools"
  fi
  for yazi_plugin in auto-layout.yazi git.yazi lazygit.yazi smart-tabs.yazi starship.yazi; do
    link_runtime_input "${yaziAssetsRoot}/plugins/$yazi_plugin" "configs/yazi/plugins/$yazi_plugin"
  done
  printf '%s\n' ${pkgs.lib.escapeShellArg runtimeVariant} > "$out/runtime_variant"
  printf '%s\n' ${pkgs.lib.escapeShellArg runtimeIdentityJson} > "$out/runtime_identity.json"
  printf '%s\n' ${pkgs.lib.escapeShellArg runtimeComponentRegistry.manifestJson} > "$out/runtime_components.json"
  printf '%s\n' ${pkgs.lib.escapeShellArg runtimeToolManifestJson} > "$out/runtime_tools.json"
  ${pkgs.lib.optionalString enableZellijKittyPassthrough ''
    mkdir -p "$out/runtime_features"
    touch "$out/runtime_features/zellij_kitty_passthrough"
  ''}
  ${pkgs.lib.optionalString (runtimeVariant == "mars" && marsTerminalPackage != null) ''
    replace_runtime_link "${marsTerminalPackage}/share/mars" "share/mars"
  ''}

  mkdir -p "$out/libexec"
  for bin_dir in ${escapedRuntimeBinDirs}; do
    if [ -d "$bin_dir" ]; then
      for entry in "$bin_dir"/*; do
        [ -e "$entry" ] || continue
        replace_runtime_link "$entry" "libexec/$(basename "$entry")"
      done
    fi
  done
  if [ -d "$out/runtime_tools" ]; then
    for bin_dir in "$out/runtime_tools"/*/bin; do
      [ -d "$bin_dir" ] || continue
      for entry in "$bin_dir"/*; do
        [ -e "$entry" ] || continue
        replace_runtime_link "$entry" "libexec/$(basename "$entry")"
      done
    done
  fi

  if [ -e "$out/libexec/yazelix_zellij_bar_widget" ]; then
    yazelix_zellij_bar_widget_target="$(readlink "$out/libexec/yazelix_zellij_bar_widget")"
    rm -f "$out/libexec/yazelix_zellij_bar_widget"
    cat > "$out/libexec/yazelix_zellij_bar_widget" <<EOF
#!/bin/sh
PATH="$out/toolbin:$out/bin:\$PATH"
export PATH
exec "$yazelix_zellij_bar_widget_target" "\$@"
EOF
    chmod +x "$out/libexec/yazelix_zellij_bar_widget"
  fi

  link_runtime_command_alias() {
    source_name="$1"
    alias_name="$2"
    if [ -e "$out/libexec/$source_name" ] && [ ! -e "$out/libexec/$alias_name" ]; then
      replace_runtime_link "$out/libexec/$source_name" "libexec/$alias_name"
    fi
  }

  link_runtime_command_alias hx helix
  link_runtime_command_alias nvim neovim
  link_runtime_command_alias lazygit lg

  ${pkgs.lib.optionalString (rustCoreHelper != null) ''
    replace_runtime_link "${rustCoreHelper}/bin/yzx" "libexec/yzx"
    replace_runtime_link "${rustCoreHelper}/bin/yzx_core" "libexec/yzx_core"
    replace_runtime_link "${rustCoreHelper}/bin/yzx_control" "libexec/yzx_control"
  ''}
  ${pkgs.lib.optionalString cursorsEnabled ''
    replace_runtime_link "${yazelixCursorsPackage}/bin/yzc" "libexec/yzc"
  ''}

  mkdir -p "$out/toolbin"
  for command_name in ${escapedExportedRuntimeCommands}; do
    if [ -e "$out/libexec/$command_name" ]; then
      replace_runtime_link "$out/libexec/$command_name" "toolbin/$command_name"
    fi
  done
  if [ -x "$out/shells/posix/yazelix_hx.sh" ] && [ -e "$out/libexec/hx" ]; then
    replace_runtime_link "$out/shells/posix/yazelix_hx.sh" "toolbin/hx"
    if [ -e "$out/toolbin/helix" ]; then
      replace_runtime_link "$out/shells/posix/yazelix_hx.sh" "toolbin/helix"
    fi
  fi

  mkdir -p "$out/bin"
  cat > "$out/bin/yzx" <<EOF
#!/bin/sh
bootstrap_path="${pkgs.nushell}/bin:/nix/var/nix/profiles/default/bin:/run/current-system/sw/bin:/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin"
if [ -n "\''${PATH:-}" ]; then
  PATH="\$bootstrap_path:\$PATH"
else
  PATH="\$bootstrap_path"
fi
export PATH
YAZELIX_INVOKED_YZX_PATH="\$0"
export YAZELIX_INVOKED_YZX_PATH
exec "$out/shells/posix/yzx_cli.sh" "\$@"
EOF
  chmod +x "$out/bin/yzx"
''
