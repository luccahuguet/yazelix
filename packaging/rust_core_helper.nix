{ pkgs, src ? ../., fenixPkgs ? null }:

let
  lib = pkgs.lib;
  rustPlatform =
    if fenixPkgs == null then
      pkgs.rustPlatform
    else
      let
        rustToolchain = fenixPkgs.combine [
          fenixPkgs.stable.cargo
          fenixPkgs.stable.rustc
        ];
      in
      pkgs.makeRustPlatform {
        cargo = rustToolchain;
        rustc = rustToolchain;
      };
  rustSource = lib.cleanSourceWith {
    name = "yazelix-rust-core-source";
    src = src;
    filter =
      path: _type:
      let
        relativePath = lib.removePrefix ((toString src) + "/") (toString path);
        isBuildArtifact =
          relativePath == "rust_core/target" || lib.hasPrefix "rust_core/target/" relativePath;
        isRustCoreSource =
          relativePath == "rust_core"
          || relativePath == "config_metadata"
          || relativePath == "settings_default.jsonc"
          || relativePath == "yazelix_cursors_default.toml"
          || lib.hasPrefix "rust_core/" relativePath
          || lib.hasPrefix "config_metadata/" relativePath;
      in
      isRustCoreSource && !isBuildArtifact;
  };
in
rustPlatform.buildRustPackage {
  pname = "yazelix-core";
  version = "0.1.0";
  dontStrip = pkgs.stdenv.hostPlatform.isDarwin;

  src = rustSource;
  cargoRoot = "rust_core";
  cargoLock = {
    lockFile = "${src}/rust_core/Cargo.lock";
    outputHashes = {
      "yazelix_cursors-0.1.0" = "sha256-4IOvm1p3A6W0uC3+Y+jp1B2y8mYNp1G7bXQzMbNYMT0=";
      "yazelix-ratconfig-0.1.0" = "sha256-H1BTLSH81nCq4dw4PDshxHvhcXbMR+Uv4o0632EXwK8=";
      "yazelix_screen-0.1.0" = "sha256-kc6vw5W/msKjcUH05hDgQQVq9OfrarTFVdNt3diTu/U=";
      "yazelix_yazi_assets-0.1.0" = "sha256-46vNIjYKL6tByQf1Pcy1ynll/A7AKJ4lrMXn7LM5yqc=";
      "yazelix_zellij_config_pack-0.1.0" = "sha256-dSS7KiyWpnV4wzWblL+zQYaEf02UKCtJLwjwTqO6ZbA=";
    };
  };
  buildAndTestSubdir = "rust_core";
  cargoBuildFlags = [
    "-p"
    "yazelix_core"
  ];

  # User package builds must be install-only. CI and maintainer commands own
  # Rust test execution so Home Manager switches do not pay test cost or depend
  # on host-only tools from package-time test cases.
  doCheck = false;

  meta = {
    description = "Private Yazelix Rust core helper";
    homepage = "https://github.com/luccahuguet/yazelix";
    license = pkgs.lib.licenses.mit;
    mainProgram = "yzx_core";
  };
}
