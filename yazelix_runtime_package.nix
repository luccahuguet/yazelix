{ pkgs, src ? ./., nixgl ? null }:

import ./packaging/mk_runtime_tree.nix {
  inherit pkgs src nixgl;
  name = "yazelix-runtime";
}
