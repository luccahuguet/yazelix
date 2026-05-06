{ lib, components ? { } }:

let
  componentDefaults = {
    screen = {
      enabled = true;
      disableable = false;
      notes = [ "Screen commands are still compiled into the Rust helper." ];
    };
    cursors = {
      enabled = true;
      disableable = false;
      notes = [ "Cursor commands and config UI are still compiled into the Rust helper." ];
    };
  };
  componentNames = builtins.attrNames components;
  unknownComponentNames = lib.filter (name: !(builtins.hasAttr name componentDefaults)) componentNames;
  invalidValueNames = lib.filter (name: !(builtins.isBool components.${name})) componentNames;
  unsupportedDisabledNames = lib.filter (
    name: components.${name} == false && !(componentDefaults.${name}.disableable or false)
  ) componentNames;
  enabledFor = name: components.${name} or componentDefaults.${name}.enabled;
  manifest = lib.mapAttrs (name: component: {
    enabled = enabledFor name;
    disableable = component.disableable;
    notes = component.notes;
  }) componentDefaults;
in
if unknownComponentNames != [ ] then
  throw "Unsupported Yazelix component(s): ${lib.concatStringsSep ", " unknownComponentNames}"
else if invalidValueNames != [ ] then
  throw "Unsupported Yazelix component value(s) for: ${lib.concatStringsSep ", " invalidValueNames}. Expected booleans."
else if unsupportedDisabledNames != [ ] then
  throw "Yazelix component disabling is not supported yet for: ${lib.concatStringsSep ", " unsupportedDisabledNames}"
else
  {
    inherit componentDefaults manifest;
    manifestJson = builtins.toJSON manifest;
  }
