//! Maintainer validation for coupled child-repo release transactions.

use crate::repo_validation::ValidationReport;
use serde_json::Value as JsonValue;
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, PartialEq, Eq)]
struct ChildInputLock {
    node: String,
    owner: String,
    repo: String,
    rev: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ZellijPluginWasmPackageContract {
    input_node: &'static str,
    package_attr: &'static str,
    system: &'static str,
    plugin_name: &'static str,
    wasm_path: &'static str,
}

type JsonObject = serde_json::Map<String, JsonValue>;

const ZELLIJ_PLUGIN_WASM_PACKAGE_CONTRACTS: &[ZellijPluginWasmPackageContract] = &[
    ZellijPluginWasmPackageContract {
        input_node: "yazelixZellijPaneOrchestrator",
        package_attr: "yazelix_zellij_pane_orchestrator",
        system: "aarch64-darwin",
        plugin_name: "yazelix-zellij-pane-orchestrator",
        wasm_path: "share/yazelix_zellij_pane_orchestrator/yazelix_pane_orchestrator.wasm",
    },
    ZellijPluginWasmPackageContract {
        input_node: "yazelixZellijPopup",
        package_attr: "yazelix_zellij_popup",
        system: "aarch64-darwin",
        plugin_name: "yazelix-zellij-popup",
        wasm_path: "share/yazelix_zellij_popup/yzpp.wasm",
    },
];

const ZELLIJ_PLUGIN_WASM_METADATA_FIELDS: &[&str] = &[
    "schemaVersion",
    "pluginName",
    "packageAttr",
    "wasmPath",
    "wasmTarget",
    "cargoAuditableDisabled",
    "cargoBuildHookDisabled",
    "preBuildPreservesNixRustToolchain",
    "wasmTargetRustcEnvPinned",
    "cargoBuildSerialized",
    "installCheckVerifiesWasm",
];

const CURSOR_PACKAGE_METADATA_FIELDS: &[&str] = &[
    "schemaVersion",
    "packageName",
    "shareRoot",
    "defaultConfig",
    "shaderRoot",
    "generatedEffectRoot",
    "requiredTargets",
    "requiredShaderFiles",
    "forbiddenShaderFiles",
];

const CURSOR_PACKAGE_REQUIRED_TARGETS: &[&str] = &[
    "ghostty",
    "mars",
    "rio",
    "ratty",
    "protocol_cursor_positions",
];

const CURSOR_PACKAGE_REQUIRED_SHADER_FILES: &[&str] = &[
    "cursor_trail_common.glsl",
    "cursor_trail_reef.glsl",
    "upstream_effects/ripple_rectangle_cursor.glsl",
    "generated_effects/tail.glsl",
];

pub fn validate_child_release_transaction(repo_root: &Path) -> Result<ValidationReport, String> {
    let mut report = ValidationReport::default();
    let lock_path = repo_root.join("flake.lock");
    let raw = fs::read_to_string(&lock_path)
        .map_err(|error| format!("Failed to read {}: {error}", lock_path.display()))?;
    let inputs = locked_child_inputs(&raw)?;
    if inputs.is_empty() {
        report
            .errors
            .push("flake.lock contains no first-party Yazelix child inputs".to_string());
        return Ok(report);
    }

    for input in &inputs {
        if let Some(warning) = local_child_checkout_warning(repo_root, input)? {
            report.warnings.push(warning);
        }
        if !remote_rev_is_fetchable(input)? {
            report.errors.push(format!(
                "Child input `{}` pins unpublished or unreachable commit {} for {}/{}. Push the child repo first, update flake.lock to the published revision, then run no-overrides validation.",
                input.node, input.rev, input.owner, input.repo
            ));
        }
    }

    report
        .errors
        .extend(validate_zellij_plugin_wasm_package_contracts(
            repo_root, &inputs,
        )?);
    report
        .errors
        .extend(validate_cursor_package_contracts(repo_root, &inputs)?);
    report
        .errors
        .extend(validate_main_cursor_shader_tree_deleted(repo_root));

    Ok(report)
}

fn locked_child_inputs(raw_lock: &str) -> Result<Vec<ChildInputLock>, String> {
    let parsed: JsonValue = serde_json::from_str(raw_lock)
        .map_err(|error| format!("Invalid flake.lock JSON: {error}"))?;
    let nodes = parsed
        .get("nodes")
        .and_then(JsonValue::as_object)
        .ok_or_else(|| "flake.lock is missing object `nodes`".to_string())?;
    let mut inputs = Vec::new();
    for (node, data) in nodes {
        let Some(locked) = data.get("locked").and_then(JsonValue::as_object) else {
            continue;
        };
        if locked.get("type").and_then(JsonValue::as_str) != Some("github") {
            continue;
        }
        let owner = locked
            .get("owner")
            .and_then(JsonValue::as_str)
            .unwrap_or("");
        let repo = locked.get("repo").and_then(JsonValue::as_str).unwrap_or("");
        let rev = locked.get("rev").and_then(JsonValue::as_str).unwrap_or("");
        if owner != "luccahuguet" || !repo.starts_with("yazelix-") || rev.is_empty() {
            continue;
        }
        inputs.push(ChildInputLock {
            node: node.to_string(),
            owner: owner.to_string(),
            repo: repo.to_string(),
            rev: rev.to_string(),
        });
    }
    inputs.sort_by(|left, right| left.node.cmp(&right.node));
    Ok(inputs)
}

fn local_child_checkout_warning(
    repo_root: &Path,
    input: &ChildInputLock,
) -> Result<Option<String>, String> {
    let Some(parent) = repo_root.parent() else {
        return Ok(None);
    };
    let checkout = parent.join(&input.repo);
    if !checkout.join(".git").exists() {
        return Ok(None);
    }

    let status = Command::new("git")
        .args([
            "-C",
            checkout.to_string_lossy().as_ref(),
            "status",
            "--short",
        ])
        .output()
        .map_err(|error| {
            format!(
                "Failed to inspect local child checkout {}: {error}",
                checkout.display()
            )
        })?;
    if !status.status.success() {
        return Err(format!(
            "Failed to inspect local child checkout {}\n{}",
            checkout.display(),
            String::from_utf8_lossy(&status.stderr).trim()
        ));
    }
    let dirty = String::from_utf8_lossy(&status.stdout).trim().to_string();
    if dirty.is_empty() {
        return Ok(None);
    }
    Ok(Some(format!(
        "Local child checkout {} has uncommitted changes; finish or stash them before running a coupled release transaction.",
        checkout.display()
    )))
}

fn remote_rev_is_fetchable(input: &ChildInputLock) -> Result<bool, String> {
    let url = format!("https://github.com/{}/{}.git", input.owner, input.repo);
    let probe_dir = remote_rev_probe_dir(input)?;
    let init = Command::new("git")
        .args(["init", "--bare"])
        .arg(&probe_dir)
        .output()
        .map_err(|error| format!("Failed to initialize {}: {error}", probe_dir.display()))?;
    if !init.status.success() {
        return Err(format!(
            "Failed to initialize {}\n{}",
            probe_dir.display(),
            String::from_utf8_lossy(&init.stderr).trim()
        ));
    }

    let fetch = Command::new("git")
        .arg("-C")
        .arg(&probe_dir)
        .args(["fetch", "--depth=1", &url, &input.rev])
        .output()
        .map_err(|error| {
            format!(
                "Failed to run `git fetch --depth=1 {url} {}`: {error}",
                input.rev
            )
        });
    let cleanup = remove_probe_dir(&probe_dir);

    let fetch = fetch?;
    cleanup?;
    if fetch.status.success() {
        return Ok(true);
    }

    let stderr = String::from_utf8_lossy(&fetch.stderr);
    if fetch_failure_means_missing_revision(&stderr) {
        return Ok(false);
    }

    Err(format!(
        "Failed to fetch locked revision {} from {url}\n{}",
        input.rev,
        stderr.trim()
    ))
}

fn remove_probe_dir(probe_dir: &Path) -> Result<(), String> {
    let mut last_error = None;
    for delay_ms in [0, 50, 100, 250, 500] {
        if delay_ms > 0 {
            thread::sleep(Duration::from_millis(delay_ms));
        }

        match fs::remove_dir_all(probe_dir) {
            Ok(()) => return Ok(()),
            Err(error) if error.kind() == ErrorKind::NotFound => return Ok(()),
            Err(error) => last_error = Some(error.to_string()),
        }
    }

    Err(format!(
        "Failed to remove {}: {}",
        probe_dir.display(),
        last_error.unwrap_or_else(|| "unknown cleanup error".to_string())
    ))
}

fn remote_rev_probe_dir(input: &ChildInputLock) -> Result<PathBuf, String> {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| format!("System clock is before UNIX_EPOCH: {error}"))?
        .as_nanos();
    Ok(std::env::temp_dir().join(format!(
        "yazelix_child_release_probe_{}_{}_{}",
        std::process::id(),
        stamp,
        input.repo
    )))
}

