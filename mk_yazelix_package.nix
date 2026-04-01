{ pkgs, src }:

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

  runtime = pkgs.symlinkJoin {
    name = "yazelix-runtime";
    paths = [ ];

    postBuild = ''
      ln -s ${src}/assets "$out/assets"
      ln -s ${src}/config_metadata "$out/config_metadata"
      ln -s ${src}/configs "$out/configs"
      ln -s ${src}/docs "$out/docs"
      ln -s ${src}/nushell "$out/nushell"
      ln -s ${src}/rust_plugins "$out/rust_plugins"
      ln -s ${src}/shells "$out/shells"

      ln -s ${src}/CHANGELOG.md "$out/CHANGELOG.md"
      ln -s ${src}/devenv.lock "$out/devenv.lock"
      ln -s ${src}/devenv.nix "$out/devenv.nix"
      ln -s ${src}/devenv.yaml "$out/devenv.yaml"
      ln -s ${src}/yazelix_default.toml "$out/yazelix_default.toml"
      ln -s ${src}/yazelix_packs_default.toml "$out/yazelix_packs_default.toml"

      mkdir -p "$out/bin"
      ln -s ${pkgs.lib.getBin pkgs.nushell}/bin/nu "$out/bin/nu"
      cat > "$out/bin/yzx" <<EOF
#!/bin/sh
PATH="${pkgs.lib.makeBinPath [ pkgs.nushell ]}:\$PATH"
export PATH
exec "\$(dirname "\$0")/../shells/posix/yzx_cli.sh" "\$@"
EOF
      chmod +x "$out/bin/yzx"
    '';
  };
in
pkgs.symlinkJoin {
  name = "yazelix";
  paths = [ runtime ];
  nativeBuildInputs = [ pkgs.makeWrapper ];

  postBuild = ''
    ln -s ${pkgs.lib.getBin pkgs.devenv}/bin/devenv "$out/bin/devenv"
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
