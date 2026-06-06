{
  pkgs,
  src ? null,
  rust_core_src ? ./.,
  nixgl ? null,
  fenixPkgs ? null,
  runtimeVariant ? "ghostty",
  runtimeToolSources ? { },
  runtimeIdentity ? { },
  name ? "yazelix-runtime",
  components ? { },
  extraRuntimePackages ? [ ],
  yaziAssets ? null,
  yazelixCursorsPackage ? null,
  yazelixTerminalPackage ? null,
  zellijPluginArtifacts ? { },
  enableZellijKittyPassthrough ? false,
}:

let
  runtimeSource =
    if src == null then
      import ./packaging/repo_source.nix {
        lib = pkgs.lib;
        src = ./.;
        inherit components;
      }
    else
      src;
  rustCoreHelper = import ./packaging/rust_core_helper.nix {
    inherit pkgs fenixPkgs;
    src = rust_core_src;
  };
in

import ./packaging/mk_runtime_tree.nix {
  inherit
    pkgs
    nixgl
    rustCoreHelper
    runtimeVariant
    runtimeToolSources
    runtimeIdentity
    components
    extraRuntimePackages
    yaziAssets
    yazelixCursorsPackage
    yazelixTerminalPackage
    zellijPluginArtifacts
    enableZellijKittyPassthrough
    ;
  src = runtimeSource;
  inherit name;
}
