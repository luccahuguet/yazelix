// Defends: zellij-render-plan.compute returns layout/plan data with one success envelope.

use assert_cmd::Command;
use serde_json::Value;

#[test]
fn zellij_render_plan_emits_ok_envelope() {
    let request = serde_json::json!({
        "enable_sidebar": true,
        "sidebar_width_percent": 20,
        "popup_width_percent": 90,
        "popup_height_percent": 90,
        "zellij_theme": "default",
        "zellij_pane_frames": "true",
        "zellij_rounded_corners": "true",
        "disable_zellij_tips": "true",
        "persistent_sessions": "false",
        "support_kitty_keyboard_protocol": "true",
        "zellij_default_mode": "normal",
        "yazelix_layout_dir": "/tmp/yazelix/layouts",
        "resolved_default_shell": "/usr/bin/nu"
    });

    let mut cmd = Command::cargo_bin("yzx_core").unwrap();
    let output = cmd
        .arg("zellij-render-plan.compute")
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
    assert_eq!(envelope["command"], "zellij-render-plan.compute");
    assert_eq!(envelope["status"], "ok");
    assert_eq!(envelope["data"]["default_layout_name"], "yzx_side");
    assert_eq!(
        envelope["data"]["layout_percentages"]["open_primary_width_percent"],
        "48%"
    );
}

#[test]
fn zellij_render_plan_rejects_bad_sidebar_width() {
    let request = serde_json::json!({
        "sidebar_width_percent": 5,
        "popup_width_percent": 90,
        "popup_height_percent": 90,
        "yazelix_layout_dir": "/tmp/y/layouts",
        "resolved_default_shell": "/bin/sh"
    });

    let mut cmd = Command::cargo_bin("yzx_core").unwrap();
    let output = cmd
        .arg("zellij-render-plan.compute")
        .arg("--request-json")
        .arg(request.to_string())
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(65));
    let envelope: Value = serde_json::from_slice(&output.stderr).unwrap();
    assert_eq!(envelope["status"], "error");
    assert_eq!(envelope["error"]["class"], "config");
    assert_eq!(envelope["error"]["code"], "invalid_sidebar_width_percent");
}
