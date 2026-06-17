{
  pkgs,
  nixgl ? null,
  rioPackage ? pkgs.rio,
  runtimeVariant ? "ghostty",
  runtimeToolSources ? { },
  marsTerminalPackage ? null,
}:

let
  registry = import ./runtime_tool_registry.nix {
    inherit pkgs nixgl rioPackage runtimeVariant runtimeToolSources marsTerminalPackage;
  };
in
registry.runtimePackages
