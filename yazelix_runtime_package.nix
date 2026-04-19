{ pkgs, src ? ./., nixgl ? null, fenixPkgs ? null }:

let
  rustCoreHelper = import ./packaging/rust_core_helper.nix {
    inherit pkgs src fenixPkgs;
  };
in

import ./packaging/mk_runtime_tree.nix {
  inherit pkgs src nixgl rustCoreHelper;
  name = "yazelix-runtime";
}