fn fetch_failure_means_missing_revision(stderr: &str) -> bool {
    stderr.contains("not our ref")
        || stderr.contains("couldn't find remote ref")
        || stderr.contains("Server does not allow request for unadvertised object")
}

fn validate_zellij_plugin_wasm_package_contracts(
    repo_root: &Path,
    inputs: &[ChildInputLock],
) -> Result<Vec<String>, String> {
    let mut errors = Vec::new();
    for contract in ZELLIJ_PLUGIN_WASM_PACKAGE_CONTRACTS {
        let input = locked_child_input(inputs, contract.input_node)?;
        let metadata =
            package_passthru_metadata(repo_root, input, contract.system, contract.package_attr)?;
        errors.extend(validate_zellij_plugin_wasm_package_contract_with(
            contract, &metadata,
        )?);
    }
    Ok(errors)
}

fn validate_cursor_package_contracts(
    repo_root: &Path,
    inputs: &[ChildInputLock],
) -> Result<Vec<String>, String> {
    let mut errors = Vec::new();
    let input = locked_child_input(inputs, "yazelixCursors")?;
    for system in ["x86_64-linux", "aarch64-darwin"] {
        let metadata = package_passthru_metadata(repo_root, input, system, "yazelix_cursors")?;
        errors.extend(validate_cursor_package_contract_with(system, &metadata)?);
    }
    Ok(errors)
}

