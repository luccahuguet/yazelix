{
  config,
  lib,
  options,
  fenixPkgs ? null,
  mkYazelixPackage ? null,
  nixgl ? null,
  pkgs,
  yazelixTerminalPackage ? null,
  ...
}:

with lib;

let
  cfg = config.programs.yazelix;
  defaultRuntimeVariant = "ghostty";
  runtimeToolSourceModes = [
    "bundled"
    "host"
    "off"
  ];
  runtimeVariants = [
    "ghostty"
    "kitty"
    "yzxterm"
    "wezterm"
  ] ++ lib.optional pkgs.stdenv.hostPlatform.isLinux "ratty";
  yzxtermProfiles = [
    "full"
    "baseline"
    "shaders"
  ];
  terminalPackageFor =
    runtimeVariant:
    if runtimeVariant == "ghostty" then
      if pkgs.stdenv.hostPlatform.isDarwin then pkgs."ghostty-bin" else pkgs.ghostty
    else if runtimeVariant == "kitty" then
      pkgs.kitty
    else if runtimeVariant == "wezterm" then
      pkgs.wezterm
    else if runtimeVariant == "ratty" then
      if pkgs.stdenv.hostPlatform.isLinux then
        pkgs.ratty
      else
        throw "programs.yazelix.extra_terminal_variants ratty is only supported on Linux"
    else if runtimeVariant == "yzxterm" then
      if yazelixTerminalPackage != null then
        yazelixTerminalPackage
      else
        throw "programs.yazelix.extra_terminal_variants yzxterm requires the yazelix-terminal child package"
    else
      throw "Unsupported Yazelix terminal variant: ${runtimeVariant}";
  runtimeDefaultTerminals =
    runtimeVariant:
    extraTerminalVariants:
    lib.unique (
      [ runtimeVariant ]
      ++ extraTerminalVariants
      ++ (
        if runtimeVariant == "wezterm" then
          [
            "ghostty"
          ]
        else if runtimeVariant == "kitty" then
          [
            "ghostty"
            "yzxterm"
            "wezterm"
          ]
        else if runtimeVariant == "ratty" then
          [
            "ghostty"
            "wezterm"
          ]
        else if runtimeVariant == "yzxterm" then
          [
            "ghostty"
            "wezterm"
          ]
        else
          [
            "wezterm"
          ]
      )
    );
  extraTerminalVariantPackages =
    let
      variants = lib.filter (variant: variant != cfg.runtime_variant) cfg.extra_terminal_variants;
    in
    lib.unique (map terminalPackageFor variants);
  componentEnabled = name: cfg.components.${name} or true;
  runtimeToolSource = name: cfg.runtime_tool_sources.${name} or "bundled";
  yzxtermProfileEnv =
    lib.optionalString (cfg.yzxterm_profile != "full")
      "YAZELIX_TERMINAL_PROFILE=${cfg.yzxterm_profile}";
  yzxtermProfileExport =
    lib.optionalString (cfg.yzxterm_profile != "full")
      "export ${yzxtermProfileEnv}";
  yzxtermDesktopExec =
    "${lib.optionalString (cfg.yzxterm_profile != "full") "env ${yzxtermProfileEnv} "}${config.home.profileDirectory}/bin/yzx desktop launch";
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
  packageBuilderArgs = {
    inherit pkgs;
    runtimeVariant = cfg.runtime_variant;
    runtimeToolSources = cfg.runtime_tool_sources;
    components = cfg.components;
    extraRuntimePackages = selectedAgentUsagePackages;
  };
  yazelixPackage =
    if cfg.package != null then
      cfg.package
    else if mkYazelixPackage != null then
      mkYazelixPackage packageBuilderArgs
    else
      import ../yazelix_package.nix (
        packageBuilderArgs
        // {
          inherit fenixPkgs nixgl;
        }
      );
  mainConfigContract = builtins.fromTOML (builtins.readFile ../config_metadata/main_config_contract.toml);
  mainContractFields = mainConfigContract.fields;
  defaultCursorConfig = builtins.fromTOML (builtins.readFile ../yazelix_ghostty_cursors_default.toml);
  mainConfigSectionOrder = [
    "core"
    "helix"
    "editor"
    "workspace"
    "shell"
    "terminal"
    "zellij"
    "yazi"
  ];
  runtimeYzxCore = "${yazelixPackage}/libexec/yzx_core";
  runtimeYzxControl = "${yazelixPackage}/libexec/yzx_control";
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
    if values == [ ] then "[]" else "[ " + (concatStringsSep ", " (map renderTomlValue values)) + " ]";

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
        else if field.kind == "string_list_map" then
          types.attrsOf (types.listOf types.str)
        else if field.kind == "int" then
          types.int
        else if field.kind == "float" then
          types.either types.int types.float
        else if field.kind == "helix_steel_plugins" then
          types.submodule {
            options = {
              enabled = mkOption {
                type = types.listOf types.str;
                default = [
                  "recentf"
                  "splash"
                  "spacemacs_theme"
                ];
                description = "Bundled Helix Steel plugin ids to load from the Yazelix plugin repository";
              };
              extra = mkOption {
                type = types.listOf (types.submodule {
                  options = {
                    id = mkOption {
                      type = types.str;
                      description = "Stable Yazelix Helix Steel plugin id";
                    };
                    source = mkOption {
                      type = types.str;
                      description = "Plugin source path below ~/.config/yazelix/helix/steel_plugins";
                    };
                    support_files = mkOption {
                      type = types.listOf types.str;
                      default = [ ];
                      description = "Additional Steel source files required by this plugin";
                    };
                    public_commands = mkOption {
                      type = types.listOf types.str;
                      default = [ ];
                      description = "Commands exposed through Helix command completion";
                    };
                    internal_commands = mkOption {
                      type = types.listOf types.str;
                      default = [ ];
                      description = "Commands imported for plugin use but kept out of completion";
                    };
                    startup_commands = mkOption {
                      type = types.listOf types.str;
                      default = [ ];
                      description = "Declared commands to run when the generated Steel module loads";
                    };
                    startup_condition = mkOption {
                      type = types.nullOr (types.enum [ "show_splash" ]);
                      default = null;
                      description = "Optional Yazelix condition required before startup_commands run";
                    };
                    command_descriptions = mkOption {
                      type = types.attrsOf types.str;
                      default = { };
                      description = "Descriptions for public and internal commands";
                    };
                  };
                });
                default = [ ];
                description = "User-owned Helix Steel plugin manifests";
              };
            };
          }
        else if field.kind == "helix_external" then
          types.submodule {
            options = {
              binary = mkOption {
                type = types.str;
                description = "Custom Helix binary path";
              };
              runtime_path = mkOption {
                type = types.str;
                description = "Runtime path matching the custom Helix binary";
              };
            };
          }
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
    else if builtins.isAttrs value then
      let
        nonNullNames =
          builtins.filter (name: builtins.getAttr name value != null) (builtins.attrNames value);
      in
      "{ "
      + concatStringsSep ", " (
        map (name: "${name} = ${renderTomlValue (builtins.getAttr name value)}") nonNullNames
      )
      + " }"
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
      else if field.kind == "helix_external" then
        null
      else if (attrOr field "parser_behavior" "") == "empty_string_to_null" then
        ""
      else
        throw "Null Home Manager value is not renderable for ${fieldPath}"
    else
      value;

  mainConfigSettingsFieldIncluded =
    fieldPath:
    let
      field = getMainField fieldPath;
      value = mainConfigValueForSettings fieldPath;
    in
    value != null || field.kind == "helix_external";

  mainConfigSettingsFieldPaths =
    builtins.filter mainConfigSettingsFieldIncluded mainConfigFieldPaths;

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
    "workspace"
    "shell"
    "terminal"
    "zellij"
    "yazi"
  ];

  settingsJsonValue = mainConfigSettingsValue;

  settingsOrderedNames =
    (builtins.filter (name: builtins.hasAttr name settingsJsonValue) settingsTopLevelOrder)
    ++ (builtins.filter (
      name: !(builtins.elem name settingsTopLevelOrder)
    ) (builtins.attrNames settingsJsonValue));

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

  cursorSettingsJsonc =
    ''
      // Generated by the Yazelix Home Manager module.
      // Edit your Home Manager configuration instead of this file.
    ''
    + builtins.toJSON defaultCursorConfig
    + "\n";

