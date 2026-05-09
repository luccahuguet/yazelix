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
    "docs"
    "nushell"
    "rust_plugins"
    "shells"
  ];
  includeFiles = [
    "CHANGELOG.md"
    "tombi.toml"
    "yazelix_ghostty_cursors_default.toml"
    "yazelix_default.toml"
  ];
  cursorRuntimePaths = [
    "yazelix_ghostty_cursors_default.toml"
    "configs/terminal_emulators/ghostty/shaders"
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
        || lib.hasPrefix "rust_core/target/" relativePath
        || relativePath == "rust_plugins/zellij_pane_orchestrator/target"
        || lib.hasPrefix "rust_plugins/zellij_pane_orchestrator/target/" relativePath;
      disabledCursorPath =
        !cursorsEnabled
        && builtins.any
          (cursorPath: relativePath == cursorPath || lib.hasPrefix "${cursorPath}/" relativePath)
          cursorRuntimePaths;
    in
    included && !isBuildArtifact && !disabledCursorPath;
}
