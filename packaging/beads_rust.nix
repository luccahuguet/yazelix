{
  pkgs,
  rustPlatform ? pkgs.rustPlatform,
}:

rustPlatform.buildRustPackage rec {
  pname = "beads_rust";
  version = "0.2.11";

  src = pkgs.fetchCrate {
    inherit pname version;
    hash = "sha256-ItmCjTQjpWujp2uQlWGdQztsTfJ3BZvk1fpGmSNUQTI=";
  };

  cargoHash = "sha256-3u7GMriV2ZG0mjjGYLXGcUDQrs83uRYDMy5NKXTdaTI=";

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
