{ pkgs, src ? ../. }:

let
  system = pkgs.stdenv.hostPlatform.system;
  lock = builtins.fromJSON (builtins.readFile (src + "/devenv.lock"));
  lockedNode =
    if lock.nodes ? devenv && lock.nodes.devenv ? locked then
      lock.nodes.devenv.locked
    else
      throw "devenv.lock does not contain a locked devenv input";
  lockedSource = pkgs.fetchFromGitHub {
    owner = lockedNode.owner;
    repo = lockedNode.repo;
    rev = lockedNode.rev;
    hash = lockedNode.narHash;
  };
  lockedCompat = import (builtins.toPath "${lockedSource}/default.nix");
in
lockedCompat.packages.${system}.devenv
