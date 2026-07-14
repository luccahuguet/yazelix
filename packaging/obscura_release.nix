{
  pkgs,
  obscuraSource,
  rustPlatform ? pkgs.rustPlatform,
}:

let
  manifest = builtins.fromTOML (builtins.readFile "${obscuraSource}/Cargo.toml");
  cargoLock = builtins.fromTOML (builtins.readFile "${obscuraSource}/Cargo.lock");
  version = manifest.workspace.package.version;
  rustyV8Version = "137.3.0";
  lockedRustyV8Versions = map (package: package.version) (
    builtins.filter (package: package.name == "v8") cargoLock.package
  );
  lockedRustyV8Version =
    if builtins.length lockedRustyV8Versions == 1 then
      builtins.head lockedRustyV8Versions
    else
      throw "Obscura's Cargo.lock must contain exactly one rusty_v8 `v8` package";
  rustyV8Hashes = {
    x86_64-linux = "sha256-omgf3lMBir0zZgGPEyYX3VmAAt948VbHvG0v9gi1ZWc=";
    aarch64-linux = "sha256-42jQy0HBecQ6mQ5OxKVeRN2XYvHTS+FWlqzEQz+KbJI=";
    x86_64-darwin = "sha256-ZnFsCn2VDqLHKqr2oMGkAqO6xV/fwLQ0H0mzjpr+zXU=";
    aarch64-darwin = "sha256-YFA9ZyTlUsRrAewmChXnnobEcVtxl8XGJ0iRG/H04HA=";
  };
  system = pkgs.stdenv.hostPlatform.system;
  rustyV8Hash =
    rustyV8Hashes.${system}
      or (throw "Obscura's rusty_v8 ${rustyV8Version} archive is unavailable for ${system}");
  rustyV8Archive = pkgs.fetchurl {
    name = "librusty_v8-${rustyV8Version}";
    url = "https://github.com/denoland/rusty_v8/releases/download/v${rustyV8Version}/librusty_v8_release_${pkgs.stdenv.hostPlatform.rust.rustcTarget}.a.gz";
    hash = rustyV8Hash;
    meta = {
      version = rustyV8Version;
      sourceProvenance = with pkgs.lib.sourceTypes; [ binaryNativeCode ];
    };
  };
in
assert pkgs.lib.assertMsg (lockedRustyV8Version == rustyV8Version) (
  "Obscura Cargo.lock uses rusty_v8 ${lockedRustyV8Version}, but packaging pins ${rustyV8Version}"
);
rustPlatform.buildRustPackage {
  pname = "obscura";
  inherit version;

  src = obscuraSource;

  cargoLock.lockFile = "${obscuraSource}/Cargo.lock";
  cargoBuildFlags = [
    "-p"
    "obscura-cli"
  ];
  buildFeatures = [ "stealth" ];

  nativeBuildInputs = [
    pkgs.cmake
    pkgs.git
    pkgs.perl
    rustPlatform.bindgenHook
  ];
  env.RUSTY_V8_ARCHIVE = rustyV8Archive;

  # The child repository owns its full test matrix. The foundation derivation
  # builds the release CLI and verifies the installed command below without
  # trying to run browser/network integration tests inside the Nix sandbox.
  doCheck = false;
  postInstall = ''
    "$out/bin/obscura" --version >/dev/null
    "$out/bin/obscura" mcp --help >/dev/null
  '';

  passthru = {
    inherit rustyV8Archive rustyV8Version;
    obscuraFeatures = [ "stealth" ];
  };

  meta = with pkgs.lib; {
    description = "Obscura headless browser and MCP runtime built from the pinned FlexNetOS source";
    homepage = "https://github.com/FlexNetOS/obscura";
    license = licenses.asl20;
    mainProgram = "obscura";
    platforms = builtins.attrNames rustyV8Hashes;
    sourceProvenance = with sourceTypes; [
      fromSource
      binaryNativeCode
    ];
  };
}
