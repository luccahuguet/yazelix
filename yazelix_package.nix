{ pkgs, src ? ./., nixgl ? null, fenixPkgs ? null }:

let
  firstPartyPlatforms = [
    "x86_64-linux"
    "aarch64-linux"
    "x86_64-darwin"
    "aarch64-darwin"
  ];
in
import ./packaging/mk_yazelix_package.nix {
  inherit pkgs src nixgl fenixPkgs;
  metaPlatforms = firstPartyPlatforms;
}
