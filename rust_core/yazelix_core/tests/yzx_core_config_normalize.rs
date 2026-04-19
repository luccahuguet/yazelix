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
