{
  defaultPackageFor,
}:
{
  config,
  lib,
  pkgs,
  ...
}: let
  cfg = config.programs.yazelix;
  tomlFormat = pkgs.formats.toml {};
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
    "yazelix/cursors.toml" = {
      option = cfg.config.cursors;
      name = "programs.yazelix.config.cursors";
    };
    "yazelix/mars/config.toml" = {
      option = cfg.config.mars;
      name = "programs.yazelix.config.mars";
    };
    "yazelix/zellij/config.kdl" = {
      option = cfg.config.zellij;
      name = "programs.yazelix.config.zellij";
    };
    "yazelix/starship.toml" = {
      option = cfg.config.starship;
      name = "programs.yazelix.config.starship";
    };
    "yazelix/helix/config.toml" = {
      option = cfg.config.helix.config;
      name = "programs.yazelix.config.helix.config";
    };
    "yazelix/helix/languages.toml" = {
      option = cfg.config.helix.languages;
      name = "programs.yazelix.config.helix.languages";
    };
    "yazelix/helix/helix.scm" = {
      option = cfg.config.helix.module;
      name = "programs.yazelix.config.helix.module";
    };
    "yazelix/helix/init.scm" = {
      option = cfg.config.helix.init;
      name = "programs.yazelix.config.helix.init";
    };
    "yazelix/yazi/init.lua" = {
      option = cfg.config.yazi.init;
      name = "programs.yazelix.config.yazi.init";
    };
    "yazelix/yazi/yazi.toml" = {
      option = cfg.config.yazi.config;
      name = "programs.yazelix.config.yazi.config";
    };
    "yazelix/yazi/keymap.toml" = {
      option = cfg.config.yazi.keymap;
      name = "programs.yazelix.config.yazi.keymap";
    };
    "yazelix/yazi/package.toml" = {
      option = cfg.config.yazi.package;
      name = "programs.yazelix.config.yazi.package";
    };
    "yazelix/yazi/starship.toml" = {
      option = cfg.config.yazi.starship;
      name = "programs.yazelix.config.yazi.starship";
    };
    "yazelix/yazi/theme.toml" = {
      option = cfg.config.yazi.theme;
      name = "programs.yazelix.config.yazi.theme";
    };
    "yazelix/nu/env.nu" = {
      option = cfg.config.nu.env;
      name = "programs.yazelix.config.nu.env";
    };
    "yazelix/nu/config.nu" = {
      option = cfg.config.nu.config;
      name = "programs.yazelix.config.nu.config";
    };
  };
  nativeConfigFiles =
    lib.mapAttrs'
    (path: spec:
      lib.nameValuePair path (
        lib.optionalAttrs (spec.option.text != null) {inherit (spec.option) text;}
        // lib.optionalAttrs (spec.option.source != null) {inherit (spec.option) source;}
      ))
    (lib.filterAttrs (_: spec: spec.option != null) nativeFiles);
in {
  options.programs.yazelix = {
    enable = lib.mkEnableOption "Yazelix Nova";

    package = lib.mkOption {
      type = lib.types.package;
      default = defaultPackageFor pkgs.stdenv.hostPlatform.system;
      defaultText = lib.literalExpression "inputs.yazelix.packages.\${pkgs.stdenv.hostPlatform.system}.yazelix";
      description = ''
        Yazelix package to install. The package owns the command and any desktop
        entry it provides.
      '';
    };

    config = {
      settings = lib.mkOption {
        type = lib.types.nullOr tomlFormat.type;
        default = null;
        description = ''
          Semantic Yazelix settings rendered to
          $XDG_CONFIG_HOME/yazelix/config.toml.
        '';
      };

      cursors = nativeFileOption "Managed Yazelix cursor configuration.";
      mars = nativeFileOption "Managed sparse Mars overrides.";
      zellij = nativeFileOption "Managed Zellij config.kdl sidecar.";
      starship = nativeFileOption "Managed sparse Starship overrides.";

      helix = {
        config = nativeFileOption "Managed Helix config.toml.";
        languages = nativeFileOption "Managed Helix languages.toml.";
        module = nativeFileOption "Managed Helix helix.scm.";
        init = nativeFileOption "Managed Helix init.scm.";
      };

      yazi = {
        config = nativeFileOption "Managed native Yazi yazi.toml.";
        init = nativeFileOption "Managed Yazi init.lua.";
        keymap = nativeFileOption "Managed Yazi keymap.toml.";
        package = nativeFileOption "Managed Yazi package.toml metadata.";
        starship = nativeFileOption "Complete Starship configuration for managed Yazi.";
        theme = nativeFileOption "Managed native Yazi theme.toml.";
      };

      nu = {
        env = nativeFileOption "Managed Nushell env.nu.";
        config = nativeFileOption "Managed Nushell config.nu.";
      };
    };
  };

  config = lib.mkIf cfg.enable {
    home.packages = [cfg.package];
    assertions =
      lib.mapAttrsToList
      (_: spec: {
        assertion =
          spec.option == null
          || ((spec.option.text != null) != (spec.option.source != null));
        message = "${spec.name} requires exactly one of text or source.";
      })
      nativeFiles;
    xdg.configFile =
      lib.optionalAttrs (cfg.config.settings != null) {
        "yazelix/config.toml".source =
          tomlFormat.generate "yazelix-config.toml" cfg.config.settings;
      }
      // nativeConfigFiles;
  };
}
