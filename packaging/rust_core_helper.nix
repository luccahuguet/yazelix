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
      "yazelix_cursors-0.1.0" = "sha256-V0wIPwYMfUFAZ3ieb2n41AkYv6cHXhgBxQifc6ZV3mk=";
      "ratconfig-0.1.0" = "sha256-axG4lSlrxHF2C7BA678sxeIEo4yw79b0gtqPCIhngrU=";
      "yazelix_screen-0.1.0" = "sha256-e8qM6kzHUNMsbBBQ21QJEAgJp5rqytDiXVIJmGaY9SE=";
      "yazelix_yazi_assets-0.1.0" = "sha256-46vNIjYKL6tByQf1Pcy1ynll/A7AKJ4lrMXn7LM5yqc=";
      "yazelix_zellij_config_pack-0.1.0" = "sha256-eGG/kBLyz0I6ZuN6RsYVH3WjamtZLyesr1AQve57Uh8=";
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
