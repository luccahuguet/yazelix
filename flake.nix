{
  description = "Yazelix flake interface";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    nixgl.url = "github:guibou/nixGL";
  };

  outputs =
    {
      self,
      nixpkgs,
      nixgl,
    }:
    let
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];
      forAllSystems = nixpkgs.lib.genAttrs systems;
      mkPkgs = system: nixpkgs.legacyPackages.${system};
      homeManagerModule = import ./home_manager/module.nix;
      runtimePackage = pkgs: import ./yazelix_runtime_package.nix { inherit pkgs nixgl; };
      yazelixPackage = pkgs: import ./yazelix_package.nix { inherit pkgs nixgl; };
    in
    {
      packages = forAllSystems (
        system:
        let
          pkgs = mkPkgs system;
          runtime = runtimePackage pkgs;
          yazelix = yazelixPackage pkgs;
          install = pkgs.writeShellScriptBin "yazelix-install" (
            builtins.replaceStrings
              [
                "@runtime@"
                "@coreutils_bin@"
                "@nu_bin@"
                "@zellij_bin@"
              ]
              [
                "${runtime}"
                "${pkgs.coreutils}/bin"
                "${pkgs.nushell}/bin/nu"
                "${pkgs.zellij}/bin"
              ]
              (builtins.readFile ./shells/posix/install_yazelix.sh.in)
          );
        in
        {
          default = yazelix;
          runtime = runtime;
          yazelix = yazelix;
          install = install;
        }
      );

      apps = forAllSystems (system: {
        default = {
          type = "app";
          program = "${self.packages.${system}.yazelix}/bin/yzx";
        };
        yazelix = {
          type = "app";
          program = "${self.packages.${system}.yazelix}/bin/yzx";
        };
        install = {
          type = "app";
          program = "${self.packages.${system}.install}/bin/yazelix-install";
        };
      });

      homeManagerModules.default = homeManagerModule;
      homeManagerModules.yazelix = homeManagerModule;
    };
}
