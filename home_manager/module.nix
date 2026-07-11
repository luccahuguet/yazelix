{
  config,
  lib,
  options,
  terminalSupport,
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
  # Single source of truth: the yazelix-terminal-support child (consumed as the
  # yazelixTerminalSupport flake input, parsed into terminalSupport).
  defaultTerminal = terminalSupport.default_terminal;
  terminalVariants = terminalSupport.launch_order;
  terminalDescriptionBullets =
    "        - \"kitty\": packaged default terminal; Yazelix launches Kitty and keeps its native config user-owned\n        - \"ghostty\": host-installed backup terminal; start Yazelix with `yzx enter`";
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
