{
  config,
  cfg,
  fenixPkgs ? null,
  lib,
  marsTerminalPackage ? null,
  mkYazelixPackage ? null,
  nixgl ? null,
  options,
  pkgs,
  rioPackage ? pkgs.rio,
  terminalMetadata,
  yazelixHelixPackage ? null,
  yazelixCursorsPackage ? null,
}:

with lib;

let
  isLinux = pkgs.stdenv.hostPlatform.isLinux;
  componentEnabled = name: cfg.components.${name} or true;
  runtimeToolSource = name: cfg.runtime_tool_sources.${name} or "bundled";
  terminalDesktopLabel = terminalMetadata.desktopLabel;
  terminalDesktopIdSuffix = terminalMetadata.desktopIdSuffix;
  marsTerminalVariant = "mars";
  desktopEntryKey = terminal: "com.yazelix.Yazelix.${terminalDesktopIdSuffix terminal}";
  desktopEntryName = terminal: "New Yazelix - ${terminalDesktopLabel terminal}";
  startupWmClassFor =
    terminal:
    if terminal == marsTerminalVariant
    then desktopEntryKey terminal
    else "com.yazelix.Yazelix";
  marsActiveFor = terminal: terminal == marsTerminalVariant;
  extraTerminalLaunchers = lib.unique cfg.extra_terminal_launchers;
  marsDesktopPackage =
    if cfg.mars_package != null then cfg.mars_package else marsTerminalPackage;
  desktopTerminalLaunchers = extraTerminalLaunchers;
  marsConfigured =
    marsActiveFor cfg.terminal || builtins.elem marsTerminalVariant desktopTerminalLaunchers;
  marsProfileActiveFor = terminal: marsActiveFor terminal && cfg.mars_profile != "full";
  marsProfileActive = marsProfileActiveFor cfg.terminal;
  marsProfileExport =
    lib.optionalString marsProfileActive
      "export MARS_PROFILE=${cfg.mars_profile}";
  marsSemanticEnvActive = marsConfigured && cfg.manage_config;
  marsAppearanceExport =
    lib.optionalString marsSemanticEnvActive
      "export MARS_APPEARANCE=${cfg.appearance_mode}";
  marsEmojiFontExport =
    lib.optionalString marsSemanticEnvActive
      "export MARS_EMOJI_FONT=${cfg.mars_emoji_font}";
  marsEmojiFontSourceExport =
    lib.optionalString marsSemanticEnvActive
      "export MARS_EMOJI_FONT_SOURCE=home-manager";

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
      inherit rioPackage yazelixHelixPackage;
    }
    // marsPackageArgs;
  yazelixPackage =
    if cfg.package != null then
      cfg.package
    else if mkYazelixPackage != null then
      mkYazelixPackage packageBuilderArgs
    else
      import ../yazelix_package.nix (
        packageBuilderArgs
        // {
          inherit fenixPkgs nixgl;
        }
      );

  packageArgsForTerminal =
    terminal:
    packageBuilderArgs
    // {
      runtimeVariant = terminal;
      name = "yazelix-${terminal}";
      runtimeName = "yazelix-runtime-${terminal}";
    };
  yazelixPackageForTerminal =
    terminal:
    if terminal == cfg.terminal then
      yazelixPackage
    else if mkYazelixPackage != null then
      mkYazelixPackage (packageArgsForTerminal terminal)
    else
      import ../yazelix_package.nix (
        packageArgsForTerminal terminal
        // {
          inherit fenixPkgs nixgl;
        }
      );

  activationTerminalVariants = [ cfg.terminal ] ++ desktopTerminalLaunchers;
  runtimeConfigGenerationPath = lib.makeBinPath [
    pkgs.coreutils
    pkgs.zellij
  ];
  terminalMaterializationActivation = lib.concatMapStringsSep "\n" (
    terminal:
    let
      terminalPackage = yazelixPackageForTerminal terminal;
      terminalEnv =
        ''PATH="${terminalPackage}/toolbin:${terminalPackage}/libexec:${terminalPackage}/bin:${runtimeConfigGenerationPath}:$PATH" YAZELIX_RUNTIME_DIR="${terminalPackage}"''
        + lib.optionalString (marsActiveFor terminal && cfg.manage_config) " MARS_APPEARANCE=${cfg.appearance_mode}"
        + lib.optionalString (marsActiveFor terminal && cfg.manage_config) " MARS_EMOJI_FONT=${cfg.mars_emoji_font}"
        + lib.optionalString (marsActiveFor terminal && cfg.manage_config) " MARS_EMOJI_FONT_SOURCE=home-manager"
        + lib.optionalString (marsProfileActiveFor terminal) " MARS_PROFILE=${cfg.mars_profile}";
    in
    ''
          $DRY_RUN_CMD env ${terminalEnv} ${terminalPackage}/libexec/yzx_core terminal-materialization.generate --from-env >/dev/null
    ''
  ) activationTerminalVariants;

  desktopExecFor =
    terminal: yzxPath: skipStableWrapperRedirect:
    let
      envVars =
        lib.optional skipStableWrapperRedirect "YAZELIX_SKIP_STABLE_WRAPPER_REDIRECT=1"
        ++ lib.optional (terminal == marsTerminalVariant) "MARS_APP_ID=${startupWmClassFor terminal}"
        ++ lib.optional (terminal == marsTerminalVariant && cfg.manage_config) "MARS_APPEARANCE=${cfg.appearance_mode}"
        ++ lib.optional (terminal == marsTerminalVariant && cfg.manage_config) "MARS_EMOJI_FONT=${cfg.mars_emoji_font}"
        ++ lib.optional (terminal == marsTerminalVariant && cfg.manage_config) "MARS_EMOJI_FONT_SOURCE=home-manager"
        ++ lib.optional (marsProfileActiveFor terminal) "MARS_PROFILE=${cfg.mars_profile}";
    in
    "${lib.optionalString (envVars != [ ]) "env ${lib.concatStringsSep " " envVars} "}${yzxPath} desktop launch";
  desktopEntryFor =
    terminal: yzxPath: skipStableWrapperRedirect:
    {
      name = desktopEntryName terminal;
      comment = "Yazi + Zellij + Helix integrated terminal environment";
      exec = desktopExecFor terminal yzxPath skipStableWrapperRedirect;
      icon = "yazelix";
      categories = [ "Development" ];
      type = "Application";
      terminal = true;
      settings = {
        StartupWMClass = startupWmClassFor terminal;
      };
    };
  extraDesktopEntries = lib.listToAttrs (
    map (terminal: {
      name = desktopEntryKey terminal;
      value = desktopEntryFor terminal "${yazelixPackageForTerminal terminal}/bin/yzx" true;
    }) desktopTerminalLaunchers
  );
  marsDesktopPackages = [ ];

  cursorGeneratorPackage =
    if componentEnabled "cursors" && yazelixCursorsPackage != null then
      [ yazelixCursorsPackage ]
    else
      [ ];
  cursorConfigRoot = "${config.xdg.configHome}/yazelix_cursors";
  cursorConfigPath = "${cursorConfigRoot}/settings.jsonc";
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
      assertion = (componentEnabled "screen") || cfg.skip_welcome_screen;
      message = "programs.yazelix.components.screen = false requires programs.yazelix.skip_welcome_screen = true";
    }
    {
      assertion = (componentEnabled "screen") || !cfg.screen_saver_enabled;
      message = "programs.yazelix.components.screen = false requires programs.yazelix.screen_saver_enabled = false";
    }
    {
      assertion = (runtimeToolSource "macchina") != "off" || !cfg.show_macchina_on_welcome;
      message = "programs.yazelix.runtime_tool_sources.macchina = \"off\" requires programs.yazelix.show_macchina_on_welcome = false";
    }
    {
      assertion = (builtins.length cfg.extra_terminal_launchers) == (builtins.length extraTerminalLaunchers);
      message = "programs.yazelix.extra_terminal_launchers must not contain duplicate terminal variants";
    }
    {
      assertion = !(builtins.elem cfg.terminal cfg.extra_terminal_launchers);
      message = "programs.yazelix.extra_terminal_launchers must not include programs.yazelix.terminal; the active terminal already gets a desktop launcher";
    }
    {
      assertion = isLinux || cfg.extra_terminal_launchers == [ ];
      message = "programs.yazelix.extra_terminal_launchers is only supported on Linux desktop environments";
    }
    {
      assertion = cfg.mars_package == null || cfg.package == null;
      message = "programs.yazelix.mars_package cannot be combined with programs.yazelix.package; use the narrow mars_package override or a whole Yazelix package replacement, not both";
    }
    {
      assertion = cfg.mars_package == null;
      message = "programs.yazelix.mars_package is dormant while Mars is not a shipped Yazelix terminal variant";
    }
  ];
