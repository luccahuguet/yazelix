{
  pkgs,
  src ? ../.,
  metaPlatforms ? null,
}:

let
  lib = pkgs.lib;
  generatorSource = lib.cleanSourceWith {
    name = "yazelix-bar-generator-source";
    src = src;
    filter =
      path: _type:
      let
        relativePath = lib.removePrefix ((toString src) + "/") (toString path);
        isBuildArtifact =
          relativePath == "rust_core/target" || lib.hasPrefix "rust_core/target/" relativePath;
      in
      (relativePath == "rust_core" || lib.hasPrefix "rust_core/" relativePath) && !isBuildArtifact;
  };
  generator = pkgs.rustPlatform.buildRustPackage {
    pname = "yazelix_bar_generator";
    version = "0.1.0";

    src = generatorSource;
    cargoRoot = "rust_core";
    cargoLock = {
      lockFile = "${src}/rust_core/Cargo.lock";
      outputHashes = {
        "yazelix_cursors-0.1.0" = "sha256-5BlGyV5ZCkpHfDvl+eMaFvsl3y51mnPR+vWFs+H4ul8=";
        "yazelix_screen-0.1.0" = "sha256-PkZ4ChP94XabPULG1ohd4vojF3ne/p0CZ6HdsLCtI9g=";
      };
    };
    buildAndTestSubdir = "rust_core";
    cargoBuildFlags = [
      "-p"
      "yazelix_bar"
      "--bin"
      "yazelix_bar_generate"
    ];
    doCheck = false;
  };
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
      || relativePath == "configs/yazelix_bar/examples"
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
    install -Dm755 ${generator}/bin/yazelix_bar_generate "$out/bin/yazelix_bar_generate"
    mkdir -p "$out/share/yazelix_bar/generated"
    "$out/bin/yazelix_bar_generate" \
      --wasm-url "file:$out/share/yazelix_bar/zjstatus.wasm" \
      > "$out/share/yazelix_bar/generated/yazelix_bar.kdl"
    cp -R configs/yazelix_bar/examples "$out/share/yazelix_bar/examples"
    install -Dm644 docs/yazelix_bar.md "$out/share/doc/yazelix_bar/README.md"

    runHook postInstall
  '';

  passthru = {
    presetPath = "share/yazelix_bar/yazelix_bar.kdl";
    templatePath = "share/yazelix_bar/yazelix_bar.template.kdl";
    generatedPresetPath = "share/yazelix_bar/generated/yazelix_bar.kdl";
    examplesPath = "share/yazelix_bar/examples";
    generatorPath = "bin/yazelix_bar_generate";
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
