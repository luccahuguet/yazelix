// Test lane: maintainer

use assert_cmd::Command;
use serde_json::Value;

// Defends: yazi-render-plan.compute returns one machine-readable success envelope for a typical normalized request.
// Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
#[test]
fn yazi_render_plan_emits_ok_envelope() {
    let request = serde_json::json!({
        "yazi_theme": "default",
        "yazi_sort_by": "alphabetical",
        "yazi_plugins": ["git", "starship"]
    });

    let output = Command::cargo_bin("yzx_core")
        .unwrap()
        .arg("yazi-render-plan.compute")
        .arg("--request-json")
        .arg(request.to_string())
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let envelope: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(envelope["schema_version"], 1);
    assert_eq!(envelope["command"], "yazi-render-plan.compute");
    assert_eq!(envelope["status"], "ok");
    assert_eq!(envelope["data"]["resolved_theme"], "default");
    assert_eq!(envelope["data"]["git_plugin_enabled"], true);
    assert_eq!(envelope["data"]["theme_flavor"]["kind"], "none");
}

// Defends: yazi-render-plan.compute rejects invalid sort_by with a single config-class error envelope.
// Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=1 total=8/10
#[test]
fn yazi_render_plan_rejects_invalid_sort_by() {
    let request = serde_json::json!({
        "yazi_sort_by": "not-a-sort"
    });

    let output = Command::cargo_bin("yzx_core")
        .unwrap()
        .arg("yazi-render-plan.compute")
        .arg("--request-json")
        .arg(request.to_string())
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(65));
    let envelope: Value = serde_json::from_slice(&output.stderr).unwrap();
    assert_eq!(envelope["status"], "error");
    assert_eq!(envelope["error"]["class"], "config");
    assert_eq!(envelope["error"]["code"], "invalid_yazi_sort_by");
}
