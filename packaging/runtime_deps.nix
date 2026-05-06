{
  pkgs,
  nixgl ? null,
  runtimeVariant ? "ghostty",
  runtimeToolSources ? { },
}:

let
  registry = import ./runtime_tool_registry.nix {
    inherit pkgs nixgl runtimeVariant runtimeToolSources;
  };
in
registry.runtimePackages
