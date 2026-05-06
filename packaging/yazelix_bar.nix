{
  pkgs,
  src ? ../.,
  metaPlatforms ? null,
}:

let
  lib = pkgs.lib;
  packageSource = lib.cleanSourceWith {
    name = "yazelix-bar-source";
    src = src;
    filter =
      path: _type:
      let
        relativePath = lib.removePrefix ((toString src) + "/") (toString path);
      in
      relativePath == "configs"
      || relativePath == "configs/yazelix_bar"
      || lib.hasPrefix "configs/yazelix_bar/" relativePath
      || relativePath == "configs/zellij"
      || relativePath == "configs/zellij/plugins"
      || relativePath == "configs/zellij/plugins/zjstatus.wasm"
      || relativePath == "docs"
      || relativePath == "docs/yazelix_bar.md";
  };
in
pkgs.stdenvNoCC.mkDerivation {
  pname = "yazelix_bar";
  version = "0.1.0";

  src = packageSource;

  dontBuild = true;

  installPhase = ''
    runHook preInstall

    install -Dm644 configs/zellij/plugins/zjstatus.wasm "$out/share/yazelix_bar/zjstatus.wasm"
    substitute configs/yazelix_bar/yazelix_bar.kdl "$out/share/yazelix_bar/yazelix_bar.kdl" \
      --replace-fail "__YAZELIX_BAR_ZJSTATUS_WASM__" "file:$out/share/yazelix_bar/zjstatus.wasm"
    install -Dm644 configs/yazelix_bar/yazelix_bar.kdl "$out/share/yazelix_bar/yazelix_bar.template.kdl"
    install -Dm644 docs/yazelix_bar.md "$out/share/doc/yazelix_bar/README.md"

    runHook postInstall
  '';

  passthru = {
    presetPath = "share/yazelix_bar/yazelix_bar.kdl";
    templatePath = "share/yazelix_bar/yazelix_bar.template.kdl";
    wasmPath = "share/yazelix_bar/zjstatus.wasm";
  };

  meta = {
    description = "Standalone Yazelix-branded zjstatus bar preset for Zellij";
    homepage = "https://github.com/luccahuguet/yazelix";
    license = lib.licenses.mit;
  } // lib.optionalAttrs (metaPlatforms != null) {
    platforms = metaPlatforms;
  };
}
