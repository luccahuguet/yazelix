{
  pkgs,
  src ? ../.,
  fenixPkgs ? null,
  metaPlatforms ? null,
}:

let
  lib = pkgs.lib;
  rustToolchain =
    if fenixPkgs == null then
      null
    else
      fenixPkgs.combine [
        fenixPkgs.stable.cargo
        fenixPkgs.stable.rustc
        fenixPkgs.targets.wasm32-wasip1.stable.rust-std
      ];
  rustPlatform =
    if rustToolchain == null then
      pkgs.rustPlatform
    else
      pkgs.makeRustPlatform {
        cargo = rustToolchain;
        rustc = rustToolchain;
      };
  pluginSource = lib.cleanSourceWith {
    name = "yazelix-zellij-popup-source";
    src = src;
    filter =
      path: _type:
      let
        relativePath = lib.removePrefix ((toString src) + "/") (toString path);
        isBuildArtifact =
          relativePath == "rust_plugins/zellij_pane_orchestrator/target"
          || lib.hasPrefix "rust_plugins/zellij_pane_orchestrator/target/" relativePath;
      in
      !isBuildArtifact
      && (
        relativePath == "rust_plugins"
        || relativePath == "rust_plugins/zellij_pane_orchestrator"
        || lib.hasPrefix "rust_plugins/zellij_pane_orchestrator/" relativePath
        || relativePath == "docs"
        || relativePath == "docs/examples"
        || relativePath == "docs/examples/zellij_popup_plain_zellij.kdl"
        || relativePath == "docs/yazelix_zellij_popup.md"
      );
  };
in
rustPlatform.buildRustPackage {
  pname = "yazelix_zellij_popup";
  version = "0.1.0";

  src = pluginSource;
  cargoRoot = "rust_plugins/zellij_pane_orchestrator";
  cargoLock = {
    lockFile = "${src}/rust_plugins/zellij_pane_orchestrator/Cargo.lock";
  };
  buildAndTestSubdir = "rust_plugins/zellij_pane_orchestrator";
  nativeBuildInputs = [
    pkgs.pkg-config
  ];
  buildInputs = [
    pkgs.openssl
  ];
  doCheck = false;

  buildPhase = ''
    runHook preBuild

    cargo build \
      --manifest-path rust_plugins/zellij_pane_orchestrator/Cargo.toml \
      --target-dir target \
      --offline \
      --profile release \
      --target wasm32-wasip1 \
      --bin yazelix_zellij_popup

    runHook postBuild
  '';

  installPhase = ''
    runHook preInstall

    install -Dm644 target/wasm32-wasip1/release/yazelix_zellij_popup.wasm \
      "$out/share/yazelix_zellij_popup/yazelix_zellij_popup.wasm"
    mkdir -p "$out/share/yazelix_zellij_popup/examples"
    substitute docs/examples/zellij_popup_plain_zellij.kdl \
      "$out/share/yazelix_zellij_popup/examples/gitui.kdl" \
      --replace-fail "__YAZELIX_ZELLIJ_POPUP_WASM__" \
      "file:$out/share/yazelix_zellij_popup/yazelix_zellij_popup.wasm"
    install -Dm644 docs/examples/zellij_popup_plain_zellij.kdl \
      "$out/share/yazelix_zellij_popup/examples/gitui.template.kdl"
    install -Dm644 docs/yazelix_zellij_popup.md \
      "$out/share/doc/yazelix_zellij_popup/README.md"

    runHook postInstall
  '';

  doInstallCheck = true;
  nativeInstallCheckInputs = [
    pkgs.coreutils
    pkgs.gnugrep
  ];
  installCheckPhase = ''
    runHook preInstallCheck

    test -s "$out/share/yazelix_zellij_popup/yazelix_zellij_popup.wasm"
    grep -q 'location="file:' "$out/share/yazelix_zellij_popup/examples/gitui.kdl"
    grep -q 'load_plugins' "$out/share/yazelix_zellij_popup/examples/gitui.kdl"
    grep -q 'MessagePlugin "yazelix_zellij_popup"' "$out/share/yazelix_zellij_popup/examples/gitui.kdl"
    grep -q 'ReadApplicationState' "$out/share/doc/yazelix_zellij_popup/README.md"
    ! grep -q '__YAZELIX_ZELLIJ_POPUP_WASM__' "$out/share/yazelix_zellij_popup/examples/gitui.kdl"

    runHook postInstallCheck
  '';

  passthru = {
    wasmPath = "share/yazelix_zellij_popup/yazelix_zellij_popup.wasm";
    examplePath = "share/yazelix_zellij_popup/examples/gitui.kdl";
    templatePath = "share/yazelix_zellij_popup/examples/gitui.template.kdl";
  };

  meta = {
    description = "Standalone Zellij plugin for toggling managed floating TUI popups";
    homepage = "https://github.com/luccahuguet/yazelix";
    license = lib.licenses.mit;
  } // lib.optionalAttrs (metaPlatforms != null) {
    platforms = metaPlatforms;
  };
}
