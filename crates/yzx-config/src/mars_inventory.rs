use ratconfig::{ConfigUiCapability, ConfigUiChoice, ConfigUiTextEncoding};
use serde_json::Value as JsonValue;

use crate::{
    catalog::MARS_APPEARANCE_PRESET_PATH,
    common::{Result, error, json_i64},
};

const INVENTORY: &str = include_str!("../../../../mars/docs/yazelix/config_inventory.v1.json");
const MARS_INVENTORY_SCHEMA_VERSION: u64 = 1;

pub(crate) struct MarsInventory {
    document: JsonValue,
}

#[derive(Clone, Copy)]
pub(crate) struct MarsField<'a> {
    inventory: &'a JsonValue,
    entry: &'a JsonValue,
}

impl MarsInventory {
    pub(crate) fn parse() -> Result<Self> {
        let document: JsonValue = serde_json::from_str(INVENTORY)?;
        if document["schema_version"] != MARS_INVENTORY_SCHEMA_VERSION
            || document["owner"] != "mars"
            || !document["entries"].is_array()
        {
            return Err(error("unsupported Mars config inventory contract"));
        }
        Ok(Self { document })
    }

    pub(crate) fn fields(&self) -> impl Iterator<Item = MarsField<'_>> {
        self.document["entries"]
            .as_array()
            .expect("validated Mars inventory")
            .iter()
            .filter(|entry| entry["path"] != MARS_APPEARANCE_PRESET_PATH)
            .map(|entry| MarsField {
                inventory: &self.document,
                entry,
            })
    }

    pub(crate) fn field(&self, path: &str) -> Option<MarsField<'_>> {
        self.fields().find(|field| field.path() == path)
    }
}

