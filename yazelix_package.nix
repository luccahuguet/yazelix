{ pkgs, src ? ./. }:

import ./packaging/mk_yazelix_package.nix {
  inherit pkgs src;
}
