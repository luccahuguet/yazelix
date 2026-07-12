{
  config,
  cfg,
  lib,
  options,
  pkgs,
}:

let
  isLinux = pkgs.stdenv.hostPlatform.isLinux;
  desktopEntryKey = "com.yazelix.Yazelix.Mars";
  yazelixPackage = cfg.package;
  stateRoot = "${config.xdg.dataHome}/yazelix";
  logsPath = "${stateRoot}/logs";
  managedConfigRoot = "${config.xdg.configHome}/yazelix";
in
{
  baseConfig = {
    home.packages = [ yazelixPackage ];

    xdg.dataFile."icons/hicolor/48x48/apps/yazelix.png".source =
      ../assets/icons/48x48/yazelix.png;
    xdg.dataFile."icons/hicolor/64x64/apps/yazelix.png".source =
      ../assets/icons/64x64/yazelix.png;
    xdg.dataFile."icons/hicolor/128x128/apps/yazelix.png".source =
      ../assets/icons/128x128/yazelix.png;
    xdg.dataFile."icons/hicolor/256x256/apps/yazelix.png".source =
      ../assets/icons/256x256/yazelix.png;

    home.activation.yazelixGeneratedRuntimeConfigs = lib.hm.dag.entryAfter [ "linkGeneration" ] ''
      export PATH="${yazelixPackage}/toolbin:${yazelixPackage}/libexec:${yazelixPackage}/bin:$PATH"
      export YAZELIX_RUNTIME_DIR="${yazelixPackage}"
      export YAZELIX_CONFIG_DIR="${managedConfigRoot}"
      export YAZELIX_STATE_DIR="${stateRoot}"
      export YAZELIX_LOGS_DIR="${logsPath}"
      $DRY_RUN_CMD ${yazelixPackage}/libexec/yzx_core runtime-materialization.repair --from-env --force --summary
      $DRY_RUN_CMD env YAZELIX_QUIET_MODE=true ${yazelixPackage}/libexec/yzx_control generate_shell_initializers
    '';
  };

  desktopConfig = lib.mkIf isLinux (
    lib.optionalAttrs (lib.hasAttrByPath [ "xdg" "desktopEntries" ] options) {
      xdg.desktopEntries.${desktopEntryKey} = {
        name = "New Yazelix - Mars";
        comment = "Yazi + Zellij + Helix integrated terminal environment";
        exec = "env MARS_APP_ID=${desktopEntryKey} ${config.home.profileDirectory}/bin/yzx desktop launch";
        icon = "yazelix";
        categories = [ "Development" ];
        type = "Application";
        terminal = false;
        settings.StartupWMClass = desktopEntryKey;
      };
    }
  );
}
