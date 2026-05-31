{
  pkgs,
  baseYaziUnwrapped ? pkgs.yazi-unwrapped,
  codeSrc,
  manSrc ? baseYaziUnwrapped.passthru.srcs.man_src,
  sourceRoot ? "yazi-yazelix-kgp-src",
  version ? "26.5.6",
  cargoHash ? "sha256-gc0uEMNJ+eCIymXK10+Swi11xuyP5cj6MbLLB/ZDgXw=",
}:

let
  pname = "yazi";
  srcs = [
    codeSrc
    manSrc
  ];
in
baseYaziUnwrapped.overrideAttrs (old: {
  inherit pname version srcs sourceRoot;

  # Keep KGP Yazi source-coupled patch metadata owned here. Consumer
  # yazi-unwrapped patches may target a different upstream source shape.
  patches = [ ];
  prePatch = "";
  postPatch = "";

  cargoDeps = pkgs.rustPlatform.fetchCargoVendor {
    inherit pname version srcs sourceRoot;
    hash = cargoHash;
  };

  passthru = old.passthru // {
    srcs = old.passthru.srcs // {
      code_src = codeSrc;
      man_src = manSrc;
    };
  };
})
