{ yazelixYaziAssets }:

let
  requireString = description: value:
    if builtins.isString value then value else throw "${description} must be a string";
  cargoManifest = builtins.fromTOML (builtins.readFile ../rust_core/yazelix_core/Cargo.toml);
  cargoRevision = requireString "Cargo dependency `yazelix_yazi_assets.rev`" (
    cargoManifest.dependencies.yazelix_yazi_assets.rev or null
  );
  inputRevision = requireString
    "flake input `yazelixYaziAssets.rev` (commit the child and use a git-backed override for local testing)"
    (yazelixYaziAssets.rev or null);
  inputNarHash = requireString "flake input `yazelixYaziAssets.narHash`" (
    yazelixYaziAssets.narHash or null
  );
in
if cargoRevision != inputRevision then
  throw "Cargo dependency `yazelix_yazi_assets` pins revision `${cargoRevision}`, but flake input `yazelixYaziAssets` pins `${inputRevision}`; update Cargo.toml, Cargo.lock, and flake.lock in one child-release transaction"
else
  {
    "yazelix_cursors-0.1.0" = "sha256-NMHeKzfTzodG+Dfk4F/CLfEa2EiKodslnfaYQDBctxw=";
    "ratconfig-2.0.0" = "sha256-NXnn7WOBEa7uQl8rs52gpIhpEGTeanRL5+au9ltjQyE=";
    "yazelix_screen-0.1.0" = "sha256-e8qM6kzHUNMsbBBQ21QJEAgJp5rqytDiXVIJmGaY9SE=";
    "yazelix_yazi_assets-0.1.0" = inputNarHash;
  }
