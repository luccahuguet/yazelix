{
  pkgs,
  src ? ../.,
  rust_core_src ? src,
  nixgl ? null,
  metaPlatforms ? null,
  fenixPkgs ? null,
  runtimeVariant ? "ghostty",
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
  yazelixTerminalPackage ? null,
  zellijPluginArtifacts ? { },
  enableZellijKittyPassthrough ? false,
}:

let
  rustCoreHelper = import ./rust_core_helper.nix {
    inherit pkgs fenixPkgs;
    src = rust_core_src;
  };
  runtime = import ./mk_runtime_tree.nix {
    inherit
      pkgs
      src
      nixgl
      rustCoreHelper
      runtimeVariant
      runtimeToolSources
      runtimeIdentity
      components
      extraRuntimePackages
      yaziAssets
      yazelixHelixPackage
      yazelixCursorsPackage
      yazelixTerminalPackage
      zellijPluginArtifacts
      enableZellijKittyPassthrough
      ;
    name = runtimeName;
  };
in
pkgs.symlinkJoin {
  inherit name;
  paths = [ runtime ];

  nativeBuildInputs = [ pkgs.makeWrapper ];

  postBuild = ''
    for entry in "$out/bin/"*; do
      local name=$(basename "$entry")
      if [ "$name" != "yzx" ]; then
        rm -f "$entry"
      fi
    done

    rm -f "$out/bin/yzx"
    makeWrapper "$out/shells/posix/yzx_cli.sh" "$out/bin/yzx" \
      --run 'export YAZELIX_INVOKED_YZX_PATH="$0"'${pkgs.lib.optionalString skipStableWrapperRedirect " \\\n      --run 'export YAZELIX_SKIP_STABLE_WRAPPER_REDIRECT=1'"}
  '';

  meta = {
    description = "Reproducible terminal IDE built from Zellij, Yazi, and Helix";
    homepage = "https://github.com/luccahuguet/yazelix";
    license = pkgs.lib.licenses.mit;
    mainProgram = "yzx";
  } // pkgs.lib.optionalAttrs (metaPlatforms != null) {
    platforms = metaPlatforms;
  };
}
