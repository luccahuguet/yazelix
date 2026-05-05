{
  config,
  lib,
  options,
  fenixPkgs ? null,
  nixgl ? null,
  pkgs,
  ...
}:

with lib;

let
  cfg = config.programs.yazelix;
  defaultRuntimeVariant = "ghostty";
  agentUsageProgramNames = [
    "tokenusage"
  ];
  agentUsagePackageMap = {
    tokenusage = import ../packaging/tokenusage.nix { inherit pkgs; };
  };
  selectedAgentUsagePackages =
    map (
      program:
      if builtins.hasAttr program agentUsagePackageMap then
        builtins.getAttr program agentUsagePackageMap
      else
        throw "programs.yazelix.agent_usage_programs contains an unsupported agent usage program"
    ) cfg.agent_usage_programs;
  yazelixPackage = import ../yazelix_package.nix {
    inherit pkgs fenixPkgs nixgl;
    runtimeVariant = cfg.runtime_variant;
    extraRuntimePackages = selectedAgentUsagePackages;
  };
  mainConfigContract = builtins.fromTOML (builtins.readFile ../config_metadata/main_config_contract.toml);
  mainContractFields = mainConfigContract.fields;
  defaultCursorConfig = builtins.fromTOML (builtins.readFile ../yazelix_cursors_default.toml);
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

  mainConfigValueForSettings =
    fieldPath:
    let
      field = getMainField fieldPath;
      value = configValueForField fieldPath;
    in
    if value == null then
      if attrOr field "home_manager_can_omit" false then
        null
      else if (attrOr field "parser_behavior" "") == "empty_string_to_null" then
        ""
      else
        throw "Null Home Manager value is not renderable for ${fieldPath}"
    else
      value;

  mainConfigSettingsFieldPaths =
    builtins.filter (fieldPath: mainConfigValueForSettings fieldPath != null) mainConfigFieldPaths;

  mainConfigSettingsValue =
    lib.foldl' (
      acc: fieldPath:
      lib.recursiveUpdate acc (
        lib.setAttrByPath (lib.splitString "." fieldPath) (mainConfigValueForSettings fieldPath)
      )
    ) { } mainConfigSettingsFieldPaths;

  settingsTopLevelOrder = [
    "core"
    "helix"
    "editor"
    "shell"
    "terminal"
    "zellij"
    "yazi"
  ];

  settingsJsonValue = mainConfigSettingsValue // { cursors = defaultCursorConfig; };

  settingsOrderedNames =
    (builtins.filter (name: builtins.hasAttr name settingsJsonValue) settingsTopLevelOrder)
    ++ (builtins.filter (
      name: name != "cursors" && !(builtins.elem name settingsTopLevelOrder)
    ) (builtins.attrNames settingsJsonValue))
    ++ (lib.optional (builtins.hasAttr "cursors" settingsJsonValue) "cursors");

  renderSettingsJsonEntry =
    name:
    "  ${builtins.toJSON name}: ${builtins.toJSON (builtins.getAttr name settingsJsonValue)}";

  settingsJsonc =
    ''
      // Generated by the Yazelix Home Manager module.
      // Edit your Home Manager configuration instead of this file.
    ''
    + "{\n"
    + concatStringsSep ",\n" (map renderSettingsJsonEntry settingsOrderedNames)
    + "\n}\n";

