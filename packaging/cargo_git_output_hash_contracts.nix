{ pkgs, yazelixYaziAssets }:

let
  hashes = import ./cargo_git_output_hashes.nix { inherit yazelixYaziAssets; };
  mismatch = builtins.tryEval (
    (import ./cargo_git_output_hashes.nix {
      yazelixYaziAssets = {
        inherit (yazelixYaziAssets) narHash;
        rev = "${yazelixYaziAssets.rev}-mismatch";
      };
    })."yazelix_yazi_assets-0.1.0"
  );
in
assert hashes."yazelix_yazi_assets-0.1.0" == yazelixYaziAssets.narHash;
assert !mismatch.success;
pkgs.runCommand "yazelix-cargo-git-output-hash-contracts" { } ''
  touch "$out"
''
