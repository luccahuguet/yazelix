{ nixpkgs, system, kgpPackages }:

let
  pkgs = nixpkgs.legacyPackages.${system};
  zellijPoisonAttrs = {
    cargoDeps = throw "consumer Zellij build-base cargoDeps leaked into Yazelix KGP Zellij";
    patches = throw "consumer Zellij build-base patches leaked into Yazelix KGP Zellij";
    prePatch = throw "consumer Zellij build-base prePatch leaked into Yazelix KGP Zellij";
    postPatch = throw "consumer Zellij build-base postPatch leaked into Yazelix KGP Zellij";
    installCheckPhase =
      throw "consumer Zellij build-base installCheckPhase leaked into Yazelix KGP Zellij";
  };
  poisonZellijBuildBase =
    zellij:
    if zellij ? unwrapped then
      zellij.overrideAttrs (_old: {
        passthru = (zellij.passthru or { }) // {
          unwrapped = zellij.unwrapped.overrideAttrs (_oldUnwrapped: zellijPoisonAttrs);
        };
      })
    else
      zellij.overrideAttrs (_old: zellijPoisonAttrs);
  poisonedConsumerPkgs = import nixpkgs {
    inherit system;
    overlays = [
      (_final: prev:
        {
          zellij = poisonZellijBuildBase prev.zellij;
        }
        // (if builtins.hasAttr "zellij-unwrapped" prev then
          {
            zellij-unwrapped = prev."zellij-unwrapped".overrideAttrs (_old: zellijPoisonAttrs);
          }
        else
          { })
        // {
          yazi-unwrapped = prev.yazi-unwrapped.overrideAttrs (_old: {
            cargoDeps = throw "consumer pkgs.yazi-unwrapped cargoDeps leaked into Yazelix KGP Yazi";
            patches = throw "consumer pkgs.yazi-unwrapped patches leaked into Yazelix KGP Yazi";
            prePatch = throw "consumer pkgs.yazi-unwrapped prePatch leaked into Yazelix KGP Yazi";
            postPatch = throw "consumer pkgs.yazi-unwrapped postPatch leaked into Yazelix KGP Yazi";
          });
        })
    ];
  };
  wrappedNoPassthruConsumerPkgs = import nixpkgs {
    inherit system;
    overlays = [
      (_final: prev:
        let
          fallbackUnwrapped =
            if builtins.hasAttr "zellij-unwrapped" prev then
              prev."zellij-unwrapped"
            else
              prev.zellij;
        in
        {
          zellij = prev.zellij.overrideAttrs (old: {
            passthru = (builtins.removeAttrs (old.passthru or { }) [ "unwrapped" ]) // {
              __yazelix_test_base = "wrapper";
            };
          });
          zellij-unwrapped = fallbackUnwrapped.overrideAttrs (old: {
            passthru = (old.passthru or { }) // {
              __yazelix_test_base = "zellij-unwrapped";
            };
          });
        })
    ];
  };
  kgpZellij = kgpPackages.mkZellij poisonedConsumerPkgs (
    kgpPackages.zellijBuildBase poisonedConsumerPkgs poisonedConsumerPkgs.zellij
  );
  wrappedNoPassthruZellijBase =
    kgpPackages.zellijBuildBase wrappedNoPassthruConsumerPkgs wrappedNoPassthruConsumerPkgs.zellij;
  kgpZellijWrappedNoPassthru =
    kgpPackages.mkZellij wrappedNoPassthruConsumerPkgs wrappedNoPassthruZellijBase;
  kgpYazi = kgpPackages.mkYazi poisonedConsumerPkgs poisonedConsumerPkgs.yazi-unwrapped;
in
assert (wrappedNoPassthruZellijBase.__yazelix_test_base or "") == "zellij-unwrapped";
assert (kgpZellijWrappedNoPassthru.version or "") == "0.44.3";
assert (kgpZellijWrappedNoPassthru.cargoDeps.name or "") == "zellij-0.44.3-vendor";
assert (kgpZellij.version or "") == "0.44.3";
assert (kgpZellij.cargoDeps.name or "") == "zellij-0.44.3-vendor";
assert (kgpZellij.patches or [ ]) == [ ];
assert (kgpZellij.prePatch or "") == "";
assert (kgpZellij.postPatch or "") == "";
assert (kgpZellij.installCheckPhase or "") == ''
  runHook preInstallCheck
  runHook postInstallCheck
'';
assert (kgpYazi.version or "") == "26.5.6";
assert (kgpYazi.cargoDeps.name or "") == "yazi-26.5.6-vendor";
assert (kgpYazi.patches or [ ]) == [ ];
assert (kgpYazi.prePatch or "") == "";
assert (kgpYazi.postPatch or "") == "";
pkgs.runCommand "yazelix-kgp-package-contracts" { } ''
  touch "$out"
''
