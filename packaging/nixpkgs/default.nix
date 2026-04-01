{ system ? builtins.currentSystem }:

let
  repoRoot = ../..;
  flake = builtins.getFlake (toString repoRoot);
  pkgs = flake.inputs.nixpkgs.legacyPackages.${system};
in
pkgs.callPackage ./yazelix_package.nix {
  src = repoRoot;
}
