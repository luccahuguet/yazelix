{
  config,
  lib,
  fenixPkgs ? null,
  nixgl ? null,
  pkgs,
  ...
}:

with lib;

let
  cfg = config.programs.yazelix;
  yazelixPackage = import ../yazelix_package.nix { inherit pkgs fenixPkgs nixgl; };
  mainConfigContract = builtins.fromTOML (builtins.readFile ../config_metadata/main_config_contract.toml);
  mainContractFields = mainConfigContract.fields;
  mainConfigSectionOrder = [
    "core"
    "helix"
    "editor"
    "shell"
    "terminal"
    "zellij"
    "yazi"
  ];
  runtimeYzxCore = "${yazelixPackage}/libexec/yzx_core";
  stateRoot = "${config.xdg.dataHome}/yazelix";
  logsPath = "${stateRoot}/logs";
  managedConfigRoot = "${config.xdg.configHome}/yazelix";
  runtimeConfigGenerationPath = lib.makeBinPath [
    pkgs.coreutils
    pkgs.zellij
  ];

  boolToToml = value: if value then "true" else "false";

  escapeString =
    value:
    let
      safe = lib.replaceStrings [ "\"" "\\" ] [ "\\\"" "\\\\" ] value;
    in
    "\"${safe}\"";

  listToToml =
    values:
    if values == [ ] then "[]" else "[ " + (concatStringsSep ", " (map escapeString values)) + " ]";

  attrOr =
    attrs: name: fallback:
    if builtins.hasAttr name attrs then builtins.getAttr name attrs else fallback;

  getMainField = fieldPath: builtins.getAttr fieldPath mainContractFields;

  mainFieldAllowsNull =
    field:
    (attrOr field "nullable" false)
    || (attrOr field "home_manager_default_is_null" false)
    || (attrOr field "home_manager_can_omit" false);

  mainFieldDefault =
    field:
    if attrOr field "home_manager_default_is_null" false then null else field.default;

  mainFieldType =
    field:
    let
      validation = attrOr field "validation" "";
      baseType =
        if validation == "enum" then
          types.enum field.allowed_values
        else if validation == "enum_string_list" then
          types.listOf (types.enum field.allowed_values)
        else if validation == "int_range" then
          types.ints.between field.min field.max
        else if validation == "float_range" then
          types.addCheck (types.either types.int types.float) (
            value: value >= field.min && value <= field.max
          )
        else if field.kind == "bool" then
          types.bool
        else if field.kind == "string" then
          types.str
        else if field.kind == "string_list" then
          types.listOf types.str
        else if field.kind == "int" then
          types.int
        else if field.kind == "float" then
          types.either types.int types.float
        else
          throw "Unsupported main config contract kind for Home Manager: ${field.kind}";
    in
    if mainFieldAllowsNull field then types.nullOr baseType else baseType;

  mkMainContractOption =
    fieldPath: extra:
    let
      field = getMainField fieldPath;
    in
    mkOption (
      {
        type = mainFieldType field;
        default = mainFieldDefault field;
      }
      // extra
    );

  mainConfigFieldPaths = lib.sort builtins.lessThan (builtins.attrNames mainContractFields);

  fieldSection = fieldPath: builtins.head (lib.splitString "." fieldPath);

  fieldTomlKey = fieldPath: builtins.elemAt (lib.splitString "." fieldPath) 1;

  mainFieldsForSection =
    section:
    builtins.filter (fieldPath: fieldSection fieldPath == section) mainConfigFieldPaths;

  configValueForField =
    fieldPath:
    let
      field = getMainField fieldPath;
    in
    builtins.getAttr field.home_manager_option cfg;

  renderTomlValue =
    value:
    if builtins.isBool value then
      boolToToml value
    else if builtins.isInt value || builtins.isFloat value then
      toString value
    else if builtins.isList value then
      listToToml value
    else
      escapeString value;

  renderMainConfigField =
    fieldPath:
    let
      field = getMainField fieldPath;
      value = configValueForField fieldPath;
      tomlKey = fieldTomlKey fieldPath;
    in
    if value == null then
      if attrOr field "home_manager_can_omit" false then
        null
      else if (attrOr field "parser_behavior" "") == "empty_string_to_null" then
        "${tomlKey} = ${escapeString ""}"
      else
        throw "Null Home Manager value is not renderable for ${fieldPath}"
    else
      "${tomlKey} = ${renderTomlValue value}";

  renderMainConfigSection =
    section:
    let
      lines = lib.filter (line: line != null) (map renderMainConfigField (mainFieldsForSection section));
    in
    if lines == [ ] then [ ] else [ "" "[${section}]" ] ++ lines;

