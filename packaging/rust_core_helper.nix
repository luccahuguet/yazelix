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
        isZellijStatusTemplateSource =
          relativePath == "configs"
          || relativePath == "configs/zellij"
          || relativePath == "configs/zellij/layouts"
          || relativePath == "configs/zellij/layouts/fragments"
          || relativePath == "configs/zellij/layouts/fragments/zjstatus_tab_template.kdl";
        isRustCoreSource =
          relativePath == "rust_core"
          || relativePath == "config_metadata"
          || relativePath == "yazelix_default.toml"
          || relativePath == "yazelix_cursors_default.toml"
          || isZellijStatusTemplateSource
          || lib.hasPrefix "rust_core/" relativePath
          || lib.hasPrefix "config_metadata/" relativePath;
      in
      isRustCoreSource && !isBuildArtifact;
  };
in
rustPlatform.buildRustPackage {
  pname = "yazelix-core";
  version = "0.1.0";

  src = rustSource;
  cargoRoot = "rust_core";
  cargoLock = {
    lockFile = "${src}/rust_core/Cargo.lock";
    outputHashes = {
      "yazelix_screen-0.1.0" = "sha256-PkZ4ChP94XabPULG1ohd4vojF3ne/p0CZ6HdsLCtI9g=";
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