in
{
  inherit agentUsageProgramNames;

  baseConfig = {
    home.packages = [ yazelixPackage ] ++ cursorGeneratorPackage ++ marsDesktopPackages;
    home.sessionVariables = mkMerge [
      (mkIf marsSemanticEnvActive {
        MARS_APPEARANCE = mkDefault cfg.appearance_mode;
        MARS_EMOJI_FONT = mkDefault cfg.mars_emoji_font;
      })
      (mkIf marsProfileActive {
        MARS_PROFILE = mkDefault cfg.mars_profile;
      })
    ];
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
      ${marsAppearanceExport}
      ${marsEmojiFontExport}
      ${marsEmojiFontSourceExport}
      ${marsProfileExport}

      $DRY_RUN_CMD ${runtimeYzxCore} runtime-materialization.repair --from-env --force --summary
${terminalMaterializationActivation}
${cursorGeneratorActivation}
      $DRY_RUN_CMD env YAZELIX_QUIET_MODE=true ${runtimeYzxControl} generate_shell_initializers
    '';
  };

  desktopConfig = mkIf isLinux (
    lib.optionalAttrs (lib.hasAttrByPath [ "xdg" "desktopEntries" ] options) {
      xdg.desktopEntries =
        {
          ${desktopEntryKey cfg.terminal} =
            desktopEntryFor cfg.terminal "${config.home.profileDirectory}/bin/yzx" false;
        }
        // extraDesktopEntries;
    }
  );
}
