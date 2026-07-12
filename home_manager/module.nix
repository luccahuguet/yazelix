{
  defaultPackageFor,
}:
{
  config,
  lib,
  options,
  pkgs,
  ...
}:

let
  cfg = config.programs.yazelix;
  tomlFormat = pkgs.formats.toml { };
  nativeFileOption = description:
    lib.mkOption {
      type = lib.types.nullOr (lib.types.submodule {
        options = {
          text = lib.mkOption {
            type = lib.types.nullOr lib.types.lines;
            default = null;
            description = "Inline file contents.";
          };
          source = lib.mkOption {
            type = lib.types.nullOr lib.types.path;
            default = null;
            description = "Path to a file to install.";
          };
        };
      });
      default = null;
      inherit description;
    };
  nativeFiles = {
    "yazelix/cursors.toml" = cfg.config.cursors;
    "yazelix/mars/config.toml" = cfg.config.mars;
    "yazelix/zellij/config.kdl" = cfg.config.zellij;
    "yazelix/starship.toml" = cfg.config.starship;
    "yazelix/helix/config.toml" = cfg.config.helix.config;
    "yazelix/helix/languages.toml" = cfg.config.helix.languages;
    "yazelix/helix/helix.scm" = cfg.config.helix.module;
    "yazelix/helix/init.scm" = cfg.config.helix.init;
    "yazelix/yazi/yazi.toml" = cfg.config.yazi.config;
    "yazelix/yazi/init.lua" = cfg.config.yazi.init;
    "yazelix/yazi/keymap.toml" = cfg.config.yazi.keymap;
    "yazelix/yazi/package.toml" = cfg.config.yazi.package;
    "yazelix/yazi/theme.toml" = cfg.config.yazi.theme;
    "yazelix/nu/env.nu" = cfg.config.nu.env;
    "yazelix/nu/config.nu" = cfg.config.nu.config;
  };
  nativeConfigFiles =
    lib.mapAttrs'
      (path: value:
        lib.nameValuePair path (
          lib.optionalAttrs (value.text != null) { inherit (value) text; }
          // lib.optionalAttrs (value.source != null) { inherit (value) source; }
        ))
      (lib.filterAttrs (_: value: value != null) nativeFiles);
  runtimeIntegration = import ./runtime_integration.nix {
    inherit
      cfg
      config
      lib
      options
      pkgs
      ;
  };
in
{
  _file = "yazelix/home_manager/module.nix";

  options.programs.yazelix = {
    enable = lib.mkEnableOption "Yazelix terminal environment";

    package = lib.mkOption {
      type = lib.types.package;
      default = defaultPackageFor pkgs.stdenv.hostPlatform.system;
      defaultText = lib.literalExpression "inputs.yazelix.packages.\${pkgs.stdenv.hostPlatform.system}.yazelix";
      description = "Complete Yazelix package to install.";
    };

    config = {
      settings = lib.mkOption {
        type = lib.types.nullOr tomlFormat.type;
        default = null;
        description = "Sparse semantic settings rendered to $XDG_CONFIG_HOME/yazelix/config.toml.";
      };

      cursors = nativeFileOption "Managed Yazelix cursor configuration.";
      mars = nativeFileOption "Managed sparse Mars overrides.";
      zellij = nativeFileOption "Managed Zellij config.kdl sidecar.";
      starship = nativeFileOption "Managed sparse Starship overrides for Nova staging.";

      helix = {
        config = nativeFileOption "Managed Helix config.toml.";
        languages = nativeFileOption "Managed Helix languages.toml.";
        module = nativeFileOption "Managed Helix helix.scm for Nova staging.";
        init = nativeFileOption "Managed Helix init.scm for Nova staging.";
      };

      yazi = {
        config = nativeFileOption "Managed native Yazi yazi.toml.";
        init = nativeFileOption "Managed Yazi init.lua.";
        keymap = nativeFileOption "Managed Yazi keymap.toml.";
        package = nativeFileOption "Managed Yazi package.toml metadata for Nova staging.";
        theme = nativeFileOption "Managed native Yazi theme.toml for Nova staging.";
      };

      nu = {
        env = nativeFileOption "Managed Nushell env.nu for Nova staging.";
        config = nativeFileOption "Managed Nushell config.nu for Nova staging.";
      };
    };
  };

  config = lib.mkIf cfg.enable (lib.mkMerge [
    runtimeIntegration.baseConfig
    runtimeIntegration.desktopConfig
    {
      assertions = lib.mapAttrsToList (path: value: {
        assertion =
          value == null
          || ((value.text != null) != (value.source != null));
        message = "Home Manager native file ${path} requires exactly one of text or source.";
      }) nativeFiles;
      xdg.configFile =
        lib.optionalAttrs (cfg.config.settings != null) {
          "yazelix/config.toml".source =
            tomlFormat.generate "yazelix-config.toml" cfg.config.settings;
        }
        // nativeConfigFiles;
    }
  ]);
}
