{
  pkgs,
  src ?
    import ./packaging/repo_source.nix {
      lib = pkgs.lib;
      src = ./.;
    },
  rust_core_src ? ./.,
  nixgl ? null,
  fenixPkgs ? null,
  runtimeVariant ? "ghostty",
  extraRuntimePackages ? [ ],
}:

let
  rustCoreHelper = import ./packaging/rust_core_helper.nix {
    inherit pkgs fenixPkgs;
    src = rust_core_src;
  };
in

import ./packaging/mk_runtime_tree.nix {
  inherit pkgs src nixgl rustCoreHelper runtimeVariant extraRuntimePackages;
  name = "yazelix-runtime";
}
