{
  config,
  cfg,
  fenixPkgs ? null,
  lib,
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
  desktopEntryKey = terminal: "com.yazelix.Yazelix.${terminalDesktopIdSuffix terminal}";
  desktopEntryName = terminal: "New Yazelix - ${terminalDesktopLabel terminal}";
  startupWmClassFor =
    terminal:
    if terminal == "yzxterm"
    then desktopEntryKey terminal
    else "com.yazelix.Yazelix";
  yzxtermActiveFor = terminal: terminal == "yzxterm";
  yzxtermConfigured =
    yzxtermActiveFor cfg.terminal || builtins.elem "yzxterm" extraTerminalLaunchers;
  yzxtermProfileActiveFor = terminal: yzxtermActiveFor terminal && cfg.yzxterm_profile != "full";
  yzxtermProfileActive = yzxtermProfileActiveFor cfg.terminal;
  yzxtermProfileExport =
    lib.optionalString yzxtermProfileActive
      "export YAZELIX_TERMINAL_PROFILE=${cfg.yzxterm_profile}";
  yzxtermAppearanceExport =
    lib.optionalString yzxtermConfigured
      "export YAZELIX_TERMINAL_APPEARANCE=${cfg.appearance_mode}";
  yzxtermEmojiFontExport =
    lib.optionalString yzxtermConfigured
      "export YAZELIX_TERMINAL_EMOJI_FONT=${cfg.yzxterm_emoji_font}";

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

  yzxtermPackageArgs = lib.optionalAttrs (cfg.yzxterm_package != null) {
    yazelixTerminalPackage = cfg.yzxterm_package;
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
    // yzxtermPackageArgs;
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

  extraTerminalLaunchers = lib.unique cfg.extra_terminal_launchers;
  activationTerminalVariants = [ cfg.terminal ] ++ extraTerminalLaunchers;
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
        + lib.optionalString (yzxtermActiveFor terminal) " YAZELIX_TERMINAL_APPEARANCE=${cfg.appearance_mode}"
        + lib.optionalString (yzxtermActiveFor terminal) " YAZELIX_TERMINAL_EMOJI_FONT=${cfg.yzxterm_emoji_font}"
        + lib.optionalString (yzxtermProfileActiveFor terminal) " YAZELIX_TERMINAL_PROFILE=${cfg.yzxterm_profile}";
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
        ++ lib.optional (terminal == "yzxterm") "YAZELIX_TERMINAL_APP_ID=${startupWmClassFor terminal}"
        ++ lib.optional (terminal == "yzxterm") "YAZELIX_TERMINAL_APPEARANCE=${cfg.appearance_mode}"
        ++ lib.optional (terminal == "yzxterm") "YAZELIX_TERMINAL_EMOJI_FONT=${cfg.yzxterm_emoji_font}"
        ++ lib.optional (yzxtermProfileActiveFor terminal) "YAZELIX_TERMINAL_PROFILE=${cfg.yzxterm_profile}";
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
    }) extraTerminalLaunchers
  );

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
      assertion = cfg.yzxterm_package == null || cfg.package == null;
      message = "programs.yazelix.yzxterm_package cannot be combined with programs.yazelix.package; use the narrow yzxterm_package override or a whole Yazelix package replacement, not both";
    }
    {
      assertion =
        cfg.yzxterm_package == null
        || cfg.terminal == "yzxterm"
        || builtins.elem "yzxterm" cfg.extra_terminal_launchers;
      message = "programs.yazelix.yzxterm_package applies only when terminal = \"yzxterm\" or extra_terminal_launchers contains \"yzxterm\"";
    }
  ];
in
{
  inherit agentUsageProgramNames;

  baseConfig = {
    home.packages = [ yazelixPackage ] ++ cursorGeneratorPackage;
    home.sessionVariables = mkMerge [
      (mkIf yzxtermConfigured {
        YAZELIX_TERMINAL_APPEARANCE = mkDefault cfg.appearance_mode;
        YAZELIX_TERMINAL_EMOJI_FONT = mkDefault cfg.yzxterm_emoji_font;
      })
      (mkIf yzxtermProfileActive {
        YAZELIX_TERMINAL_PROFILE = mkDefault cfg.yzxterm_profile;
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
      ${yzxtermAppearanceExport}
      ${yzxtermEmojiFontExport}
      ${yzxtermProfileExport}

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
