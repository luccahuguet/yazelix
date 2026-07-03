{
  agentUsagePackages,
  beadsRustPackage,
  kgpPackages,
  mkYazelix,
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
    inherit pkgs fenixPkgs;
    src = ../.;
  };
  yazelix_zellij_pane_orchestrator =
    yazelixZellijPaneOrchestrator.packages.${system}.yazelix_zellij_pane_orchestrator;
  yazelix_zellij_popup = yazelixZellijPopup.packages.${system}.yzpp;
  yazelix_yazi_assets = yazelixYaziAssets.packages.${system}.yazelix_yazi_assets;
  beads_rust = beadsRustPackage system pkgs;
  install_check = import ./install_check.nix { inherit pkgs; };
  flexnetos_foundation_claude = pkgs."claude-code";
  flexnetos_foundation_codex = import ./codex_cli_release.nix {
    inherit pkgs system;
    version = "0.143.0-alpha.35";
  };
  flexnetos_foundation_git_kb = import ./git_kb_local_binary.nix {
    inherit pkgs;
    version = "0.2.12";
  };
  flexnetos_foundation_rtk = import ./rtk_local_binary.nix {
    inherit pkgs;
  };
  yazelix_flexnetos_foundation = mkYazelix {
    inherit pkgs;
    runtimeVariant = "mars";
    name = "yazelix-flexnetos-foundation";
    runtimeName = "yazelix-flexnetos-foundation-runtime";
    extraRuntimePackages = defaultRuntimePackages ++ [
      flexnetos_foundation_claude
      flexnetos_foundation_codex
      flexnetos_foundation_git_kb
      flexnetos_foundation_rtk
    ];
    extraRuntimeCommands = [
      "claude"
      "codex"
      "git-kb"
      "rtk"
    ];
    exportedBinCommands = [
      "claude"
      "codex"
      "git-kb"
      "rtk"
    ];
  };
  packages =
    {
      br = beads_rust;
      claude = flexnetos_foundation_claude;
      codex = flexnetos_foundation_codex;
      git_kb = flexnetos_foundation_git_kb;
      rtk = flexnetos_foundation_rtk;
      inherit beads_rust install_check;
      inherit runtime_mars yazelix_mars;
      inherit yazelix_flexnetos_foundation;
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
