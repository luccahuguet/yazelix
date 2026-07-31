use ratconfig::{ConfigUiSchemaField, collect_resolved_config_ui_schema_fields};
use serde_json::Value as JsonValue;

use crate::common::{Result, error};

pub(crate) const PACKAGED_STARSHIP_DEFAULT_CONFIG_TOML: &str = include_str!(env!(
    "YAZELIX_STARSHIP_DEFAULT_CONFIG",
    "YAZELIX_STARSHIP_DEFAULT_CONFIG must point to `starship print-config --default` output"
));
const SCHEMA: &str = include_str!(env!(
    "YAZELIX_STARSHIP_CONFIG_SCHEMA",
    "YAZELIX_STARSHIP_CONFIG_SCHEMA must point to the packaged Starship JSON Schema"
));

pub(crate) struct StarshipInventory(Vec<ConfigUiSchemaField>);

impl StarshipInventory {
    pub(crate) fn parse() -> Result<Self> {
        let schema: JsonValue = serde_json::from_str(SCHEMA)?;
        if schema["title"] != "FullConfig" {
            return Err(error("unsupported Starship config schema"));
        }
        let fields = collect_resolved_config_ui_schema_fields(&schema, "root")
            .into_iter()
            .map(|mut field| {
                field.path = field
                    .path
                    .strip_prefix("root.")
                    .unwrap_or(&field.path)
                    .to_string();
                field
            })
            .collect::<Vec<_>>();
        if !fields.iter().any(|field| field.path == "character.format")
            || !fields.iter().any(|field| field.path == "custom")
        {
            return Err(error("incomplete Starship config schema"));
        }
        Ok(Self(fields))
    }

    pub(crate) fn fields(&self) -> impl Iterator<Item = &ConfigUiSchemaField> {
        self.0.iter()
    }

    pub(crate) fn field(&self, path: &str) -> Option<&ConfigUiSchemaField> {
        self.0.iter().find(|field| field.path == path)
    }
}

pub(crate) fn starship_field_is_editable(field: &ConfigUiSchemaField) -> bool {
    matches!(field.kind.as_str(), "boolean" | "string")
}

pub(crate) fn validate_starship_field(
    field: &ConfigUiSchemaField,
    value: &JsonValue,
) -> Result<()> {
    if !starship_field_is_editable(field) {
        return Err(error(format!(
            "Starship config path {} has no schema-backed inline editor",
            field.path
        )));
    }
    let valid = match field.kind.as_str() {
        "boolean" => value.is_boolean(),
        "string" => value.is_string(),
        _ => false,
    };
    if !valid {
        return Err(error(format!(
            "{} must be a Starship {}",
            field.path, field.kind
        )));
    }
    Ok(())
}
