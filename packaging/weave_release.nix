{
  pkgs,
  weaveSource,
  rustPlatform ? pkgs.rustPlatform,
}:

let
  manifest = builtins.fromTOML (builtins.readFile "${weaveSource}/Cargo.toml");
in
rustPlatform.buildRustPackage {
  pname = "weave";
  version = manifest.workspace.package.version;

  src = weaveSource;

  cargoLock = {
    lockFile = "${weaveSource}/Cargo.lock";
    outputHashes = {
      "fnx-algorithms-0.1.0" = "sha256-p6xDFS2uVdUYYwYdFLEqaUWeto4sUGviJ1Iiox53cOU=";
    };
  };
  # workspace carries weave + weave-mcp + benches; the `weave` bin (which also
  # serves MCP via `weave mcp`) is the shipped surface.
  cargoBuildFlags = [
    "-p"
    "weave"
  ];
  doCheck = false;

  meta = with pkgs.lib; {
    description = "weave A2A session mesh CLI built from the FlexNetOS/weave source (WL-084 identity)";
    homepage = "https://github.com/FlexNetOS/weave";
    license = licenses.asl20;
    mainProgram = "weave";
    platforms = [ "x86_64-linux" ];
  };
}
