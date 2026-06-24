{
  description = "Yazelix Next";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    mars = {
      url = "github:luccahuguet/mars";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    yazelixZellij = {
      url = "github:luccahuguet/yazelix-zellij/yazelix_kgp_preview";
      flake = false;
    };
  };

  outputs = {
    self,
    nixpkgs,
    mars,
    yazelixZellij,
  }: let
    systems = [
      "x86_64-linux"
      "aarch64-linux"
    ];
    eachSystem = nixpkgs.lib.genAttrs systems;
    mkYazelixZellij = pkgs: let
      baseZellij =
        if pkgs.zellij ? unwrapped
        then pkgs.zellij.unwrapped
        else if builtins.hasAttr "zellij-unwrapped" pkgs
        then pkgs."zellij-unwrapped"
        else pkgs.zellij;
    in
      baseZellij.overrideAttrs (_old: {
        pname = "zellij";
        version = "0.44.3";
        src = yazelixZellij;
        patches = [];
        prePatch = "";
        postPatch = "";
        installCheckPhase = ''
          runHook preInstallCheck
          runHook postInstallCheck
        '';
        cargoDeps = pkgs.rustPlatform.fetchCargoVendor {
          pname = "zellij";
          version = "0.44.3";
          src = yazelixZellij;
          hash = "sha256-966FpfSsF9I10SrYe3+YNsfM2kLLv+gd0/Aw8vLp4Lk=";
        };
        doCheck = false;
      });
  in {
    packages = eachSystem (system: let
      pkgs = import nixpkgs {inherit system;};
      marsPackage = mars.packages.${system}.mars;
      yazelixZellijPackage = mkYazelixZellij pkgs;
      yzn = pkgs.writeShellApplication {
        name = "yzn";
        text = ''
          exec ${marsPackage}/bin/mars -e ${yazelixZellijPackage}/bin/zellij "$@"
        '';
      };
    in {
      yazelix_zellij = yazelixZellijPackage;
      inherit yzn;
      default = yzn;
    });

    apps = eachSystem (system: let
      yzn = {
        type = "app";
        program = "${self.packages.${system}.yzn}/bin/yzn";
      };
    in {
      inherit yzn;
      default = yzn;
    });
  };
}