fn validate_cursor_package_contract_with(
    system: &str,
    raw_passthru: &str,
) -> Result<Vec<String>, String> {
    let parsed: JsonValue = serde_json::from_str(raw_passthru)
        .map_err(|error| format!("Invalid package passthru JSON: {error}"))?;
    let passthru = parsed
        .as_object()
        .ok_or_else(|| "Package passthru metadata must be a JSON object".to_string())?;
    let Some(metadata) = passthru
        .get("yazelixCursorPackageContract")
        .and_then(JsonValue::as_object)
    else {
        return Ok(vec![format!(
            "`yazelix_cursors` {system} package passthru must expose `yazelixCursorPackageContract`."
        )]);
    };

    let mut errors = Vec::new();
    let context = format!("`yazelix_cursors` {system} yazelixCursorPackageContract");
    require_exact_contract_fields(
        &mut errors,
        &context,
        metadata,
        CURSOR_PACKAGE_METADATA_FIELDS,
    );
    require_contract_number(&mut errors, &context, metadata, "schemaVersion", 1);
    require_contract_string(
        &mut errors,
        &context,
        metadata,
        "packageName",
        "yazelix-cursors",
    );
    require_contract_string(
        &mut errors,
        &context,
        metadata,
        "shareRoot",
        "share/yazelix/yazelix_cursors",
    );
    require_contract_string(
        &mut errors,
        &context,
        metadata,
        "defaultConfig",
        "share/yazelix/yazelix_cursors/cursors.toml",
    );
    require_contract_string(
        &mut errors,
        &context,
        metadata,
        "shaderRoot",
        "share/yazelix/yazelix_cursors/shaders",
    );
    require_contract_string(
        &mut errors,
        &context,
        metadata,
        "generatedEffectRoot",
        "share/yazelix/yazelix_cursors/shaders/generated_effects",
    );
    require_contract_string_array(
        &mut errors,
        &context,
        metadata,
        "requiredTargets",
        CURSOR_PACKAGE_REQUIRED_TARGETS,
    );
    require_contract_string_array(
        &mut errors,
        &context,
        metadata,
        "requiredShaderFiles",
        CURSOR_PACKAGE_REQUIRED_SHADER_FILES,
    );
    require_contract_string_array(
        &mut errors,
        &context,
        metadata,
        "forbiddenShaderFiles",
        &["build_shaders.nu"],
    );
    Ok(errors)
}

fn validate_main_cursor_shader_tree_deleted(repo_root: &Path) -> Vec<String> {
    let path = repo_root
        .join("configs")
        .join("terminal_emulators")
        .join("ghostty")
        .join("shaders");
    if path.exists() {
        vec![format!(
            "Main repo must not own Ghostty cursor shader assets at {}; consume `yazelix_cursors` package assets instead.",
            path.display()
        )]
    } else {
        Vec::new()
    }
}

