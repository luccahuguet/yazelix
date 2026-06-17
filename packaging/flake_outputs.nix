{
  agentUsagePackages,
  beadsRustPackage,
  defaultRuntimeIdentity,
  kgpPackages,
  lib,
  mkYazelix,
  pkgs,
  runtimePackage,
  runtimePackageWith,
  system,
  terminalMetadata,
  yazelixCursors,
  yazelixPackage,
  yazelixScreen,
  marsTerminal,
  yazelixYaziAssets,
  yazelixZellijBar,
  yazelixZellijConfigPack,
  yazelixZellijPaneOrchestrator,
  yazelixZellijPopup,
}:

let
  defaultRuntimeVariant = terminalMetadata.default;
  defaultRuntimePackages = agentUsagePackages system;
  terminalPackageEntries =
    terminal:
    [
      { name = terminalMetadata.runtimeOutput terminal; value = runtimePackage system pkgs terminal defaultRuntimePackages; }
      { name = terminalMetadata.packageOutput terminal; value = yazelixPackage system pkgs terminal defaultRuntimePackages; }
    ];
  terminalPackages = lib.listToAttrs (lib.concatMap terminalPackageEntries terminalMetadata.supported);
  runtime_default = builtins.getAttr (terminalMetadata.runtimeOutput defaultRuntimeVariant) terminalPackages;
  marsFastRuntimeIdentity = defaultRuntimeIdentity // {
    package_profile = "mars-fast";
    mars_terminal_package = "mars-fast";
  };
  marsFastTerminalPackage = marsTerminal.packages.${system}.mars-fast;
  runtime_mars_fast = runtimePackageWith system pkgs "mars" defaultRuntimePackages {
    name = "yazelix-runtime-mars-fast";
    runtimeIdentity = marsFastRuntimeIdentity;
    marsTerminalPackage = marsFastTerminalPackage;
  };
  yazelix_default = builtins.getAttr (terminalMetadata.packageOutput defaultRuntimeVariant) terminalPackages;
  mars_fast = mkYazelix system {
    inherit pkgs;
    name = "yazelix-mars-fast";
    runtimeName = "yazelix-runtime-mars-fast";
    runtimeVariant = "mars";
    runtimeIdentity = marsFastRuntimeIdentity;
    extraRuntimePackages = defaultRuntimePackages;
    skipStableWrapperRedirect = true;
    marsTerminalPackage = marsFastTerminalPackage;
  };
  runtime_agent_tools = runtimePackage system pkgs defaultRuntimeVariant defaultRuntimePackages;
  yazelix_agent_tools = yazelixPackage system pkgs defaultRuntimeVariant defaultRuntimePackages;
  yazelix_zellij_bar = yazelixZellijBar.packages.${system}.yazelix_zellij_bar;
  yazelix_screen = yazelixScreen.packages.${system}.yzs;
  yazelix_cursors = yazelixCursors.packages.${system}.yazelix_cursors;
  yazelix_helix = kgpPackages.helixPackage system;
  yazelix_zellij_config_pack =
    yazelixZellijConfigPack.packages.${system}.yazelix_zellij_config_pack;
  yazelix_zellij_pane_orchestrator =
    yazelixZellijPaneOrchestrator.packages.${system}.yazelix_zellij_pane_orchestrator;
  yazelix_zellij_popup = yazelixZellijPopup.packages.${system}.yzpp;
  yazelix_yazi_assets = yazelixYaziAssets.packages.${system}.yazelix_yazi_assets;
  beads_rust = beadsRustPackage system pkgs;
  packages =
    {
      br = beads_rust;
      inherit beads_rust runtime_agent_tools runtime_mars_fast mars_fast;
      inherit yazelix_agent_tools yazelix_cursors yazelix_helix yazelix_screen;
      inherit yazelix_yazi_assets yazelix_zellij_bar yazelix_zellij_config_pack;
      inherit yazelix_zellij_pane_orchestrator yazelix_zellij_popup;
      default = yazelix_default;
      runtime = runtime_default;
      yazelix = yazelix_default;
      yazelix_kgp_zellij = (kgpPackages.graphicsPkgs pkgs).zellij;
      yzs = yazelix_screen;
    }
    // terminalPackages;

  appFor = packageName: binName: {
    type = "app";
    program = "${packages.${packageName}}/bin/${binName}";
  };
  yzxApp = packageName: appFor packageName "yzx";
  terminalApps = lib.listToAttrs (
    map (terminal: {
      name = terminalMetadata.packageOutput terminal;
      value = yzxApp (terminalMetadata.packageOutput terminal);
    }) terminalMetadata.supported
  );
in
{
  inherit packages;

  apps =
    {
      default = yzxApp "yazelix";
      yazelix = yzxApp "yazelix";
      mars_fast = yzxApp "mars_fast";
      yazelix_agent_tools = yzxApp "yazelix_agent_tools";
      yazelix_screen = appFor "yazelix_screen" "yzs";
      yzs = appFor "yazelix_screen" "yzs";
      yazelix_cursors = appFor "yazelix_cursors" "yzc";
      yzc = appFor "yazelix_cursors" "yzc";
    }
    // terminalApps;
}