in
{
  _file = "yazelix/home_manager/module.nix";

  options.programs.yazelix = {
    enable = mkEnableOption "Yazelix terminal environment";

    # Configuration options (mirrors yazelix_default.toml structure)
    default_shell = mkMainContractOption "shell.default_shell" {
      description = "Default shell for Zellij sessions";
    };

    terminals = mkMainContractOption "terminal.terminals" {
      description = "Ordered terminal emulator list (first is primary, rest are fallbacks)";
    };

    terminal_config_mode = mkMainContractOption "terminal.config_mode" {
      description = ''
        How Yazelix selects terminal configs:
        - "yazelix": use Yazelix-managed configs in ~/.local/share/yazelix (default)
        - "user": load the terminal's native user config path and fail if it does not exist
      '';
    };

    ghostty_trail_color = mkMainContractOption "terminal.ghostty_trail_color" {
      description = ''
        Ghostty cursor color palette and Kitty cursor-trail fallback preset.
        Disable the palette and fallback trail: "none"
        Supported by Ghostty: "none", "blaze", "snow", "cosmic", "ocean", "forest", "sunset", "neon", "party", "eclipse", "dusk", "orchid", "reef", "inferno", "random"
        Supported by Ghostty and Kitty: "snow"
        "random" rerolls a Ghostty color palette for each Yazelix Ghostty window (excluding "party")
      '';
    };

    ghostty_trail_effect = mkMainContractOption "terminal.ghostty_trail_effect" {
      description = ''
        Ghostty trail effect for cursor movement.
        Set to null to disable extra tail effects.
        Valid values: "tail", "warp", "sweep", "random"
      '';
    };

    ghostty_mode_effect = mkMainContractOption "terminal.ghostty_mode_effect" {
      description = ''
        Ghostty mode-change effect, triggered when the editor changes cursor mode
        such as Neovim switching between normal and insert.
        Set to null to disable mode-change effects.
        Valid values: "ripple", "sonic_boom", "rectangle_boom", "ripple_rectangle", "random"
      '';
    };

    ghostty_trail_glow = mkMainContractOption "terminal.ghostty_trail_glow" {
      description = ''
        Glow level around Ghostty cursor trails and related cursor effects.

        - "none": keep the cursor/trail color effect but remove the extra spatial glow
        - "low": a tighter, subtler aura
        - "medium": the current Yazelix look (default)
        - "high": a larger, brighter aura
      '';
    };

    transparency = mkMainContractOption "terminal.transparency" {
      description = ''
        Terminal transparency level for all terminals.

        - "none": No transparency (opacity = 1.0)
        - "very_low": Minimal transparency (opacity = 0.95)
        - "low": Light transparency (opacity = 0.90)
        - "medium": Medium transparency (opacity = 0.85)
        - "high": High transparency (opacity = 0.80)
        - "very_high": Very high transparency (opacity = 0.70)
        - "super_high": Maximum transparency (opacity = 0.60)
      '';
    };

    # Editor configuration
    editor_command = mkMainContractOption "editor.command" {
      description = ''
        Editor command - yazelix will always set this as EDITOR.

        - null (default): Use yazelix's Nix-provided Helix - full integration
        - "nvim": Use Neovim - first-class support with full integration
        - "hx": Use system Helix from PATH (set helix_runtime_path only when your runtime lives outside Helix's normal discovery paths)
        - Other editors: "vim", "nano", "emacs", etc. (basic integration only)
      '';
    };

    helix_runtime_path = mkMainContractOption "helix.runtime_path" {
      description = ''
        Custom Helix runtime path - only set this if editor_command points to a custom Helix build.

        IMPORTANT: The runtime MUST match your Helix binary version to avoid startup errors.
        Example: "/home/user/helix/runtime" for a custom Helix build in ~/helix
      '';
    };

    enable_sidebar = mkMainContractOption "editor.enable_sidebar" {
      description = "Enable or disable the Yazi sidebar";
    };

    sidebar_width_percent = mkMainContractOption "editor.sidebar_width_percent" {
      description = "Width of the open Yazi sidebar as a percentage of the tab.";
    };

    disable_zellij_tips = mkMainContractOption "zellij.disable_tips" {
      description = "Disable Zellij tips popup on startup for cleaner launches";
    };

    zellij_pane_frames = mkMainContractOption "zellij.pane_frames" {
      description = "Show Zellij pane frames";
    };

    zellij_rounded_corners = mkMainContractOption "zellij.rounded_corners" {
      description = "Enable rounded corners for Zellij pane frames";
    };

    support_kitty_keyboard_protocol = mkMainContractOption "zellij.support_kitty_keyboard_protocol" {
      description = "Enable Kitty keyboard protocol in Zellij (disable if dead keys stop working)";
    };

    zellij_theme = mkMainContractOption "zellij.theme" {
      description = ''
        Zellij color theme (37 built-in themes available).

        Dark themes: ansi, ao, atelier-sulphurpool, ayu_mirage, ayu_dark, catppuccin-frappe,
        catppuccin-macchiato, cyber-noir, blade-runner, retro-wave, dracula, everforest-dark,
        gruvbox-dark, iceberg-dark, kanagawa, lucario, menace, molokai-dark, night-owl, nightfox,
        nord, one-half-dark, onedark, solarized-dark, tokyo-night-dark, tokyo-night-storm,
        tokyo-night, vesper

        Light themes: ayu_light, catppuccin-latte, everforest-light, gruvbox-light,
        iceberg-light, dayfox, pencil-light, solarized-light, tokyo-night-light
      '';
    };

    zellij_widget_tray = mkMainContractOption "zellij.widget_tray" {
      description = "Zjstatus widget tray order (editor/shell/term/cpu/ram)";
    };

    zellij_custom_text = mkMainContractOption "zellij.custom_text" {
      description = "Optional short zjstatus badge shown before YAZELIX. Trimmed and capped at 8 characters.";
    };

    popup_program = mkMainContractOption "zellij.popup_program" {
      description = ''
        Default transient popup command for `yzx popup` and the default popup keybinding.
        Use an argv-style list, eg. [ "lazygit" ], [ "editor" ] to reuse `editor.command`,
        or [ "codex" ].
      '';
    };

    popup_width_percent = mkMainContractOption "zellij.popup_width_percent" {
      description = "Width of the managed popup as a percentage of the current tab.";
    };

    popup_height_percent = mkMainContractOption "zellij.popup_height_percent" {
      description = "Height of the managed popup as a percentage of the current tab.";
    };

    yazi_plugins = mkMainContractOption "yazi.plugins" {
      description = "Yazi plugins to load (core plugins auto_layout and sidebar_status are always loaded)";
    };

    yazi_command = mkMainContractOption "yazi.command" {
      description = "Custom Yazi binary for Yazelix-managed Yazi launches. Null uses `yazi` from PATH.";
    };

    yazi_ya_command = mkMainContractOption "yazi.ya_command" {
      description = "Custom `ya` CLI for Yazelix-managed reveal and sidebar-sync actions. Null uses `ya` from PATH.";
    };

    yazi_theme = mkMainContractOption "yazi.theme" {
      description = ''
        Yazi color theme (flavor). 25 built-in flavors available (19 dark + 5 light + default).
        Use "default" to keep Yazi's upstream built-in theme.
        Use "random-dark" or "random-light" to pick a different theme on each yazelix restart.
        Browse flavors: https://github.com/yazi-rs/flavors
      '';
    };

    yazi_sort_by = mkMainContractOption "yazi.sort_by" {
      description = "Default file sorting method";
    };

    debug_mode = mkMainContractOption "core.debug_mode" {
      description = "Enable verbose debug logging";
    };

    skip_welcome_screen = mkMainContractOption "core.skip_welcome_screen" {
      description = "Skip the welcome screen on startup";
    };

    welcome_style = mkMainContractOption "core.welcome_style" {
      description = ''
        Welcome screen style.
        - "static": show the resting Yazelix logo frame only
        - "logo": show the branded animated logo reveal
        - "boids": show the animated flocking style
        - "game_of_life_gliders": show the glider-swarm Game of Life style
        - "game_of_life_oscillators": show the oscillator-garden Game of Life style
        - "game_of_life_bloom": show the bloom-field Game of Life style
        - "random": choose one Game of Life variant at random (never "static")
      '';
    };

    welcome_duration_seconds = mkMainContractOption "core.welcome_duration_seconds" {
      description = ''
        Welcome animation duration in seconds for animated styles.
        The logo style keeps its fixed timing and ignores this value.
        Default: 1.0.
        Valid range: 0.2 to 8.0.
      '';
    };

    show_macchina_on_welcome = mkMainContractOption "core.show_macchina_on_welcome" {
      description = "Show macchina system info on welcome screen";
    };

    persistent_sessions = mkMainContractOption "zellij.persistent_sessions" {
      description = "Enable persistent Zellij sessions";
    };

    session_name = mkMainContractOption "zellij.session_name" {
      description = "Session name for persistent sessions";
    };

    zellij_default_mode = mkMainContractOption "zellij.default_mode" {
      description = ''
        Startup mode for new Zellij sessions.
        - "normal": Yazelix default, starts unlocked
        - "locked": start in Zellij locked mode for compatibility with other TUIs
      '';
    };

  };

  config = mkIf cfg.enable {
    # Expose the packaged Yazelix runtime through the Home Manager profile.
    home.packages = [ yazelixPackage ];

    # Desktop icon integration.
    xdg.dataFile."icons/hicolor/48x48/apps/yazelix.png".source = ../assets/icons/48x48/yazelix.png;
    xdg.dataFile."icons/hicolor/64x64/apps/yazelix.png".source = ../assets/icons/64x64/yazelix.png;
    xdg.dataFile."icons/hicolor/128x128/apps/yazelix.png".source =
      ../assets/icons/128x128/yazelix.png;
    xdg.dataFile."icons/hicolor/256x256/apps/yazelix.png".source =
      ../assets/icons/256x256/yazelix.png;

    # Desktop entry for application launcher
    xdg.desktopEntries.yazelix = {
      name = "Yazelix";
      comment = "Yazi + Zellij + Helix integrated terminal environment";
      exec = "${config.home.profileDirectory}/bin/yzx desktop launch";
      icon = "yazelix";
      categories = [ "Development" ];
      type = "Application";
      terminal = true;
      settings = {
        StartupWMClass = "com.yazelix.Yazelix";
      };
    };

    home.activation.yazelixGeneratedRuntimeConfigs = lib.hm.dag.entryAfter [ "linkGeneration" ] ''
      export PATH="${runtimeConfigGenerationPath}:$PATH"
      export YAZELIX_RUNTIME_DIR="${yazelixPackage}"
      export YAZELIX_CONFIG_DIR="${managedConfigRoot}"
      export YAZELIX_STATE_DIR="${stateRoot}"
      export YAZELIX_LOGS_DIR="${logsPath}"

      $DRY_RUN_CMD ${runtimeYzxCore} runtime-materialization.repair --from-env --force --summary
    '';

    # Generate yazelix.toml configuration file
    xdg.configFile."yazelix/user_configs/yazelix.toml" = {
      text =
        lib.concatStringsSep "\n" (
          [
            "# Generated by the Yazelix Home Manager module."
            "# Edit your Home Manager configuration instead of this file."
          ]
          ++ lib.concatLists (map renderMainConfigSection mainConfigSectionOrder)
        )
        + "\n";
    };
  };
}
