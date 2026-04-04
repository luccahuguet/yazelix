{ pkgs, src ? ../. }:

let
  lockedDevenv = import ./locked_devenv_package.nix { inherit pkgs src; };
  runtimeDeps = [
    pkgs.nushell
    lockedDevenv
    pkgs.nix
    pkgs.coreutils
    pkgs.findutils
    pkgs.gnugrep
    pkgs.gnused
    pkgs.jq
    pkgs.util-linux
    pkgs.bash
  ];
  runtimeBinPath = pkgs.lib.makeBinPath runtimeDeps;

  runtime = import ./mk_runtime_tree.nix {
    inherit pkgs src;
    name = "yazelix-runtime";
  };
in
pkgs.symlinkJoin {
  name = "yazelix";
  paths = [ runtime ];
  nativeBuildInputs = [ pkgs.makeWrapper ];

  postBuild = ''
    rm -f "$out/bin/yzx"
    makeWrapper "$out/shells/posix/yzx_cli.sh" "$out/bin/yzx" \
      --prefix PATH : "${runtimeBinPath}"
  '';

  meta = {
    description = "Reproducible terminal IDE built from Zellij, Yazi, and Helix";
    homepage = "https://github.com/luccahuguet/yazelix";
    license = pkgs.lib.licenses.mit;
    mainProgram = "yzx";
    platforms = pkgs.lib.platforms.linux;
  };
}