fn package_passthru_metadata(
    repo_root: &Path,
    input: &ChildInputLock,
    system: &str,
    package_attr: &str,
) -> Result<String, String> {
    let flake_attr = package_passthru_flake_attr(input, system, package_attr);
    let eval = Command::new("nix")
        .current_dir(repo_root)
        .args(["eval", "--json", &flake_attr, "--accept-flake-config"])
        .output()
        .map_err(|error| format!("Failed to run `nix eval --json {flake_attr}`: {error}"))?;
    if !eval.status.success() {
        return Err(format!(
            "Failed to evaluate child package passthru metadata `{flake_attr}`\n{}",
            String::from_utf8_lossy(&eval.stderr).trim()
        ));
    }

    String::from_utf8(eval.stdout)
        .map_err(|error| format!("Invalid UTF-8 from `nix eval --json {flake_attr}`: {error}"))
}

fn locked_child_input<'a>(
    inputs: &'a [ChildInputLock],
    node: &str,
) -> Result<&'a ChildInputLock, String> {
    inputs
        .iter()
        .find(|input| input.node == node)
        .ok_or_else(|| format!("flake.lock is missing required child input `{node}`"))
}

fn package_passthru_flake_attr(input: &ChildInputLock, system: &str, package_attr: &str) -> String {
    format!(
        "github:{}/{}/{}#packages.{system}.{package_attr}.passthru",
        input.owner, input.repo, input.rev
    )
}

fn validate_zellij_plugin_wasm_package_contract_with(
    contract: &ZellijPluginWasmPackageContract,
    raw_passthru: &str,
) -> Result<Vec<String>, String> {
    let parsed: JsonValue = serde_json::from_str(raw_passthru)
        .map_err(|error| format!("Invalid package passthru JSON: {error}"))?;
    let passthru = parsed
        .as_object()
        .ok_or_else(|| "Package passthru metadata must be a JSON object".to_string())?;
    let mut errors = Vec::new();
    require_passthru_wasm_path(&mut errors, contract, passthru);
    let Some(metadata) = passthru
        .get("zellijPluginWasmPackageContract")
        .and_then(JsonValue::as_object)
    else {
        errors.push(format!(
            "`{}` {} package passthru must expose `zellijPluginWasmPackageContract`.",
            contract.package_attr, contract.system
        ));
        return Ok(errors);
    };

    let context = format!(
        "`{}` {} zellijPluginWasmPackageContract",
        contract.package_attr, contract.system
    );
    require_exact_contract_fields(
        &mut errors,
        &context,
        metadata,
        ZELLIJ_PLUGIN_WASM_METADATA_FIELDS,
    );
    require_contract_number(&mut errors, &context, metadata, "schemaVersion", 1);
    require_contract_string(
        &mut errors,
        &context,
        metadata,
        "pluginName",
        contract.plugin_name,
    );
    require_contract_string(
        &mut errors,
        &context,
        metadata,
        "packageAttr",
        contract.package_attr,
    );
    require_contract_string(
        &mut errors,
        &context,
        metadata,
        "wasmPath",
        contract.wasm_path,
    );
    require_contract_string(
        &mut errors,
        &context,
        metadata,
        "wasmTarget",
        "wasm32-wasip1",
    );
    for key in [
        "cargoAuditableDisabled",
        "cargoBuildHookDisabled",
        "preBuildPreservesNixRustToolchain",
        "wasmTargetRustcEnvPinned",
        "cargoBuildSerialized",
        "installCheckVerifiesWasm",
    ] {
        require_contract_bool(&mut errors, &context, metadata, key, true);
    }

    Ok(errors)
}

fn require_passthru_wasm_path(
    errors: &mut Vec<String>,
    contract: &ZellijPluginWasmPackageContract,
    passthru: &JsonObject,
) {
    let actual = passthru.get("wasmPath").and_then(JsonValue::as_str);
    if actual != Some(contract.wasm_path) {
        errors.push(format!(
            "`{}` {} package passthru must expose wasmPath `{}`; found {:?}.",
            contract.package_attr, contract.system, contract.wasm_path, actual
        ));
    }
}

