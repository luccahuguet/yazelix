{ pkgs }:

pkgs.stdenvNoCC.mkDerivation {
  pname = "yazelix-install-check";
  version = "0";

  src = ../shells/posix/install_check.sh;
  dontUnpack = true;

  installPhase = ''
    runHook preInstall
    install -Dm755 "$src" "$out/bin/install_check"
    runHook postInstall
  '';

  postFixup = ''
    patchShebangs "$out/bin/install_check"
  '';

  meta = {
    description = "Read-only bootstrap checks for installing Yazelix";
    mainProgram = "install_check";
    platforms = pkgs.lib.platforms.unix;
  };
}
