{ pkgs }:

pkgs.rustPlatform.buildRustPackage rec {
  pname = "cargo-crap";
  version = "0.2.2";

  src = pkgs.fetchCrate {
    inherit pname version;
    hash = "sha256-cZ30mdHHLXzpvMhkC6XoPMgfqAdsmdqhEfHq8T15Fmw=";
  };

  cargoHash = "sha256-vzkGNzQrVOtfpGLniGTdPRQfwA9jn5elXhudrFC7w9g=";

  # The crates.io archive omits tests/fixtures/sample_workspace, which these
  # workspace tests require. Keep the rest of the upstream test suite enabled.
  checkFlags = [
    "--skip=workspace_human_output_includes_per_crate_summary"
    "--skip=workspace_json_includes_crate_field"
    "--skip=workspace_summary_flag_shows_only_crate_table"
  ];

  meta = {
    description = "Change Risk Anti-Patterns metric for Rust projects";
    homepage = "https://github.com/minikin/cargo-crap";
    license = pkgs.lib.licenses.mit;
    mainProgram = "cargo-crap";
  };
}
