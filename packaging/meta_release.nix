{
  pkgs,
  version ? "0.2.22",
}:

let
  platform =
    if pkgs.stdenv.hostPlatform.system == "x86_64-linux" then
      "linux-x64"
    else
      throw "meta ${version} is only packaged for x86_64-linux in the FlexNetOS foundation runtime";
in
pkgs.stdenvNoCC.mkDerivation {
  pname = "meta";
  inherit version;
  # The meta CLI's workspace member crates are not present in the public
  # repo trees (peer-repo routing restructure in flight), so the published
  # release binaries are the only buildable source of truth — same pattern
  # as git_kb_release.nix.
  src = pkgs.fetchurl {
    url = "https://github.com/gitkb/meta/releases/download/v${version}/meta-${platform}.tar.gz";
    hash = "sha256-KJRgRpJ9pG0E0tqsXbGmTaC+xG9l+JH9ywv0yeQg1ow=";
  };

  dontConfigure = true;
  dontBuild = true;
  sourceRoot = ".";

  installPhase = ''
    runHook preInstall
    mkdir -p "$out/bin"
    for bin in loop meta meta-git meta-mcp meta-project; do
      install -m 755 "$bin" "$out/bin/$bin"
    done
    runHook postInstall
  '';

  meta = with pkgs.lib; {
    description = "meta multi-repo management CLI packaged from the published gitkb/meta release";
    homepage = "https://github.com/gitkb/meta";
    license = licenses.mit;
    mainProgram = "meta";
    platforms = [ "x86_64-linux" ];
  };
}
