{
  pkgs,
  version ? "0.2.12",
}:

let
  platform =
    if pkgs.stdenv.hostPlatform.system == "x86_64-linux" then
      "linux-x64"
    else
      throw "git-kb ${version} is only packaged for x86_64-linux in the FlexNetOS foundation runtime";
in
pkgs.stdenvNoCC.mkDerivation {
  pname = "git-kb";
  inherit version;
  src = pkgs.fetchurl {
    url = "https://github.com/gitkb/gitkb-releases/releases/download/v${version}/gitkb-${platform}.tar.gz";
    hash = "sha256-YOJFkRv0wfxB7pH1GAIOgv1nTGmo37BWdpeHHueVUX8=";
  };

  dontConfigure = true;
  dontBuild = true;
  sourceRoot = ".";

  installPhase = ''
    runHook preInstall
    mkdir -p "$out/bin"
    install -m 755 git-kb "$out/bin/git-kb"
    runHook postInstall
  '';

  meta = with pkgs.lib; {
    description = "GitKB CLI packaged from the published GitKB release";
    license = licenses.mit;
    mainProgram = "git-kb";
    platforms = [ "x86_64-linux" ];
  };
}
