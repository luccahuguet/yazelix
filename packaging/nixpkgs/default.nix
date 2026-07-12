{ system ? builtins.currentSystem }:

let
  repoRoot = ../..;
  flake = builtins.getFlake (toString repoRoot);
  pkgs = flake.inputs.nixpkgs.legacyPackages.${system};
  cargoGitOutputHashes = import ../cargo_git_output_hashes.nix {
    yazelixCursors = flake.inputs.yazelixCursors;
    yazelixYaziAssets = flake.inputs.yazelixYaziAssets;
  };
in
import ./yazelix_package.nix {
  inherit cargoGitOutputHashes pkgs;
  mars_terminal = flake.inputs.mars.packages.${system}.mars;
  src = repoRoot;
  yazelix_cursors = flake.inputs.yazelixCursors.packages.${system}.yazelix_cursors;
  yazelix_helix = flake.inputs.yazelixHelix.packages.${system}.yazelix_helix;
  yazelix_yazi_assets = flake.inputs.yazelixYaziAssets.packages.${system}.yazelix_yazi_assets;
  yazelix_zellij_pane_orchestrator =
    flake.inputs.yazelixZellijPaneOrchestrator.packages.${system}.yazelix_zellij_pane_orchestrator;
  yazelix_zellij_popup = flake.inputs.yazelixZellijPopup.packages.${system}.yzpp;
  yazelix_zjstatus = flake.inputs.zjstatus.packages.${system}.default;
}
