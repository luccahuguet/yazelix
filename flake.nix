{
  description = "Yazelix flake interface";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs =
    {
      self,
      nixpkgs,
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
      runtimePackage = pkgs: import ./yazelix_runtime_package.nix { inherit pkgs; };
      yazelixPackage = pkgs: import ./yazelix_package.nix { inherit pkgs; };
    in
    {
      packages = forAllSystems (
        system:
        let
          pkgs = mkPkgs system;
          lockedDevenv = import ./packaging/locked_devenv_package.nix {
            inherit pkgs;
            src = ./.;
          };
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
          default = runtime;
          locked_devenv = lockedDevenv;
          runtime = runtime;
          yazelix = yazelix;
          install = install;
        }
      );

      apps = forAllSystems (system: {
        install = {
          type = "app";
          program = "${self.packages.${system}.install}/bin/yazelix-install";
        };
      });
    };
}
