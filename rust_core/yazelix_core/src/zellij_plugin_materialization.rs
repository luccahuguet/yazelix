// Test lane: maintainer

use crate::bridge::{CoreError, ErrorClass};
use crate::zellij_materialization_io::{
    hash_file, read_text, write_bytes_atomic, write_text_atomic,
};
use directories::ProjectDirs;
use serde_json::json;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

const PANE_ORCHESTRATOR_REQUIRED_PERMISSIONS: &[&str] = &[
    "ReadApplicationState",
    "OpenTerminalsOrPlugins",
    "ChangeApplicationState",
    "RunCommands",
    "WriteToStdin",
    "ReadCliPipes",
    "MessageAndLaunchOtherPlugins",
    "ReadSessionEnvironmentVariables",
];
const YZPP_REQUIRED_PERMISSIONS: &[&str] = &[
    "ReadApplicationState",
    "ChangeApplicationState",
    "OpenTerminalsOrPlugins",
    "RunCommands",
    "ReadCliPipes",
];

#[derive(Debug, Clone)]
pub(crate) struct PluginArtifact {
    pub(crate) name: &'static str,
    pub(crate) wasm_name: &'static str,
    pub(crate) tracked_path: PathBuf,
    pub(crate) tracked_hash: String,
    pub(crate) runtime_path: PathBuf,
    pub(crate) required_permissions: &'static [&'static str],
}

#[derive(Debug, Clone)]
struct PermissionBlock {
    path: String,
    permissions: Vec<String>,
}

pub(crate) fn resolve_plugin_artifacts(
    runtime_dir: &Path,
    state_dir: &Path,
) -> Result<[PluginArtifact; 3], CoreError> {
    let plugin_dir = state_dir.join("configs").join("zellij").join("plugins");
    Ok([
        resolve_plugin_artifact(
            runtime_dir,
            &plugin_dir,
            "pane_orchestrator",
            "yazelix_pane_orchestrator.wasm",
            PANE_ORCHESTRATOR_REQUIRED_PERMISSIONS,
        )?,
        resolve_plugin_artifact(
            runtime_dir,
            &plugin_dir,
            "zjstatus",
            "zjstatus.wasm",
            &[
                "ReadApplicationState",
                "ChangeApplicationState",
                "RunCommands",
            ],
        )?,
        resolve_plugin_artifact(
            runtime_dir,
            &plugin_dir,
            "yzpp",
            "yzpp.wasm",
            YZPP_REQUIRED_PERMISSIONS,
        )?,
    ])
}

fn resolve_plugin_artifact(
    runtime_dir: &Path,
    plugin_dir: &Path,
    name: &'static str,
    wasm_name: &'static str,
    required_permissions: &'static [&'static str],
) -> Result<PluginArtifact, CoreError> {
    let tracked_path = runtime_dir
        .join("configs")
        .join("zellij")
        .join("plugins")
        .join(wasm_name);
    if !tracked_path.exists() {
        return Err(CoreError::classified(
            ErrorClass::Io,
            "missing_tracked_zellij_plugin",
            format!(
                "Tracked {name} wasm not found at: {}",
                tracked_path.to_string_lossy()
            ),
            "Reinstall Yazelix so the runtime includes all tracked Zellij plugin wasm artifacts.",
            json!({ "path": tracked_path.to_string_lossy(), "plugin": name }),
        ));
    }
    Ok(PluginArtifact {
        name,
        wasm_name,
        tracked_hash: hash_file(&tracked_path)?,
        runtime_path: plugin_dir.join(wasm_name),
        tracked_path,
        required_permissions,
    })
}

pub(crate) fn sync_plugin_artifacts(
    plugin_artifacts: &[PluginArtifact; 3],
    seed_plugin_permissions: bool,
) -> Result<(), CoreError> {
    for artifact in plugin_artifacts.iter() {
        sync_plugin_artifact(artifact)?;
        let prefix = artifact
            .wasm_name
            .strip_suffix(".wasm")
            .unwrap_or(artifact.wasm_name);
        let plugin_dir = artifact
            .runtime_path
            .parent()
            .expect("plugin path has parent");
        remove_runtime_plugins_by_prefix_in_dir(plugin_dir, prefix, Some(&artifact.runtime_path))?;
        preserve_plugin_permissions(
            prefix,
            &artifact.tracked_path,
            &artifact.runtime_path,
            artifact.required_permissions,
        )?;
    }
    if seed_plugin_permissions {
        upsert_plugin_permission_blocks(plugin_artifacts)?;
    }
    Ok(())
}

