// Test lane: maintainer

use assert_cmd::Command;
use pretty_assertions::assert_eq;
use serde_json::Value;
use std::fs;
use std::path::PathBuf;
use tempfile::tempdir;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repo root")
}

// Defends: config.normalize emits a single machine-readable success envelope for valid config input.
// Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=1 total=8/10
#[test]
fn config_normalize_prints_one_success_json_envelope() {
    let repo = repo_root();
    let output = Command::cargo_bin("yzx_core")
        .unwrap()
        .arg("config.normalize")
        .arg("--config")
        .arg(repo.join("yazelix_default.toml"))
        .arg("--default-config")
        .arg(repo.join("yazelix_default.toml"))
        .arg("--contract")
        .arg(repo.join("config_metadata/main_config_contract.toml"))
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let envelope: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(envelope["schema_version"], 1);
    assert_eq!(envelope["command"], "config.normalize");
    assert_eq!(envelope["status"], "ok");
    assert_eq!(
        envelope["data"]["normalized_config"]["terminal_config_mode"],
        "yazelix"
    );
}

// Defends: config.normalize emits a single machine-readable config error envelope for invalid input.
// Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=1 total=8/10
#[test]
fn config_normalize_prints_one_error_json_envelope() {
    let repo = repo_root();
    let tmp = tempdir().unwrap();
    let config_path = tmp.path().join("yazelix.toml");
    fs::write(&config_path, "[shell]\ndefault_shell = \"powershell\"\n").unwrap();

    let output = Command::cargo_bin("yzx_core")
        .unwrap()
        .arg("config.normalize")
        .arg("--config")
        .arg(&config_path)
        .arg("--default-config")
        .arg(repo.join("yazelix_default.toml"))
        .arg("--contract")
        .arg(repo.join("config_metadata/main_config_contract.toml"))
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(65));
    assert!(output.stdout.is_empty());
    let envelope: Value = serde_json::from_slice(&output.stderr).unwrap();
    assert_eq!(envelope["schema_version"], 1);
    assert_eq!(envelope["command"], "config.normalize");
    assert_eq!(envelope["status"], "error");
    assert_eq!(envelope["error"]["class"], "config");
    assert_eq!(envelope["error"]["code"], "unsupported_config");
}

// Defends: unknown helper commands report the requested command in the usage error envelope.
// Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=1 total=8/10
#[test]
fn unsupported_command_reports_requested_command_in_error_envelope() {
    let output = Command::cargo_bin("yzx_core")
        .unwrap()
        .arg("config.unknown")
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(64));
    assert!(output.stdout.is_empty());
    let envelope: Value = serde_json::from_slice(&output.stderr).unwrap();
    assert_eq!(envelope["command"], "config.unknown");
    assert_eq!(envelope["error"]["class"], "usage");
    assert_eq!(envelope["error"]["code"], "invalid_arguments");
}

// Defends: config-state.compute returns a machine-readable state envelope with stable hash fields.
// Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
#[test]
fn config_state_compute_prints_machine_readable_state_envelope() {
    let repo = repo_root();
    let tmp = tempdir().unwrap();
    let state_path = tmp.path().join("state/rebuild_hash");

    let output = Command::cargo_bin("yzx_core")
        .unwrap()
        .arg("config-state.compute")
        .arg("--config")
        .arg(repo.join("yazelix_default.toml"))
        .arg("--default-config")
        .arg(repo.join("yazelix_default.toml"))
        .arg("--contract")
        .arg(repo.join("config_metadata/main_config_contract.toml"))
        .arg("--runtime-dir")
        .arg(&repo)
        .arg("--state-path")
        .arg(&state_path)
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let envelope: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(envelope["schema_version"], 1);
    assert_eq!(envelope["command"], "config-state.compute");
    assert_eq!(envelope["status"], "ok");
    assert_eq!(envelope["data"]["config"]["default_shell"], "nu");
    assert_eq!(
        envelope["data"]["config_hash"],
        "2f7b0e3920d8a8862d243edcc6c39867042e88390a8b16546783d1482dcb6988"
    );
    assert_eq!(envelope["data"]["needs_refresh"], true);
}