in
{
  _file = "yazelix/home_manager/module.nix";

  options.programs.yazelix = {
    enable = mkEnableOption "Yazelix terminal environment";

    package = mkOption {
      type = types.nullOr types.package;
      default = null;
      description = ''
        Yazelix package to expose through the Home Manager profile.

        The default builds Yazelix from this module's runtime options. Set this
        only when selecting a specific prebuilt package output instead.
      '';
    };

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

    manage_cursor_config = mkOption {
      type = types.bool;
      default = false;
      description = ''
        Whether Home Manager generates ~/.config/yazelix_ghostty_cursors/settings.jsonc.

        Cursor settings are independent from the main Yazelix settings file so
        the standalone yzc command and full Yazelix can share one cursor source.
        Set this to true only when you want Home Manager to own the cursor
        registry declaratively.
      '';
    };

    runtime_variant = mkOption {
      type = types.enum runtimeVariants;
      default = defaultRuntimeVariant;
      description = ''
        Packaged terminal runtime variant.

        - "ghostty": default packaged runtime with Yazelix cursor trails, Ghostty config effects, and Yazi image previews through Zellij
        - "kitty": packaged Kitty runtime with generated Kitty config and the Yazelix Zellij/Yazi Kitty graphics bridge
        - "wezterm": explicit alternate packaged runtime
        - "yzxterm": experimental Yazelix-owned Rio fork with Rio trail cursor defaults and opt-in shader support
        - "ratty": experimental Linux packaged runtime with Ratty and the Yazelix Zellij/Yazi Kitty graphics bridge
      '';
    };

    yzxterm_profile = mkOption {
      type = types.enum yzxtermProfiles;
      default = "full";
      description = ''
        Yazelix Terminal profile used by generated runtime configs and the
        Linux desktop entry.

        - "full": Rio trail cursor defaults without custom shaders
        - "baseline": no cursor effects
        - "shaders": Rio trail cursor plus generated Yazelix cursor shaders
      '';
    };

    extra_terminal_variants = mkOption {
      type = types.listOf (types.enum runtimeVariants);
      default = [ ];
      example = [
        "ghostty"
      ];
      description = ''
        Additional bundled terminal emulator packages to install beside the primary runtime variant.

        This is for users who want, for example, `runtime_variant = "yzxterm"` as the primary
        Yazelix runtime while also keeping Ghostty available to `yzx launch --terminal ghostty`.

        These packages install only the terminal emulator commands into the Home Manager profile.
        They do not install additional Yazelix `yzx` wrappers, so they avoid profile collisions
        with the selected primary runtime package.
      '';
    };

    runtime_tool_sources = mkOption {
      type = types.attrsOf (types.enum runtimeToolSourceModes);
      default = { };
      description = ''
        Per-tool runtime source modes. Omitted tools default to "bundled",
        except mise and tombi, which default to "host".

        Supported values:
        - "bundled": include the Yazelix-packaged tool and export its commands
        - "host": omit the package/export and rely on the inherited host PATH
        - "off": omit the package/export when the tool explicitly supports disabling

        Host mode is supported for leaf tools such as lazygit, bottom, helix, steel,
        neovim, yazi, fzf, zoxide, starship, carapace, macchina, mise, tombi, git, jq,
        fd, and ripgrep. Bootstrap tools such as Nushell, Zellij, the selected
        terminal, Nix, POSIX utilities, and graphics wrappers remain bundled.

        Off mode is supported for optional helpers such as steel, macchina, p7zip,
        poppler, and resvg. Disabled helpers are intentionally omitted from the
        packaged runtime and reported as disabled instead of missing.
      '';
    };

    components = mkOption {
      type = types.attrsOf types.bool;
      default = { };
      example = {
        cursors = false;
        screen = false;
      };
      description = ''
        Optional Yazelix runtime components. Omitted components default to true.

        Supported components:
        - "cursors": Yazelix cursor shader assets and shared cursor config integration
        - "screen": startup welcome animation and `yzx screen` renderer integration
      '';
    };

    agent_usage_programs = mkOption {
      type = types.listOf (types.enum agentUsageProgramNames);
      default = [ "tokenusage" ];
      description = ''
        Usage binaries to include in the Yazelix runtime.

        These support zellij.widget_tray usage entries:
        - "tokenusage": claude_usage, codex_usage

        codex_usage is a combined 5h/week token and quota widget.
        claude_usage is a combined 5h/week token and quota widget.
        opencode_go_usage reads OpenCode's local SQLite database directly and does
        not require an extra usage binary. Configure rendered windows with
        zellij_codex_usage_periods, zellij_claude_usage_periods, and
        zellij_opencode_go_usage_periods.

        Set this to [] only if the Claude and Codex usage widgets are removed
        from zellij_widget_tray or intentionally host-provided.
      '';
    };

    # Configuration options (mirrors settings_default.jsonc structure)
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

        Cursor presets and cursor effects live in ~/.config/yazelix_ghostty_cursors/settings.jsonc
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
        - "hx": Use the packaged Helix command from the Yazelix runtime
        - Other editors: "vim", "nano", "emacs", etc. (basic integration only)
      '';
    };

    helix_external = mkMainContractOption "helix.external" {
      description = ''
        Custom Helix binary/runtime pair.

        Set this only when running a user-owned Helix fork. Both binary and
        runtime_path are required because the runtime MUST match the Helix
        binary version.

        Example:
          {
            binary = "/home/user/helix/target/release/hx";
            runtime_path = "/home/user/helix/runtime";
          }
      '';
    };

    helix_steel_plugins = mkMainContractOption "helix.steel_plugins" {
      description = ''
        Helix Steel plugin selection.

        enabled selects bundled plugin ids from Yazelix's packaged plugin
        repository. extra declares user-owned plugin manifests whose source
        files are resolved below ~/.config/yazelix/helix/steel_plugins and
        copied into the generated Yazelix Helix runtime config.
      '';
    };

    hide_sidebar_on_file_open = mkMainContractOption "editor.hide_sidebar_on_file_open" {
      description = ''
        Whether Yazelix should hide the managed sidebar after opening a file from
        the Yazi file-tree sidebar.
      '';
    };

    left_sidebar_command = mkMainContractOption "workspace.left_sidebar.command" {
      description = "Terminal command used for the managed left sidebar pane. Defaults to `yzx`.";
    };

    left_sidebar_args = mkMainContractOption "workspace.left_sidebar.args" {
      description = ''
        Arguments passed to the managed left sidebar command.

        The default launches Yazelix's managed Yazi file-tree adapter with `yzx sidebar yazi`.
      '';
    };

    left_sidebar_width_percent = mkMainContractOption "workspace.left_sidebar.width_percent" {
      description = "Width of the open left sidebar as a percentage of the tab.";
    };

    right_sidebar_command = mkMainContractOption "workspace.right_sidebar.command" {
      description = "Terminal command used for the managed right sidebar pane. Defaults to host-installed Codex.";
    };

    right_sidebar_args = mkMainContractOption "workspace.right_sidebar.args" {
      description = "Arguments passed to the managed right sidebar command.";
    };

    right_sidebar_width_percent = mkMainContractOption "workspace.right_sidebar.width_percent" {
      description = "Width of the open right sidebar as a percentage of the tab.";
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

    zellij_tab_label_mode = mkMainContractOption "zellij.tab_label_mode" {
      description = ''
        Zjstatus tab-label mode.

        - "full": show tab index and tab name
        - "compact": show tab index and state indicators only
      '';
    };

    zellij_codex_usage_display = mkMainContractOption "zellij.codex_usage_display" {
      description = "Codex usage widget display mode: token, quota, or both";
    };

    zellij_codex_usage_periods = mkMainContractOption "zellij.codex_usage_periods" {
      description = "Periods shown by the codex_usage widget: 5h, week";
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
        Default transient popup command for `yzx popup`.
        Use an argv-style list, eg. [ "lazygit" ], [ "editor" ] to reuse `editor.command`,
        or [ "codex" ].
      '';
    };

    popup_commands = mkMainContractOption "zellij.popup_commands" {
      description = ''
        Commands for named Yazelix popup surfaces.
        Defaults: bottom_popup = [ "lazygit" ], top_popup = [ "yzx" "config" "ui" ],
        menu = [ "yzx" "menu" ], btm = [ "btm" ].
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
        Browse bundled Yazelix flavors: https://github.com/luccahuguet/yazelix-yazi-assets/tree/main/flavors
      '';
    };

    yazi_sort_by = mkMainContractOption "yazi.sort_by" {
      description = "Default file sorting method";
    };

    yazi_keybindings = mkMainContractOption "yazi.keybindings" {
      description = ''
        Semantic remaps for Yazelix-owned Yazi integration actions.

        Keys are action ids such as "open_directory_as_workspace_pane" and
        "open_zoxide_in_editor"; values are lists of generated Yazi bindings
        such as "<A-p>". Use an empty list to disable the generated binding for
        one action.
      '';
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
        - "magician": show the 1mposter ASCII magician GIF animation
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

    zellij_keybindings = mkMainContractOption "zellij.keybindings" {
      description = ''
        Semantic remaps for Yazelix-owned Zellij actions.

        Keys are action ids such as "bottom_popup", "top_popup", "menu", "btm",
        "toggle_left_sidebar", and "move_focus_left_or_tab"; values are lists of
        Zellij key strings. Use an empty list to disable the generated binding
        for one action.
      '';
    };

    zellij_native_keybindings = mkMainContractOption "zellij.native_keybindings" {
      description = ''
        Curated native Zellij key policy remaps and unbinds managed by Yazelix.

        Keys are policy ids such as "scroll_mode", "scroll_mode_unbind",
        "move_tab_left", "move_pane_down", and "move_tab_left_unbind"; values
        are lists of Zellij key strings. Use an empty list to disable one native
        policy entry.
      '';
    };

  };

  config = mkIf cfg.enable (mkMerge [
    {
      # Expose the packaged Yazelix runtime through the Home Manager profile.
      home.packages = [ yazelixPackage ] ++ extraTerminalVariantPackages;
      home.sessionVariables = mkIf (cfg.yzxterm_profile != "full") {
        YAZELIX_TERMINAL_PROFILE = mkDefault cfg.yzxterm_profile;
      };

      programs.yazelix.terminals = mkDefault (
        runtimeDefaultTerminals cfg.runtime_variant cfg.extra_terminal_variants
      );

      assertions = [
        {
          assertion = (componentEnabled "cursors") || !cfg.manage_cursor_config;
          message = "programs.yazelix.manage_cursor_config requires programs.yazelix.components.cursors to remain enabled";
        }
        {
          assertion = (componentEnabled "screen") || cfg.skip_welcome_screen;
          message = "programs.yazelix.components.screen = false requires programs.yazelix.skip_welcome_screen = true";
        }
        {
          assertion = (componentEnabled "screen") || !cfg.screen_saver_enabled;
          message = "programs.yazelix.components.screen = false requires programs.yazelix.screen_saver_enabled = false";
        }
        {
          assertion = (runtimeToolSource "macchina") != "off" || !cfg.show_macchina_on_welcome;
          message = "programs.yazelix.runtime_tool_sources.macchina = \"off\" requires programs.yazelix.show_macchina_on_welcome = false";
        }
      ];

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
        export PATH="${yazelixPackage}/toolbin:${yazelixPackage}/libexec:${yazelixPackage}/bin:${runtimeConfigGenerationPath}:$PATH"
        export YAZELIX_RUNTIME_DIR="${yazelixPackage}"
        export YAZELIX_CONFIG_DIR="${managedConfigRoot}"
        export YAZELIX_STATE_DIR="${stateRoot}"
        export YAZELIX_LOGS_DIR="${logsPath}"
        ${yzxtermProfileExport}

        $DRY_RUN_CMD ${runtimeYzxCore} runtime-materialization.repair --from-env --force --summary
        $DRY_RUN_CMD env YAZELIX_QUIET_MODE=true ${runtimeYzxControl} generate_shell_initializers
      '';
    }
    (mkIf pkgs.stdenv.hostPlatform.isLinux (
      lib.optionalAttrs (lib.hasAttrByPath [ "xdg" "desktopEntries" ] options) {
        # Linux desktop entry for application launchers.
        xdg.desktopEntries.yazelix = {
          name = "Yazelix";
          comment = "Yazi + Zellij + Helix integrated terminal environment";
          exec = yzxtermDesktopExec;
          icon = "yazelix";
          categories = [ "Development" ];
          type = "Application";
          terminal = false;
          settings = {
            StartupWMClass = "com.yazelix.Yazelix";
          };
        };
      }
    ))
    (mkIf cfg.manage_config {
      # Generate settings.jsonc configuration file
      xdg.configFile."yazelix/settings.jsonc" = {
        text = settingsJsonc;
      };
    })
    (mkIf cfg.manage_cursor_config {
      # Generate shared Yazelix cursor settings for both Yazelix and yzc.
      xdg.configFile."yazelix_ghostty_cursors/settings.jsonc" = {
        text = cursorSettingsJsonc;
      };
    })
  ]);
}
