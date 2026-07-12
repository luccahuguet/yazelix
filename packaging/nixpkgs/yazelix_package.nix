{
  cargoGitOutputHashes,
  mars_terminal,
  pkgs,
  src,
  yazelix_cursors,
  yazelix_helix,
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
# submission targets Linux. The first-party flake assembles the shared builder
# with the broader set of platforms it exports publicly. Do not widen this
# without an explicit product decision.

import ../mk_yazelix_package.nix {
  inherit cargoGitOutputHashes pkgs src;
  marsTerminalPackage = mars_terminal;
  metaPlatforms = pkgs.lib.platforms.linux;
  yazelixCursorsPackage = yazelix_cursors;
  yazelixHelixPackage = yazelix_helix;
  yaziAssets = yazelix_yazi_assets;
  zellijPluginArtifacts = {
    pane_orchestrator = "${yazelix_zellij_pane_orchestrator}/${yazelix_zellij_pane_orchestrator.wasmPath}";
    zjstatus = "${yazelix_zjstatus}/bin/zjstatus.wasm";
    yzpp = "${yazelix_zellij_popup}/${yazelix_zellij_popup.wasmPath}";
  };
}
