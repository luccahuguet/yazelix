# devenv.nix - Production configuration for Yazelix
# Mirrors the legacy flake-based shell while benefiting from devenv caching
{ pkgs, lib, inputs, ... }:

let
  inherit (pkgs.stdenv) isLinux isDarwin;

  nixglPackages = if isLinux then inputs.nixgl.packages.${pkgs.system} else null;
  nixglIntel = if nixglPackages != null && nixglPackages ? nixGLIntel then nixglPackages.nixGLIntel else null;

  # Import user configuration from TOML
  # IMPORTANT: yazelix.toml is gitignored, which makes it invisible to pure Nix evaluation
  # We must use --impure mode and read from an absolute path via $HOME
  # All devenv shell calls include --impure flag to enable this
  homeDir = builtins.getEnv "HOME";
  tomlConfigFile = if homeDir != "" then "${homeDir}/.config/yazelix/yazelix.toml" else "";
  defaultTomlConfigFile = ./yazelix_default.toml;

  rawConfig =
    if tomlConfigFile != "" && builtins.pathExists (builtins.toPath tomlConfigFile) then
      builtins.fromTOML (builtins.readFile tomlConfigFile)
    else
      builtins.fromTOML (builtins.readFile defaultTomlConfigFile);

  # Parse TOML config into the format devenv.nix expects
  userConfig = {
    recommended_deps = rawConfig.core.recommended_deps or true;
    yazi_extensions = rawConfig.core.yazi_extensions or true;
    yazi_media = rawConfig.core.yazi_media or false;
    debug_mode = rawConfig.core.debug_mode or false;
    skip_welcome_screen = rawConfig.core.skip_welcome_screen or false;
    show_macchina_on_welcome = rawConfig.core.show_macchina_on_welcome or false;

    helix_mode = rawConfig.helix.mode or "release";
    helix_runtime_path = rawConfig.helix.runtime_path or null;

    # Treat empty string as null for editor_command
    editor_command = let cmd = rawConfig.editor.command or null; in if cmd == "" then null else cmd;
    enable_sidebar = rawConfig.editor.enable_sidebar or true;

    default_shell = rawConfig.shell.default_shell or "nu";
    extra_shells = rawConfig.shell.extra_shells or [];
    enable_atuin = rawConfig.shell.enable_atuin or false;

    preferred_terminal = rawConfig.terminal.preferred_terminal or "ghostty";
    extra_terminals = rawConfig.terminal.extra_terminals or [];
    terminal_config_mode = rawConfig.terminal.config_mode or "yazelix";
    cursor_trail = rawConfig.terminal.cursor_trail or "random";
    transparency = rawConfig.terminal.transparency or "low";

    disable_zellij_tips = rawConfig.zellij.disable_tips or true;
    zellij_rounded_corners = rawConfig.zellij.rounded_corners or true;
    persistent_sessions = rawConfig.zellij.persistent_sessions or false;
    session_name = rawConfig.zellij.session_name or "yazelix";

    ascii_art_mode = rawConfig.ascii.mode or "static";

    language_packs = rawConfig.packs.language or [];
    tool_packs = rawConfig.packs.tools or [];
    user_packages = map (name: pkgs.${name}) (rawConfig.packs.user_packages or []);
  };

  boolToString = value: if value then "true" else "false";
  filterNull = builtins.filter (x: x != null);

  recommendedDepsEnabled = userConfig.recommended_deps or true;
  yaziExtensionsEnabled = userConfig.yazi_extensions or true;
  yaziMediaEnabled = userConfig.yazi_media or true;
  atuinEnabled = userConfig.enable_atuin or false;

  defaultShell = userConfig.default_shell or "nu";
  extraShells = userConfig.extra_shells or [ ];
  shellsToInclude = lib.unique (["nu" "bash" defaultShell] ++ extraShells);
  includeFish = lib.elem "fish" shellsToInclude;
  includeZsh = lib.elem "zsh" shellsToInclude;

  helixMode = userConfig.helix_mode or "release";
  buildHelixFromSource = helixMode == "source";

  helixPackage =
    if buildHelixFromSource then
      if inputs ? helix then
        inputs.helix.packages.${pkgs.system}.default
      else
        throw ''
          helix_mode = "source" requires the helix input.
          Add it to devenv.yaml and update devenv.lock.
        ''
    else
      pkgs.helix;

  helixRuntimePath = userConfig.helix_runtime_path or null;
  helixRuntime = if helixRuntimePath != null then helixRuntimePath else "${helixPackage}/lib/runtime";

  editorCommand =
    if (userConfig.editor_command or null) == null
    then "${helixPackage}/bin/hx"
    else userConfig.editor_command;

  preferredTerminal = userConfig.preferred_terminal or "ghostty";
  extraTerminals = userConfig.extra_terminals or [ ];
  terminalConfigMode = userConfig.terminal_config_mode or "yazelix";

  debugMode = userConfig.debug_mode or false;
  skipWelcomeScreen = userConfig.skip_welcome_screen or false;
  asciiArtMode = userConfig.ascii_art_mode or "static";
  enableSidebar = userConfig.enable_sidebar or true;
  showMacchinaOnWelcome = userConfig.show_macchina_on_welcome or false;

  yazelixLayoutName =
    if enableSidebar then
      "yzx_side"
    else
      "yzx_no_side";

  extraShellsStr =
    if extraShells == [ ]
    then "NONE"
    else builtins.concatStringsSep "," extraShells;

  # Terminal wrappers replicate the flake-based launchers
  ghosttyWrapper = if isLinux then
    pkgs.writeShellScriptBin "yazelix-ghostty" ''
      MODE="''${YAZELIX_TERMINAL_CONFIG_MODE:-${terminalConfigMode}}"
      MODE="''${MODE:-auto}"
      USER_CONF="$HOME/.config/ghostty/config"
      YZ_CONF="$HOME/.local/share/yazelix/configs/terminal_emulators/ghostty/config"
      CONF="$YZ_CONF"
      if [ "$MODE" = "user" ] || [ "$MODE" = "auto" ]; then
        if [ -f "$USER_CONF" ]; then CONF="$USER_CONF"; fi
      fi
      exec ${lib.optionalString (nixglIntel != null) "${nixglIntel}/bin/nixGLIntel "}${pkgs.ghostty}/bin/ghostty \
        --config-file="$CONF" \
        --class="com.yazelix.Yazelix" \
        --x11-instance-name="yazelix" \
        --title="Yazelix - Ghostty" "$@"
    ''
  else null;

  kittyWrapper =
    if preferredTerminal == "kitty" || lib.elem "kitty" extraTerminals then
      pkgs.writeShellScriptBin "yazelix-kitty" ''
        MODE="''${YAZELIX_TERMINAL_CONFIG_MODE:-${terminalConfigMode}}"
        MODE="''${MODE:-auto}"
        USER_CONF="$HOME/.config/kitty/kitty.conf"
        YZ_CONF="$HOME/.local/share/yazelix/configs/terminal_emulators/kitty/kitty.conf"
        CONF="$YZ_CONF"
        if [ "$MODE" = "user" ] || [ "$MODE" = "auto" ]; then
          if [ -f "$USER_CONF" ]; then CONF="$USER_CONF"; fi
        fi
        exec ${lib.optionalString (isLinux && nixglIntel != null) "${nixglIntel}/bin/nixGLIntel "}${pkgs.kitty}/bin/kitty \
          --config="$CONF" \
          --class="com.yazelix.Yazelix" \
          --title="Yazelix - Kitty" "$@"
      ''
    else null;

  weztermWrapper =
    if preferredTerminal == "wezterm" || lib.elem "wezterm" extraTerminals then
      pkgs.writeShellScriptBin "yazelix-wezterm" ''
        MODE="''${YAZELIX_TERMINAL_CONFIG_MODE:-${terminalConfigMode}}"
        MODE="''${MODE:-auto}"
        USER_CONF_MAIN="$HOME/.wezterm.lua"
        USER_CONF_ALT="$HOME/.config/wezterm/wezterm.lua"
        if [ -f "$USER_CONF_MAIN" ]; then USER_CONF="$USER_CONF_MAIN"; else USER_CONF="$USER_CONF_ALT"; fi
        YZ_CONF="$HOME/.local/share/yazelix/configs/terminal_emulators/wezterm/.wezterm.lua"
        CONF="$YZ_CONF"
        if [ "$MODE" = "user" ] || [ "$MODE" = "auto" ]; then
          if [ -f "$USER_CONF" ]; then CONF="$USER_CONF"; fi
        fi
        exec ${lib.optionalString (isLinux && nixglIntel != null) "${nixglIntel}/bin/nixGLIntel "}${pkgs.wezterm}/bin/wezterm \
          --config-file="$CONF" \
          --config 'window_decorations="NONE"' \
          --config enable_tab_bar=false \
          start --class=com.yazelix.Yazelix "$@"
      ''
    else null;

  alacrittyWrapper =
    if preferredTerminal == "alacritty" || lib.elem "alacritty" extraTerminals then
      pkgs.writeShellScriptBin "yazelix-alacritty" ''
        MODE="''${YAZELIX_TERMINAL_CONFIG_MODE:-${terminalConfigMode}}"
        MODE="''${MODE:-auto}"
        USER_CONF="$HOME/.config/alacritty/alacritty.toml"
        YZ_CONF="$HOME/.local/share/yazelix/configs/terminal_emulators/alacritty/alacritty.toml"
        CONF="$YZ_CONF"
        if [ "$MODE" = "user" ] || [ "$MODE" = "auto" ]; then
          if [ -f "$USER_CONF" ]; then CONF="$USER_CONF"; fi
        fi
        exec ${lib.optionalString (isLinux && nixglIntel != null) "${nixglIntel}/bin/nixGLIntel "}${pkgs.alacritty}/bin/alacritty \
          --config-file="$CONF" \
          --class="com.yazelix.Yazelix" \
          --title="Yazelix - Alacritty" "$@"
      ''
    else null;

  footWrapper =
    if isLinux && (preferredTerminal == "foot" || lib.elem "foot" extraTerminals) then
      pkgs.writeShellScriptBin "yazelix-foot" ''
        MODE="''${YAZELIX_TERMINAL_CONFIG_MODE:-${terminalConfigMode}}"
        MODE="''${MODE:-auto}"
        USER_CONF="$HOME/.config/foot/foot.ini"
        YZ_CONF="$HOME/.local/share/yazelix/configs/terminal_emulators/foot/foot.ini"
        CONF="$YZ_CONF"
        if [ "$MODE" = "user" ] || [ "$MODE" = "auto" ]; then
          if [ -f "$USER_CONF" ]; then CONF="$USER_CONF"; fi
        fi
        exec ${lib.optionalString (nixglIntel != null) "${nixglIntel}/bin/nixGLIntel "}${pkgs.foot}/bin/foot \
          --config="$CONF" \
          --app-id="com.yazelix.Yazelix" "$@"
      ''
    else null;

  yazelixDesktopLauncher =
    if isLinux then
      pkgs.writeShellScriptBin "yazelix-desktop-launcher" ''
        cd ~/.config/yazelix
        export YAZELIX_DIR="$HOME/.config/yazelix"
        exec devenv shell nu "$YAZELIX_DIR/nushell/scripts/core/launch_yazelix.nu"
      ''
    else null;

  yazelixDesktopEntry =
    if isLinux && yazelixDesktopLauncher != null then
      pkgs.makeDesktopItem {
        name = "com.yazelix.Yazelix";
        exec = "${yazelixDesktopLauncher}/bin/yazelix-desktop-launcher";
        icon = "yazelix";
        desktopName = "Yazelix";
        comment = "Yazi + Zellij + Helix integrated terminal environment";
        categories = [ "Development" ];
        startupWMClass = "com.yazelix.Yazelix";
      }
    else null;

  ghosttyDeps = if isLinux then filterNull [ ghosttyWrapper pkgs.ghostty ] else [ ];
  kittyDeps =
    if preferredTerminal == "kitty" || lib.elem "kitty" extraTerminals then
      filterNull [ kittyWrapper ]
      ++ [
        pkgs.kitty
        pkgs.nerd-fonts.fira-code
        pkgs.nerd-fonts.symbols-only
      ]
    else [ ];
  weztermDeps =
    if preferredTerminal == "wezterm" || lib.elem "wezterm" extraTerminals then
      filterNull [ weztermWrapper ] ++ [ pkgs.wezterm ]
    else [ ];
  alacrittyDeps =
    if preferredTerminal == "alacritty" || lib.elem "alacritty" extraTerminals then
      filterNull [ alacrittyWrapper ]
      ++ [
        pkgs.alacritty
        pkgs.nerd-fonts.fira-code
        pkgs.nerd-fonts.symbols-only
      ]
    else [ ];
  footDeps =
    if isLinux && (preferredTerminal == "foot" || lib.elem "foot" extraTerminals) then
      filterNull [ footWrapper ] ++ [ pkgs.foot ]
    else [ ];

  essentialDeps =
    with pkgs;
    [
      zellij
      helixPackage
      yazi
      nushell
      fzf
      zoxide
      starship
      bashInteractive
      macchina
      mise
    ]
    ++ lib.optionals isLinux [ libnotify ]
    ++ filterNull [ yazelixDesktopLauncher yazelixDesktopEntry ]
    ++ (if isLinux && nixglIntel != null then [ nixglIntel ] else [ ])
    ++ ghosttyDeps
    ++ kittyDeps
    ++ weztermDeps
    ++ alacrittyDeps
    ++ footDeps;

  extraShellDeps =
    (if includeFish then [ pkgs.fish ] else [ ])
    ++ (if includeZsh then [ pkgs.zsh ] else [ ]);

  recommendedDeps =
    with pkgs;
    [
      lazygit
      atuin
      carapace
      markdown-oxide
    ];

  yaziExtensionsDeps =
    with pkgs;
    [
      p7zip
      jq
      fd
      ripgrep
      poppler
    ];

  yaziMediaDeps =
    with pkgs;
    [
      ffmpeg
      imagemagick
    ];

  packDefinitions = {
    python = with pkgs; [
      ruff
      uv
      ty
      python3Packages.ipython
    ];
    ts = with pkgs; [
      nodePackages.typescript-language-server
      biome
      oxlint
      bun
    ];
    rust = with pkgs; [
      cargo-update
      cargo-binstall
      cargo-edit
      cargo-watch
      cargo-audit
      cargo-nextest
    ];
    config = with pkgs; [
      taplo
      mpls
    ];
    file-management = with pkgs; [
      ouch
      erdtree
      serpl
    ];
    git = with pkgs; [
      onefetch
      gh
      delta
      gitleaks
      jujutsu
      prek
    ];
    nix = with pkgs; [
      nil
      nixd
      nixfmt-rfc-style
    ];
    go = with pkgs; [
      gopls
      golangci-lint
      delve
      air
      govulncheck
    ];
    kotlin = with pkgs; [
      kotlin-language-server
      ktlint
      detekt
      gradle
    ];
    gleam = with pkgs; [
      gleam
    ];
  };

  selectedLanguagePacks = userConfig.language_packs or [ ];
  selectedToolPacks = userConfig.tool_packs or [ ];
  selectedPacks = selectedLanguagePacks ++ selectedToolPacks;
  packPackages = builtins.concatLists (
    map (packName:
      if builtins.hasAttr packName packDefinitions then
        packDefinitions.${packName}
      else
        throw "Unknown pack '${packName}'. Available packs: ${builtins.concatStringsSep ", " (builtins.attrNames packDefinitions)}"
    ) selectedPacks
  );

  allDeps =
    essentialDeps
    ++ extraShellDeps
    ++ (if recommendedDepsEnabled then recommendedDeps else [ ])
    ++ (if yaziExtensionsEnabled then yaziExtensionsDeps else [ ])
    ++ (if yaziMediaEnabled then yaziMediaDeps else [ ])
    ++ packPackages
    ++ (userConfig.user_packages or [ ]);

