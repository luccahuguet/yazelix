{ pkgs }:

let
  # Ghostty is the single first-party terminal Yazelix owns across platforms.
  ghosttyPackage =
    if pkgs.stdenv.hostPlatform.isDarwin then
      pkgs."ghostty-bin"
    else
      pkgs.ghostty;
in
with pkgs;
[
  bashInteractive
  nushell
  zellij
  ghosttyPackage
  helix
  neovim
  yazi
  fzf
  zoxide
  starship
  lazygit
  carapace
  macchina
  mise
  taplo
  fish
  zsh
  git
  jq
  fd
  ripgrep
  p7zip
  poppler
  nix
  coreutils
  findutils
  gnugrep
  gnused
  util-linux
]
