{
  pkgs,
  baseZellij ? pkgs.zellij.unwrapped,
  src,
  version ? "0.44.3",
  cargoHash ? "sha256-966FpfSsF9I10SrYe3+YNsfM2kLLv+gd0/Aw8vLp4Lk=",
}:

let
  pname = "zellij";
in
baseZellij.overrideAttrs (_old: {
  inherit pname version src;

  # Keep KGP Zellij source-coupled patch and install-check metadata owned
  # here. Consumer zellij-unwrapped hooks may target a different upstream source shape.
  patches = [ ];
  prePatch = "";
  postPatch = "";
  installCheckPhase = ''
    runHook preInstallCheck
    runHook postInstallCheck
  '';

  cargoDeps = pkgs.rustPlatform.fetchCargoVendor {
    inherit pname version src;
    hash = cargoHash;
  };

  # Runtime package builds install Yazelix's forked Zellij. CI and maintainer
  # checks own test execution so Home Manager switches do not compile Zellij's
  # release test graph.
  doCheck = false;
})
