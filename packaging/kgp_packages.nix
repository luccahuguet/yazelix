{ yazelixZellij, yazelixHelix }:

let
  zellijBuildBase =
    pkgs: zellij:
    if zellij ? unwrapped then
      zellij.unwrapped
    else if builtins.hasAttr "zellij-unwrapped" pkgs then
      pkgs."zellij-unwrapped"
    else
      zellij;
  mkZellij =
    pkgs: baseZellij:
    import ./yazelix_kgp_zellij.nix { inherit pkgs baseZellij; src = yazelixZellij; };
  helixPackage = system: yazelixHelix.packages.${system}.yazelix_helix;
  helixPkgs = system: pkgs: pkgs.extend (_final: _prev: { helix = helixPackage system; });
  graphicsPkgs =
    pkgs:
    pkgs.extend (final: prev: {
      zellij = mkZellij final (zellijBuildBase prev prev.zellij);
    });
in
{
  inherit graphicsPkgs helixPackage helixPkgs mkZellij zellijBuildBase;
}
