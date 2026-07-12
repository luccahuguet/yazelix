{
  config,
  cfg,
  lib,
  marsTerminalPackage ? null,
  mkYazelixPackage ? null,
  options,
  pkgs,
  yazelixHelixPackage ? null,
  yazelixCursorsPackage ? null,
}:

with lib;

let
  isLinux = pkgs.stdenv.hostPlatform.isLinux;
  componentEnabled = name: cfg.components.${name} or true;
  runtimeToolSource = name: cfg.runtime_tool_sources.${name} or "bundled";
  desktopEntryKey = "com.yazelix.Yazelix.Mars";
  desktopEntryName = "New Yazelix - Mars";
  marsDesktopPackage =
    if cfg.mars_package != null then cfg.mars_package else marsTerminalPackage;
  marsConfigured = cfg.terminal == "mars";

  agentUsageProgramNames = [
    "tokenusage"
  ];
  agentUsagePackageMap = {
    tokenusage = import ../packaging/tokenusage.nix { inherit pkgs; };
  };
  selectedAgentUsagePackages =
    map (
      program:
      if builtins.hasAttr program agentUsagePackageMap then
        builtins.getAttr program agentUsagePackageMap
      else
        throw "programs.yazelix.agent_usage_programs contains an unsupported agent usage program"
    ) cfg.agent_usage_programs;

  marsPackageArgs = lib.optionalAttrs (marsDesktopPackage != null) {
    marsTerminalPackage = marsDesktopPackage;
  };
  packageBuilderArgs =
    {
      inherit pkgs;
      runtimeVariant = cfg.terminal;
      runtimeToolSources = cfg.runtime_tool_sources;
      components = cfg.components;
      extraRuntimePackages = selectedAgentUsagePackages;
      inherit yazelixHelixPackage;
    }
    // marsPackageArgs;
  yazelixPackage =
    if cfg.package != null then
      cfg.package
    else if mkYazelixPackage != null then
      mkYazelixPackage packageBuilderArgs
    else
      throw "programs.yazelix requires the Yazelix flake Home Manager module or an explicit programs.yazelix.package";

  runtimeConfigGenerationPath = lib.makeBinPath [
    pkgs.coreutils
    pkgs.zellij
  ];
  desktopExec =
    let
      envVars =
        lib.optional marsConfigured "MARS_APP_ID=${desktopEntryKey}";
    in
    "${lib.optionalString (envVars != [ ]) "env ${lib.concatStringsSep " " envVars} "}${config.home.profileDirectory}/bin/yzx desktop launch";
  desktopEntry = {
    name = desktopEntryName;
    comment = "Yazi + Zellij + Helix integrated terminal environment";
    exec = desktopExec;
    icon = "yazelix";
    categories = [ "Development" ];
    type = "Application";
    terminal = false;
    settings = {
      StartupWMClass = desktopEntryKey;
    };
  };
  cursorGeneratorPackage =
    if componentEnabled "cursors" && yazelixCursorsPackage != null then
      [ yazelixCursorsPackage ]
    else
      [ ];
  cursorConfigRoot = "${config.xdg.configHome}/yazelix";
  cursorConfigPath = "${cursorConfigRoot}/cursors.toml";
  cursorGeneratorActivation = lib.optionalString (cursorGeneratorPackage != [ ]) ''
        if [ -f ${lib.escapeShellArg cursorConfigPath} ]; then
          $DRY_RUN_CMD ${yazelixCursorsPackage}/bin/yzc --config-dir ${lib.escapeShellArg cursorConfigRoot} generate ghostty >/dev/null
        fi
  '';
  stateRoot = "${config.xdg.dataHome}/yazelix";
  logsPath = "${stateRoot}/logs";
  managedConfigRoot = "${config.xdg.configHome}/yazelix";
  runtimeYzxCore = "${yazelixPackage}/libexec/yzx_core";
  runtimeYzxControl = "${yazelixPackage}/libexec/yzx_control";

  assertions = [
    {
      assertion = (componentEnabled "cursors") || !cfg.manage_cursor_config;
      message = "programs.yazelix.manage_cursor_config requires programs.yazelix.components.cursors to remain enabled";
    }
    {
      assertion = (componentEnabled "screen") || cfg.skip_welcome_screen == true;
      message = "programs.yazelix.components.screen = false requires programs.yazelix.skip_welcome_screen = true";
    }
    {
      assertion = (componentEnabled "screen") || cfg.screen_saver_enabled == false;
      message = "programs.yazelix.components.screen = false requires programs.yazelix.screen_saver_enabled = false";
    }
    {
      assertion = (runtimeToolSource "macchina") != "off" || cfg.show_macchina_on_welcome == false;
      message = "programs.yazelix.runtime_tool_sources.macchina = \"off\" requires programs.yazelix.show_macchina_on_welcome = false";
    }
    {
      assertion = cfg.mars_package == null || cfg.package == null;
      message = "programs.yazelix.mars_package cannot be combined with programs.yazelix.package; use the narrow mars_package override or a whole Yazelix package replacement, not both";
    }
  ];
in
{
  inherit agentUsageProgramNames;

  baseConfig = {
    home.packages = [ yazelixPackage ] ++ cursorGeneratorPackage;
    inherit assertions;

    xdg.dataFile."icons/hicolor/48x48/apps/yazelix.png".source =
      ../assets/icons/48x48/yazelix.png;
    xdg.dataFile."icons/hicolor/64x64/apps/yazelix.png".source =
      ../assets/icons/64x64/yazelix.png;
    xdg.dataFile."icons/hicolor/128x128/apps/yazelix.png".source =
      ../assets/icons/128x128/yazelix.png;
    xdg.dataFile."icons/hicolor/256x256/apps/yazelix.png".source =
      ../assets/icons/256x256/yazelix.png;

    home.activation.yazelixGeneratedRuntimeConfigs = lib.hm.dag.entryAfter [ "linkGeneration" ] ''
      export PATH="${yazelixPackage}/toolbin:${yazelixPackage}/libexec:${yazelixPackage}/bin:${runtimeConfigGenerationPath}:$PATH"
      export YAZELIX_RUNTIME_DIR="${yazelixPackage}"
      export YAZELIX_CONFIG_DIR="${managedConfigRoot}"
      export YAZELIX_STATE_DIR="${stateRoot}"
      export YAZELIX_LOGS_DIR="${logsPath}"
      $DRY_RUN_CMD ${runtimeYzxCore} runtime-materialization.repair --from-env --force --summary
${cursorGeneratorActivation}
      $DRY_RUN_CMD env YAZELIX_QUIET_MODE=true ${runtimeYzxControl} generate_shell_initializers
    '';
  };

  desktopConfig = mkIf isLinux (
    lib.optionalAttrs (lib.hasAttrByPath [ "xdg" "desktopEntries" ] options) {
      xdg.desktopEntries =
        {
          ${desktopEntryKey} = desktopEntry;
        };
    }
  );
}
