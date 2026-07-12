// Test lane: maintainer

use crate::bridge::CoreError;
use crate::public_command_surface::{
    YzxPublicRootRoute, classify_yzx_root_route, yzx_command_metadata,
};
use crate::runtime_components::{
    read_optional_runtime_component_manifest, read_optional_runtime_tool_manifest,
    runtime_components_manifest_path, runtime_tools_manifest_path,
};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

const SCHEMA_VERSION: u8 = 1;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct RuntimeOwnershipGraphRequest {
    pub runtime_dir: PathBuf,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct RuntimeOwnershipGraphData {
    pub schema_version: u8,
    pub runtime_dir: String,
    pub command_owners: Vec<CommandOwnerEntry>,
    pub config_owners: Vec<SurfaceOwnerEntry>,
    pub generated_state_owners: Vec<SurfaceOwnerEntry>,
    pub runtime_tools: RuntimeManifestSection<RuntimeToolOwnerEntry>,
    pub runtime_components: RuntimeManifestSection<RuntimeComponentOwnerEntry>,
    pub validation_commands: Vec<ValidationCommandEntry>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct CommandOwnerEntry {
    pub command: String,
    pub owner: String,
    pub route: String,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct SurfaceOwnerEntry {
    pub surface: String,
    pub owner: String,
    pub source: String,
    pub validation: Vec<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct RuntimeManifestSection<T> {
    pub status: String,
    pub path: String,
    pub entries: Vec<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RuntimeToolOwnerEntry {
    pub name: String,
    pub source: String,
    pub commands: Vec<String>,
    pub required_commands: Vec<String>,
    pub hostable: bool,
    pub disableable: bool,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RuntimeComponentOwnerEntry {
    pub name: String,
    pub enabled: bool,
    pub disableable: bool,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ValidationCommandEntry {
    pub subsystem: String,
    pub command: String,
    pub owner: String,
}

pub fn compute_runtime_ownership_graph(
    request: &RuntimeOwnershipGraphRequest,
) -> Result<RuntimeOwnershipGraphData, CoreError> {
    Ok(RuntimeOwnershipGraphData {
        schema_version: SCHEMA_VERSION,
        runtime_dir: request.runtime_dir.to_string_lossy().to_string(),
        command_owners: command_owners(),
        config_owners: config_owners(),
        generated_state_owners: generated_state_owners(),
        runtime_tools: runtime_tool_manifest(&request.runtime_dir)?,
        runtime_components: runtime_component_manifest(&request.runtime_dir)?,
        validation_commands: validation_commands(),
    })
}

fn command_owners() -> Vec<CommandOwnerEntry> {
    yzx_command_metadata()
        .into_iter()
        .map(|metadata| {
            let tail = metadata
                .name
                .split_whitespace()
                .skip(1)
                .map(str::to_string)
                .collect::<Vec<_>>();
            let (owner, route) = match classify_yzx_root_route(&tail) {
                Ok(YzxPublicRootRoute::Help) => ("rust_root", "help"),
                Ok(YzxPublicRootRoute::Version) => ("rust_root", "version"),
                Ok(YzxPublicRootRoute::VersionFull) => ("rust_root", "version_full"),
                Ok(YzxPublicRootRoute::RustControl) => ("rust_control", "rust_control"),
                Err(_) => ("unknown", "unclassified"),
            };
            CommandOwnerEntry {
                command: metadata.name.to_string(),
                owner: owner.to_string(),
                route: route.to_string(),
                source: "rust_core/yazelix_core/src/public_command_surface.rs".to_string(),
            }
        })
        .collect()
}

fn config_owners() -> Vec<SurfaceOwnerEntry> {
    vec![
        SurfaceOwnerEntry {
            surface: "~/.config/yazelix/config.toml".to_string(),
            owner: "Rust yazelix_core config normalization and settings surface".to_string(),
            source: "config_metadata/main_config_contract.toml".to_string(),
            validation: vec!["yzx_repo_validator validate-config-surface-contract".to_string()],
        },
        SurfaceOwnerEntry {
            surface: "~/.config/yazelix/cursors.toml".to_string(),
            owner: "yazelix_cursors plus Yazelix terminal materialization".to_string(),
            source: "yazelix-cursors package contract".to_string(),
            validation: vec!["yzx_repo_validator validate-config-surface-contract".to_string()],
        },
        SurfaceOwnerEntry {
            surface: "programs.yazelix Home Manager options".to_string(),
            owner: "home_manager/module.nix".to_string(),
            source: "config_metadata/main_config_contract.toml".to_string(),
            validation: vec![
                "yzx_repo_validator validate-config-surface-contract".to_string(),
                "yzx_repo_validator validate-nix-customization-api".to_string(),
            ],
        },
    ]
}

fn generated_state_owners() -> Vec<SurfaceOwnerEntry> {
    vec![
        SurfaceOwnerEntry {
            surface: "~/.local/share/yazelix/configs/yazi".to_string(),
            owner: "Rust yazi_materialization".to_string(),
            source: "rust_core/yazelix_core/src/yazi_materialization.rs".to_string(),
            validation: vec!["yzx dev test".to_string()],
        },
        SurfaceOwnerEntry {
            surface: "~/.local/share/yazelix/configs/zellij".to_string(),
            owner: "Rust zellij_materialization".to_string(),
            source: "rust_core/yazelix_core/src/zellij_materialization.rs".to_string(),
            validation: vec![
                "yzx_repo_validator validate-workspace-session-contract".to_string(),
                "yzx dev test".to_string(),
            ],
        },
        SurfaceOwnerEntry {
            surface: "~/.local/share/yazelix/configs/terminal_emulators/ghostty".to_string(),
            owner: "Standalone Ghostty cursor materialization".to_string(),
            source: "rust_core/yazelix_core/src/terminal_cursor_materialization.rs".to_string(),
            validation: vec!["yzx dev test".to_string()],
        },
        SurfaceOwnerEntry {
            surface: "~/.local/share/yazelix/initializers".to_string(),
            owner: "Rust launch/setup preflight and initializer generation".to_string(),
            source: "rust_core/yazelix_core/src/launch_commands/enter.rs".to_string(),
            validation: vec![
                "yzx_repo_validator validate-nushell-syntax".to_string(),
                "yzx dev test".to_string(),
            ],
        },
    ]
}

fn runtime_tool_manifest(
    runtime_dir: &Path,
) -> Result<RuntimeManifestSection<RuntimeToolOwnerEntry>, CoreError> {
    let path = runtime_tools_manifest_path(runtime_dir);
    let Some(entries) = read_optional_runtime_tool_manifest(runtime_dir)? else {
        return missing_manifest(
            path,
            "runtime tool manifest is available in packaged runtimes",
        );
    };
    Ok(RuntimeManifestSection {
        status: "available".to_string(),
        path: path.to_string_lossy().to_string(),
        entries: entries
            .into_iter()
            .map(|(name, entry)| RuntimeToolOwnerEntry {
                name,
                source: entry.source,
                commands: entry.commands,
                required_commands: entry.required_commands,
                hostable: entry.hostable,
                disableable: entry.disableable,
                notes: entry.notes,
            })
            .collect(),
        note: None,
    })
}

fn runtime_component_manifest(
    runtime_dir: &Path,
) -> Result<RuntimeManifestSection<RuntimeComponentOwnerEntry>, CoreError> {
    let path = runtime_components_manifest_path(runtime_dir);
    let Some(entries) = read_optional_runtime_component_manifest(runtime_dir)? else {
        return missing_manifest(
            path,
            "runtime component manifest is available in packaged runtimes",
        );
    };
    Ok(RuntimeManifestSection {
        status: "available".to_string(),
        path: path.to_string_lossy().to_string(),
        entries: entries
            .into_iter()
            .map(|(name, entry)| RuntimeComponentOwnerEntry {
                name,
                enabled: entry.enabled,
                disableable: entry.disableable,
                notes: entry.notes,
            })
            .collect(),
        note: None,
    })
}

fn missing_manifest<T>(
    path: PathBuf,
    note: &'static str,
) -> Result<RuntimeManifestSection<T>, CoreError> {
    Ok(RuntimeManifestSection {
        status: "missing".to_string(),
        path: path.to_string_lossy().to_string(),
        entries: Vec::new(),
        note: Some(note.to_string()),
    })
}

fn validation_commands() -> Vec<ValidationCommandEntry> {
    [
        (
            "runtime control plane",
            "yzx dev test",
            "rust_core/yazelix_core",
        ),
        (
            "workspace session orchestration",
            "yzx_repo_validator validate-workspace-session-contract",
            "rust_core/yazelix_maintainer",
        ),
        (
            "distribution and host integration",
            "yzx_repo_validator validate-flake-interface",
            "rust_core/yazelix_maintainer",
        ),
        (
            "shipped runtime data and assets",
            "yzx_repo_validator validate-config-surface-contract",
            "rust_core/yazelix_maintainer",
        ),
        (
            "maintainer workflow and validation",
            "yzx_repo_validator validate-rust-test-traceability",
            "rust_core/yazelix_maintainer",
        ),
    ]
    .into_iter()
    .map(|(subsystem, command, owner)| ValidationCommandEntry {
        subsystem: subsystem.to_string(),
        command: command.to_string(),
        owner: owner.to_string(),
    })
    .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    // Defends: the ownership graph exposes public command routing from the Rust command metadata source.
    #[test]
    fn command_owners_include_rust_and_nu_routes() {
        let graph = compute_runtime_ownership_graph(&RuntimeOwnershipGraphRequest {
            runtime_dir: PathBuf::from("/missing-runtime-manifests"),
        })
        .unwrap();

        assert!(graph.command_owners.iter().any(|entry| {
            entry.command == "yzx launch"
                && entry.owner == "rust_control"
                && entry.route == "rust_control"
        }));
        assert!(graph.command_owners.iter().any(|entry| {
            entry.command == "yzx menu"
                && entry.owner == "rust_control"
                && entry.route == "rust_control"
        }));
    }

    // Defends: packaged runtime tool/component manifests feed the graph instead of a hand-maintained docs table.
    #[test]
    fn graph_reads_packaged_runtime_manifests_when_present() {
        let tmp = TempDir::new().unwrap();
        fs::write(
            tmp.path().join("runtime_tools.json"),
            r#"{
              "yazi": {
                "source": "host",
                "commands": ["yazi", "ya"],
                "required_commands": ["yazi"],
                "hostable": true,
                "disableable": false,
                "notes": []
              }
            }"#,
        )
        .unwrap();
        fs::write(
            tmp.path().join("runtime_components.json"),
            r#"{
              "cursors": {
                "enabled": false,
                "disableable": true,
                "notes": ["test component"]
              }
            }"#,
        )
        .unwrap();

        let graph = compute_runtime_ownership_graph(&RuntimeOwnershipGraphRequest {
            runtime_dir: tmp.path().to_path_buf(),
        })
        .unwrap();

        assert_eq!(graph.runtime_tools.status, "available");
        assert_eq!(graph.runtime_tools.entries[0].name, "yazi");
        assert_eq!(graph.runtime_tools.entries[0].source, "host");
        assert_eq!(graph.runtime_components.status, "available");
        assert_eq!(graph.runtime_components.entries[0].name, "cursors");
        assert!(!graph.runtime_components.entries[0].enabled);
    }
}
