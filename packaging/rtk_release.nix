{
  pkgs,
  rtkSource,
  rustPlatform ? pkgs.rustPlatform,
}:

let
  manifest = builtins.fromTOML (builtins.readFile "${rtkSource}/Cargo.toml");
in
rustPlatform.buildRustPackage {
  pname = "rtk";
  version = manifest.package.version;

  src = rtkSource;

  cargoLock.lockFile = "${rtkSource}/Cargo.lock";
  doCheck = false;

  meta = with pkgs.lib; {
    description = "RTK CLI built from the FlexNetOS rtk-tokenkill source";
    homepage = "https://github.com/FlexNetOS/rtk-tokenkill";
    license = licenses.asl20;
    mainProgram = "rtk";
    platforms = [ "x86_64-linux" ];
  };
}
