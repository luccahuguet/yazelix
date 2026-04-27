{
  pkgs,
  nixgl ? null,
  runtimeVariant ? if pkgs.stdenv.hostPlatform.isLinux then "wezterm" else "ghostty",
}:

let
  ghosttyPackage =
    if pkgs.stdenv.hostPlatform.isDarwin then
      pkgs."ghostty-bin"
    else
      pkgs.ghostty;
  terminalPackage =
    if runtimeVariant == "ghostty" then
      ghosttyPackage
    else if runtimeVariant == "wezterm" then
      pkgs.wezterm
    else
      throw "Unsupported Yazelix runtimeVariant: ${runtimeVariant}";
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
  terminalPackage
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
  resvg
  nixVersions.latest
  coreutils
  findutils
  gnugrep
  gnused
  util-linux
] ++ pkgs.lib.optionals (linuxGlWrapperPackage != null) [
  linuxGlWrapperPackage
]
++ pkgs.lib.optionals pkgs.stdenv.hostPlatform.isLinux [
  procps
  xclip
  wl-clipboard
  xsel
]
