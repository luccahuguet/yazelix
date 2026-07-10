{
  pkgs,
  version ? "0.8.0a3",
}:

pkgs.python3Packages.buildPythonApplication {
  pname = "notebooklm-py";
  inherit version;
  pyproject = true;

  src = pkgs.fetchPypi {
    pname = "notebooklm_py";
    inherit version;
    hash = "sha256-X0r68aM6LUEjHR09ZejCq69YIJmbC/z5qilN7P7zJEA=";
  };

  build-system = with pkgs.python3Packages; [
    hatchling
    hatch-fancy-pypi-readme
  ];

  # Base deps plus the [browser] extra (playwright, used by `notebooklm
  # login` / headless reauth). This matches the capability of the prior
  # host-local `uv tool install "notebooklm-py[browser]"` it replaces.
  dependencies = with pkgs.python3Packages; [
    click
    filelock
    httpx
    rich
    playwright
  ];

  # Upstream caps (httpx<0.29, rich<16, ...) can trail the nixpkgs pin;
  # the foundation build proves the CLI against the pinned versions.
  pythonRelaxDeps = true;

  # The mcp/server extras are not packaged: fastmcp==3.4.2 is an exact pin
  # nixpkgs does not carry. Their console scripts would ImportError exactly
  # as they did in the uv install, so drop them instead of shipping known-
  # broken commands. Re-add alongside the extras when packaging them.
  postInstall = ''
    rm "$out/bin/notebooklm-mcp" "$out/bin/notebooklm-server"
  '';

  pythonImportsCheck = [ "notebooklm" ];

  meta = with pkgs.lib; {
    description = "NotebookLM CLI packaged from the published notebooklm-py PyPI release";
    homepage = "https://github.com/teng-lin/notebooklm-py";
    license = licenses.mit;
    mainProgram = "notebooklm";
    platforms = platforms.linux;
  };
}
