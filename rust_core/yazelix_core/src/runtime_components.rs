// Test lane: default
//! Runtime self-description manifest helpers.

use crate::bridge::{CoreError, ErrorClass};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::BTreeMap;
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

const OPTIONAL_HOST_INTEGRATION_NOTE: &str = "optional_host_integration";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MissingManifest {
    Error,
    Ok,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct RuntimeToolManifestEntry {
    pub source: String,
    #[serde(default)]
    pub commands: Vec<String>,
    #[serde(default)]
    pub required_commands: Vec<String>,
    #[serde(default)]
    pub hostable: bool,
    #[serde(default)]
    pub disableable: bool,
    #[serde(default)]
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct RuntimeComponentManifestEntry {
    pub enabled: bool,
    pub disableable: bool,
    pub notes: Vec<String>,
}

pub(crate) fn runtime_components_manifest_path(runtime_dir: &Path) -> PathBuf {
    runtime_dir.join("runtime_components.json")
}

pub fn runtime_tools_manifest_path(runtime_dir: &Path) -> PathBuf {
    runtime_dir.join("runtime_tools.json")
}

fn read_optional_runtime_manifest<T>(
    path: &Path,
    missing_manifest: MissingManifest,
    read_code: &'static str,
    read_message: &'static str,
    read_remediation: &'static str,
    parse_code: &'static str,
    parse_message: &'static str,
    parse_remediation: &'static str,
) -> Result<Option<BTreeMap<String, T>>, CoreError>
where
    T: for<'de> Deserialize<'de>,
{
    let raw = match fs::read_to_string(path) {
        Ok(raw) => raw,
        Err(source)
            if source.kind() == ErrorKind::NotFound && missing_manifest == MissingManifest::Ok =>
        {
            return Ok(None);
        }
        Err(source) => {
            return Err(CoreError::io(
                read_code,
                read_message,
                read_remediation,
                path.to_string_lossy(),
                source,
            ));
        }
    };

    serde_json::from_str(&raw).map(Some).map_err(|source| {
        CoreError::classified(
            ErrorClass::Runtime,
            parse_code,
            parse_message,
            parse_remediation,
            json!({
                "path": path.to_string_lossy(),
                "error": source.to_string(),
            }),
        )
    })
}

pub fn read_runtime_component_manifest(
    runtime_dir: &Path,
) -> Result<BTreeMap<String, RuntimeComponentManifestEntry>, CoreError> {
    let manifest_path = runtime_components_manifest_path(runtime_dir);
    read_optional_runtime_manifest(
        &manifest_path,
        MissingManifest::Error,
        "read_runtime_component_manifest",
        "Could not read the Yazelix runtime component manifest",
        "Reinstall Yazelix so runtime_components.json is present in the runtime root.",
        "parse_runtime_component_manifest",
        "Could not parse the Yazelix runtime component manifest.",
        "Reinstall Yazelix so runtime_components.json is valid.",
    )?
    .ok_or_else(|| unreachable!("required manifest helper returns an error when missing"))
}

pub fn read_optional_runtime_component_manifest(
    runtime_dir: &Path,
) -> Result<Option<BTreeMap<String, RuntimeComponentManifestEntry>>, CoreError> {
    read_optional_runtime_manifest(
        &runtime_components_manifest_path(runtime_dir),
        MissingManifest::Ok,
        "read_runtime_component_manifest",
        "Could not read the Yazelix runtime component manifest",
        "Reinstall Yazelix so runtime_components.json is present in the runtime root.",
        "parse_runtime_component_manifest",
        "Could not parse the Yazelix runtime component manifest.",
        "Reinstall Yazelix so runtime_components.json is valid.",
    )
}

pub fn read_optional_runtime_tool_manifest(
    runtime_dir: &Path,
) -> Result<Option<BTreeMap<String, RuntimeToolManifestEntry>>, CoreError> {
    read_optional_runtime_manifest(
        &runtime_tools_manifest_path(runtime_dir),
        MissingManifest::Ok,
        "read_runtime_tool_manifest",
        "Could not read the Yazelix runtime tool manifest",
        "Reinstall Yazelix so runtime_tools.json is present and readable.",
        "parse_runtime_tool_manifest",
        "Could not parse the Yazelix runtime tool manifest.",
        "Reinstall Yazelix so runtime_tools.json is valid.",
    )
}

pub(crate) fn runtime_tool_required_commands(tool: &RuntimeToolManifestEntry) -> &[String] {
    if tool.required_commands.is_empty() {
        &tool.commands
    } else {
        &tool.required_commands
    }
}

pub(crate) fn runtime_tool_is_optional_host_integration(tool: &RuntimeToolManifestEntry) -> bool {
    tool.notes
        .iter()
        .any(|note| note == OPTIONAL_HOST_INTEGRATION_NOTE)
}

pub fn runtime_component_enabled(runtime_dir: &Path, component: &str) -> Result<bool, CoreError> {
    let manifest = read_runtime_component_manifest(runtime_dir)?;
    manifest
        .get(component)
        .map(|entry| entry.enabled)
        .ok_or_else(|| {
            CoreError::classified(
                ErrorClass::Runtime,
                "unknown_runtime_component",
                format!(
                    "Yazelix runtime component `{component}` is not described by this runtime."
                ),
                "Reinstall Yazelix or remove the unsupported component override.",
                json!({ "component": component }),
            )
        })
}

pub fn require_runtime_component_enabled(
    runtime_dir: &Path,
    component: &str,
    label: &str,
) -> Result<(), CoreError> {
    if runtime_component_enabled(runtime_dir, component)? {
        return Ok(());
    }

    Err(CoreError::classified(
        ErrorClass::Runtime,
        "disabled_runtime_component",
        format!("{label} is disabled in this Yazelix runtime."),
        format!(
            "Install a Yazelix package with the `{component}` component enabled, or stop using {label}."
        ),
        json!({
            "component": component,
            "label": label,
        }),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    // Defends: runtime component off mode is read from the packaged manifest instead of inferred from file presence.
    #[test]
    fn component_enabled_reads_packaged_manifest() {
        let tmp = TempDir::new().unwrap();
        std::fs::write(
            tmp.path().join("runtime_components.json"),
            r#"{
              "cursors": { "enabled": false, "disableable": true, "notes": [] },
              "screen": { "enabled": true, "disableable": true, "notes": [] }
            }"#,
        )
        .unwrap();

        assert!(!runtime_component_enabled(tmp.path(), "cursors").unwrap());
        assert!(runtime_component_enabled(tmp.path(), "screen").unwrap());
    }

    // Defends: the packaged runtime component manifest fails fast on stale fields instead of letting runtime and metadata drift.
    #[test]
    fn component_manifest_rejects_unknown_fields() {
        let tmp = TempDir::new().unwrap();
        std::fs::write(
            tmp.path().join("runtime_components.json"),
            r#"{ "screen": { "enabled": true, "disableable": true, "notes": [], "stale": true } }"#,
        )
        .unwrap();

        let err = read_runtime_component_manifest(tmp.path()).unwrap_err();
        let details = err.details();
        let detail = details["error"].as_str().unwrap_or_default();

        assert_eq!(err.code(), "parse_runtime_component_manifest");
        assert!(detail.contains("unknown field"));
        assert!(detail.contains("stale"));
    }

    // Regression: disabled public runtime surfaces fail with a component-specific error instead of falling through to missing assets.
    #[test]
    fn disabled_component_reports_unavailable_surface() {
        let tmp = TempDir::new().unwrap();
        std::fs::write(
            tmp.path().join("runtime_components.json"),
            r#"{ "screen": { "enabled": false, "disableable": true, "notes": [] } }"#,
        )
        .unwrap();

        let err =
            require_runtime_component_enabled(tmp.path(), "screen", "yzx screen").unwrap_err();

        assert_eq!(err.code(), "disabled_runtime_component");
    }
}
