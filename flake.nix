{
  description = "Yazelix flake interface";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    nixgl.url = "github:guibou/nixGL";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    beads = {
      url = "github:steveyegge/beads/v1.0.0";
    };
    zjstatus = {
      url = "github:dj95/zjstatus";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      nixgl,
      fenix,
      beads,
      zjstatus,
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
      maintainerShell =
        system: pkgs:
        import ./maintainer_shell.nix {
          inherit pkgs nixgl;
          lib = nixpkgs.lib;
          fenixPkgs = fenix.packages.${system};
          bdPackage = (pkgs.callPackage "${beads}/default.nix" { self = beads; }).overrideAttrs (old: {
            vendorHash = "sha256-7DJgqJX2HDa9gcGD8fLNHLIXvGAEivYeDYx3snCUyCE=";
            nativeBuildInputs = (old.nativeBuildInputs or [ ]) ++ [ pkgs.pkg-config ];
            buildInputs = (old.buildInputs or [ ]) ++ [ pkgs.icu ];
          });
          repoRoot = ./.;
        };
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
              ]
              [
                "${runtime}"
                "${pkgs.coreutils}/bin"
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

      devShells = forAllSystems (
        system:
        let
          pkgs = mkPkgs system;
        in
        {
          default = maintainerShell system pkgs;
        }
      );

      homeManagerModules.default = homeManagerModule;
      homeManagerModules.yazelix = homeManagerModule;
    };
}
