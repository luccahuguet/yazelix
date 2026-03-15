# devenv.nix - Production configuration for Yazelix
# Mirrors the legacy flake-based shell while benefiting from devenv caching
{
  pkgs,
  lib,
  inputs,
  ...
}:

let
  inherit (pkgs.stdenv) isLinux isDarwin;
  system = pkgs.stdenv.hostPlatform.system;

  fenixPkgs = if inputs ? fenix then inputs.fenix.packages.${system} else null;
  nixglPackages = if isLinux then inputs.nixgl.packages.${system} else null;

  # LLM agents packages from numtide/llm-agents.nix (daily updates)
  llmAgentsPkgs =
    if inputs ? llm-agents then
      inputs.llm-agents.packages.${system}
    else
      { };

  # Packages to resolve from llm-agents instead of nixpkgs
  llmAgentsPackageNames = [
    "amp"
    "beads"
    "beads-rust"
    "beads-viewer"
    "ccusage"
    "ccusage-amp"
    "ccusage-codex"
    "ccusage-opencode"
    "claude-code"
    "coderabbit-cli"
    "code"
    "codex"
    "cursor-agent"
    "gemini-cli"
    "goose-cli"
    "openclaw"
    "pi"
    "picoclaw"
    "opencode"
    "zeroclaw"
  ];

  # Packages explicitly blocked in Yazelix packs/user_packages.
  blockedPackageNames = [ ];

  nixglIntel =
    if nixglPackages != null && nixglPackages ? nixGLIntel then nixglPackages.nixGLIntel else null;

  # Import user configuration from TOML
  # IMPORTANT: yazelix.toml is gitignored, so Yazelix reads it via an absolute path under $HOME.
  # Current devenv releases expose that path to evaluation without requiring a separate --impure flag.
  homeDir = builtins.getEnv "HOME";
  tomlConfigFile = if homeDir != "" then "${homeDir}/.config/yazelix/yazelix.toml" else "";
  defaultTomlConfigFile = ./yazelix_default.toml;

  rawConfig =
    if tomlConfigFile != "" && builtins.pathExists (builtins.toPath tomlConfigFile) then
      builtins.fromTOML (builtins.readFile tomlConfigFile)
    else
      builtins.fromTOML (builtins.readFile defaultTomlConfigFile);

  rawPacks = rawConfig.packs or { };
  _ =
    if rawPacks ? language || rawPacks ? tools then
      throw ''
        packs.language and packs.tools are deprecated.
        Use packs.enabled and packs.declarations instead.
      ''
    else
      null;

  # Parse TOML config into the format devenv.nix expects
  userConfig = {
    recommended_deps = rawConfig.core.recommended_deps or true;
    yazi_extensions = rawConfig.core.yazi_extensions or true;
    yazi_media = rawConfig.core.yazi_media or false;
    debug_mode = rawConfig.core.debug_mode or false;
    skip_welcome_screen = rawConfig.core.skip_welcome_screen or false;
    show_macchina_on_welcome = rawConfig.core.show_macchina_on_welcome or false;
    build_cores = rawConfig.core.build_cores or "max_minus_one";

    helix_mode = rawConfig.helix.mode or "release";
    helix_runtime_path = rawConfig.helix.runtime_path or null;

    # Treat empty string as null for editor_command
    editor_command =
      let
        cmd = rawConfig.editor.command or null;
      in
      if cmd == "" then null else cmd;
    enable_sidebar = rawConfig.editor.enable_sidebar or true;

    default_shell = rawConfig.shell.default_shell or "nu";
    extra_shells = rawConfig.shell.extra_shells or [ ];

    terminals = rawConfig.terminal.terminals or [ "ghostty" ];
    manage_terminals = rawConfig.terminal.manage_terminals or true;
    terminal_config_mode = rawConfig.terminal.config_mode or "yazelix";
    ghostty_trail_color = rawConfig.terminal.ghostty_trail_color or "random";
    transparency = rawConfig.terminal.transparency or "low";

    disable_zellij_tips = rawConfig.zellij.disable_tips or true;
    zellij_rounded_corners = rawConfig.zellij.rounded_corners or true;
    persistent_sessions = rawConfig.zellij.persistent_sessions or false;
    session_name = rawConfig.zellij.session_name or "yazelix";

    ascii_art_mode = rawConfig.ascii.mode or "static";

    pack_names = rawPacks.enabled or [ ];
    pack_declarations = rawPacks.declarations or { };
    user_packages = map resolvePkg (rawPacks.user_packages or [ ]);
  };

  boolToString = value: if value then "true" else "false";
  filterNull = builtins.filter (x: x != null);

  recommendedDepsEnabled = userConfig.recommended_deps or true;
  yaziExtensionsEnabled = userConfig.yazi_extensions or true;
  yaziMediaEnabled = userConfig.yazi_media or true;

  defaultShell = userConfig.default_shell or "nu";
  extraShells = userConfig.extra_shells or [ ];
  shellsToInclude = lib.unique (
    [
      "nu"
      "bash"
      defaultShell
    ]
    ++ extraShells
  );
  includeFish = lib.elem "fish" shellsToInclude;
  includeZsh = lib.elem "zsh" shellsToInclude;

  helixMode = userConfig.helix_mode or "release";
  buildHelixFromSource = helixMode == "source";

  helixPackage =
    if buildHelixFromSource then
      if inputs ? helix then
        inputs.helix.packages.${system}.default
      else
        throw ''
          helix_mode = "source" requires the helix input.
          Add it to devenv.yaml and update devenv.lock.
        ''
    else
      pkgs.helix;

  zjstatusPkg =
    if inputs ? zjstatus then
      inputs.zjstatus.packages.${system}.default
    else
      throw ''
        zjstatus input missing.
        Add it to devenv.yaml and update devenv.lock.
      '';

  helixRuntimePath = userConfig.helix_runtime_path or null;

  configuredEditor = userConfig.editor_command or null;
  isNamedNeovimEditor =
    configuredEditor != null
    && (
      configuredEditor == "nvim"
      || configuredEditor == "neovim"
    );

  editorCommand =
    if configuredEditor == null then
      "${helixPackage}/bin/hx"
    else if isNamedNeovimEditor then
      "${pkgs.neovim}/bin/nvim"
    else
      configuredEditor;

  terminalList = lib.unique (userConfig.terminals or [ ]);
  manageTerminals = userConfig.manage_terminals or true;
  preferredTerminal = if terminalList == [ ] then "unknown" else builtins.elemAt terminalList 0;
  terminalConfigMode = userConfig.terminal_config_mode or "yazelix";

  debugMode = userConfig.debug_mode or false;
  skipWelcomeScreen = userConfig.skip_welcome_screen or false;
  asciiArtMode = userConfig.ascii_art_mode or "static";
  enableSidebar = userConfig.enable_sidebar or true;
  showMacchinaOnWelcome = userConfig.show_macchina_on_welcome or false;
  yazelixNixConfig = ''
    warn-dirty = false
    extra-substituters = https://cache.numtide.com
    extra-trusted-public-keys = niks3.numtide.com-1:DTx8wZduET09hRmMtKdQDxNNthLQETkc/yaX7M4qK0g=
  '';

  yazelixLayoutName = if enableSidebar then "yzx_side" else "yzx_no_side";

  # Terminal wrappers replicate the flake-based launchers
  ghosttyWrapper = pkgs.writeShellScriptBin "yazelix-ghostty" (
    if isLinux then
      ''
        MODE="''${YAZELIX_TERMINAL_CONFIG_MODE:-${terminalConfigMode}}"
        MODE="''${MODE:-auto}"
        USER_CONF="$HOME/.config/ghostty/config"
        YZ_CONF="$HOME/.local/share/yazelix/configs/terminal_emulators/ghostty/config"
        CONF="$YZ_CONF"
        if [ "$MODE" = "user" ] || [ "$MODE" = "auto" ]; then
          if [ -f "$USER_CONF" ]; then CONF="$USER_CONF"; fi
        fi
        # On Wayland, stale IM variables (e.g. GTK_IM_MODULE=ibus without daemon)
        # can break dead-key/compose input in GTK terminals.
        if [ -n "''${WAYLAND_DISPLAY:-}" ]; then
          use_simple_im=0
          if [ -z "''${GTK_IM_MODULE:-}" ]; then
            use_simple_im=1
          fi
          if [ "''${GTK_IM_MODULE:-}" = "ibus" ]; then
            if ! command -v pgrep >/dev/null 2>&1 || ! pgrep -x ibus-daemon >/dev/null 2>&1; then
              use_simple_im=1
            fi
          fi
          case "''${GTK_IM_MODULE:-}" in
            fcitx|fcitx5)
              if ! command -v pgrep >/dev/null 2>&1 || { ! pgrep -x fcitx5 >/dev/null 2>&1 && ! pgrep -x fcitx >/dev/null 2>&1; }; then
                use_simple_im=1
              fi
              ;;
          esac
          if [ "$use_simple_im" -eq 1 ]; then
            export GTK_IM_MODULE="simple"
            unset QT_IM_MODULE XMODIFIERS
          fi
        elif [ -z "''${GTK_IM_MODULE:-}" ]; then
          # X11 fallback when no IM is configured.
          export GTK_IM_MODULE="simple"
        fi
        exec ${
          lib.optionalString (nixglIntel != null) "${nixglIntel}/bin/nixGLIntel "
        }${pkgs.ghostty}/bin/ghostty \
          --config-file="$CONF" \
          --gtk-single-instance=false \
          --class="com.yazelix.Yazelix" \
          --x11-instance-name="yazelix" \
          --title="Yazelix - Ghostty" "$@"
      ''
    else
      ''
        # macOS: Detect Homebrew-installed Ghostty
        MODE="''${YAZELIX_TERMINAL_CONFIG_MODE:-${terminalConfigMode}}"
        MODE="''${MODE:-auto}"
        USER_CONF="$HOME/.config/ghostty/config"
        YZ_CONF="$HOME/.local/share/yazelix/configs/terminal_emulators/ghostty/config"
        CONF="$YZ_CONF"
        if [ "$MODE" = "user" ] || [ "$MODE" = "auto" ]; then
          if [ -f "$USER_CONF" ]; then CONF="$USER_CONF"; fi
        fi

        # Try to find Ghostty binary (Homebrew installation)
        GHOSTTY_BIN=""
        if [ -x "/Applications/Ghostty.app/Contents/MacOS/ghostty" ]; then
          GHOSTTY_BIN="/Applications/Ghostty.app/Contents/MacOS/ghostty"
        elif [ -x "$HOME/Applications/Ghostty.app/Contents/MacOS/ghostty" ]; then
          GHOSTTY_BIN="$HOME/Applications/Ghostty.app/Contents/MacOS/ghostty"
        else
          echo "Error: Ghostty not found. Please install via Homebrew:"
          echo "  brew install --cask ghostty"
          exit 1
        fi

        exec "$GHOSTTY_BIN" \
          --config-file="$CONF" \
          --title="Yazelix - Ghostty" "$@"
      ''
  );

  kittyWrapper =
    if lib.elem "kitty" terminalList then
      pkgs.writeShellScriptBin "yazelix-kitty" ''
        MODE="''${YAZELIX_TERMINAL_CONFIG_MODE:-${terminalConfigMode}}"
        MODE="''${MODE:-auto}"
        USER_CONF="$HOME/.config/kitty/kitty.conf"
        YZ_CONF="$HOME/.local/share/yazelix/configs/terminal_emulators/kitty/kitty.conf"
        CONF="$YZ_CONF"
        if [ "$MODE" = "user" ] || [ "$MODE" = "auto" ]; then
          if [ -f "$USER_CONF" ]; then CONF="$USER_CONF"; fi
        fi
        exec ${
          lib.optionalString (isLinux && nixglIntel != null) "${nixglIntel}/bin/nixGLIntel "
        }${pkgs.kitty}/bin/kitty \
          --config="$CONF" \
          --class="com.yazelix.Yazelix" \
          --title="Yazelix - Kitty" "$@"
      ''
    else
      null;

  weztermWrapper =
    if lib.elem "wezterm" terminalList then
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
        exec ${
          lib.optionalString (isLinux && nixglIntel != null) "${nixglIntel}/bin/nixGLIntel "
        }${pkgs.wezterm}/bin/wezterm \
          --config-file="$CONF" \
          --config 'window_decorations="NONE"' \
          --config enable_tab_bar=false \
          start --class=com.yazelix.Yazelix "$@"
      ''
    else
      null;

  alacrittyWrapper =
    if lib.elem "alacritty" terminalList then
      pkgs.writeShellScriptBin "yazelix-alacritty" ''
        MODE="''${YAZELIX_TERMINAL_CONFIG_MODE:-${terminalConfigMode}}"
        MODE="''${MODE:-auto}"
        USER_CONF="$HOME/.config/alacritty/alacritty.toml"
        YZ_CONF="$HOME/.local/share/yazelix/configs/terminal_emulators/alacritty/alacritty.toml"
        CONF="$YZ_CONF"
        if [ "$MODE" = "user" ] || [ "$MODE" = "auto" ]; then
          if [ -f "$USER_CONF" ]; then CONF="$USER_CONF"; fi
        fi
        exec ${
          lib.optionalString (isLinux && nixglIntel != null) "${nixglIntel}/bin/nixGLIntel "
        }${pkgs.alacritty}/bin/alacritty \
          --config-file="$CONF" \
          --class="com.yazelix.Yazelix" \
          --title="Yazelix - Alacritty" "$@"
      ''
    else
      null;

  footWrapper =
    if isLinux && (lib.elem "foot" terminalList) then
      pkgs.writeShellScriptBin "yazelix-foot" ''
        MODE="''${YAZELIX_TERMINAL_CONFIG_MODE:-${terminalConfigMode}}"
        MODE="''${MODE:-auto}"
        USER_CONF="$HOME/.config/foot/foot.ini"
        YZ_CONF="$HOME/.local/share/yazelix/configs/terminal_emulators/foot/foot.ini"
        CONF="$YZ_CONF"
        if [ "$MODE" = "user" ] || [ "$MODE" = "auto" ]; then
          if [ -f "$USER_CONF" ]; then CONF="$USER_CONF"; fi
        fi
        exec ${
          lib.optionalString (nixglIntel != null) "${nixglIntel}/bin/nixGLIntel "
        }${pkgs.foot}/bin/foot \
          --config="$CONF" \
          --app-id="com.yazelix.Yazelix" "$@"
      ''
    else
      null;

  yazelixDesktopLauncher =
    if isLinux then
      pkgs.writeShellScriptBin "yazelix-desktop-launcher" ''
        exec "$HOME/.config/yazelix/shells/posix/desktop_launcher.sh"
      ''
    else
      null;

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
    else
      null;

  ghosttyDeps =
    if manageTerminals && (lib.elem "ghostty" terminalList) then
      filterNull (
        [ ghosttyWrapper ] # Wrapper available on both Linux and macOS
        ++ lib.optionals isLinux [ pkgs.ghostty ] # Package only on Linux
      )
    else
      [ ];
  kittyDeps =
    if manageTerminals && (lib.elem "kitty" terminalList) then
      filterNull [ kittyWrapper ]
      ++ [
        pkgs.kitty
        pkgs.nerd-fonts.fira-code
        pkgs.nerd-fonts.symbols-only
      ]
    else
      [ ];
  weztermDeps =
    if manageTerminals && (lib.elem "wezterm" terminalList) then
      filterNull [ weztermWrapper ] ++ [ pkgs.wezterm ]
    else
      [ ];
  alacrittyDeps =
    if manageTerminals && (lib.elem "alacritty" terminalList) then
      filterNull [ alacrittyWrapper ]
      ++ [
        pkgs.alacritty
        pkgs.nerd-fonts.fira-code
        pkgs.nerd-fonts.symbols-only
      ]
    else
      [ ];
  footDeps =
    if isLinux && manageTerminals && (lib.elem "foot" terminalList) then
      filterNull [ footWrapper ] ++ [ pkgs.foot ]
    else
      [ ];

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
      taplo # TOML toolkit for yazelix.toml configuration
    ]
    ++ lib.optionals isNamedNeovimEditor [ neovim ]
    ++ lib.optionals isLinux [ libnotify ]
    ++ filterNull [
      yazelixDesktopLauncher
      yazelixDesktopEntry
    ]
    ++ (if isLinux && nixglIntel != null then [ nixglIntel ] else [ ])
    ++ ghosttyDeps
    ++ kittyDeps
    ++ weztermDeps
    ++ alacrittyDeps
    ++ footDeps;

  extraShellDeps =
    (if includeFish then [ pkgs.fish ] else [ ]) ++ (if includeZsh then [ pkgs.zsh ] else [ ]);

  recommendedDeps = with pkgs; [
    lazygit
    carapace
  ];

  yaziExtensionsDeps = with pkgs; [
    p7zip
    jq
    fd
    ripgrep
    poppler
  ];

  yaziMediaDeps = with pkgs; [
    ffmpeg
    imagemagick
  ];

  justcodePkg =
    if llmAgentsPkgs ? code then
      pkgs.writeShellScriptBin "justcode" ''
        exec ${llmAgentsPkgs.code}/bin/code "$@"
      ''
    else
      null;

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

  rustToolchain =
    if fenixPkgs != null then
      fenixPkgs.combine [
        fenixPkgs.stable.cargo
        fenixPkgs.stable.rustc
        fenixPkgs.stable.rustfmt
        fenixPkgs.stable.clippy
      ]
    else
      null;

  truPkg = pkgs.rustPlatform.buildRustPackage rec {
    pname = "tru";
    version = "0.2.1";

    src = pkgs.fetchFromGitHub {
      owner = "Dicklesworthstone";
      repo = "toon_rust";
      rev = "v${version}";
      hash = "sha256-rvqCkf14zC1PldutoO/u2cdxZGi7VDrlWErILjmA3Jo=";
    };

    cargoHash = "sha256-kNgpOdkxCBjW8I2WcYIyFL0kd3e/Hb9cj51RghSwuFw=";

    postInstall = ''
      ln -s $out/bin/toon $out/bin/tru
    '';

    meta = with lib; {
      description = "TOON reference implementation in Rust (JSON <-> TOON)";
      homepage = "https://github.com/Dicklesworthstone/toon_rust";
      license = licenses.mit;
      mainProgram = "tru";
    };
  };

  resolvePkg =
    name:
    let
      canonicalName = name;
      # Check if this package should come from llm-agents
      isLlmAgentsPkg = builtins.elem canonicalName llmAgentsPackageNames;
      llmAgentsValue = if isLlmAgentsPkg then llmAgentsPkgs.${canonicalName} or null else null;
      # Fall back to nixpkgs (supports nested paths like "python3Packages.foo")
      path = lib.splitString "." canonicalName;
      nixpkgsValue = lib.attrByPath path null pkgs;
    in
    if builtins.elem name blockedPackageNames then
      throw "Package '${name}' is blocked in Yazelix. Remove it from packs/user_packages."
    else if name == "justcode" then
      if justcodePkg != null then
        justcodePkg
      else
        throw "Package 'justcode' requires llm-agents.nix package 'code', but it was not found"
    else if name == "rust_wasi_toolchain" then
      if rustWasiToolchain != null then
        rustWasiToolchain
      else
        throw "Package 'rust_wasi_toolchain' requires the fenix input, but it was not found in devenv.yaml"
    else if name == "rust_toolchain" then
      if rustToolchain != null then
        rustToolchain
      else
        throw "Package 'rust_toolchain' requires the fenix input, but it was not found in devenv.yaml"
    else if name == "tru" then
      truPkg
    else if llmAgentsValue != null then
      llmAgentsValue
    else if nixpkgsValue != null then
      nixpkgsValue
    else if isLlmAgentsPkg then
      throw "Package '${name}' resolves to '${canonicalName}', but it was not found in llm-agents.nix (is the input added to devenv.yaml?)"
    else
      throw "Package '${name}' not found in nixpkgs";

  packDeclarations =
    if builtins.isAttrs (userConfig.pack_declarations or { }) then
      userConfig.pack_declarations
    else
      throw "packs.declarations must be a table of pack definitions";

  packDefinitions = lib.mapAttrs (
    packName: pkgNames:
    if builtins.isList pkgNames then
      map resolvePkg pkgNames
    else
      throw "Pack '${packName}' must be a list of package names"
  ) packDeclarations;

  selectedPacks = userConfig.pack_names or [ ];
  packPackages = builtins.concatLists (
    map (
      packName:
      if builtins.hasAttr packName packDefinitions then
        packDefinitions.${packName}
      else
        throw "Unknown pack '${packName}'. Declare it under packs.declarations. Available packs: ${builtins.concatStringsSep ", " (builtins.attrNames packDefinitions)}"
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

in
{
  # Pull binaries from caches to speed up builds
  cachix.pull = [
    "numtide"        # llm-agents.nix AI tools
    "helix"          # Helix editor builds
    "nix-community"  # General community packages
  ];

  packages = allDeps;

  env = {
    YAZELIX_DIR = "$HOME/.config/yazelix";
    IN_YAZELIX_SHELL = "true";
    NIX_CONFIG = yazelixNixConfig;
    YAZELIX_DEBUG_MODE = boolToString debugMode;
    ZELLIJ_DEFAULT_LAYOUT = yazelixLayoutName;
    YAZELIX_ENABLE_SIDEBAR = boolToString enableSidebar;
    YAZI_CONFIG_HOME = "$HOME/.local/share/yazelix/configs/yazi";
    YAZELIX_HELIX_MODE = helixMode;
    YAZELIX_PREFERRED_TERMINAL = preferredTerminal;
    YAZELIX_TERMINAL_CONFIG_MODE = terminalConfigMode;
    YAZELIX_ASCII_ART_MODE = asciiArtMode;
    EDITOR = editorCommand;
  } // lib.optionalAttrs (helixRuntimePath != null) {
    HELIX_RUNTIME = helixRuntimePath;
  };

  enterShell = ''
    if [ -z "$HOME" ]; then
      export HOME="$(dirname "$(dirname "$DEVENV_ROOT")")"
    fi

    export YAZELIX_DIR="$HOME/.config/yazelix"
    export IN_YAZELIX_SHELL="true"
    export NIX_CONFIG='${yazelixNixConfig}'
    export YAZELIX_DEBUG_MODE="${boolToString debugMode}"
    export ZELLIJ_DEFAULT_LAYOUT="${yazelixLayoutName}"
    export YAZELIX_ENABLE_SIDEBAR="${boolToString enableSidebar}"
    export YAZI_CONFIG_HOME="$HOME/.local/share/yazelix/configs/yazi"
    export YAZELIX_HELIX_MODE="${helixMode}"
    export YAZELIX_PREFERRED_TERMINAL="${preferredTerminal}"
    export YAZELIX_TERMINAL_CONFIG_MODE="${terminalConfigMode}"
    export YAZELIX_ASCII_ART_MODE="${asciiArtMode}"
    export EDITOR="${editorCommand}"
    ${lib.optionalString (helixRuntimePath != null) ''
      export HELIX_RUNTIME="${helixRuntimePath}"
    ''}

    if [ "$YAZELIX_ENV_ONLY" != "true" ]; then
      echo "📝 Set EDITOR to: ${editorCommand}"
    fi

    # Environment setup now reads directly from yazelix.toml (single source of truth)
    if [ "$YAZELIX_SHELLHOOK_SKIP_WELCOME" = "true" ]; then
      nu "$YAZELIX_DIR/nushell/scripts/setup/environment.nu" --skip-welcome
      unset YAZELIX_SHELLHOOK_SKIP_WELCOME
    else
      nu "$YAZELIX_DIR/nushell/scripts/setup/environment.nu"
    fi

    # Save config hash after successful environment setup
    if command -v nu >/dev/null 2>&1; then
      nu -c 'use ~/.config/yazelix/nushell/scripts/utils/config_state.nu [compute_config_state mark_config_state_applied]; let state = compute_config_state; mark_config_state_applied $state' 2>/dev/null || true
    fi
  '';
}
