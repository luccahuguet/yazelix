{ pkgs, src }:

# Local upstream-prep draft:
# keep src injected here so the package body stays directly testable from the
# current repo. The real nixpkgs submission should replace this with the chosen
# release/version fetcher stanza once the upstream PR is opened.

import ../mk_yazelix_package.nix {
  inherit pkgs src;
}
