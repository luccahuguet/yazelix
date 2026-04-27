{
  description = "Yazelix flake interface";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    home-manager = {
      url = "github:nix-community/home-manager";
      inputs.nixpkgs.follows = "nixpkgs";
    };
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
      home-manager,
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
      homeManagerModule = { pkgs, ... }: {
        _module.args.nixgl = nixgl;
        _module.args.fenixPkgs = fenix.packages.${pkgs.stdenv.hostPlatform.system};
        imports = [ ./home_manager/module.nix ];
      };
      runtimePackage = system: pkgs: runtimeVariant:
        import ./yazelix_runtime_package.nix {
          inherit pkgs nixgl runtimeVariant;
          fenixPkgs = fenix.packages.${system};
        };
      yazelixPackage = system: pkgs: runtimeVariant:
        import ./yazelix_package.nix {
          inherit pkgs nixgl runtimeVariant;
          fenixPkgs = fenix.packages.${system};
        };
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
          defaultRuntimeVariant = "ghostty";
          runtime_default = runtimePackage system pkgs defaultRuntimeVariant;
          runtime_ghostty = runtimePackage system pkgs "ghostty";
          runtime_wezterm = runtimePackage system pkgs "wezterm";
          yazelix_default = yazelixPackage system pkgs defaultRuntimeVariant;
          yazelix_ghostty = yazelixPackage system pkgs "ghostty";
          yazelix_wezterm = yazelixPackage system pkgs "wezterm";
        in
        {
          default = yazelix_default;
          runtime = runtime_default;
          runtime_ghostty = runtime_ghostty;
          runtime_wezterm = runtime_wezterm;
          yazelix = yazelix_default;
          yazelix_ghostty = yazelix_ghostty;
          yazelix_wezterm = yazelix_wezterm;
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
        yazelix_ghostty = {
          type = "app";
          program = "${self.packages.${system}.yazelix_ghostty}/bin/yzx";
        };
        yazelix_wezterm = {
          type = "app";
          program = "${self.packages.${system}.yazelix_wezterm}/bin/yzx";
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
