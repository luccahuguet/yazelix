{
  config,
  lib,
  options,
  fenixPkgs ? null,
  mkYazelixPackage ? null,
  marsTerminalPackage ? null,
  nixgl ? null,
  pkgs,
  rioPackage ? pkgs.rio,
  yazelixHelixPackage ? null,
  yazelixCursorsPackage ? null,
  ...
}:

with lib;

let
  cfg = config.programs.yazelix;
  terminalMetadata = import ../packaging/terminal_variants.nix {
    inherit (pkgs.stdenv.hostPlatform) isLinux;
  };
  defaultTerminal = terminalMetadata.default;
  terminalVariants = terminalMetadata.supported;
  terminalDescriptionBullets = lib.concatMapStringsSep "\n" (
    terminal: "        - \"${terminal}\": ${terminalMetadata.description terminal}"
  ) terminalVariants;
  runtimeToolSourceModes = [
    "bundled"
    "host"
    "off"
  ];
  yzxtermProfiles = [
    "full"
    "baseline"
    "shaders"
  ];
  yzxtermEmojiFonts = [
    "noto"
    "twitter"
    "serenityos"
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
      fenixPkgs
      lib
      marsTerminalPackage
      mkYazelixPackage
      nixgl
      options
      pkgs
      rioPackage
      terminalMetadata
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
      yzxtermEmojiFonts
      yzxtermProfiles
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
  ]);
}
