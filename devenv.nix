# devenv.nix - Fixed maintainer shell for Yazelix v15 work
{
  pkgs,
  lib,
  inputs,
  ...
}:

let
  system = pkgs.stdenv.hostPlatform.system;
  runtimeDeps = import ./packaging/runtime_deps.nix { inherit pkgs; };

  fenixPkgs =
    if inputs ? fenix then
      inputs.fenix.packages.${system}
    else
      null;

  llmAgentsPkgs =
    if inputs ? llm-agents then
      inputs.llm-agents.packages.${system}
    else
      { };

  homeDir = builtins.getEnv "HOME";
  configRoot = if homeDir != "" then "${homeDir}/.config/yazelix" else "";
  userConfigDir = if configRoot != "" then "${configRoot}/user_configs" else "";
  tomlConfigFile = if userConfigDir != "" then "${userConfigDir}/yazelix.toml" else "";
  legacyTomlConfigFile = if configRoot != "" then "${configRoot}/yazelix.toml" else "";
  defaultTomlConfigFile = ./yazelix_default.toml;

  mainConfigPath =
    if tomlConfigFile != "" && builtins.pathExists (builtins.toPath tomlConfigFile) then
      tomlConfigFile
    else if legacyTomlConfigFile != "" && builtins.pathExists (builtins.toPath legacyTomlConfigFile) then
      legacyTomlConfigFile
    else
      defaultTomlConfigFile;

  rawConfig = builtins.fromTOML (builtins.readFile mainConfigPath);
  rawEditor = rawConfig.editor or { };
  rawHelix = rawConfig.helix or { };
  rawShell = rawConfig.shell or { };

  configuredEditor =
    let
      cmd = rawEditor.command or null;
    in
    if cmd == "" then null else cmd;

  helixRuntimePath =
    let
      runtimePath = rawHelix.runtime_path or null;
    in
    if runtimePath == "" then null else runtimePath;

  enableSidebar = rawEditor.enable_sidebar or true;
  defaultShell = rawShell.default_shell or "nu";

  isNamedNeovimEditor =
    configuredEditor != null
    && (
      configuredEditor == "nvim"
      || configuredEditor == "neovim"
    );

  isNamedHelixEditor =
    configuredEditor == null
    || configuredEditor == "hx"
    || configuredEditor == "helix"
    || lib.hasSuffix "/hx" configuredEditor
    || lib.hasSuffix "/helix" configuredEditor;

  editorCommand =
    if configuredEditor == null then
      "${pkgs.helix}/bin/hx"
    else if isNamedNeovimEditor then
      "${pkgs.neovim}/bin/nvim"
    else
      configuredEditor;

  managedEditorKind =
    if isNamedHelixEditor then
      "helix"
    else if isNamedNeovimEditor then
      "neovim"
    else
      "";

  editorLauncher =
    if managedEditorKind == "helix" then
      "$DEVENV_ROOT/shells/posix/yazelix_hx.sh"
    else
      editorCommand;

  rustWasiToolchain =
    if fenixPkgs != null then
      fenixPkgs.combine [
        fenixPkgs.stable.cargo
        fenixPkgs.stable.rustc
        fenixPkgs.stable.rustfmt
        fenixPkgs.stable.clippy
        fenixPkgs.targets.wasm32-wasip1.stable.rust-std
      ]
    else
      null;

  maintainerDeps =
    [ pkgs.github-cli ]
    ++ lib.optionals (builtins.hasAttr "beads-rust" llmAgentsPkgs) [ llmAgentsPkgs."beads-rust" ]
    ++ lib.optionals (builtins.hasAttr "beads-viewer" llmAgentsPkgs) [ llmAgentsPkgs."beads-viewer" ]
    ++ lib.optionals (rustWasiToolchain != null) [ rustWasiToolchain ];

  allDeps = lib.unique (runtimeDeps ++ maintainerDeps);

  yazelixNixConfig = ''
    warn-dirty = false
    extra-substituters = https://cache.numtide.com
    extra-trusted-public-keys = niks3.numtide.com-1:DTx8wZduET09hRmMtKdQDxNNthLQETkc/yaX7M4qK0g=
  '';

  yazelixLayoutName = if enableSidebar then "yzx_side" else "yzx_no_side";
in
{
  devenv.warnOnNewVersion = false;

  cachix.pull = [
    "numtide"
    "helix"
    "nix-community"
  ];

  packages = allDeps;

  env = {
    IN_YAZELIX_SHELL = "true";
    NIX_CONFIG = yazelixNixConfig;
    ZELLIJ_DEFAULT_LAYOUT = yazelixLayoutName;
    YAZELIX_DEFAULT_SHELL = defaultShell;
  };

  enterShell = ''
    if [ -z "$HOME" ]; then
      export HOME="$(dirname "$(dirname "$DEVENV_ROOT")")"
    fi

    runtime_root="$DEVENV_ROOT"
    unset YAZELIX_DIR
    export YAZELIX_RUNTIME_DIR="$runtime_root"
    export YAZELIX_CONFIG_DIR="$HOME/.config/yazelix"
    export YAZELIX_STATE_DIR="$HOME/.local/share/yazelix"
    export YAZELIX_LOGS_DIR="$YAZELIX_STATE_DIR/logs"
    export IN_YAZELIX_SHELL="true"
    export NIX_CONFIG='${yazelixNixConfig}'
    export ZELLIJ_DEFAULT_LAYOUT="${yazelixLayoutName}"
    export YAZELIX_DEFAULT_SHELL="${defaultShell}"
    export YAZI_CONFIG_HOME="$YAZELIX_STATE_DIR/configs/yazi"
    export EDITOR="${editorLauncher}"
    ${lib.optionalString (managedEditorKind == "helix") ''
      export YAZELIX_MANAGED_HELIX_BINARY="${editorCommand}"
    ''}
    ${lib.optionalString (helixRuntimePath != null) ''
      export HELIX_RUNTIME="${helixRuntimePath}"
    ''}

    if [ "$YAZELIX_ENV_ONLY" != "true" ] && [ "$YAZELIX_SHELLHOOK_SKIP_WELCOME" != "true" ]; then
      echo "🧭 Yazelix maintainer shell"
      echo "   Fixed runtime + maintainer toolchain; no dynamic pack graph or terminal wrapper ownership."
      echo "   Default shell preference: ${defaultShell}"
      echo "   EDITOR: $EDITOR"
    fi

    if [ "$YAZELIX_SHELLHOOK_SKIP_WELCOME" = "true" ]; then
      ${pkgs.nushell}/bin/nu "$runtime_root/nushell/scripts/setup/environment.nu" --skip-welcome
      unset YAZELIX_SHELLHOOK_SKIP_WELCOME
    else
      ${pkgs.nushell}/bin/nu "$runtime_root/nushell/scripts/setup/environment.nu"
    fi
  '';
}
