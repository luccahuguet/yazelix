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
          || relativePath == "settings_default.jsonc"
          || relativePath == "yazelix_ghostty_cursors_default.toml"
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
      "yazelix_cursors-0.1.0" = "sha256-8IRD7cSz4kMl0rE8dM6H9OwC3e4sm+MgmsKSTdLcuVs=";
      "yazelix-ratconfig-0.1.0" = "sha256-oN9UDnzZXloopc1F3+noDJEPcBoRH3keUf0wD7J5Eho=";
      "yazelix_screen-0.1.0" = "sha256-sYPJlmIqnExw3KoX+V9wjzM6tYw6ZrtIfFxc+a6pMYg=";
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
