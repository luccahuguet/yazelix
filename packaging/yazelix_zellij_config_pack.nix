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
    name = "yazelix_zellij_config_pack_source";
    src = src;
    filter =
      path: _type:
      let
        relativePath = lib.removePrefix ((toString src) + "/") (toString path);
        isBuildArtifact =
          relativePath == "rust_core/target" || lib.hasPrefix "rust_core/target/" relativePath;
        isRustCoreSource =
          relativePath == "rust_core" || lib.hasPrefix "rust_core/" relativePath;
      in
      isRustCoreSource && !isBuildArtifact;
  };
in
rustPlatform.buildRustPackage {
  pname = "yazelix_zellij_config_pack";
  version = "0.1.0";

  src = rustSource;
  cargoRoot = "rust_core";
  cargoLock = {
    lockFile = "${src}/rust_core/Cargo.lock";
    outputHashes = {
      "yazelix_cursors-0.1.0" = "sha256-lgs+e/rm/WTXHM2veZFiswKxO8H6zCrdiv0uYhGdZeQ=";
      "ratconfig-0.1.0" = "sha256-gibgrdrrTe9Zsu17k5xNevPIFChqFCQ9WEavPsISId8=";
      "yazelix_screen-0.1.0" = "sha256-e8qM6kzHUNMsbBBQ21QJEAgJp5rqytDiXVIJmGaY9SE=";
      "yazelix_yazi_assets-0.1.0" = "sha256-dXYwpt5HRdT7L2F5UTznpLs8WjH94idvsGJDHvmYOOo=";
    };
  };
  buildAndTestSubdir = "rust_core";
  cargoBuildFlags = [
    "-p"
    "yazelix_zellij_config_pack"
  ];
  cargoTestFlags = [
    "-p"
    "yazelix_zellij_config_pack"
  ];
  nativeBuildInputs = [ pkgs.findutils ];

  installPhase = ''
    runHook preInstall

    renderer_bin="$(find target -type f -path '*/release/yazelix_zellij_config_pack' -perm -111 | head -n 1)"
    if [ -z "$renderer_bin" ]; then
      echo "could not find built yazelix_zellij_config_pack binary" >&2
      exit 1
    fi
    install -Dm755 "$renderer_bin" "$out/bin/yazelix_zellij_config_pack"
    mkdir -p "$out/share/yazelix_zellij_config_pack"
    cp -R rust_core/yazelix_zellij_config_pack/layouts "$out/share/yazelix_zellij_config_pack/layouts"
    install -Dm644 rust_core/yazelix_zellij_config_pack/config_metadata/zellij_layout_families.toml \
      "$out/share/yazelix_zellij_config_pack/config_metadata/zellij_layout_families.toml"
    install -Dm644 rust_core/yazelix_zellij_config_pack/README.md "$out/share/doc/yazelix_zellij_config_pack/README.md"
    install -Dm644 rust_core/yazelix_zellij_config_pack/LICENSE "$out/share/doc/yazelix_zellij_config_pack/LICENSE"

    runHook postInstall
  '';

  doInstallCheck = true;
  nativeInstallCheckInputs = [
    pkgs.coreutils
    pkgs.findutils
    pkgs.gnugrep
  ];
  installCheckPhase = ''
    runHook preInstallCheck

    test -x "$out/bin/yazelix_zellij_config_pack"
    test -f "$out/share/yazelix_zellij_config_pack/layouts/yzx_side.kdl"
    test -f "$out/share/yazelix_zellij_config_pack/layouts/yzx_side.swap.kdl"
    test -f "$out/share/yazelix_zellij_config_pack/layouts/fragments/swap_sidebar_open.kdl"
    test -f "$out/share/yazelix_zellij_config_pack/config_metadata/zellij_layout_families.toml"
    "$out/bin/yazelix_zellij_config_pack" --schema-version | grep -q '^2$'

    runHook postInstallCheck
  '';

  passthru = {
    rendererSchemaVersion = 2;
    layoutsPath = "share/yazelix_zellij_config_pack/layouts";
    layoutFamiliesPath = "share/yazelix_zellij_config_pack/config_metadata/zellij_layout_families.toml";
  };

  meta = {
    description = "Deterministic Zellij config and layout renderer from Yazelix";
    homepage = "https://github.com/luccahuguet/yazelix";
    license = pkgs.lib.licenses.asl20;
    mainProgram = "yazelix_zellij_config_pack";
  };
}
