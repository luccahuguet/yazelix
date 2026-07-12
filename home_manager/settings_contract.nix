{
  cfg,
  lib,
}:

with lib;

let
  mainConfigContract = builtins.fromTOML (builtins.readFile ../config_metadata/main_config_contract.toml);
  mainContractFields = mainConfigContract.fields;

  attrOr =
    attrs: name: fallback:
    if builtins.hasAttr name attrs then builtins.getAttr name attrs else fallback;

  getMainField = fieldPath: builtins.getAttr fieldPath mainContractFields;

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
        else if field.kind == "custom_popup_list" then
          types.listOf (types.submodule {
            options = {
              id = mkOption {
                type = types.str;
                description = "Stable custom popup id";
              };
              command = mkOption {
                type = types.listOf types.str;
                description = "Command argv used by this popup";
              };
              keybindings = mkOption {
                type = types.listOf types.str;
                default = [ ];
                description = "Zellij key strings that toggle this popup";
              };
              keep_alive = mkOption {
                type = types.nullOr types.bool;
                default = null;
                description = "Whether focused toggle hides this popup instead of closing it. Null uses the Yazelix default for the popup id and command.";
              };
            };
          })
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
                description = "Bundled Helix Steel plugin ids to load from the packaged yazelix-helix plugin repository";
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
                description = "Yazelix-compatible Helix fork binary path";
              };
              runtime_path = mkOption {
                type = types.str;
                description = "Runtime path matching the Yazelix-compatible Helix fork binary";
              };
            };
          }
        else
          throw "Unsupported main config contract kind for Home Manager: ${field.kind}";
    in
    types.nullOr baseType;

  mkMainContractOption =
    fieldPath: extra:
    let
      field = getMainField fieldPath;
    in
    mkOption (
      {
        type = mainFieldType field;
        default = null;
      }
      // extra
    );

  mainConfigFieldPaths = lib.sort builtins.lessThan (builtins.attrNames mainContractFields);

  configValueForField =
    fieldPath:
    let
      field = getMainField fieldPath;
    in
    builtins.getAttr field.home_manager_option cfg;

  mainConfigValueForSettings =
    fieldPath:
    let
      field = getMainField fieldPath;
      value = configValueForField fieldPath;
    in
    if value == null then
      null
    else if field.kind == "custom_popup_list" then
      map (popup: lib.filterAttrs (_name: popupValue: popupValue != null) popup) value
    else
      value;

  mainConfigSettingsFieldIncluded = fieldPath: mainConfigValueForSettings fieldPath != null;

  mainConfigSettingsFieldPaths =
    builtins.filter mainConfigSettingsFieldIncluded mainConfigFieldPaths;

  mainConfigSettingsValue =
    lib.foldl' (
      acc: fieldPath:
      lib.recursiveUpdate acc (
        lib.setAttrByPath (lib.splitString "." fieldPath) (mainConfigValueForSettings fieldPath)
      )
    ) { } mainConfigSettingsFieldPaths;

  configTomlValue = mainConfigSettingsValue;
in
{
  inherit configTomlValue mkMainContractOption;
}