fn require_exact_contract_fields(
    errors: &mut Vec<String>,
    context: &str,
    metadata: &JsonObject,
    required_fields: &[&str],
) {
    for required in required_fields {
        if !metadata.contains_key(*required) {
            errors.push(format!("{context} is missing required field `{required}`."));
        }
    }
    for actual in metadata.keys() {
        if !required_fields.contains(&actual.as_str()) {
            errors.push(format!("{context} has unsupported field `{actual}`."));
        }
    }
}

fn require_contract_number(
    errors: &mut Vec<String>,
    context: &str,
    metadata: &JsonObject,
    key: &str,
    expected: u64,
) {
    let actual = metadata.get(key).and_then(JsonValue::as_u64);
    if actual != Some(expected) {
        errors.push(format!(
            "{context} field `{key}` must be {expected}; found {actual:?}."
        ));
    }
}

fn require_contract_string(
    errors: &mut Vec<String>,
    context: &str,
    metadata: &JsonObject,
    key: &str,
    expected: &str,
) {
    let actual = metadata.get(key).and_then(JsonValue::as_str);
    if actual != Some(expected) {
        errors.push(format!(
            "{context} field `{key}` must be `{expected}`; found {actual:?}."
        ));
    }
}

fn require_contract_bool(
    errors: &mut Vec<String>,
    context: &str,
    metadata: &JsonObject,
    key: &str,
    expected: bool,
) {
    let actual = metadata.get(key).and_then(JsonValue::as_bool);
    if actual != Some(expected) {
        errors.push(format!(
            "{context} field `{key}` must be {expected}; found {actual:?}."
        ));
    }
}

fn require_contract_string_array(
    errors: &mut Vec<String>,
    context: &str,
    metadata: &JsonObject,
    key: &str,
    expected: &[&str],
) {
    let actual = metadata
        .get(key)
        .and_then(JsonValue::as_array)
        .map(|values| {
            values
                .iter()
                .filter_map(JsonValue::as_str)
                .collect::<Vec<_>>()
        });
    if actual.as_deref() != Some(expected) {
        errors.push(format!(
            "{context} field `{key}` must be {expected:?}; found {actual:?}."
        ));
    }
}

