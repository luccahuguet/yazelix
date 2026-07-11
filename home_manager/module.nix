{
  config,
  lib,
  options,
  mkYazelixPackage ? null,
  marsTerminalPackage ? null,
  pkgs,
  yazelixHelixPackage ? null,
  yazelixCursorsPackage ? null,
  ...
}:

with lib;

let
  cfg = config.programs.yazelix;
  defaultTerminal = "mars";
  terminalVariants = [ "mars" ];
  terminalDescriptionBullets =
    "        - \"mars\": packaged Rust terminal merging its package base with the optional programs.yazelix.config.mars override";
  runtimeToolSourceModes = [
    "bundled"
    "host"
    "off"
  ];
  settingsContract = import ./settings_contract.nix { inherit cfg lib; };
  inherit (settingsContract)
    cursorSettingsJsonc
    mkMainContractOption
    settingsJsonc
    ;
  nativeConfig =
    name: path: value:
    mkIf (value != null) {
      assertions = [
        {
          assertion = (value.text != null) != (value.source != null);
          message = "programs.yazelix.config.${name} requires exactly one of text or source";
        }
      ];
      xdg.configFile.${path} =
        (lib.optionalAttrs (value.text != null) { inherit (value) text; })
        // (lib.optionalAttrs (value.source != null) { inherit (value) source; });
    };

  runtimeIntegration = import ./runtime_integration.nix {
    inherit
      cfg
      config
      lib
      marsTerminalPackage
      mkYazelixPackage
      options
      pkgs
      yazelixHelixPackage
      yazelixCursorsPackage
      ;
  };
in
{
  _file = "yazelix/home_manager/module.nix";

  options.programs.yazelix = import ./options.nix {
    inherit
      defaultTerminal
      lib
      mkMainContractOption
      runtimeToolSourceModes
      terminalDescriptionBullets
      terminalVariants
      ;
    inherit (runtimeIntegration) agentUsageProgramNames;
  };

  config = mkIf cfg.enable (mkMerge [
    runtimeIntegration.baseConfig
    runtimeIntegration.desktopConfig
    (mkIf cfg.manage_config {
      xdg.configFile."yazelix/settings.jsonc".text = settingsJsonc;
    })
    (mkIf cfg.manage_cursor_config {
      xdg.configFile."yazelix_cursors/settings.jsonc".text = cursorSettingsJsonc;
    })
    (nativeConfig "mars" "yazelix/mars/config.toml" cfg.config.mars)
    (nativeConfig "zellij" "yazelix/zellij/config.kdl" cfg.config.zellij)
  ]);
}
