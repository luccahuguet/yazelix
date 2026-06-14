// Test lane: default

use serde_json::Value;
mod support;

use support::commands::yzx_core_command_in_fixture;
use support::envelopes::ok_envelope;
use support::fixtures::{
    managed_config_fixture, write_session_config_snapshot, write_session_config_snapshot_with_id,
};

// Defends: popup-session-facts.compute keeps popup geometry under one Rust-owned facts surface for transient popup requests.
#[test]
fn popup_session_facts_compute_reports_geometry() {
    let fixture = managed_config_fixture(
        r#"[zellij]
popup_width_percent = 82
popup_height_percent = 76
"#,
    );

    let output = yzx_core_command_in_fixture(&fixture, "popup-session-facts.compute")
        .output()
        .unwrap();
    let envelope: Value = ok_envelope(&output);

    assert_eq!(envelope["command"], "popup-session-facts.compute");
    assert_eq!(envelope["data"]["popup_width_percent"], 82);
    assert_eq!(envelope["data"]["popup_height_percent"], 76);
}

// Regression: popup session facts come from the per-session snapshot so popup geometry keeps launch-time config.
#[test]
fn popup_session_facts_compute_prefers_session_snapshot_over_stale_config() {
    let fixture = managed_config_fixture(
        r#"[zellij]
popup_width_percent = 11
popup_height_percent = 12

[terminal]
ghostty_trail_color = "random"
"#,
    );
    let snapshot = write_session_config_snapshot(
        &fixture,
        &[
            ("popup_width_percent", serde_json::json!(82)),
            ("popup_height_percent", serde_json::json!(76)),
        ],
    );

    let output = yzx_core_command_in_fixture(&fixture, "popup-session-facts.compute")
        .env("YAZELIX_SESSION_CONFIG_PATH", snapshot)
        .output()
        .unwrap();
    let envelope: Value = ok_envelope(&output);

    assert_eq!(envelope["command"], "popup-session-facts.compute");
    assert_eq!(envelope["data"]["popup_width_percent"], 82);
    assert_eq!(envelope["data"]["popup_height_percent"], 76);
}

// Regression: different Yazelix windows keep the config snapshot they launched with, even after live config edits.
#[test]
fn popup_session_facts_compute_uses_window_snapshot_identity() {
    let fixture = managed_config_fixture(
        r#"[zellij]
popup_width_percent = 40

[terminal]
ghostty_trail_color = "random"
"#,
    );
    let old_snapshot = write_session_config_snapshot_with_id(
        &fixture,
        "old-window",
        &[("popup_width_percent", serde_json::json!(41))],
    );
    let new_snapshot = write_session_config_snapshot_with_id(
        &fixture,
        "new-window",
        &[("popup_width_percent", serde_json::json!(77))],
    );

    let old_output = yzx_core_command_in_fixture(&fixture, "popup-session-facts.compute")
        .env("YAZELIX_SESSION_CONFIG_PATH", old_snapshot)
        .output()
        .unwrap();
    let new_output = yzx_core_command_in_fixture(&fixture, "popup-session-facts.compute")
        .env("YAZELIX_SESSION_CONFIG_PATH", new_snapshot)
        .output()
        .unwrap();
    let old_envelope: Value = ok_envelope(&old_output);
    let new_envelope: Value = ok_envelope(&new_output);

    assert_eq!(old_envelope["data"]["popup_width_percent"], 41);
    assert_eq!(new_envelope["data"]["popup_width_percent"], 77);
}
