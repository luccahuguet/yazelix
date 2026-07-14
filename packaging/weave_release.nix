{
  pkgs,
  weaveSource,
  backend ? "sqlite",
  rustPlatform ? pkgs.rustPlatform,
}:

let
  manifest = builtins.fromTOML (builtins.readFile "${weaveSource}/Cargo.toml");
  validBackend = backend == "sqlite" || backend == "libsql";
in
assert validBackend;
rustPlatform.buildRustPackage {
  pname = if backend == "sqlite" then "weave" else "weave-libsql";
  version = manifest.workspace.package.version;

  src = weaveSource;

  cargoLock = {
    lockFile = "${weaveSource}/Cargo.lock";
    outputHashes = {
      "fnx-algorithms-0.1.0" = "sha256-p6xDFS2uVdUYYwYdFLEqaUWeto4sUGviJ1Iiox53cOU=";
    };
  };
  # workspace carries weave + weave-mcp + benches; the `weave` bin (which also
  # serves MCP via `weave mcp`) is the shipped surface.
  cargoBuildFlags = [
    "-p"
    "weave"
  ];
  # The installed profile intentionally uses the default SQLite backend while
  # enabling every orthogonal production surface. libSQL is exposed as a
  # separate derivation because the two store backends are mutually exclusive.
  buildNoDefaultFeatures = backend == "libsql";
  buildFeatures = [
    "sign"
    "llm"
    "surfaces"
    "obscura"
  ]
  ++ pkgs.lib.optional (backend == "libsql") "libsql";
  doCheck = false;
  postInstall = ''
    "$out/bin/weave" --version >weave-version.txt
    grep -F 'backends: ${backend}' weave-version.txt >/dev/null
    for command_name in key summarize dashboard telegram slack push web; do
      "$out/bin/weave" "$command_name" --help >/dev/null
    done
    mkdir -p "$TMPDIR/weave-post-install-home"
    HOME="$TMPDIR/weave-post-install-home" \
      WEAVE_DB="$TMPDIR/weave-post-install.db" \
      "$out/bin/weave" web --list >weave-web-list.txt
    grep -F 'tab_list' weave-web-list.txt >/dev/null
  '';

  passthru = {
    inherit backend;
    weaveFeatures = [
      (if backend == "sqlite" then "sqlite" else "libsql")
      "sign"
      "llm"
      "surfaces"
      "obscura"
    ];
  };

  meta = with pkgs.lib; {
    description = "weave A2A session mesh CLI (${backend}, sign, llm, surfaces, obscura) built from the FlexNetOS/weave source";
    homepage = "https://github.com/FlexNetOS/weave";
    license = licenses.asl20;
    mainProgram = "weave";
    platforms = [ "x86_64-linux" ];
  };
}
