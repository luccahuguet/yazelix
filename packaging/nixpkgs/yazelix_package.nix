{
  pkgs,
  src,
  yazelix_yazi_assets,
  yazelix_screen,
  yazelix_zellij_pane_orchestrator,
  yazelix_zellij_popup,
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
  screenAssets = yazelix_screen;
  yaziAssets = yazelix_yazi_assets;
  zellijPluginArtifacts = {
    pane_orchestrator = "${yazelix_zellij_pane_orchestrator}/${yazelix_zellij_pane_orchestrator.wasmPath}";
    yzpp = "${yazelix_zellij_popup}/${yazelix_zellij_popup.wasmPath}";
  };
}
