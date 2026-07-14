{
  pkgs,
  version ? "0.10.0",
  rev ? "3b619093e1e9907bca65386715540ea445947fe0",
  srcHash ? "sha256-+shHEqFa/ixttYwl8aUfQ/MJfQJXpI38mPdHoa2Oyhg=",
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

  cargoHash = "sha256-8cGTNNx6M5e53xcb7h8BAjKFoLnQCmN3girtXKUv6rw=";

  doCheck = false;

  meta = {
    description = "Content-addressed compiler cache used by the FlexNetOS/Yazelix Rust toolchain";
    homepage = "https://github.com/kunobi-ninja/kache";
    license = pkgs.lib.licenses.asl20;
    mainProgram = "kache";
  };
}
