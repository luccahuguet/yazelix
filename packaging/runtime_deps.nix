{
  pkgs,
  nixgl ? null,
  runtimeVariant ? "ghostty",
  runtimeToolSources ? { },
  yazelixTerminalPackage ? null,
}:

let
  registry = import ./runtime_tool_registry.nix {
    inherit pkgs nixgl runtimeVariant runtimeToolSources yazelixTerminalPackage;
  };
in
registry.runtimePackages
