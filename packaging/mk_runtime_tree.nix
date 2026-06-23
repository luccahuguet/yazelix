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
  rioPackage ? pkgs.rio,
  yazelixHelixPackage ? null,
  yazelixCursorsPackage ? null,
  marsTerminalPackage ? null,
  zellijPluginArtifacts ? { },
  enableZellijKittyPassthrough ? false,
}:

let
  runtimeToolRegistry = import ./runtime_tool_registry.nix {
    inherit pkgs nixgl rioPackage runtimeVariant runtimeToolSources marsTerminalPackage;
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
        source = "${src}/config_metadata";
        target = "config_metadata";
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
    }
    ++ pkgs.lib.optionals (runtimeVariant == "rio") [
      {
        source = "${pkgs.nerd-fonts.fira-code}/share/fonts/truetype/NerdFonts/FiraCode";
        target = "share/yazelix/rio_fonts/fira_code_nerd";
      }
      {
        source = "${pkgs.nerd-fonts.symbols-only}/share/fonts/truetype/NerdFonts/Symbols";
        target = "share/yazelix/rio_fonts/symbols_nerd";
      }
      {
        source = "${pkgs.noto-fonts-color-emoji}/share/fonts/noto";
        target = "share/yazelix/rio_fonts/noto_color_emoji";
      }
    ];
  renderRuntimeInputLink =
    { source, target }:
    ''
      link_runtime_input ${pkgs.lib.escapeShellArg source} ${pkgs.lib.escapeShellArg target}
    '';
  renderedRuntimeInputLinks = pkgs.lib.concatMapStrings renderRuntimeInputLink runtimeInputLinks;
  renderedTerminalAppBundleLink = pkgs.lib.optionalString (runtimeToolRegistry.terminalAppBundlePath != null) ''
    test -d ${pkgs.lib.escapeShellArg runtimeToolRegistry.terminalAppBundlePath}
    link_runtime_input ${pkgs.lib.escapeShellArg runtimeToolRegistry.terminalAppBundlePath} "Applications/Ghostty.app"
  '';
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
  ${renderedTerminalAppBundleLink}
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
  for terminal_entry in ${src}/configs/terminal_emulators/*; do
    terminal_name="$(basename "$terminal_entry")"
    if [ "$terminal_name" = "ghostty" ]; then
      link_runtime_input "$terminal_entry/config" "configs/terminal_emulators/ghostty/config"
      ${pkgs.lib.optionalString cursorsEnabled ''
        ${pkgs.lib.concatMapStringsSep "\n        " (shaderFile: ''test -s "${cursorShaderRoot}/${shaderFile}"'') cursorPackageContract.requiredShaderFiles}
        test ! -e "${cursorShaderRoot}/build_shaders.nu"
        link_runtime_input "${cursorShaderRoot}" "configs/terminal_emulators/ghostty/shaders"
      ''}
    else
      link_runtime_input "$terminal_entry" "configs/terminal_emulators/$terminal_name"
    fi
  done
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
  for yazi_plugin in auto-layout.yazi git.yazi lazygit.yazi starship.yazi; do
    link_runtime_input "${yaziAssetsRoot}/plugins/$yazi_plugin" "configs/yazi/plugins/$yazi_plugin"
  done
  printf '%s\n' ${pkgs.lib.escapeShellArg runtimeVariant} > "$out/runtime_variant"
  printf '%s\n' ${pkgs.lib.escapeShellArg runtimeIdentityJson} > "$out/runtime_identity.json"
  printf '%s\n' ${pkgs.lib.escapeShellArg runtimeComponentRegistry.manifestJson} > "$out/runtime_components.json"
  printf '%s\n' ${pkgs.lib.escapeShellArg runtimeToolRegistry.manifestJson} > "$out/runtime_tools.json"
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
  ${pkgs.lib.optionalString (rustCoreHelper != null) ''
    replace_runtime_link "${rustCoreHelper}/bin/yzx" "libexec/yzx"
    replace_runtime_link "${rustCoreHelper}/bin/yzx_core" "libexec/yzx_core"
    replace_runtime_link "${rustCoreHelper}/bin/yzx_control" "libexec/yzx_control"
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
