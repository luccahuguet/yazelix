{
  pkgs,
  nixgl ? null,
  rioPackage ? pkgs.rio,
  runtimeVariant ? "ghostty",
  runtimeToolSources ? { },
  yazelixTerminalPackage ? null,
}:

let
  registry = import ./runtime_tool_registry.nix {
    inherit pkgs nixgl rioPackage runtimeVariant runtimeToolSources yazelixTerminalPackage;
  };
in
registry.runtimePackages
