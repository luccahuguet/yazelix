{
  lib,
  src ? ../.,
  components ? { },
}:

let
  cursorsEnabled =
    if builtins.hasAttr "cursors" components && builtins.isBool components.cursors then
      components.cursors
    else
      true;
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
    "yazelix_ghostty_cursors_default.toml"
    "settings_default.jsonc"
  ];
  cursorRuntimePaths = [
    "yazelix_ghostty_cursors_default.toml"
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
      disabledCursorPath =
        !cursorsEnabled
        && builtins.any
          (cursorPath: relativePath == cursorPath || lib.hasPrefix "${cursorPath}/" relativePath)
          cursorRuntimePaths;
    in
    included && !isBuildArtifact && !disabledCursorPath;
}
