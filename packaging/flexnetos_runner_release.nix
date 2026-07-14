{
  pkgs,
  runnerSource,
}:

pkgs.rustPlatform.buildRustPackage {
  pname = "flexnetos_runner";
  version = "0.1.0";
  src = runnerSource;

  cargoLock.lockFile = "${runnerSource}/Cargo.lock";
  cargoBuildFlags = ["--workspace" "--bins"];
  doCheck = false;

  meta = {
    description = "Profile-owned FlexNetOS runner, Actions supervisor, and dispatcher";
    homepage = "https://github.com/FlexNetOS/flexnetos_runner";
    license = pkgs.lib.licenses.mit;
    mainProgram = "fxrun";
  };
}
