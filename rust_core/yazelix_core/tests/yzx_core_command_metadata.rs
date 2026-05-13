// Test lane: maintainer

use assert_cmd::Command;
use serde_json::Value;
use std::fs;
use tempfile::TempDir;

// Regression: yzx_core owns generated extern bridge sync, so startup no longer needs the Nushell compatibility wrapper.
#[test]
fn command_metadata_sync_externs_writes_generated_bridge() {
    let runtime = TempDir::new().unwrap();
    let state = TempDir::new().unwrap();
    fs::write(runtime.path().join("settings_default.jsonc"), "").unwrap();

    let first = Command::cargo_bin("yzx_core")
        .unwrap()
        .arg("yzx-command-metadata.sync-externs")
        .arg("--runtime-dir")
        .arg(runtime.path())
        .arg("--state-dir")
        .arg(state.path())
        .output()
        .unwrap();

    assert!(first.status.success());
    assert!(first.stderr.is_empty());

    let envelope: Value = serde_json::from_slice(&first.stdout).unwrap();
    assert_eq!(envelope["command"], "yzx-command-metadata.sync-externs");
    assert_eq!(envelope["status"], "ok");
    assert_eq!(envelope["data"]["status"], "updated");

    let extern_path = envelope["data"]["extern_path"].as_str().unwrap();
    let fingerprint_path = envelope["data"]["fingerprint_path"].as_str().unwrap();
    let generated = fs::read_to_string(extern_path).unwrap();
    assert!(generated.contains("export extern \"yzx env\""));
    assert!(generated.contains("export extern \"yzx run\""));
    assert!(fs::metadata(fingerprint_path).unwrap().is_file());
}

// Regression: Rust-owned extern bridge sync ignores host Nushell config so migrated leaves do not get rendered twice on startup.
#[test]
fn command_metadata_sync_externs_ignores_host_nushell_config() {
    let runtime = TempDir::new().unwrap();
    let state = TempDir::new().unwrap();
    let home = TempDir::new().unwrap();
    let xdg_config_home = home.path().join(".config");
    let nushell_config_dir = xdg_config_home.join("nushell");
    let initializers_dir = state.path().join("initializers").join("nushell");
    fs::create_dir_all(&nushell_config_dir).unwrap();
    fs::create_dir_all(&initializers_dir).unwrap();
    fs::write(runtime.path().join("settings_default.jsonc"), "").unwrap();
    fs::write(initializers_dir.join("yazelix_init.nu"), "").unwrap();
    fs::write(initializers_dir.join("yazelix_user_hook.nu"), "").unwrap();

    let sync_output = Command::cargo_bin("yzx_core")
        .unwrap()
        .arg("yzx-command-metadata.sync-externs")
        .arg("--runtime-dir")
        .arg(runtime.path())
        .arg("--state-dir")
        .arg(state.path())
        .output()
        .unwrap();

    assert!(sync_output.status.success());
    let envelope: Value = serde_json::from_slice(&sync_output.stdout).unwrap();
    let extern_path = envelope["data"]["extern_path"].as_str().unwrap();
    fs::write(
        nushell_config_dir.join("config.nu"),
        format!("source \"{extern_path}\"\n"),
    )
    .unwrap();

    let second_sync = Command::cargo_bin("yzx_core")
        .unwrap()
        .env("HOME", home.path())
        .env("XDG_CONFIG_HOME", &xdg_config_home)
        .arg("yzx-command-metadata.sync-externs")
        .arg("--runtime-dir")
        .arg(runtime.path())
        .arg("--state-dir")
        .arg(state.path())
        .output()
        .unwrap();

    assert!(
        second_sync.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&second_sync.stderr)
    );
    let generated = fs::read_to_string(extern_path).unwrap();
    assert_eq!(generated.matches("export extern \"yzx env\"").count(), 1);
    assert_eq!(generated.matches("export extern \"yzx run\"").count(), 1);
}

// Defends: the machine-readable runtime ownership graph is emitted by yzx_core from Rust command metadata and packaged manifests.
#[test]
fn runtime_ownership_graph_includes_command_and_manifest_owners() {
    let runtime = TempDir::new().unwrap();
    fs::write(
        runtime.path().join("runtime_tools.json"),
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
        runtime.path().join("runtime_components.json"),
        r#"{
          "screen": {
            "enabled": true,
            "disableable": true,
            "notes": []
          }
        }"#,
    )
    .unwrap();

    let output = Command::cargo_bin("yzx_core")
        .unwrap()
        .arg("runtime-ownership.graph")
        .arg("--runtime-dir")
        .arg(runtime.path())
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let envelope: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(envelope["command"], "runtime-ownership.graph");
    assert_eq!(envelope["status"], "ok");
    assert_eq!(envelope["data"]["runtime_tools"]["status"], "available");
    assert_eq!(
        envelope["data"]["runtime_tools"]["entries"][0]["name"],
        "yazi"
    );
    assert_eq!(
        envelope["data"]["runtime_components"]["entries"][0]["name"],
        "screen"
    );

    let command_owners = envelope["data"]["command_owners"].as_array().unwrap();
    assert!(
        command_owners
            .iter()
            .any(|entry| { entry["command"] == "yzx launch" && entry["owner"] == "rust_control" })
    );
    assert!(
        command_owners
            .iter()
            .any(|entry| entry["command"] == "yzx menu" && entry["owner"] == "nushell")
    );
}
