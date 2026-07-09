{
  pkgs,
  version ? "2.1.202",
}:

let
  stdenv = pkgs.stdenvNoCC;
  platformKey = "${stdenv.hostPlatform.node.platform}-${stdenv.hostPlatform.node.arch}";
  checksums = {
    "2.1.202" = {
      "darwin-arm64" = "7414f707861e2fe5afef33a466f888a8d2170e5028f5e9d2858f1d3ef45ffca5";
      "darwin-x64" = "0dc578bb294094f5041e99a0444030ac6ae7236b387e56f00d4a5214816763bd";
      "linux-arm64" = "de5e0bb28e2b32409444ed4c1431e2931001c05ed270a3dc96c6706b0693867f";
      "linux-x64" = "71590202249892db3805ecd5b867f831f04b8129eaabd3f9a5bd4ba16b52c839";
      "linux-arm64-musl" = "80405fead329dd67d786b2a3d49bb121797a157937c99dedae2e36fcc77b55e6";
      "linux-x64-musl" = "bd62d47b677b8867e34f32642ee13f9fb87ad31b8acfdd326307eeffec02ec89";
    };
  };
in
pkgs."claude-code".overrideAttrs (_old: {
  inherit version;

  src = pkgs.fetchurl {
    url = "https://downloads.claude.ai/claude-code-releases/${version}/${platformKey}/claude";
    sha256 = checksums.${version}.${platformKey};
  };
})
