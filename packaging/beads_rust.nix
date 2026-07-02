{
  pkgs,
  rustPlatform ? pkgs.rustPlatform,
}:

rustPlatform.buildRustPackage rec {
  pname = "beads_rust";
  version = "0.2.16";

  src = pkgs.fetchCrate {
    inherit pname version;
    hash = "sha256-6QM4WLC4hQArtvB2FyAxYDl/HqCEoUO7FRu6rrAFP4c=";
  };

  cargoHash = "sha256-I8R0UWt+dlG05RGdASDCBo56m2vx4wSTg/pzP9eHYGg=";

  cargoBuildFlags = [
    "--bin"
    "br"
  ];
  buildNoDefaultFeatures = true;
  doCheck = false;

  meta = {
    description = "Agent-first issue tracker with SQLite and JSONL storage";
    homepage = "https://github.com/Dicklesworthstone/beads_rust";
    license = pkgs.lib.licenses.mit;
    mainProgram = "br";
  };
}
