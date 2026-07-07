{
  pkgs,
  version ? "0.8.0",
  rev ? "9bd87ccd18f6301534152d39427f0e3dd21b1fba",
  srcHash ? "sha256-Zhi6HpqKws5JUV99IHfXs4iBX5fSVqSdY7HKuCBrp7c=",
}:

pkgs.rustPlatform.buildRustPackage {
  pname = "kache";
  inherit version;

  src = pkgs.fetchFromGitHub {
    owner = "kunobi-ninja";
    repo = "kache";
    inherit rev;
    hash = srcHash;
  };

  cargoHash = "sha256-ol83gvXeXhJfJy5+O1+ZXw+fQdJvR3y5M/DmsYJG1vM=";

  doCheck = false;

  meta = {
    description = "Content-addressed compiler cache used by the FlexNetOS/Yazelix Rust toolchain";
    homepage = "https://github.com/kunobi-ninja/kache";
    license = pkgs.lib.licenses.asl20;
    mainProgram = "kache";
  };
}