in
{
  _file = "yazelix/home_manager/module.nix";

  options.programs.yazelix = {
    enable = mkEnableOption "Yazelix terminal environment";

    manage_config = mkOption {
      type = types.bool;
      default = false;
      description = ''
        Whether Home Manager generates ~/.config/yazelix/settings.jsonc.

        The default keeps Home Manager responsible for the Yazelix
        package/runtime/desktop integration while leaving settings.jsonc as a
        normal mutable user file managed through `yzx edit` or your editor.

        Set this to true only when you want Home Manager to generate and own
        settings.jsonc declaratively from programs.yazelix options.
      '';
    };

    runtime_variant = mkOption {
      type = types.enum [ "ghostty" "wezterm" ];
      default = defaultRuntimeVariant;
      description = ''
        Packaged terminal runtime variant.

        - "ghostty": default packaged runtime with Yazelix cursor trails and Ghostty config effects
        - "wezterm": explicit compatibility runtime, especially for users who prefer WezTerm image-preview behavior
      '';
    };

    agent_usage_programs = mkOption {
      type = types.listOf (types.enum agentUsageProgramNames);
      default = [ ];
      description = ''
        Opt-in usage binaries to include in the Yazelix runtime.

        These support zellij.widget_tray usage entries:
        - "tokenusage": claude_usage, codex_usage

        codex_usage is a combined 5h/week token and quota widget.
        claude_usage is a combined 5h/week token and quota widget.
        opencode_go_usage reads OpenCode's local SQLite database directly and does
        not require an extra usage binary. Configure its rendered windows with
        zellij_opencode_go_usage_periods.
      '';
    };

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

        Ghostty cursor presets and cursor effects live in ~/.config/yazelix/settings.jsonc under cursors
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
      description = ''
        Legacy compatibility toggle for older configs.

        Prefer hide_sidebar_on_file_open for the file-open workflow that hides
        the managed sidebar after opening a file.
      '';
    };

    hide_sidebar_on_file_open = mkMainContractOption "editor.hide_sidebar_on_file_open" {
      description = ''
        Whether Yazelix should hide the managed sidebar after opening a file from
        the Yazi file-tree sidebar.
      '';
    };

    sidebar_width_percent = mkMainContractOption "editor.sidebar_width_percent" {
      description = "Width of the open sidebar as a percentage of the tab; the default sidebar is a Yazi file tree.";
    };

    sidebar_command = mkMainContractOption "editor.sidebar_command" {
      description = "Terminal command used for the managed sidebar pane. Defaults to Nu running the Yazelix Yazi file-tree adapter.";
    };

    sidebar_args = mkMainContractOption "editor.sidebar_args" {
      description = ''
        Arguments passed to the managed sidebar command.

        The default launches Yazelix's managed Yazi file-tree adapter with the default Nu command.
        When sidebar_command is changed and sidebar_args remains at that default adapter
        path, Yazelix renders the custom sidebar command with no inherited Yazi argument.
      '';
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
      description = "Zjstatus widget tray order (editor/shell/term/workspace/cursor/usage/cpu/ram); dynamic entries read from a window-local cache";
    };

    zellij_codex_usage_display = mkMainContractOption "zellij.codex_usage_display" {
      description = "Codex usage widget display mode: token, quota, or both";
    };

    zellij_claude_usage_display = mkMainContractOption "zellij.claude_usage_display" {
      description = "Claude usage widget display mode: token, quota, or both";
    };

    zellij_opencode_go_usage_display = mkMainContractOption "zellij.opencode_go_usage_display" {
      description = "OpenCode Go usage widget display mode: token, quota, or both";
    };

    zellij_opencode_go_usage_periods = mkMainContractOption "zellij.opencode_go_usage_periods" {
      description = "Periods shown by the opencode_go_usage widget: 5h, week, month";
    };

    zellij_claude_usage_periods = mkMainContractOption "zellij.claude_usage_periods" {
      description = "Periods shown by the claude_usage widget: 5h, week";
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

    screen_saver_enabled = mkMainContractOption "zellij.screen_saver_enabled" {
      description = "Enable the opt-in idle `yzx screen` pane-orchestrator screen saver.";
    };

    screen_saver_idle_seconds = mkMainContractOption "zellij.screen_saver_idle_seconds" {
      description = "Seconds of Zellij input inactivity before the screen saver opens.";
    };

    screen_saver_style = mkMainContractOption "zellij.screen_saver_style" {
      description = "Animated `yzx screen` style to run when the idle screen saver opens.";
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
        - "boids": alias for "boids_predator"
        - "boids_predator": show boids with predator/prey motion
        - "boids_schools": show species-separated boids schools
        - "mandelbrot": show the Seahorse/Misiurewicz Mandelbrot zoom
        - "game_of_life_gliders": show the glider-swarm Game of Life style
        - "game_of_life_oscillators": show the oscillator-garden Game of Life style
        - "game_of_life_bloom": show the bloom-field Game of Life style
        - "random": choose evenly across Game of Life, boids, and Mandelbrot families (never "static" or "logo")
      '';
    };

    welcome_duration_seconds = mkMainContractOption "core.welcome_duration_seconds" {
      description = ''
        Welcome animation duration in seconds for animated styles.
        The logo style keeps its fixed timing and ignores this value.
        Default: 2.0.
        Valid range: 0.2 to 8.0.
      '';
    };

    game_of_life_cell_style = mkMainContractOption "core.game_of_life_cell_style" {
      description = ''
        Game of Life cell rendering style.
        - "full_block": solid cells matching the old Nushell renderer
        - "dotted": braille scale-4 texture with the same footprint
      '';
    };

    show_macchina_on_welcome = mkMainContractOption "core.show_macchina_on_welcome" {
      description = "Show macchina system info on welcome screen";
    };

    zellij_default_mode = mkMainContractOption "zellij.default_mode" {
      description = ''
        Startup mode for new Zellij sessions.
        - "normal": Yazelix default, starts unlocked
        - "locked": start in Zellij locked mode for compatibility with other TUIs
      '';
    };

  };

  config = mkIf cfg.enable (mkMerge [
    {
      # Expose the packaged Yazelix runtime through the Home Manager profile.
      home.packages = [ yazelixPackage ];

      # Desktop icon integration.
      xdg.dataFile."icons/hicolor/48x48/apps/yazelix.png".source =
        ../assets/icons/48x48/yazelix.png;
      xdg.dataFile."icons/hicolor/64x64/apps/yazelix.png".source =
        ../assets/icons/64x64/yazelix.png;
      xdg.dataFile."icons/hicolor/128x128/apps/yazelix.png".source =
        ../assets/icons/128x128/yazelix.png;
      xdg.dataFile."icons/hicolor/256x256/apps/yazelix.png".source =
        ../assets/icons/256x256/yazelix.png;

      home.activation.yazelixGeneratedRuntimeConfigs = lib.hm.dag.entryAfter [ "linkGeneration" ] ''
        export PATH="${runtimeConfigGenerationPath}:$PATH"
        export YAZELIX_RUNTIME_DIR="${yazelixPackage}"
        export YAZELIX_CONFIG_DIR="${managedConfigRoot}"
        export YAZELIX_STATE_DIR="${stateRoot}"
        export YAZELIX_LOGS_DIR="${logsPath}"

        $DRY_RUN_CMD ${runtimeYzxCore} runtime-materialization.repair --from-env --force --summary
      '';
    }
    (lib.optionalAttrs (lib.hasAttrByPath [ "xdg" "desktopEntries" ] options) {
      # Linux desktop entry for application launchers.
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
    })
    (mkIf cfg.manage_config {
      # Generate settings.jsonc configuration file
      xdg.configFile."yazelix/settings.jsonc" = {
        text = settingsJsonc;
      };
    })
  ]);
}
