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
    (mkIf (cfg.config.mars != null) {
      assertions = [
        {
          assertion = (cfg.config.mars.text != null) != (cfg.config.mars.source != null);
          message = "programs.yazelix.config.mars requires exactly one of text or source";
        }
      ];
      xdg.configFile."yazelix/mars/config.toml" =
        (lib.optionalAttrs (cfg.config.mars.text != null) { inherit (cfg.config.mars) text; })
        // (lib.optionalAttrs (cfg.config.mars.source != null) { inherit (cfg.config.mars) source; });
    })
  ]);
}
