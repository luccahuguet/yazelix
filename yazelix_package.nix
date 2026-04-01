{ pkgs, src ? ./. }:

import ./mk_yazelix_package.nix {
  inherit pkgs src;
}
