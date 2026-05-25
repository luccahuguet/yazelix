{
  pkgs,
  baseZellij ? pkgs.zellij,
  src,
  version ? "0.44.3",
  cargoHash ? "sha256-966FpfSsF9I10SrYe3+YNsfM2kLLv+gd0/Aw8vLp4Lk=",
}:

let
  pname = "zellij";
in
baseZellij.overrideAttrs (_old: {
  inherit pname version src;

  cargoDeps = pkgs.rustPlatform.fetchCargoVendor {
    inherit pname version src;
    hash = cargoHash;
  };
})
