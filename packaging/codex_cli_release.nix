{
  pkgs,
  version,
  system ? pkgs.stdenv.hostPlatform.system,
  releases ? {
    x86_64-linux = {
      systemSuffix = "x86_64-unknown-linux-musl";
      sha256 = "sha256-YOG+oQa4EHj1qXVMbyHKqIEGMIUyniRhMqGrnqJBpCc=";
    };
    aarch64-linux = {
      systemSuffix = "aarch64-unknown-linux-musl";
      sha256 = "sha256-tL+C0TpDJCg32lb81JcgD+b3/4ysYGEV5cdwft+9QNQ=";
    };
    x86_64-darwin = {
      systemSuffix = "x86_64-apple-darwin";
      sha256 = "sha256-gw7M2qUNmPNUJbln/wqOLk59vx2Y/N4ZDmM7ENL8e5A=";
    };
    aarch64-darwin = {
      systemSuffix = "aarch64-apple-darwin";
      sha256 = "sha256-1Q3UAzo7O7UuUv7MlRYNhSUrAQl/98DAkoDms7tXwhY=";
    };
  },
}:

let
  release =
    releases.${system} or (throw "unsupported Codex release system for Yazelix foundation: ${system}");
  packageName = "codex-package-${release.systemSuffix}";
  archive = pkgs.fetchurl {
    url = "https://github.com/openai/codex/releases/download/rust-v${version}/${packageName}.tar.gz";
    inherit (release) sha256;
  };
in
pkgs.stdenvNoCC.mkDerivation {
  pname = "codex-cli";
  inherit version;
  src = archive;
  dontConfigure = true;
  dontBuild = true;
  sourceRoot = ".";

  installPhase = ''
    runHook preInstall
    mkdir -p "$out"
    cp -R bin codex-package.json codex-path codex-resources "$out/"
    runHook postInstall
  '';

  meta = with pkgs.lib; {
    description = "OpenAI Codex CLI release binary";
    license = licenses.asl20;
    mainProgram = "codex";
    platforms = builtins.attrNames releases;
  };
}
