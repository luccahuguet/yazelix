// Test lane: maintainer

use pretty_assertions::assert_eq;
use serde_json::{Value, json};
use std::fs;
use tempfile::tempdir;

mod support;

use support::commands::yzx_core_command;
use support::envelopes::{error_envelope, ok_envelope};

// Defends: runtime-env.compute returns one machine-readable env envelope with filtered PATH entries and managed Helix wrapping.
// Contract: CRCP-002
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

    let output = yzx_core_command()
        .arg("runtime-env.compute")
        .arg("--request-json")
        .arg(request.to_string())
        .output()
        .unwrap();

    let envelope: Value = ok_envelope(&output);
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
        "yzx_side_closed"
    );
}

// Defends: runtime-env.compute rejects malformed JSON request payloads with one usage envelope.
// Contract: CRCP-002
// Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=1 total=8/10
#[test]
fn runtime_env_compute_rejects_invalid_request_json() {
    let output = yzx_core_command()
        .arg("runtime-env.compute")
        .arg("--request-json")
        .arg("{")
        .output()
        .unwrap();

    let envelope: Value = error_envelope(&output, 64);
    assert_eq!(envelope["command"], "runtime-env.compute");
    assert_eq!(envelope["error"]["class"], "usage");
    assert_eq!(envelope["error"]["code"], "invalid_request_json");
}

// Defends: runtime-env.compute can build the canonical runtime env from process env plus optional config JSON without Nu request assembly.
// Contract: CRCP-002
// Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
#[test]
fn runtime_env_compute_from_env_accepts_config_json() {
    let tmp = tempdir().unwrap();
    let runtime_dir = tmp.path().join("runtime");
    let home_dir = tmp.path().join("home");

    fs::create_dir_all(runtime_dir.join("toolbin")).unwrap();
    fs::create_dir_all(runtime_dir.join("bin")).unwrap();
    fs::create_dir_all(&home_dir).unwrap();

    let runtime_toolbin = runtime_dir.join("toolbin");
    let runtime_bin = runtime_dir.join("bin");
    let config_json = json!({
        "enable_sidebar": true,
        "initial_sidebar_state": "closed",
        "editor_command": "hx",
        "helix_runtime_path": "/tmp/managed-helix-runtime",
    });

    let output = yzx_core_command()
        .env_clear()
        .env("HOME", &home_dir)
        .env(
            "PATH",
            format!("{}:{}", runtime_toolbin.display(), runtime_bin.display()),
        )
        .env("YAZELIX_RUNTIME_DIR", &runtime_dir)
        .arg("runtime-env.compute")
        .arg("--from-env")
        .arg("--config-json")
        .arg(config_json.to_string())
        .output()
        .unwrap();

    let envelope: Value = ok_envelope(&output);
    let expected_wrapper = runtime_dir
        .join("shells")
        .join("posix")
        .join("yazelix_hx.sh")
        .to_string_lossy()
        .to_string();

    assert_eq!(envelope["status"], "ok");
    assert_eq!(envelope["command"], "runtime-env.compute");
    assert_eq!(
        envelope["data"]["runtime_env"]["PATH"],
        json!([
            runtime_toolbin.to_string_lossy().to_string(),
            runtime_bin.to_string_lossy().to_string(),
        ])
    );
    assert_eq!(envelope["data"]["runtime_env"]["EDITOR"], expected_wrapper);
    assert_eq!(envelope["data"]["runtime_env"]["VISUAL"], expected_wrapper);
    assert_eq!(
        envelope["data"]["runtime_env"]["ZELLIJ_DEFAULT_LAYOUT"],
        "yzx_side_closed"
    );
    assert_eq!(
        envelope["data"]["runtime_env"]["HELIX_RUNTIME"],
        "/tmp/managed-helix-runtime"
    );
}
