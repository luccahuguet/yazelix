{
  pkgs,
  version,
  system ? pkgs.stdenv.hostPlatform.system,
  releases ? {
    x86_64-linux = {
      systemSuffix = "x86_64-unknown-linux-musl";
      sha256 = "sha256-awPS2JkQh0+lvie2F2Iddjj5BuiR/Yy0CvPSh2qKNv0=";
    };
    aarch64-linux = {
      systemSuffix = "aarch64-unknown-linux-musl";
      sha256 = "sha256-1YvgTm7oBIM8JbWGhp8fpn8n8L3D85EFoqm6zvFnrkI=";
    };
    x86_64-darwin = {
      systemSuffix = "x86_64-apple-darwin";
      sha256 = "sha256-EFbICViGOxPevV2u5et7m9b4YjahFx0hsAni3O6odj4=";
    };
    aarch64-darwin = {
      systemSuffix = "aarch64-apple-darwin";
      sha256 = "sha256-RYSiQ/+KZxJQvHFvicWlDtWZF6mDkKz9/6Psts/luzQ=";
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
