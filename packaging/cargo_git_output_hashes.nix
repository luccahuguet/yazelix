{ yazelixCursors, yazelixYaziAssets }:

let
  requireString = description: value:
    if builtins.isString value then value else throw "${description} must be a string";
  cargoManifest = builtins.fromTOML (builtins.readFile ../rust_core/yazelix_core/Cargo.toml);
  cursorCargoRevision = requireString "Cargo dependency `yazelix_cursors.rev`" (
    cargoManifest.dependencies.yazelix_cursors.rev or null
  );
  cursorInputRevision = requireString
    "flake input `yazelixCursors.rev` (commit the child and use a git-backed override for local testing)"
    (yazelixCursors.rev or null);
  cursorInputNarHash = requireString "flake input `yazelixCursors.narHash`" (
    yazelixCursors.narHash or null
  );
  yaziCargoRevision = requireString "Cargo dependency `yazelix_yazi_assets.rev`" (
    cargoManifest.dependencies.yazelix_yazi_assets.rev or null
  );
  yaziInputRevision = requireString
    "flake input `yazelixYaziAssets.rev` (commit the child and use a git-backed override for local testing)"
    (yazelixYaziAssets.rev or null);
  yaziInputNarHash = requireString "flake input `yazelixYaziAssets.narHash`" (
    yazelixYaziAssets.narHash or null
  );
in
if cursorCargoRevision != cursorInputRevision then
  throw "Cargo dependency `yazelix_cursors` pins revision `${cursorCargoRevision}`, but flake input `yazelixCursors` pins `${cursorInputRevision}`; update Cargo.toml, Cargo.lock, and flake.lock in one child-release transaction"
else if yaziCargoRevision != yaziInputRevision then
  throw "Cargo dependency `yazelix_yazi_assets` pins revision `${yaziCargoRevision}`, but flake input `yazelixYaziAssets` pins `${yaziInputRevision}`; update Cargo.toml, Cargo.lock, and flake.lock in one child-release transaction"
else
  {
    "yazelix_cursors-0.1.0" = cursorInputNarHash;
    "ratconfig-2.0.0" = "sha256-NXnn7WOBEa7uQl8rs52gpIhpEGTeanRL5+au9ltjQyE=";
    "ratconfig-3.0.0" = "sha256-G47dgqPWzpN0bNkEU9nkqbTnqS+3aAOdCNYugamP0Pg=";
    "yazelix_screen-0.1.0" = "sha256-e8qM6kzHUNMsbBBQ21QJEAgJp5rqytDiXVIJmGaY9SE=";
    "yazelix_yazi_assets-0.1.0" = yaziInputNarHash;
  }
