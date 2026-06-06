{ system ? builtins.currentSystem }:

let
  repoRoot = ../..;
  flake = builtins.getFlake (toString repoRoot);
  pkgs = flake.inputs.nixpkgs.legacyPackages.${system};
in
import ./yazelix_package.nix {
  inherit pkgs;
  src = repoRoot;
  yazelix_screen = flake.inputs.yazelixScreen.packages.${system}.yzs;
  yazelix_yazi_assets = flake.inputs.yazelixYaziAssets.packages.${system}.yazelix_yazi_assets;
  yazelix_zellij_pane_orchestrator =
    flake.inputs.yazelixZellijPaneOrchestrator.packages.${system}.yazelix_zellij_pane_orchestrator;
  yazelix_zellij_popup = flake.inputs.yazelixZellijPopup.packages.${system}.yzpp;
  yazelix_zjstatus = flake.inputs.zjstatus.packages.${system}.default;
}
