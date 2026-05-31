{
  pkgs,
  baseHelix ? pkgs.helix,
  baseHelixUnwrapped ? pkgs.helix-unwrapped,
  src,
  version ? "25.07.1",
  cargoHash ? "sha256-6bu8sIM4So3AbnHHYbh8uu+rEB4IjMQjDgh7/AkLQs0=",
}:

let
  pname = "yazelix-helix-unwrapped";
  yazelixFork = {
    repo = "luccahuguet/yazelix-helix";
    steel = true;
    configDirFlag = true;
  };
  unwrapped = baseHelixUnwrapped.overrideAttrs (old: {
    inherit pname version src;

    patches = [ ];

    cargoBuildFeatures = pkgs.lib.unique ((old.cargoBuildFeatures or [ ]) ++ [ "steel" ]);
    cargoCheckFeatures = pkgs.lib.unique ((old.cargoCheckFeatures or [ ]) ++ [ "steel" ]);

    cargoDeps = pkgs.rustPlatform.fetchCargoVendor {
      inherit pname version src;
      hash = cargoHash;
    };

    passthru = (old.passthru or { }) // {
      inherit yazelixFork;
    };

    meta = (old.meta or { }) // {
      description = "Yazelix-owned Helix Steel editor runtime";
      homepage = "https://github.com/luccahuguet/yazelix-helix";
    };
  });
  wrapped = baseHelix.override {
    helix-unwrapped = unwrapped;
  };
in
wrapped.overrideAttrs (old: {
  passthru = (old.passthru or { }) // {
    inherit yazelixFork;
  };

  meta = (old.meta or { }) // {
    description = "Yazelix-owned Helix Steel editor runtime";
    homepage = "https://github.com/luccahuguet/yazelix-helix";
  };
})
