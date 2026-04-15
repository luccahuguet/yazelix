{ pkgs, src ? ../., nixgl ? null }:

let
  runtime = import ./mk_runtime_tree.nix {
    inherit pkgs src nixgl;
    name = "yazelix-runtime";
  };
in
pkgs.symlinkJoin {
  name = "yazelix";
  paths = [ runtime ];

  nativeBuildInputs = [ pkgs.makeWrapper ];

  postBuild = ''
    for entry in "$out/bin/"*; do
      local name=$(basename "$entry")
      if [ "$name" != "yzx" ]; then
        rm -f "$entry"
      fi
    done

    rm -f "$out/bin/yzx"
    makeWrapper "$out/shells/posix/yzx_cli.sh" "$out/bin/yzx" \
      --run 'export YAZELIX_INVOKED_YZX_PATH="$0"' \
      --prefix PATH : "${runtime}/libexec"
  '';

  meta = {
    description = "Reproducible terminal IDE built from Zellij, Yazi, and Helix";
    homepage = "https://github.com/luccahuguet/yazelix";
    license = pkgs.lib.licenses.mit;
    mainProgram = "yzx";
    platforms = pkgs.lib.platforms.linux;
  };
}
