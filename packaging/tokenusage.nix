{ pkgs }:

pkgs.rustPlatform.buildRustPackage rec {
  pname = "tokenusage";
  version = "1.5.2";

  src = pkgs.fetchCrate {
    inherit pname version;
    hash = "sha256-SDXBO4qxtdOd2FsdVU+E4SMn6rh4ZwCCXFxgc+0c6KQ=";
  };

  cargoHash = "sha256-gZMMhpL5OkLX74HurQEJKZ1R6QZD/9OTTG2LTbiAmZc=";

  nativeBuildInputs = [ pkgs.pkg-config ];

  buildInputs = pkgs.lib.optionals pkgs.stdenv.hostPlatform.isLinux [
    pkgs.fontconfig
    pkgs.libxkbcommon
    pkgs.libx11
    pkgs.libxcursor
    pkgs.libxi
    pkgs.libxrandr
    pkgs.wayland
  ];

  doCheck = false;

  meta = {
    description = "Fast local token usage tracker for Codex and Claude Code";
    homepage = "https://github.com/hanbu97/tokenusage";
    license = pkgs.lib.licenses.mit;
    mainProgram = "tu";
  };
}
