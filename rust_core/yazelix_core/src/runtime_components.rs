// Test lane: default
//! Runtime component manifest helpers.

use crate::bridge::{CoreError, ErrorClass};
use serde::Deserialize;
use serde_json::json;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Deserialize)]
pub struct RuntimeComponentManifestEntry {
    pub enabled: bool,
    #[allow(dead_code)]
    pub disableable: bool,
    #[allow(dead_code)]
    pub notes: Vec<String>,
}

fn runtime_components_manifest_path(runtime_dir: &Path) -> PathBuf {
    runtime_dir.join("runtime_components.json")
}

pub fn read_runtime_component_manifest(
    runtime_dir: &Path,
) -> Result<BTreeMap<String, RuntimeComponentManifestEntry>, CoreError> {
    let manifest_path = runtime_components_manifest_path(runtime_dir);
    let raw = fs::read_to_string(&manifest_path).map_err(|source| {
        CoreError::io(
            "read_runtime_component_manifest",
            "Could not read the Yazelix runtime component manifest",
            "Reinstall Yazelix so runtime_components.json is present in the runtime root.",
            manifest_path.to_string_lossy(),
            source,
        )
    })?;

    serde_json::from_str(&raw).map_err(|source| {
        CoreError::classified(
            ErrorClass::Runtime,
            "parse_runtime_component_manifest",
            "Could not parse the Yazelix runtime component manifest.",
            "Reinstall Yazelix so runtime_components.json is valid.",
            json!({
                "path": manifest_path.to_string_lossy(),
                "error": source.to_string(),
            }),
        )
    })
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
        format!("Enable programs.yazelix.components.{component}, or stop using {label}."),
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
