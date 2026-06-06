{ yazelixZellij, yazelixYazi, yazelixHelix }:

let
  yaziCodeSrc = builtins.path { path = yazelixYazi; name = "yazi-yazelix-kgp-src"; };
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
  mkYazi = pkgs: baseYaziUnwrapped: import ./yazelix_kgp_yazi.nix { inherit pkgs baseYaziUnwrapped; codeSrc = yaziCodeSrc; };
  helixPackage = system: yazelixHelix.packages.${system}.yazelix_helix;
  helixPkgs = system: pkgs: pkgs.extend (_final: _prev: { helix = helixPackage system; });
  graphicsPkgs =
    pkgs:
    pkgs.extend (final: prev: {
      zellij = mkZellij final (zellijBuildBase prev prev.zellij);
      yazi-unwrapped = mkYazi final prev.yazi-unwrapped;
      yazi = prev.yazi.override {
        yazi-unwrapped = final.yazi-unwrapped;
      };
    });
in
{
  inherit graphicsPkgs helixPackage helixPkgs mkYazi mkZellij zellijBuildBase;
}
