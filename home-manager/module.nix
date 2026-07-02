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
in {
  options.programs.yazelix = {
    enable = lib.mkEnableOption "Yazelix Next";

    package = lib.mkOption {
      type = lib.types.package;
      default = defaultPackageFor pkgs.stdenv.hostPlatform.system;
      defaultText = lib.literalExpression "inputs.yazelix-next.packages.\${pkgs.stdenv.hostPlatform.system}.yzn";
      description = ''
        Yazelix package to install. The package owns the command and desktop
        entry; this module does not manage Yazelix runtime configuration.
      '';
    };
  };

  config = lib.mkIf cfg.enable {
    home.packages = [cfg.package];
  };
}
