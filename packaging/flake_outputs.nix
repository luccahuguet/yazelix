{
  agentUsagePackages,
  beadsRustPackage,
  cargoGitOutputHashes,
  kgpPackages,
  pkgs,
  runtimePackage,
  system,
  yazelixCursors,
  yazelixPackage,
  yazelixScreen,
  yazelixYaziAssets,
  yazelixZellijBar,
  yazelixZellijPaneOrchestrator,
  yazelixZellijPopup,
  fenixPkgs,
}:

let
  defaultRuntimePackages = agentUsagePackages system;
  runtime_mars = runtimePackage system pkgs "mars" defaultRuntimePackages;
  yazelix_mars = yazelixPackage system pkgs "mars" defaultRuntimePackages;
  yazelix_zellij_bar = yazelixZellijBar.packages.${system}.yazelix_zellij_bar;
  yazelix_screen = yazelixScreen.packages.${system}.yzs;
  yazelix_cursors = yazelixCursors.packages.${system}.yazelix_cursors;
  yazelix_helix = kgpPackages.helixPackage system;
  yazelix_zellij_config_pack = import ./yazelix_zellij_config_pack.nix {
    inherit cargoGitOutputHashes pkgs fenixPkgs;
    src = ../.;
  };
  yazelix_zellij_pane_orchestrator =
    yazelixZellijPaneOrchestrator.packages.${system}.yazelix_zellij_pane_orchestrator;
  yazelix_zellij_popup = yazelixZellijPopup.packages.${system}.yzpp;
  yazelix_yazi_assets = yazelixYaziAssets.packages.${system}.yazelix_yazi_assets;
  beads_rust = beadsRustPackage system pkgs;
  install_check = import ./install_check.nix { inherit pkgs; };
  packages =
    {
      br = beads_rust;
      inherit beads_rust install_check;
      inherit runtime_mars yazelix_mars;
      inherit yazelix_cursors yazelix_helix yazelix_screen;
      inherit yazelix_yazi_assets yazelix_zellij_bar yazelix_zellij_config_pack;
      inherit yazelix_zellij_pane_orchestrator yazelix_zellij_popup;
      default = yazelix_mars;
      runtime = runtime_mars;
      runtime_agent_tools = runtime_mars;
      yazelix = yazelix_mars;
      yazelix_agent_tools = yazelix_mars;
      yazelix_kgp_zellij = (kgpPackages.graphicsPkgs pkgs).zellij;
      yzs = yazelix_screen;
    };

  appFor = packageName: binName: {
    type = "app";
    program = "${packages.${packageName}}/bin/${binName}";
  };
  yzxApp = packageName: appFor packageName "yzx";
in
{
  inherit packages;

  apps = {
    default = yzxApp "yazelix";
    yazelix = yzxApp "yazelix";
    yazelix_agent_tools = yzxApp "yazelix_agent_tools";
    yazelix_mars = yzxApp "yazelix_mars";
    yazelix_screen = appFor "yazelix_screen" "yzs";
    yzs = appFor "yazelix_screen" "yzs";
    yazelix_cursors = appFor "yazelix_cursors" "yzc";
    yzc = appFor "yazelix_cursors" "yzc";
    install_check = appFor "install_check" "install_check";
  };
}