fn sync_plugin_artifact(artifact: &PluginArtifact) -> Result<(), CoreError> {
    if artifact.runtime_path.exists() && hash_file(&artifact.runtime_path)? == artifact.tracked_hash
    {
        return Ok(());
    }

    copy_file_atomic(&artifact.tracked_path, &artifact.runtime_path)
}

fn remove_runtime_plugins_by_prefix_in_dir(
    runtime_dir: &Path,
    prefix: &str,
    excluded_path: Option<&Path>,
) -> Result<(), CoreError> {
    if !runtime_dir.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(runtime_dir).map_err(|source| {
        CoreError::io(
            "read_zellij_plugin_runtime_dir",
            "Could not inspect the managed Zellij plugin directory",
            "Check permissions for the Yazelix state directory and retry.",
            runtime_dir.to_string_lossy(),
            source,
        )
    })? {
        let path = entry
            .map_err(|source| {
                CoreError::io(
                    "read_zellij_plugin_runtime_entry",
                    "Could not inspect a managed Zellij plugin entry",
                    "Check permissions for the Yazelix state directory and retry.",
                    runtime_dir.to_string_lossy(),
                    source,
                )
            })?
            .path();
        if excluded_path.is_some_and(|excluded| excluded == path.as_path()) {
            continue;
        }
        let file_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("");
        if plugin_name_matches_prefix(file_name, prefix) {
            fs::remove_file(&path).map_err(|source| {
                CoreError::io(
                    "remove_stale_zellij_plugin",
                    "Could not remove a stale managed Zellij plugin",
                    "Check permissions for the Yazelix state directory and retry.",
                    path.to_string_lossy(),
                    source,
                )
            })?;
        }
    }
    Ok(())
}

fn plugin_name_matches_prefix(file_name: &str, prefix: &str) -> bool {
    file_name == format!("{prefix}.wasm")
        || (file_name.starts_with(&format!("{prefix}_")) && file_name.ends_with(".wasm"))
}

fn preserve_plugin_permissions(
    prefix: &str,
    tracked_path: &Path,
    runtime_path: &Path,
    required_permissions: &[&str],
) -> Result<(), CoreError> {
    let permissions_cache_path = zellij_permissions_cache_path()?;
    if !permissions_cache_path.exists() {
        return Ok(());
    }
    let blocks = parse_permission_blocks(&read_text(
        &permissions_cache_path,
        "read_zellij_permissions_cache",
    )?);
    if !blocks
        .iter()
        .any(|block| plugin_name_matches_prefix(path_basename(&block.path), prefix))
    {
        return Ok(());
    }
    let mut retained = blocks
        .into_iter()
        .filter(|block| !plugin_name_matches_prefix(path_basename(&block.path), prefix))
        .map(|block| build_permission_block(&block.path, &block.permissions))
        .collect::<Vec<_>>();
    retained.extend(required_permission_blocks(
        tracked_path,
        runtime_path,
        required_permissions,
    ));
    write_text_atomic(&permissions_cache_path, &retained.join("\n\n"))?;
    Ok(())
}

fn parse_permission_blocks(content: &str) -> Vec<PermissionBlock> {
    let mut blocks = Vec::new();
    let mut current_path: Option<String> = None;
    let mut current_permissions = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if current_path.is_none() {
            if trimmed.ends_with('{')
                && let Some(path) = trimmed
                    .strip_prefix('"')
                    .and_then(|rest| rest.split('"').next())
            {
                current_path = Some(path.to_string());
            }
            continue;
        }
        if trimmed == "}"
            && let Some(path) = current_path.take()
        {
            blocks.push(PermissionBlock {
                path,
                permissions: std::mem::take(&mut current_permissions),
            });
            continue;
        }
        if !trimmed.is_empty() {
            current_permissions.push(trimmed.to_string());
        }
    }
    blocks
}

