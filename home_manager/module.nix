{
  config,
  lib,
  options,
  fenixPkgs ? null,
  mkYazelixPackage ? null,
  marsTerminalPackage ? null,
  nixgl ? null,
  pkgs,
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
  marsProfiles = [
    "full"
    "baseline"
    "shaders"
  ];
  defaultMarsEmojiFonts = [
    "noto"
    "twitter"
    "serenityos"
  ];
  marsPackageMetadata =
    if marsTerminalPackage != null && builtins.isAttrs (marsTerminalPackage.passthru.marsPackageMetadata or null) then
      marsTerminalPackage.passthru.marsPackageMetadata
    else
      null;
  marsEmojiFonts =
    if marsPackageMetadata != null && builtins.isList (marsPackageMetadata.supported_emoji_fonts or null) then
      marsPackageMetadata.supported_emoji_fonts
    else
      defaultMarsEmojiFonts;
  marsEmojiFontDescriptions = {
    noto = "Noto Color Emoji fallback";
    twitter = "Twitter/Twemoji color emoji fallback";
    serenityos = "SerenityOS emoji fallback";
  };
  marsEmojiFontDescriptionBullets = lib.concatMapStringsSep "\n" (
    emojiFont:
    "        - \"${emojiFont}\": ${marsEmojiFontDescriptions.${emojiFont} or "package-advertised emoji fallback"}"
  ) marsEmojiFonts;

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
      marsEmojiFontDescriptionBullets
      marsEmojiFonts
      marsProfiles
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