// Defends: config-state.record persists state only for the managed main config surface.
// Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
#[test]
fn config_state_record_writes_only_managed_surface_state() {
    let tmp = tempdir().unwrap();
    let managed_config = tmp.path().join("config/user_configs/yazelix.toml");
    let state_path = tmp.path().join("state/rebuild_hash");

    let output = Command::cargo_bin("yzx_core")
        .unwrap()
        .arg("config-state.record")
        .arg("--config-file")
        .arg(&managed_config)
        .arg("--managed-config")
        .arg(&managed_config)
        .arg("--state-path")
        .arg(&state_path)
        .arg("--config-hash")
        .arg("cfg")
        .arg("--runtime-hash")
        .arg("runtime")
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let envelope: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(envelope["command"], "config-state.record");
    assert_eq!(envelope["data"]["recorded"], true);
    let recorded: Value = serde_json::from_str(&fs::read_to_string(state_path).unwrap()).unwrap();
    assert_eq!(
        recorded,
        serde_json::json!({"config_hash":"cfg","runtime_hash":"runtime"})
    );
}

// Defends: runtime-materialization.plan reports missing artifacts without forcing refresh when hashes are current.
// Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
#[test]
fn runtime_materialization_plan_reports_missing_artifacts_with_current_state() {
    let repo = repo_root();
    let tmp = tempdir().unwrap();
    let managed_config = tmp.path().join("config/user_configs/yazelix.toml");
    let state_path = tmp.path().join("state/rebuild_hash");
    let yazi_dir = tmp.path().join("configs/yazi");
    let zellij_dir = tmp.path().join("configs/zellij");
    let zellij_layout_dir = zellij_dir.join("layouts");

    fs::create_dir_all(managed_config.parent().unwrap()).unwrap();
    fs::create_dir_all(&zellij_layout_dir).unwrap();
    fs::copy(repo.join("yazelix_default.toml"), &managed_config).unwrap();

    let state_output = Command::cargo_bin("yzx_core")
        .unwrap()
        .arg("config-state.compute")
        .arg("--config")
        .arg(&managed_config)
        .arg("--default-config")
        .arg(repo.join("yazelix_default.toml"))
        .arg("--contract")
        .arg(repo.join("config_metadata/main_config_contract.toml"))
        .arg("--runtime-dir")
        .arg(&repo)
        .arg("--state-path")
        .arg(&state_path)
        .output()
        .unwrap();
    assert!(state_output.status.success());
    let state_envelope: Value = serde_json::from_slice(&state_output.stdout).unwrap();

    let record_output = Command::cargo_bin("yzx_core")
        .unwrap()
        .arg("config-state.record")
        .arg("--config-file")
        .arg(&managed_config)
        .arg("--managed-config")
        .arg(&managed_config)
        .arg("--state-path")
        .arg(&state_path)
        .arg("--config-hash")
        .arg(state_envelope["data"]["config_hash"].as_str().unwrap())
        .arg("--runtime-hash")
        .arg(state_envelope["data"]["runtime_hash"].as_str().unwrap())
        .output()
        .unwrap();
    assert!(record_output.status.success());

    let output = Command::cargo_bin("yzx_core")
        .unwrap()
        .arg("runtime-materialization.plan")
        .arg("--config")
        .arg(&managed_config)
        .arg("--default-config")
        .arg(repo.join("yazelix_default.toml"))
        .arg("--contract")
        .arg(repo.join("config_metadata/main_config_contract.toml"))
        .arg("--runtime-dir")
        .arg(&repo)
        .arg("--state-path")
        .arg(&state_path)
        .arg("--yazi-config-dir")
        .arg(&yazi_dir)
        .arg("--zellij-config-dir")
        .arg(&zellij_dir)
        .arg("--zellij-layout-dir")
        .arg(&zellij_layout_dir)
        .output()
        .unwrap();

    assert!(output.status.success());
    let envelope: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(envelope["command"], "runtime-materialization.plan");
    assert_eq!(envelope["data"]["status"], "repair_missing_artifacts");
    assert_eq!(envelope["data"]["needs_refresh"], false);
    assert_eq!(
        envelope["data"]["missing_artifacts"]
            .as_array()
            .unwrap()
            .len(),
        5
    );
}

// Defends: runtime-materialization.apply rejects missing expected artifacts with a runtime error envelope.
// Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
#[test]
fn runtime_materialization_apply_rejects_missing_artifacts() {
    let tmp = tempdir().unwrap();
    let managed_config = tmp.path().join("config/user_configs/yazelix.toml");
    let state_path = tmp.path().join("state/rebuild_hash");
    let expected_artifacts = serde_json::json!([
        {
            "label": "generated Yazi config",
            "path": tmp.path().join("configs/yazi/yazi.toml").to_string_lossy().to_string()
        }
    ]);

    let output = Command::cargo_bin("yzx_core")
        .unwrap()
        .arg("runtime-materialization.apply")
        .arg("--config-file")
        .arg(&managed_config)
        .arg("--managed-config")
        .arg(&managed_config)
        .arg("--state-path")
        .arg(&state_path)
        .arg("--config-hash")
        .arg("cfg")
        .arg("--runtime-hash")
        .arg("runtime")
        .arg("--expected-artifacts-json")
        .arg(expected_artifacts.to_string())
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(70));
    assert!(output.stdout.is_empty());
    let envelope: Value = serde_json::from_slice(&output.stderr).unwrap();
    assert_eq!(envelope["command"], "runtime-materialization.apply");
    assert_eq!(envelope["error"]["class"], "runtime");
    assert_eq!(envelope["error"]["code"], "missing_generated_artifacts");
}

