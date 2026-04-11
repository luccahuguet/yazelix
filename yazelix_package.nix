{ pkgs, src ? ./., nixgl ? null }:

import ./packaging/mk_yazelix_package.nix {
  inherit pkgs src nixgl;
}