// Test lane: maintainer
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    // Defends: the child-release validator scopes itself to first-party Yazelix GitHub inputs instead of every flake dependency.
    #[test]
    fn locked_child_inputs_collects_first_party_yazelix_nodes_only() {
        let inputs = locked_child_inputs(
            r#"{
              "nodes": {
                "nixpkgs": {
                  "locked": {
                    "type": "github",
                    "owner": "NixOS",
                    "repo": "nixpkgs",
                    "rev": "1111111111111111111111111111111111111111"
                  }
                },
                "yazelixScreen": {
                  "locked": {
                    "type": "github",
                    "owner": "luccahuguet",
                    "repo": "yazelix-screen",
                    "rev": "2222222222222222222222222222222222222222"
                  }
                }
              }
            }"#,
        )
        .unwrap();

        assert_eq!(inputs.len(), 1);
        assert_eq!(inputs[0].node, "yazelixScreen");
        assert_eq!(inputs[0].repo, "yazelix-screen");
    }

    // Regression: successful child-revision fetch probes must not fail release validation because cleanup sees nested or already-removed probe dirs.
    #[test]
    fn remove_probe_dir_removes_nested_dirs_and_accepts_missing_paths() {
        let temp = tempdir().unwrap();
        let probe_dir = temp.path().join("probe");
        let nested = probe_dir.join("objects").join("pack");
        fs::create_dir_all(&nested).unwrap();
        fs::write(nested.join("pack.keep"), b"temporary git probe").unwrap();

        remove_probe_dir(&probe_dir).unwrap();
        assert!(!probe_dir.exists());

        remove_probe_dir(&probe_dir).unwrap();
    }

    // Regression: unpublished-child detection must treat a missing fetched object as a validation error without conflating transport failures with unpublished commits.
    #[test]
    fn fetch_failure_classifier_identifies_unreachable_revision() {
        assert!(fetch_failure_means_missing_revision(
            "fatal: remote error: upload-pack: not our ref cccccccccccccccccccccccccccccccccccccccc"
        ));
        assert!(fetch_failure_means_missing_revision(
            "fatal: couldn't find remote ref cccccccccccccccccccccccccccccccccccccccc"
        ));
        assert!(!fetch_failure_means_missing_revision(
            "fatal: unable to access 'https://github.com/example/repo.git/': Could not resolve host: github.com"
        ));
    }

    // Regression: contracted main flakes must validate package metadata at the exact locked child revision instead of requiring deleted main package mirrors.
    #[test]
    fn package_passthru_flake_attr_targets_locked_child_owner() {
        let input = ChildInputLock {
            node: "yazelixZellijPopup".to_string(),
            owner: "luccahuguet".to_string(),
            repo: "yazelix-zellij-popup".to_string(),
            rev: "2222222222222222222222222222222222222222".to_string(),
        };

        assert_eq!(
            package_passthru_flake_attr(&input, "aarch64-darwin", "yazelix_zellij_popup"),
            "github:luccahuguet/yazelix-zellij-popup/2222222222222222222222222222222222222222#packages.aarch64-darwin.yazelix_zellij_popup.passthru"
        );
    }

    // Regression: first-party Zellij plugin child packages must publish the child-owned wasm package contract that replaced main-side buildPhase inspection.
    #[test]
    fn zellij_plugin_wasm_contract_accepts_declared_package_metadata() {
        let contract = popup_contract();
        let metadata = zellij_plugin_wasm_passthru_json(serde_json::json!({
            "schemaVersion": 1,
            "pluginName": "yazelix-zellij-popup",
            "packageAttr": "yazelix_zellij_popup",
            "wasmPath": "share/yazelix_zellij_popup/yzpp.wasm",
            "wasmTarget": "wasm32-wasip1",
            "cargoAuditableDisabled": true,
            "cargoBuildHookDisabled": true,
            "preBuildPreservesNixRustToolchain": true,
            "wasmTargetRustcEnvPinned": true,
            "cargoBuildSerialized": true,
            "installCheckVerifiesWasm": true,
        }));

        let errors =
            validate_zellij_plugin_wasm_package_contract_with(&contract, &metadata).unwrap();

        assert!(errors.is_empty());
    }

    // Regression: stale child-declared artifact paths must fail before main packages a missing wasm.
    #[test]
    fn zellij_plugin_wasm_contract_rejects_stale_wasm_paths() {
        let contract = popup_contract();
        let metadata = zellij_plugin_wasm_passthru_json(serde_json::json!({
            "schemaVersion": 1,
            "pluginName": "yazelix-zellij-popup",
            "packageAttr": "yazelix_zellij_popup",
            "wasmPath": "share/yazelix_zellij_popup/old.wasm",
            "wasmTarget": "wasm32-wasip1",
            "cargoAuditableDisabled": true,
            "cargoBuildHookDisabled": true,
            "preBuildPreservesNixRustToolchain": true,
            "wasmTargetRustcEnvPinned": true,
            "cargoBuildSerialized": true,
            "installCheckVerifiesWasm": true,
        }));

        let errors =
            validate_zellij_plugin_wasm_package_contract_with(&contract, &metadata).unwrap();

        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("field `wasmPath`"));
    }

    // Regression: issue 604 remains guarded by child-declared Darwin wasm hardening metadata.
    #[test]
    fn zellij_plugin_wasm_contract_rejects_disabled_hardening_flags() {
        let contract = popup_contract();
        let metadata = zellij_plugin_wasm_passthru_json(serde_json::json!({
            "schemaVersion": 1,
            "pluginName": "yazelix-zellij-popup",
            "packageAttr": "yazelix_zellij_popup",
            "wasmPath": "share/yazelix_zellij_popup/yzpp.wasm",
            "wasmTarget": "wasm32-wasip1",
            "cargoAuditableDisabled": false,
            "cargoBuildHookDisabled": false,
            "preBuildPreservesNixRustToolchain": false,
            "wasmTargetRustcEnvPinned": false,
            "cargoBuildSerialized": false,
            "installCheckVerifiesWasm": false,
        }));

        let errors =
            validate_zellij_plugin_wasm_package_contract_with(&contract, &metadata).unwrap();

        assert_eq!(errors.len(), 6);
        assert!(
            errors
                .iter()
                .any(|error| error.contains("cargoAuditableDisabled"))
        );
        assert!(
            errors
                .iter()
                .any(|error| error.contains("cargoBuildHookDisabled"))
        );
        assert!(
            errors
                .iter()
                .any(|error| error.contains("preBuildPreservesNixRustToolchain"))
        );
        assert!(
            errors
                .iter()
                .any(|error| error.contains("wasmTargetRustcEnvPinned"))
        );
        assert!(
            errors
                .iter()
                .any(|error| error.contains("cargoBuildSerialized"))
        );
    }

    // Defends: child metadata must use the exact supported contract shape instead of growing unvalidated future fields.
    #[test]
    fn zellij_plugin_wasm_contract_rejects_unknown_metadata_fields() {
        let contract = popup_contract();
        let metadata = zellij_plugin_wasm_passthru_json(serde_json::json!({
            "schemaVersion": 1,
            "pluginName": "yazelix-zellij-popup",
            "packageAttr": "yazelix_zellij_popup",
            "wasmPath": "share/yazelix_zellij_popup/yzpp.wasm",
            "wasmTarget": "wasm32-wasip1",
            "cargoAuditableDisabled": true,
            "cargoBuildHookDisabled": true,
            "preBuildPreservesNixRustToolchain": true,
            "wasmTargetRustcEnvPinned": true,
            "cargoBuildSerialized": true,
            "installCheckVerifiesWasm": true,
            "futureBuildPhaseHint": "do not accept this",
        }));

        let errors =
            validate_zellij_plugin_wasm_package_contract_with(&contract, &metadata).unwrap();

        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("unsupported field `futureBuildPhaseHint`"));
    }

    // Defends: cursor package metadata must stay exact so main cannot silently consume stale or future child artifact shapes.
    #[test]
    fn cursor_package_contract_accepts_declared_metadata() {
        let metadata = cursor_package_passthru_json(valid_cursor_package_contract_json());

        let errors = validate_cursor_package_contract_with("x86_64-linux", &metadata).unwrap();

        assert!(errors.is_empty());
    }

    // Defends: child cursor package metadata cannot grow unused planning fields that main never validates.
    #[test]
    fn cursor_package_contract_rejects_unknown_metadata_fields() {
        let mut contract = valid_cursor_package_contract_json();
        contract
            .as_object_mut()
            .expect("test fixture is an object")
            .insert("requiredRuntimeScripts".to_string(), serde_json::json!([]));
        let metadata = cursor_package_passthru_json(contract);

        let errors = validate_cursor_package_contract_with("x86_64-linux", &metadata).unwrap();

        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("unsupported field `requiredRuntimeScripts`"));
    }

    fn valid_cursor_package_contract_json() -> JsonValue {
        serde_json::json!({
            "schemaVersion": 1,
            "packageName": "yazelix-cursors",
            "shareRoot": "share/yazelix/yazelix_cursors",
            "defaultConfig": "share/yazelix/yazelix_cursors/cursors.toml",
            "shaderRoot": "share/yazelix/yazelix_cursors/shaders",
            "generatedEffectRoot": "share/yazelix/yazelix_cursors/shaders/generated_effects",
            "requiredTargets": CURSOR_PACKAGE_REQUIRED_TARGETS,
            "requiredShaderFiles": CURSOR_PACKAGE_REQUIRED_SHADER_FILES,
            "forbiddenShaderFiles": ["build_shaders.nu"],
        })
    }

    // Regression: the main repo must not reintroduce a mirrored Ghostty cursor shader source tree after child ownership.
    #[test]
    fn child_release_validation_rejects_main_cursor_shader_tree() {
        let temp = tempdir().unwrap();
        let shader_root = temp
            .path()
            .join("configs")
            .join("terminal_emulators")
            .join("ghostty")
            .join("shaders");
        fs::create_dir_all(&shader_root).unwrap();

        let errors = validate_main_cursor_shader_tree_deleted(temp.path());

        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("Main repo must not own Ghostty cursor shader assets"));
    }

    fn popup_contract() -> ZellijPluginWasmPackageContract {
        ZellijPluginWasmPackageContract {
            input_node: "yazelixZellijPopup",
            package_attr: "yazelix_zellij_popup",
            system: "aarch64-darwin",
            plugin_name: "yazelix-zellij-popup",
            wasm_path: "share/yazelix_zellij_popup/yzpp.wasm",
        }
    }

    fn zellij_plugin_wasm_passthru_json(contract: JsonValue) -> String {
        serde_json::json!({
            "wasmPath": "share/yazelix_zellij_popup/yzpp.wasm",
            "zellijPluginWasmPackageContract": contract,
        })
        .to_string()
    }

    fn cursor_package_passthru_json(contract: JsonValue) -> String {
        serde_json::json!({
            "yazelixCursorPackageContract": contract,
        })
        .to_string()
    }
}
