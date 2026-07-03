{
  pkgs,
  version ? "local",
  binaryPath ? builtins.getEnv "FLEXNETOS_RTK_PATH",
}:

let
  resolvedBinaryPath =
    if binaryPath == "" then
      throw "FLEXNETOS_RTK_PATH must point at the rtk binary when building the FlexNetOS foundation package"
    else
      /. + binaryPath;
in
pkgs.stdenvNoCC.mkDerivation {
  pname = "rtk";
  inherit version;
  src = resolvedBinaryPath;
  dontUnpack = true;

  installPhase = ''
    runHook preInstall
    mkdir -p "$out/bin"
    install -m 755 "$src" "$out/bin/rtk"
    runHook postInstall
  '';

  meta = with pkgs.lib; {
    description = "RTK CLI packaged from the current FlexNetOS local binary";
    license = licenses.mit;
    mainProgram = "rtk";
    platforms = [ "x86_64-linux" ];
  };
}
