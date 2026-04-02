{ pkgs, src ? ./. }:

let
  system = pkgs.stdenv.hostPlatform.system;
  lock = builtins.fromJSON (builtins.readFile (src + "/devenv.lock"));
  lockedNode =
    if lock.nodes ? devenv && lock.nodes.devenv ? locked then
      lock.nodes.devenv.locked
    else
      throw "devenv.lock does not contain a locked devenv input";
  lockedTree = builtins.fetchTree {
    type = lockedNode.type;
    owner = lockedNode.owner;
    repo = lockedNode.repo;
    rev = lockedNode.rev;
    narHash = lockedNode.narHash;
  };
  lockedSource = builtins.path {
    path = lockedTree.outPath;
    name = "locked-devenv-source";
  };
  lockedCompat = import (lockedSource + "/default.nix");
in
lockedCompat.packages.${system}.devenv
