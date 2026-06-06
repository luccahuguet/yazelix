{
  pkgs,
  src,
  yazelix_helix,
  yazelix_screen ? null,
  yazelix_yazi_assets,
  yazelix_zellij_pane_orchestrator,
  yazelix_zellij_popup,
  yazelix_zjstatus,
}:

# Local upstream-prep draft:
# keep src injected here so the package body stays directly testable from the
# current repo. The real nixpkgs submission should replace this with the chosen
# release/version fetcher stanza once the upstream PR is opened.
#
# meta.platforms is intentionally Linux-only here because the first nixpkgs
# submission targets Linux. The first-party flake package at
# ../../yazelix_package.nix claims a broader set of platforms matching the
# exported flake outputs. Do not widen this without an explicit product decision.

import ../mk_yazelix_package.nix {
  inherit pkgs src;
  metaPlatforms = pkgs.lib.platforms.linux;
  yazelixHelixPackage = yazelix_helix;
  yaziAssets = yazelix_yazi_assets;
  zellijPluginArtifacts = {
    pane_orchestrator = "${yazelix_zellij_pane_orchestrator}/${yazelix_zellij_pane_orchestrator.wasmPath}";
    zjstatus = "${yazelix_zjstatus}/bin/zjstatus.wasm";
    yzpp = "${yazelix_zellij_popup}/${yazelix_zellij_popup.wasmPath}";
  };
}
