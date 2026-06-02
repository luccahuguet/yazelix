{
  pkgs,
  src ? null,
  rust_core_src ? ./.,
  nixgl ? null,
  fenixPkgs ? null,
  runtimeVariant ? "ghostty",
  runtimeToolSources ? { },
  runtimeIdentity ? { },
  components ? { },
  extraRuntimePackages ? [ ],
  screenAssets,
  yaziAssets ? null,
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
  firstPartyPlatforms = [
    "x86_64-linux"
    "aarch64-linux"
    "x86_64-darwin"
    "aarch64-darwin"
  ];
in
import ./packaging/mk_yazelix_package.nix {
  inherit
    pkgs
    rust_core_src
    nixgl
    fenixPkgs
    runtimeVariant
    runtimeToolSources
    runtimeIdentity
    components
    extraRuntimePackages
    screenAssets
    yaziAssets
    yazelixTerminalPackage
    zellijPluginArtifacts
    enableZellijKittyPassthrough
    ;
  src = runtimeSource;
  metaPlatforms = firstPartyPlatforms;
}
