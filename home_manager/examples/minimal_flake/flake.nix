{
  description = "Minimal Yazelix Home Manager flake example";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    home-manager = {
      url = "github:nix-community/home-manager";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    # This repo-local path keeps the example buildable inside the Yazelix tree.
    # In a copied user setup, replace it with:
    # yazelix-hm.url = "github:luccahuguet/yazelix";
    yazelix-hm = {
      url = "path:../../..";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = inputs@{ home-manager, nixpkgs, ... }:
    let
      system = "x86_64-linux"; # Change to your system, for example aarch64-darwin
      pkgs = import nixpkgs { inherit system; };
    in {
      homeConfigurations.demo = home-manager.lib.homeManagerConfiguration {
        inherit pkgs;
        modules = [
          inputs.yazelix-hm.homeManagerModules.default
          ./home.nix
        ];
      };
    };
}
