//! User-facing workspace asset drift checks for `yzx doctor`.

use crate::zellij_materialization::{
    generated_zellij_config_has_yazelix_markers, generated_zellij_layout_has_yazelix_markers,
};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use yazelix_zellij_config_pack as zellij_config_pack;

const ZELLIJ_PLUGIN_WASMS: &[&str] = &[
    "yazelix_pane_orchestrator.wasm",
    "zjstatus.wasm",
    "yzpp.wasm",
];
const RUNTIME_WORKSPACE_ASSETS: &[&str] = &[
    "configs/zellij/yazelix_overrides.kdl",
    "configs/zellij/plugins/yazelix_pane_orchestrator.wasm",
    "configs/zellij/plugins/zjstatus.wasm",
    "configs/zellij/plugins/yzpp.wasm",
];
const REPO_WORKSPACE_ASSETS: &[&str] = &["configs/zellij/yazelix_overrides.kdl"];

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkspaceAssetEvaluateRequest {
    pub runtime_dir: PathBuf,
    pub state_dir: PathBuf,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct WorkspaceAssetFinding {
    pub status: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
    pub fix_available: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fix_action: Option<String>,
    pub owner_surface: String,
    pub workspace_asset_check: String,
}

pub fn evaluate_workspace_asset_report(
    request: &WorkspaceAssetEvaluateRequest,
) -> Vec<WorkspaceAssetFinding> {
    vec![
        runtime_workspace_assets_finding(&request.runtime_dir),
        generated_workspace_state_finding(&request.runtime_dir, &request.state_dir),
    ]
}

pub fn validate_workspace_assets_for_repo(repo_root: &Path) -> Result<Vec<String>, String> {
    let mut errors = Vec::new();
    errors.extend(
        missing_workspace_assets(repo_root, REPO_WORKSPACE_ASSETS)
            .into_iter()
            .map(|path| {
                format!(
                    "Missing tracked workspace runtime asset from repo/runtime source: {}",
                    path.display()
                )
            }),
    );
    Ok(errors)
}

fn runtime_workspace_assets_finding(runtime_dir: &Path) -> WorkspaceAssetFinding {
    let missing = missing_workspace_assets(runtime_dir, RUNTIME_WORKSPACE_ASSETS);
    if missing.is_empty() {
        return WorkspaceAssetFinding {
            status: "ok".into(),
            message: "Workspace runtime assets are present".into(),
            details: Some(
                "Zellij overrides and plugin artifacts are available in the active runtime.".into(),
            ),
            fix_available: false,
            fix_action: None,
            owner_surface: "doctor".into(),
            workspace_asset_check: "runtime_workspace_assets".into(),
        };
    }

    WorkspaceAssetFinding {
        status: "error".into(),
        message: "Workspace runtime assets are missing from the active runtime".into(),
        details: Some(
            missing
                .into_iter()
                .map(|path| format!("missing: {}", path.display()))
                .collect::<Vec<_>>()
                .join("\n"),
        ),
        fix_available: false,
        fix_action: None,
        owner_surface: "doctor".into(),
        workspace_asset_check: "runtime_workspace_assets".into(),
    }
}

fn generated_workspace_state_finding(
    runtime_dir: &Path,
    state_dir: &Path,
) -> WorkspaceAssetFinding {
    let mut issues = Vec::new();
    let zellij_state_dir = state_dir.join("configs").join("zellij");
    let generated_config = zellij_state_dir.join("config.kdl");
    let generation_metadata = zellij_state_dir.join(".yazelix_generation.json");
    let generation_fingerprint = generated_zellij_generation_fingerprint(&generation_metadata);
    if !generated_config.is_file() {
        issues.push(format!(
            "missing generated Zellij config: {}",
            generated_config.display()
        ));
    } else {
        match generated_zellij_config_has_yazelix_markers(&generated_config) {
            Ok(true) => {}
            Ok(false) => issues.push(format!(
                "invalid generated Zellij config missing Yazelix overlay markers: {}",
                generated_config.display()
            )),
            Err(error) => issues.push(format!(
                "could not validate generated Zellij config: {}",
                error.message()
            )),
        }
    }
    match &generation_fingerprint {
        Ok(Some(_)) => {}
        Ok(None) => issues.push(format!(
            "missing Zellij generation fingerprint: {}",
            generation_metadata.display()
        )),
        Err(error) => issues.push(error.clone()),
    }

    let generated_layouts_dir = zellij_state_dir.join("layouts");
    for layout in expected_zellij_generated_layout_files() {
        let generated = generated_layouts_dir.join(&layout);
        if !generated.is_file() {
            issues.push(format!(
                "missing generated Zellij layout: {}",
                generated.display()
            ));
            continue;
        }
        match generated_zellij_layout_has_yazelix_markers(
            &generated,
            generation_fingerprint.as_ref().ok().and_then(Option::as_deref),
        ) {
            Ok(true) => {}
            Ok(false) => issues.push(format!(
                "invalid generated Zellij layout missing Yazelix generation markers or current fingerprint: {}",
                generated.display()
            )),
            Err(error) => issues.push(format!(
                "could not validate generated Zellij layout: {}",
                error.message()
            )),
        }
    }

    for wasm_name in ZELLIJ_PLUGIN_WASMS {
        let tracked = runtime_dir
            .join("configs")
            .join("zellij")
            .join("plugins")
            .join(wasm_name);
        let generated = zellij_state_dir.join("plugins").join(wasm_name);
        if !generated.is_file() {
            issues.push(format!(
                "missing generated Zellij plugin artifact: {}",
                generated.display()
            ));
            continue;
        }
        match (file_sha256_hex(&tracked), file_sha256_hex(&generated)) {
            (Ok(tracked_hash), Ok(generated_hash)) if tracked_hash == generated_hash => {}
            (Ok(tracked_hash), Ok(generated_hash)) => issues.push(format!(
                "generated Zellij plugin artifact is stale: {} (runtime sha256 {}, generated sha256 {})",
                generated.display(),
                tracked_hash,
                generated_hash
            )),
            (Err(error), _) | (_, Err(error)) => issues.push(error),
        }
    }

    if issues.is_empty() {
        return WorkspaceAssetFinding {
            status: "ok".into(),
            message: "Generated workspace assets match the active runtime".into(),
            details: Some(
                "Generated Zellij config, layouts, and plugin artifacts are present and fresh."
                    .into(),
            ),
            fix_available: false,
            fix_action: None,
            owner_surface: "doctor".into(),
            workspace_asset_check: "generated_workspace_assets".into(),
        };
    }

    WorkspaceAssetFinding {
        status: "error".into(),
        message: "Generated workspace assets are missing or stale".into(),
        details: Some(issues.join("\n")),
        fix_available: true,
        fix_action: Some("repair_generated_runtime_state".into()),
        owner_surface: "doctor".into(),
        workspace_asset_check: "generated_workspace_assets".into(),
    }
}

fn generated_zellij_generation_fingerprint(metadata_path: &Path) -> Result<Option<String>, String> {
    if !metadata_path.is_file() {
        return Ok(None);
    }
    let raw = fs::read_to_string(metadata_path).map_err(|error| {
        format!(
            "could not read Zellij generation fingerprint: {} ({error})",
            metadata_path.display()
        )
    })?;
    let parsed = serde_json::from_str::<JsonValue>(&raw).map_err(|error| {
        format!(
            "could not parse Zellij generation fingerprint: {} ({error})",
            metadata_path.display()
        )
    })?;
    let fingerprint = parsed
        .get("fingerprint")
        .and_then(JsonValue::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned);
    Ok(fingerprint)
}

fn missing_workspace_assets(root: &Path, assets: &[&str]) -> Vec<PathBuf> {
    assets
        .iter()
        .map(|relative| {
            relative
                .split('/')
                .fold(root.to_path_buf(), |path, segment| path.join(segment))
        })
        .filter(|path| !path.is_file())
        .collect()
}

fn expected_zellij_generated_layout_files() -> BTreeSet<String> {
    zellij_config_pack::bundled_layout_templates()
        .into_iter()
        .map(|template| template.relative_path)
        .collect()
}

fn file_sha256_hex(path: &Path) -> Result<String, String> {
    let bytes =
        fs::read(path).map_err(|error| format!("failed to read {}: {error}", path.display()))?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}

// Test lane: default
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn write_workspace_fixture() -> (tempfile::TempDir, PathBuf, PathBuf) {
        let tmp = tempdir().unwrap();
        let root = tmp.path();
        let runtime = root.join("runtime");
        let state = root.join("state");
        let runtime_plugins = runtime.join("configs").join("zellij").join("plugins");
        let state_zellij = state.join("configs").join("zellij");
        fs::create_dir_all(&runtime_plugins).unwrap();
        fs::create_dir_all(state_zellij.join("layouts")).unwrap();
        fs::create_dir_all(state_zellij.join("plugins")).unwrap();
        fs::write(
            runtime
                .join("configs")
                .join("zellij")
                .join("yazelix_overrides.kdl"),
            "",
        )
        .unwrap();
        for wasm in ZELLIJ_PLUGIN_WASMS {
            fs::write(runtime_plugins.join(wasm), format!("{wasm}-bytes")).unwrap();
            fs::write(
                state_zellij.join("plugins").join(wasm),
                format!("{wasm}-bytes"),
            )
            .unwrap();
        }
        fs::write(
            state_zellij.join("config.kdl"),
            "GENERATED ZELLIJ CONFIG (YAZELIX)\nyazelix_pane_orchestrator\nyzpp\n",
        )
        .unwrap();
        fs::write(
            state_zellij.join(".yazelix_generation.json"),
            r#"{"fingerprint":"fresh-fingerprint"}"#,
        )
        .unwrap();
        let generated_layout = "// GENERATED ZELLIJ LAYOUT (YAZELIX)\n// generation_fingerprint: fresh-fingerprint\nlayout {}\n";
        fs::write(
            state_zellij.join("layouts").join("yzx_side.kdl"),
            generated_layout,
        )
        .unwrap();
        fs::write(
            state_zellij.join("layouts").join("yzx_side.swap.kdl"),
            generated_layout,
        )
        .unwrap();
        (tmp, runtime, state)
    }

    // Defends: doctor can report a healthy generated workspace asset surface without invoking Zellij or Nix.
    #[test]
    fn workspace_asset_report_accepts_fresh_generated_state() {
        let (_tmp, runtime, state) = write_workspace_fixture();
        let findings = evaluate_workspace_asset_report(&WorkspaceAssetEvaluateRequest {
            runtime_dir: runtime,
            state_dir: state,
        });

        assert!(findings.iter().all(|finding| finding.status == "ok"));
    }

    // Regression: stale generated plugin artifacts should become a fixable doctor finding instead of a mystery runtime failure.
    #[test]
    fn workspace_asset_report_flags_stale_generated_plugin_as_fixable() {
        let (_tmp, runtime, state) = write_workspace_fixture();
        fs::write(
            state
                .join("configs")
                .join("zellij")
                .join("plugins")
                .join("yazelix_pane_orchestrator.wasm"),
            "stale",
        )
        .unwrap();

        let findings = evaluate_workspace_asset_report(&WorkspaceAssetEvaluateRequest {
            runtime_dir: runtime,
            state_dir: state,
        });
        let generated = findings
            .iter()
            .find(|finding| finding.workspace_asset_check == "generated_workspace_assets")
            .unwrap();

        assert_eq!(generated.status, "error");
        assert!(generated.fix_available);
        assert_eq!(
            generated.fix_action.as_deref(),
            Some("repair_generated_runtime_state")
        );
        assert!(generated.details.as_ref().unwrap().contains("stale"));
    }

    // Regression: doctor should catch a native config copied into the generated Zellij config path.
    #[test]
    fn workspace_asset_report_flags_plain_generated_zellij_config_as_fixable() {
        let (_tmp, runtime, state) = write_workspace_fixture();
        fs::write(
            state.join("configs").join("zellij").join("config.kdl"),
            "keybinds clear-defaults=true {\n    normal {}\n}\n",
        )
        .unwrap();

        let findings = evaluate_workspace_asset_report(&WorkspaceAssetEvaluateRequest {
            runtime_dir: runtime,
            state_dir: state,
        });
        let generated = findings
            .iter()
            .find(|finding| finding.workspace_asset_check == "generated_workspace_assets")
            .unwrap();

        assert_eq!(generated.status, "error");
        assert!(generated.fix_available);
        assert_eq!(
            generated.fix_action.as_deref(),
            Some("repair_generated_runtime_state")
        );
        assert!(
            generated
                .details
                .as_ref()
                .unwrap()
                .contains("overlay markers")
        );
    }

    // Regression: doctor should catch stale generated Zellij layouts instead of accepting existing files blindly.
    #[test]
    fn workspace_asset_report_flags_stale_generated_zellij_layout_as_fixable() {
        let (_tmp, runtime, state) = write_workspace_fixture();
        fs::write(
            state
                .join("configs")
                .join("zellij")
                .join("layouts")
                .join("yzx_side.kdl"),
            "// GENERATED ZELLIJ LAYOUT (YAZELIX)\n// generation_fingerprint: stale-fingerprint\nlayout {}\n",
        )
        .unwrap();

        let findings = evaluate_workspace_asset_report(&WorkspaceAssetEvaluateRequest {
            runtime_dir: runtime,
            state_dir: state,
        });
        let generated = findings
            .iter()
            .find(|finding| finding.workspace_asset_check == "generated_workspace_assets")
            .unwrap();

        assert_eq!(generated.status, "error");
        assert!(generated.fix_available);
        assert_eq!(
            generated.fix_action.as_deref(),
            Some("repair_generated_runtime_state")
        );
        assert!(
            generated
                .details
                .as_ref()
                .unwrap()
                .contains("invalid generated Zellij layout")
        );
    }
}
