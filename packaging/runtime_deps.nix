{ pkgs, nixgl ? null }:

let
  # Ghostty is the single first-party terminal Yazelix owns across platforms.
  ghosttyPackage =
    if pkgs.stdenv.hostPlatform.isDarwin then
      pkgs."ghostty-bin"
    else
      pkgs.ghostty;
  linuxGlWrapperPackage =
    if pkgs.stdenv.hostPlatform.isLinux && (nixgl != null) then
      (
        import "${nixgl}/default.nix" {
          pkgs = pkgs;
          enable32bits = pkgs.stdenv.hostPlatform.isx86_64;
          enableIntelX86Extensions = pkgs.stdenv.hostPlatform.isx86_64;
        }
      ).nixGLMesa
    else
      null;
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
] ++ pkgs.lib.optionals (linuxGlWrapperPackage != null) [
  linuxGlWrapperPackage
]
