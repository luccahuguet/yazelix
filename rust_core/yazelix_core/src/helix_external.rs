use serde::{Deserialize, Serialize};
use serde_json::{Map as JsonMap, Value as JsonValue, json};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HelixExternalPair {
    pub binary: String,
    pub runtime_path: String,
}

impl HelixExternalPair {
    pub fn normalized(binary: &str, runtime_path: &str) -> Option<Self> {
        let binary = non_empty_string(binary)?;
        let runtime_path = non_empty_string(runtime_path)?;
        Some(Self {
            binary,
            runtime_path,
        })
    }

    pub fn from_json(value: &JsonValue) -> Option<Self> {
        let object = value.as_object()?;
        Self::from_json_object(object)
    }

    pub fn from_json_object(object: &JsonMap<String, JsonValue>) -> Option<Self> {
        Self::normalized(
            object.get("binary")?.as_str()?,
            object.get("runtime_path")?.as_str()?,
        )
    }

    pub fn as_json(&self) -> JsonValue {
        json!({
            "binary": self.binary,
            "runtime_path": self.runtime_path,
        })
    }
}

pub fn non_empty_string(value: &str) -> Option<String> {
    let trimmed = value.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_string())
}

pub fn is_helix_command(command: &str) -> bool {
    let normalized = command.trim();
    normalized == "hx"
        || normalized == "helix"
        || normalized.ends_with("/hx")
        || normalized.ends_with("/helix")
}

pub fn is_custom_helix_binary_command(command: &str) -> bool {
    let trimmed = command.trim();
    if trimmed == "hx" || trimmed == "helix" || trimmed.is_empty() {
        return false;
    }
    let Some(file_name) = Path::new(trimmed)
        .file_name()
        .and_then(|name| name.to_str())
    else {
        return false;
    };
    matches!(file_name, "hx" | "helix")
}