impl<'a> MarsField<'a> {
    pub(crate) fn path(self) -> &'a str {
        self.entry["path"].as_str().expect("Mars inventory path")
    }

    pub(crate) fn group(self) -> &'a str {
        self.entry["group"].as_str().expect("Mars inventory group")
    }

    pub(crate) fn description(self) -> String {
        let mut text = self.entry["description"]
            .as_str()
            .expect("Mars inventory description")
            .to_string();
        if let Some(note) = self.entry.get("constraints").and_then(availability) {
            text.push_str(&format!(" Availability: {note}."));
        }
        let choices = self
            .choices()
            .filter_map(|choice| {
                availability(choice)
                    .map(|note| format!("{}: {note}", display_value(choice_value(choice))))
            })
            .collect::<Vec<_>>();
        if !choices.is_empty() {
            text.push_str(&format!(" Choice availability: {}.", choices.join("; ")));
        }
        if let Some(default) = self.entry["default"]
            .get("built_in")
            .and_then(JsonValue::as_str)
        {
            text.push_str(&format!(" Built-in default: {default}"));
        }
        text
    }

    pub(crate) fn type_label(self) -> &'a str {
        let shape = self.resolved_shape();
        shape
            .as_str()
            .or_else(|| shape["kind"].as_str())
            .unwrap_or("structured")
    }

    pub(crate) fn default(self) -> Option<&'a JsonValue> {
        self.entry["default"].get("value").or_else(|| {
            self.entry["default"]
                .get("platform")
                .and_then(|values| values.get(std::env::consts::OS))
        })
    }

    pub(crate) fn validation(self) -> String {
        match self.type_label() {
            "boolean" => "true or false".to_string(),
            "string" => "a string".to_string(),
            "enum" | "union" if self.entry["choices"].is_array() => format!(
                "one of: {}",
                self.available_choices()
                    .map(choice_value)
                    .map(display_value)
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            "integer" if matches!(self.path(), "window.width" | "window.height") => {
                "a positive integer".to_string()
            }
            "float" if self.path() == "window.opacity" => "a number from 0.0 to 1.0".to_string(),
            "float" if matches!(self.path(), "fonts.size" | "line-height") => {
                "a positive number".to_string()
            }
            kind => format!("Mars {kind} value"),
        }
    }

    pub(crate) fn capability(self) -> ConfigUiCapability {
        if !available_on_current_platform(&self.entry["constraints"]) {
            return ConfigUiCapability::ReadOnly {
                reason: format!(
                    "This setting is available only on {}.",
                    availability(&self.entry["constraints"]).expect("platform constraint")
                ),
                file_action_id: None,
            };
        }
        match self.type_label() {
            "boolean" => ConfigUiCapability::Toggle {
                off: ConfigUiChoice::new(JsonValue::Bool(false)),
                on: ConfigUiChoice::new(JsonValue::Bool(true)),
            },
            "string" => ConfigUiCapability::FreeText {
                encoding: ConfigUiTextEncoding::String,
            },
            "enum" | "union" if self.entry["choices"].is_array() => ConfigUiCapability::Choice {
                choices: self
                    .available_choices()
                    .map(choice_value)
                    .cloned()
                    .map(ConfigUiChoice::new)
                    .collect(),
            },
            "integer" | "float" if self.has_numeric_validator() => ConfigUiCapability::FreeText {
                encoding: ConfigUiTextEncoding::Json,
            },
            kind => ConfigUiCapability::ReadOnly {
                reason: format!("The Mars {kind} shape has no safe inline editor."),
                file_action_id: None,
            },
        }
    }

    pub(crate) fn is_editable(self) -> bool {
        !matches!(self.capability(), ConfigUiCapability::ReadOnly { .. })
    }

    pub(crate) fn validate(self, value: &JsonValue) -> Result<()> {
        match self.type_label() {
            "boolean" if value.is_boolean() => Ok(()),
            "string" if value.is_string() => Ok(()),
            "enum" | "union" if self.accepts_choice(value) => Ok(()),
            "integer" if self.has_numeric_validator() => {
                if json_i64(self.path(), value)? > 0 {
                    Ok(())
                } else {
                    Err(error(format!("{} must be positive", self.path())))
                }
            }
            "float" if self.has_numeric_validator() => {
                let number = value
                    .as_f64()
                    .ok_or_else(|| error(format!("{} must be a number", self.path())))?;
                if self.path() == "window.opacity" && !(0.0..=1.0).contains(&number) {
                    return Err(error("window.opacity must be between 0.0 and 1.0"));
                }
                if matches!(self.path(), "fonts.size" | "line-height") && number <= 0.0 {
                    return Err(error(format!("{} must be positive", self.path())));
                }
                Ok(())
            }
            _ => Err(error(format!(
                "{} must be {}",
                self.path(),
                self.validation()
            ))),
        }
    }

    fn choices(self) -> impl Iterator<Item = &'a JsonValue> {
        self.entry["choices"].as_array().into_iter().flatten()
    }

    fn available_choices(self) -> impl Iterator<Item = &'a JsonValue> {
        self.choices()
            .filter(|choice| available_on_current_platform(choice))
    }

    fn accepts_choice(self, value: &JsonValue) -> bool {
        self.available_choices()
            .any(|choice| choice_value(choice) == value)
    }

    fn resolved_shape(self) -> &'a JsonValue {
        let shape = &self.entry["shape"];
        shape
            .get("ref")
            .and_then(JsonValue::as_str)
            .map_or(shape, |name| &self.inventory["shape_definitions"][name])
    }

    fn has_numeric_validator(self) -> bool {
        matches!(
            self.path(),
            "window.width" | "window.height" | "window.opacity" | "fonts.size" | "line-height"
        )
    }
}

fn choice_value(choice: &JsonValue) -> &JsonValue {
    choice.get("value").unwrap_or(choice)
}

fn display_value(value: &JsonValue) -> String {
    value
        .as_str()
        .map(str::to_string)
        .unwrap_or_else(|| value.to_string())
}

fn availability(value: &JsonValue) -> Option<String> {
    if let Some(note) = value.get("availability").and_then(JsonValue::as_str) {
        return Some(note.trim_end_matches('.').to_string());
    }
    let mut parts = Vec::new();
    if let Some(platforms) = value.get("platforms").and_then(JsonValue::as_array) {
        parts.push(
            platforms
                .iter()
                .filter_map(JsonValue::as_str)
                .map(|platform| match platform {
                    "linux" => "Linux",
                    "macos" => "macOS",
                    "windows" => "Windows",
                    other => other,
                })
                .collect::<Vec<_>>()
                .join(", "),
        );
    }
    if let Some(features) = value.get("features").and_then(JsonValue::as_array) {
        parts.push(format!(
            "requires {}",
            features
                .iter()
                .filter_map(JsonValue::as_str)
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
    (!parts.is_empty()).then(|| parts.join("; "))
}

fn available_on_current_platform(value: &JsonValue) -> bool {
    value["platforms"].as_array().is_none_or(|platforms| {
        platforms
            .iter()
            .any(|platform| platform == std::env::consts::OS)
    })
}
