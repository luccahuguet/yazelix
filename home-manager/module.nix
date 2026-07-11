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
  defaultSettings = builtins.fromTOML (builtins.readFile ../config.toml);
  contractSettings = {
    ratconfig.contract = {
      applied_change_ids = [];
      contract_id = "yazelix-next.config";
      schema_version = 1;
      version = 1;
    };
  };
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
    "yazelix-next/mars/config.toml" = {
      option = cfg.config.mars;
      name = "programs.yazelix.config.mars";
    };
    "yazelix-next/zellij/config.kdl" = {
      option = cfg.config.zellij;
      name = "programs.yazelix.config.zellij";
    };
    "yazelix-next/starship.toml" = {
      option = cfg.config.starship;
      name = "programs.yazelix.config.starship";
    };
    "yazelix-next/helix/config.toml" = {
      option = cfg.config.helix.config;
      name = "programs.yazelix.config.helix.config";
    };
    "yazelix-next/helix/languages.toml" = {
      option = cfg.config.helix.languages;
      name = "programs.yazelix.config.helix.languages";
    };
    "yazelix-next/helix/helix.scm" = {
      option = cfg.config.helix.module;
      name = "programs.yazelix.config.helix.module";
    };
    "yazelix-next/helix/init.scm" = {
      option = cfg.config.helix.init;
      name = "programs.yazelix.config.helix.init";
    };
    "yazelix-next/yazi/init.lua" = {
      option = cfg.config.yazi.init;
      name = "programs.yazelix.config.yazi.init";
    };
    "yazelix-next/yazi/yazi.toml" = {
      option = cfg.config.yazi.config;
      name = "programs.yazelix.config.yazi.config";
    };
    "yazelix-next/yazi/keymap.toml" = {
      option = cfg.config.yazi.keymap;
      name = "programs.yazelix.config.yazi.keymap";
    };
    "yazelix-next/yazi/package.toml" = {
      option = cfg.config.yazi.package;
      name = "programs.yazelix.config.yazi.package";
    };
    "yazelix-next/yazi/theme.toml" = {
      option = cfg.config.yazi.theme;
      name = "programs.yazelix.config.yazi.theme";
    };
    "yazelix-next/nu/env.nu" = {
      option = cfg.config.nu.env;
      name = "programs.yazelix.config.nu.env";
    };
    "yazelix-next/nu/config.nu" = {
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
      defaultText = lib.literalExpression "inputs.yazelix-next.packages.\${pkgs.stdenv.hostPlatform.system}.yzn";
      description = ''
        Yazelix package to install. The package owns the command and desktop
        entry.
      '';
    };

    config = {
      settings = lib.mkOption {
        type = lib.types.nullOr tomlFormat.type;
        default = null;
        description = ''
          Semantic Yazelix settings rendered to
          $XDG_CONFIG_HOME/yazelix-next/config.toml.
        '';
      };

      mars = nativeFileOption "Managed Mars config.toml.";
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
        "yazelix-next/config.toml".source =
          tomlFormat.generate "yazelix-next-config.toml" (
            (lib.recursiveUpdate defaultSettings cfg.config.settings)
            // contractSettings
          );
      }
      // nativeConfigFiles;
  };
}
