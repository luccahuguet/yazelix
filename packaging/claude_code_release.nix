{
  pkgs,
  version ? "2.1.207",
}:

let
  stdenv = pkgs.stdenvNoCC;
  platformKey = "${stdenv.hostPlatform.node.platform}-${stdenv.hostPlatform.node.arch}";
  checksums = {
    "2.1.207" = {
      "darwin-arm64" = "1397a062c6889675055e3314dd956376ac51262a7734ad9e819c26975d71547a";
      "darwin-x64" = "8a4355d251a60c90d8cf08f32fdb22a8157dd3d085542f95d0da0475f9a2c57c";
      "linux-arm64" = "8bc14a284065383460f37981d724b8f7aa7ca93c9849d2fe367e08f03383f454";
      "linux-x64" = "85e7e988a392d859f90802ca21fb26e89d3c9ab527f5ed0b08df3955e34d5c83";
      "linux-arm64-musl" = "ec3b657344dcf6693f434fe11ffe4592381d31d4e6a7976649c1a610770dcc74";
      "linux-x64-musl" = "09a43ff41e33cbb0c4903a4939353933ee8f0d1964abab4b837004a951edb9ee";
    };
    "2.1.205" = {
      "darwin-arm64" = "33e28624c5ae84f2bd7d2d8761e5d2e77997ba965cb11b6448de6b6e2c566f9c";
      "darwin-x64" = "4299a3f48551ef365f2d056f24d87e84b822c4c10b6acc46979446b7b5c60ceb";
      "linux-arm64" = "c1874c85bcd3a88b70439fd50ff5910b7e6ac5371c14dd49d4ccc2878a592d09";
      "linux-x64" = "dd8734c0b6a503fe1d17425184e57b397c30bb0337a33f1470d9985febfe5b09";
      "linux-arm64-musl" = "a8cd2a626d7d0b5fb3516164a4cf3b4acbbadb053a5b1b2a2462ccbd2ebf6bde";
      "linux-x64-musl" = "20018df16e75f4287c3bfb088e04019452cf262f66ee43041e285113c4e479d8";
    };
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