in {
  packages = allDeps;

  env = {
    YAZELIX_DIR = "$HOME/.config/yazelix";
    IN_YAZELIX_SHELL = "true";
    NIX_CONFIG = "warn-dirty = false";
    YAZELIX_DEBUG_MODE = boolToString debugMode;
    ZELLIJ_DEFAULT_LAYOUT = yazelixLayoutName;
    YAZELIX_DEFAULT_SHELL = defaultShell;
    YAZELIX_ENABLE_SIDEBAR = boolToString enableSidebar;
    YAZI_CONFIG_HOME = "$HOME/.local/share/yazelix/configs/yazi";
    YAZELIX_HELIX_MODE = helixMode;
    YAZELIX_PREFERRED_TERMINAL = preferredTerminal;
    YAZELIX_TERMINAL_CONFIG_MODE = terminalConfigMode;
    YAZELIX_ASCII_ART_MODE = asciiArtMode;
    EDITOR = editorCommand;
    HELIX_RUNTIME = helixRuntime;
  };

  enterShell = ''
    if [ -z "$HOME" ]; then
      export HOME="$(dirname "$(dirname "$DEVENV_ROOT")")"
    fi

    export YAZELIX_DIR="$HOME/.config/yazelix"
    export IN_YAZELIX_SHELL="true"
    export NIX_CONFIG="warn-dirty = false"
    export YAZELIX_DEBUG_MODE="${boolToString debugMode}"
    export ZELLIJ_DEFAULT_LAYOUT="${yazelixLayoutName}"
    export YAZELIX_DEFAULT_SHELL="${defaultShell}"
    export YAZELIX_ENABLE_SIDEBAR="${boolToString enableSidebar}"
    export YAZI_CONFIG_HOME="$HOME/.local/share/yazelix/configs/yazi"
    export YAZELIX_HELIX_MODE="${helixMode}"
    export YAZELIX_PREFERRED_TERMINAL="${preferredTerminal}"
    export YAZELIX_TERMINAL_CONFIG_MODE="${terminalConfigMode}"
    export YAZELIX_ASCII_ART_MODE="${asciiArtMode}"
    export EDITOR="${editorCommand}"
    export HELIX_RUNTIME="${helixRuntime}"

    if [ "$YAZELIX_ENV_ONLY" != "true" ]; then
      echo "📝 Set EDITOR to: ${editorCommand}"
    fi

    echo "🔁 Yazelix shell config: default=${defaultShell}, extras=${extraShellsStr}, includeFish=${if includeFish then "true" else "false"}, includeZsh=${if includeZsh then "true" else "false"}"

    nu "$YAZELIX_DIR/nushell/scripts/setup/environment.nu" \
      "$YAZELIX_DIR" \
      "${boolToString recommendedDepsEnabled}" \
      "${boolToString atuinEnabled}" \
      "${boolToString buildHelixFromSource}" \
      "${defaultShell}" \
      "${boolToString debugMode}" \
      "${extraShellsStr}" \
      "${boolToString skipWelcomeScreen}" \
      "${helixMode}" \
      "${asciiArtMode}" \
      "${boolToString showMacchinaOnWelcome}"
  '';
}