// Defends: runtime-contract.evaluate emits one machine-readable checks envelope for batched preflight requests.
// Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
#[test]
fn runtime_contract_evaluate_prints_machine_readable_checks_envelope() {
    let tmp = tempdir().unwrap();
    let host_bin = tmp.path().join("host-bin");
    fs::create_dir_all(&host_bin).unwrap();
    fs::write(host_bin.join("ghostty"), "#!/bin/sh\nexit 0\n").unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut permissions = fs::metadata(host_bin.join("ghostty"))
            .unwrap()
            .permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(host_bin.join("ghostty"), permissions).unwrap();
    }

    let request = serde_json::json!({
        "working_dir": {
            "kind": "launch",
            "path": tmp.path().join("missing-dir").to_string_lossy().to_string()
        },
        "terminal_support": {
            "owner_surface": "launch",
            "requested_terminal": "",
            "terminals": ["ghostty"],
            "command_search_paths": [host_bin.to_string_lossy().to_string()]
        }
    });

    let output = Command::cargo_bin("yzx_core")
        .unwrap()
        .arg("runtime-contract.evaluate")
        .arg("--request-json")
        .arg(request.to_string())
        .output()
        .unwrap();

    assert!(output.status.success());
    let envelope: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(envelope["command"], "runtime-contract.evaluate");
    assert_eq!(envelope["status"], "ok");
    assert_eq!(
        envelope["data"]["checks"][0]["message"],
        format!(
            "Launch directory does not exist: {}",
            tmp.path().join("missing-dir").to_string_lossy()
        )
    );
    assert_eq!(
        envelope["data"]["checks"][1]["message"],
        "A configured terminal command is available"
    );
}

// Defends: runtime-contract.evaluate rejects malformed request JSON as a usage-surface error.
// Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
#[test]
fn runtime_contract_evaluate_reports_invalid_request_json_as_usage_error() {
    let output = Command::cargo_bin("yzx_core")
        .unwrap()
        .arg("runtime-contract.evaluate")
        .arg("--request-json")
        .arg("{not-json")
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(64));
    assert!(output.stdout.is_empty());
    let envelope: Value = serde_json::from_slice(&output.stderr).unwrap();
    assert_eq!(envelope["command"], "runtime-contract.evaluate");
    assert_eq!(envelope["error"]["class"], "usage");
    assert_eq!(envelope["error"]["code"], "invalid_request_json");
}

// Defends: install-ownership.evaluate emits one machine-readable report envelope for explicit request paths.
// Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
#[test]
fn install_ownership_evaluate_prints_ok_envelope() {
    let repo = repo_root();
    let tmp = tempdir().unwrap();
    let home = tmp.path().join("home");
    let xdg_data = home.join(".local/share");
    let xdg_config = home.join(".config");
    let state = xdg_data.join("yazelix");
    let main = home.join(".config/yazelix/yazelix.toml");
    fs::create_dir_all(main.parent().unwrap()).unwrap();
    fs::write(&main, "[core]\n").unwrap();

    let request = serde_json::json!({
        "runtime_dir": repo.to_string_lossy(),
        "home_dir": home.to_string_lossy(),
        "user": null,
        "xdg_config_home": xdg_config.to_string_lossy(),
        "xdg_data_home": xdg_data.to_string_lossy(),
        "yazelix_state_dir": state.to_string_lossy(),
        "main_config_path": main.to_string_lossy(),
        "invoked_yzx_path": null,
        "redirected_from_stale_yzx_path": null,
        "shell_resolved_yzx_path": null,
    });

    let output = Command::cargo_bin("yzx_core")
        .unwrap()
        .arg("install-ownership.evaluate")
        .arg("--request-json")
        .arg(request.to_string())
        .output()
        .unwrap();

    assert!(output.status.success());
    let envelope: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(envelope["command"], "install-ownership.evaluate");
    assert_eq!(envelope["status"], "ok");
    assert!(envelope["data"]["install_owner"].is_string());
    assert!(envelope["data"]["desktop_entry_freshness"]["message"].is_string());
}