fn build_permission_block(plugin_path: &str, permissions: &[String]) -> String {
    let mut block = format!("\"{plugin_path}\" {{");
    for permission in permissions {
        block.push_str(&format!("\n    {permission}"));
    }
    block.push_str("\n}");
    block
}

fn upsert_plugin_permission_blocks(
    plugin_artifacts: &[PluginArtifact; 3],
) -> Result<(), CoreError> {
    let permissions_cache_path = zellij_permissions_cache_path()?;
    let existing_blocks = if permissions_cache_path.exists() {
        parse_permission_blocks(&read_text(
            &permissions_cache_path,
            "read_zellij_permissions_cache",
        )?)
    } else {
        Vec::new()
    };
    let managed_prefixes = plugin_artifacts
        .iter()
        .map(|artifact| {
            artifact
                .wasm_name
                .strip_suffix(".wasm")
                .unwrap_or(artifact.wasm_name)
        })
        .collect::<BTreeSet<_>>();
    let mut updated = existing_blocks
        .into_iter()
        .filter(|block| {
            !managed_prefixes
                .iter()
                .any(|prefix| plugin_name_matches_prefix(path_basename(&block.path), prefix))
        })
        .map(|block| build_permission_block(&block.path, &block.permissions))
        .collect::<Vec<_>>();

    for artifact in plugin_artifacts.iter() {
        updated.extend(required_permission_blocks(
            &artifact.tracked_path,
            &artifact.runtime_path,
            artifact.required_permissions,
        ));
    }

    write_text_atomic(&permissions_cache_path, &updated.join("\n\n"))
}

fn required_permission_blocks(
    tracked_path: &Path,
    runtime_path: &Path,
    required_permissions: &[&str],
) -> [String; 2] {
    let permissions = required_permissions
        .iter()
        .map(|permission| (*permission).to_string())
        .collect::<Vec<_>>();
    [
        build_permission_block(&tracked_path.to_string_lossy(), &permissions),
        build_permission_block(&runtime_path.to_string_lossy(), &permissions),
    ]
}

fn copy_file_atomic(source: &Path, target: &Path) -> Result<(), CoreError> {
    let bytes = fs::read(source).map_err(|source_err| {
        CoreError::io(
            "read_zellij_plugin_source",
            "Could not read tracked Zellij plugin artifact",
            "Reinstall Yazelix so the runtime includes readable Zellij plugin artifacts.",
            source.to_string_lossy(),
            source_err,
        )
    })?;
    write_bytes_atomic(target, &bytes)
}

fn path_basename(path: &str) -> &str {
    path.rsplit('/').next().unwrap_or(path)
}

pub(crate) fn zellij_permissions_cache_path() -> Result<PathBuf, CoreError> {
    ProjectDirs::from("org", "Zellij Contributors", "Zellij")
        .map(|dirs| dirs.cache_dir().join("permissions.kdl"))
        .ok_or_else(|| {
            CoreError::classified(
                ErrorClass::Runtime,
                "resolve_zellij_permissions_cache",
                "Could not resolve Zellij's plugin permission cache directory.",
                "Ensure HOME is set, then retry.",
                json!({}),
            )
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    // Regression: macOS Zellij reads plugin permissions from ProjectDirs' Library/Caches path, not ~/.cache/zellij.
    #[cfg(target_os = "macos")]
    #[test]
    fn zellij_permissions_cache_path_uses_macos_cache_location() {
        assert!(
            zellij_permissions_cache_path()
                .unwrap()
                .to_string_lossy()
                .ends_with("/Library/Caches/org.Zellij-Contributors.Zellij/permissions.kdl")
        );
    }

    // Regression: legacy plugin permission blocks are recognized by both stable and hashed wasm names.
    #[test]
    fn plugin_prefix_matches_stable_and_hashed_names() {
        for name in ["zjstatus.wasm", "zjstatus_abc123.wasm"] {
            assert!(plugin_name_matches_prefix(name, "zjstatus"));
        }
        assert!(!plugin_name_matches_prefix("not_zjstatus.wasm", "zjstatus"));
    }
}
