{ pkgs, yazelixCursors, yazelixYaziAssets }:

let
  hashes = import ./cargo_git_output_hashes.nix {
    inherit yazelixCursors yazelixYaziAssets;
  };
  yaziMismatch = builtins.tryEval (
    (import ./cargo_git_output_hashes.nix {
      inherit yazelixCursors;
      yazelixYaziAssets = {
        inherit (yazelixYaziAssets) narHash;
        rev = "${yazelixYaziAssets.rev}-mismatch";
      };
    })."yazelix_yazi_assets-0.1.0"
  );
  cursorMismatch = builtins.tryEval (
    (import ./cargo_git_output_hashes.nix {
      inherit yazelixYaziAssets;
      yazelixCursors = {
        inherit (yazelixCursors) narHash;
        rev = "${yazelixCursors.rev}-mismatch";
      };
    })."yazelix_cursors-0.1.0"
  );
in
assert hashes."yazelix_yazi_assets-0.1.0" == yazelixYaziAssets.narHash;
assert hashes."yazelix_cursors-0.1.0" == yazelixCursors.narHash;
assert !yaziMismatch.success;
assert !cursorMismatch.success;
pkgs.runCommand "yazelix-cargo-git-output-hash-contracts" { } ''
  touch "$out"
''
