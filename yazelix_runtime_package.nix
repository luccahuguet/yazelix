{ pkgs, src ? ./. }:

import ./packaging/mk_runtime_tree.nix {
  inherit pkgs src;
  name = "yazelix-runtime";
}
