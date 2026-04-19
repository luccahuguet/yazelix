// Test lane: maintainer

use assert_cmd::Command;
use pretty_assertions::assert_eq;
use serde_json::{Value, json};
use std::fs;
use tempfile::tempdir;

// Defends: runtime-env.compute returns one machine-readable env envelope with filtered PATH entries and managed Helix wrapping.
// Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
#[test]
fn runtime_env_compute_prints_machine_readable_env_envelope() {
    let tmp = tempdir().unwrap();
    let runtime_dir = tmp.path().join("runtime");
    let home_dir = tmp.path().join("home");

    fs::create_dir_all(runtime_dir.join("toolbin")).unwrap();
    fs::create_dir_all(runtime_dir.join("bin")).unwrap();
    fs::create_dir_all(runtime_dir.join("libexec")).unwrap();
    fs::create_dir_all(&home_dir).unwrap();

    let runtime_libexec = runtime_dir.join("libexec");
    let runtime_toolbin = runtime_dir.join("toolbin");
    let runtime_bin = runtime_dir.join("bin");
    let request = json!({
        "runtime_dir": runtime_dir,
        "home_dir": home_dir,
        "current_path": format!(
            "{}:{}:/usr/local/bin:{}:/usr/bin",
            runtime_libexec.to_string_lossy(),
            runtime_toolbin.to_string_lossy(),
            runtime_bin.to_string_lossy(),
        ),
        "enable_sidebar": false,
        "editor_command": "hx",
        "helix_runtime_path": "/tmp/helix-runtime",
    });

    let output = Command::cargo_bin("yzx_core")
        .unwrap()
        .arg("runtime-env.compute")
        .arg("--request-json")
        .arg(request.to_string())
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(output.stderr.is_empty());

    let envelope: Value = serde_json::from_slice(&output.stdout).unwrap();
    let expected_wrapper = runtime_dir
        .join("shells")
        .join("posix")
        .join("yazelix_hx.sh")
        .to_string_lossy()
        .to_string();
    let expected_home = tmp
        .path()
        .join("home")
        .join(".local")
        .join("share")
        .join("yazelix")
        .join("configs")
        .join("yazi")
        .to_string_lossy()
        .to_string();

    assert_eq!(envelope["schema_version"], 1);
    assert_eq!(envelope["command"], "runtime-env.compute");
    assert_eq!(envelope["status"], "ok");
    assert_eq!(envelope["data"]["editor_kind"], "helix");
    assert_eq!(
        envelope["data"]["path_entries"],
        json!([
            runtime_toolbin.to_string_lossy().to_string(),
            runtime_bin.to_string_lossy().to_string(),
            "/usr/local/bin",
            "/usr/bin"
        ])
    );
    assert_eq!(
        envelope["data"]["runtime_env"]["PATH"],
        envelope["data"]["path_entries"]
    );
    assert_eq!(envelope["data"]["runtime_env"]["EDITOR"], expected_wrapper);
    assert_eq!(envelope["data"]["runtime_env"]["VISUAL"], expected_wrapper);
    assert_eq!(
        envelope["data"]["runtime_env"]["YAZELIX_MANAGED_HELIX_BINARY"],
        "hx"
    );
    assert_eq!(
        envelope["data"]["runtime_env"]["HELIX_RUNTIME"],
        "/tmp/helix-runtime"
    );
    assert_eq!(
        envelope["data"]["runtime_env"]["YAZI_CONFIG_HOME"],
        expected_home
    );
    assert_eq!(
        envelope["data"]["runtime_env"]["ZELLIJ_DEFAULT_LAYOUT"],
        "yzx_no_side"
    );
}

// Defends: runtime-env.compute rejects malformed JSON request payloads with one usage envelope.
// Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=1 total=8/10
#[test]
fn runtime_env_compute_rejects_invalid_request_json() {
    let output = Command::cargo_bin("yzx_core")
        .unwrap()
        .arg("runtime-env.compute")
        .arg("--request-json")
        .arg("{")
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(64));
    assert!(output.stdout.is_empty());

    let envelope: Value = serde_json::from_slice(&output.stderr).unwrap();
    assert_eq!(envelope["command"], "runtime-env.compute");
    assert_eq!(envelope["status"], "error");
    assert_eq!(envelope["error"]["class"], "usage");
    assert_eq!(envelope["error"]["code"], "invalid_request_json");
}
