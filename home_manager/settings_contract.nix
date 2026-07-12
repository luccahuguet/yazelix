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
        else if field.kind == "bool" then
          types.bool
        else if field.kind == "string" then
          types.str
        else if field.kind == "string_list" then
          types.listOf types.str
        else if field.kind == "int" then
          types.int
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

  mainConfigValueForSettings = fieldPath: configValueForField fieldPath;

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

  configTomlValue =
    mainConfigSettingsValue
    // lib.optionalAttrs (cfg.popups != null) {
      popups = lib.mapAttrs (_id: popup: lib.filterAttrs (_name: value: value != null) popup) cfg.popups;
    };
in
{
  inherit configTomlValue mkMainContractOption;
}
