// Test lane: default

use serde_json::Value;
use std::fs;

mod support;

use support::commands::yzx_root_command;
use support::fixtures::{repo_root, write_runtime_contract_assets};

// Defends: issue reporters can copy one root-command payload with release, runtime path, variant, and packaged source/input revisions.
#[test]
fn yzx_version_full_prints_packaged_runtime_identity() {
    let repo = repo_root();
    let temp = tempfile::tempdir().unwrap();
    let runtime_dir = temp.path().join("runtime");
    write_runtime_contract_assets(&repo, &runtime_dir);
    fs::write(
        runtime_dir.join("runtime_identity.json"),
        r#"{
          "schema_version": 1,
          "runtime_variant": "ghostty",
          "source": {
            "revision": "0123456789abcdef0123456789abcdef01234567",
            "short_revision": "0123456",
            "last_modified_date": "20260602123456"
          },
          "inputs": {
            "yazelix_zellij_pane_orchestrator": {
              "revision": "abcdefabcdefabcdefabcdefabcdefabcdefabcd",
              "short_revision": "abcdefa",
              "last_modified_date": "20260602110000"
            }
          }
        }"#,
    )
    .unwrap();

    let output = yzx_root_command()
        .env_clear()
        .env("YAZELIX_RUNTIME_DIR", &runtime_dir)
        .arg("--version-full")
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let payload: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(payload["schema_version"], 1);
    assert_eq!(payload["version"], "v-test");
    assert_eq!(
        payload["runtime_dir"],
        runtime_dir.to_string_lossy().as_ref()
    );
    assert_eq!(payload["runtime_identity"]["runtime_variant"], "ghostty");
    assert_eq!(
        payload["runtime_identity"]["source"]["revision"],
        "0123456789abcdef0123456789abcdef01234567"
    );
    assert_eq!(
        payload["runtime_identity"]["inputs"]["yazelix_zellij_pane_orchestrator"]["revision"],
        "abcdefabcdefabcdefabcdefabcdefabcdefabcd"
    );
}
