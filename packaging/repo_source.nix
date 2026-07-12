{
  lib,
  src ? ../.,
}:

let
  includeRoots = [
    "assets"
    "config_metadata"
    "configs"
    "nushell"
    "shells"
  ];
  includeFiles = [
    "CHANGELOG.md"
    "docs"
    "docs/upgrade_notes.toml"
    "config_default.toml"
  ];
in
lib.cleanSourceWith {
  name = "yazelix-package-source";
  inherit src;
  filter =
    path: _type:
    let
      relativePath = lib.removePrefix ((toString src) + "/") (toString path);
      included =
        builtins.elem relativePath includeFiles
        || builtins.any
          (root: relativePath == root || lib.hasPrefix "${root}/" relativePath)
          includeRoots;
      isBuildArtifact =
        relativePath == "rust_core/target"
        || lib.hasPrefix "rust_core/target/" relativePath;
    in
    included && !isBuildArtifact;
}
