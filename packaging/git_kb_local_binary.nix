{
  pkgs,
  version ? "0.2.12",
  binaryPath ? builtins.getEnv "FLEXNETOS_GIT_KB_PATH",
}:

let
  resolvedBinaryPath =
    if binaryPath == "" then
      throw "FLEXNETOS_GIT_KB_PATH must point at the git-kb binary when building the FlexNetOS foundation package"
    else
      /. + binaryPath;
in
pkgs.stdenvNoCC.mkDerivation {
  pname = "git-kb";
  inherit version;
  src = resolvedBinaryPath;
  dontUnpack = true;

  installPhase = ''
    runHook preInstall
    mkdir -p "$out/bin"
    install -m 755 "$src" "$out/bin/git-kb"
    runHook postInstall
  '';

  meta = with pkgs.lib; {
    description = "GitKB CLI packaged from the current FlexNetOS foundation binary";
    license = licenses.mit;
    mainProgram = "git-kb";
    platforms = [ "x86_64-linux" ];
  };
}
