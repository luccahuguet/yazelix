{ pkgs, runtime ? import ./yazelix_runtime_package.nix { inherit pkgs; } }:

let
  runtimeDeps = [
    pkgs.nushell
    pkgs.devenv
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
in
pkgs.symlinkJoin {
  name = "yazelix";
  paths = [ runtime ];

  postBuild = ''
    ln -s ${pkgs.devenv}/bin/devenv "$out/bin/devenv"
    rm -f "$out/bin/yzx"
    cat > "$out/bin/yzx" <<EOF
#!/bin/sh
PATH="${runtimeBinPath}:\$PATH"
export PATH
exec "\$(dirname "\$0")/../shells/posix/yzx_cli.sh" "\$@"
EOF
    chmod +x "$out/bin/yzx"
  '';

  meta = with pkgs.lib; {
    description = "Reproducible terminal IDE built from Zellij, Yazi, and Helix";
    homepage = "https://github.com/luccahuguet/yazelix";
    license = licenses.mit;
    platforms = platforms.linux;
    mainProgram = "yzx";
  };
}
