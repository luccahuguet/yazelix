{
  pkgs,
  icmSource,
  rustPlatform ? pkgs.rustPlatform,
}:

let
  # icm is a cargo workspace; the CLI crate owns the `icm` binary + version.
  manifest = builtins.fromTOML (builtins.readFile "${icmSource}/crates/icm-cli/Cargo.toml");
in
rustPlatform.buildRustPackage {
  pname = "icm";
  version = manifest.package.version;

  src = icmSource;

  cargoLock.lockFile = "${icmSource}/Cargo.lock";
  cargoBuildFlags = [
    "-p"
    "icm-cli"
  ];
  doCheck = false;

  # icm pulls openssl-sys and ort-sys (ONNX Runtime for embeddings). ort-sys
  # downloads binaries at build time unless pkg-config resolves a system
  # onnxruntime — supply both via nixpkgs so the sandboxed build links locally.
  nativeBuildInputs = [ pkgs.pkg-config ];
  buildInputs = [
    pkgs.openssl
    pkgs.onnxruntime
  ];
  OPENSSL_NO_VENDOR = "1";

  meta = with pkgs.lib; {
    description = "ICM CLI (permanent memory for AI agents) built from the upstream FlexNetOS/icm source";
    homepage = "https://github.com/FlexNetOS/icm";
    license = licenses.asl20;
    mainProgram = "icm";
    platforms = [ "x86_64-linux" ];
  };
}
