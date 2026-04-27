{
  pkgs,
  src ?
    import ./packaging/repo_source.nix {
      lib = pkgs.lib;
      src = ./.;
    },
  rust_core_src ? ./.,
  nixgl ? null,
  fenixPkgs ? null,
  runtimeVariant ? if pkgs.stdenv.hostPlatform.isLinux then "wezterm" else "ghostty",
}:

let
  firstPartyPlatforms = [
    "x86_64-linux"
    "aarch64-linux"
    "x86_64-darwin"
    "aarch64-darwin"
  ];
in
import ./packaging/mk_yazelix_package.nix {
  inherit pkgs src rust_core_src nixgl fenixPkgs runtimeVariant;
  metaPlatforms = firstPartyPlatforms;
}
