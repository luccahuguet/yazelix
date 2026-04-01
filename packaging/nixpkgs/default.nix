{ system ? builtins.currentSystem }:

let
  repoRoot = ../..;
  flake = builtins.getFlake (toString repoRoot);
  pkgs = flake.inputs.nixpkgs.legacyPackages.${system};
in
import ./yazelix_package.nix {
  inherit pkgs;
  src = repoRoot;
}
