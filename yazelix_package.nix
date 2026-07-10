{
  cargoGitOutputHashes,
  pkgs,
  src ? null,
  rust_core_src ? ./.,
  nixgl ? null,
  fenixPkgs ? null,
  runtimeVariant ? "mars",
  runtimeToolSources ? { },
  runtimeIdentity ? { },
  name ? "yazelix",
  runtimeName ? "yazelix-runtime",
  skipStableWrapperRedirect ? false,
  components ? { },
  extraRuntimePackages ? [ ],
  yaziAssets ? null,
  yazelixHelixPackage ? null,
  yazelixCursorsPackage ? null,
  marsTerminalPackage ? null,
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
    cargoGitOutputHashes
    pkgs
    rust_core_src
    nixgl
    fenixPkgs
    runtimeVariant
    runtimeToolSources
    runtimeIdentity
    name
    runtimeName
    skipStableWrapperRedirect
    components
    extraRuntimePackages
    yaziAssets
    yazelixHelixPackage
    yazelixCursorsPackage
    marsTerminalPackage
    zellijPluginArtifacts
    enableZellijKittyPassthrough
    ;
  src = runtimeSource;
  metaPlatforms = firstPartyPlatforms;
}
