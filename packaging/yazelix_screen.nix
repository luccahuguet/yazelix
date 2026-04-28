{
  pkgs,
  src ? ../.,
  fenixPkgs ? null,
}:

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
    name = "yazelix-screen-source";
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
in
rustPlatform.buildRustPackage {
  pname = "yazelix-screen";
  version = "0.1.0";

  src = rustSource;
  cargoRoot = "rust_core";
  cargoLock.lockFile = "${src}/rust_core/Cargo.lock";
  buildAndTestSubdir = "rust_core";
  cargoBuildFlags = [
    "-p"
    "yazelix_screen"
  ];

  doCheck = false;

  meta = {
    description = "Standalone terminal screen animations from Yazelix";
    homepage = "https://github.com/luccahuguet/yazelix";
    license = pkgs.lib.licenses.mit;
    mainProgram = "yazelix_screen";
  };
}
