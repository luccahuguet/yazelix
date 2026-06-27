{
  pkgs,
  nixgl ? null,
  runtimeVariant ? "mars",
  runtimeToolSources ? { },
  marsTerminalPackage ? null,
}:

let
  registry = import ./runtime_tool_registry.nix {
    inherit pkgs nixgl runtimeVariant runtimeToolSources marsTerminalPackage;
  };
in
registry.runtimePackages
