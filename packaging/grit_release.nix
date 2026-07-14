{
  pkgs,
  gritSource,
  rustPlatform ? pkgs.rustPlatform,
}:

let
  manifest = builtins.fromTOML (builtins.readFile "${gritSource}/Cargo.toml");
in
rustPlatform.buildRustPackage {
  pname = "grit";
  version = manifest.package.version;

  src = gritSource;

  cargoLock.lockFile = "${gritSource}/Cargo.lock";
  doCheck = false;

  # grit pulls openssl-sys; resolve it via pkg-config against nixpkgs openssl
  nativeBuildInputs = [ pkgs.pkg-config ];
  buildInputs = [ pkgs.openssl ];
  OPENSSL_NO_VENDOR = "1";

  meta = with pkgs.lib; {
    description = "grit CLI built from the upstream FlexNetOS/grit source";
    homepage = "https://github.com/FlexNetOS/grit";
    license = licenses.asl20;
    mainProgram = "grit";
    platforms = [ "x86_64-linux" ];
  };
}
