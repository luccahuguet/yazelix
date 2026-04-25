// Test lane: default

use serde_json::Value;
mod support;

use support::commands::yzx_core_command_in_fixture;
use support::envelopes::ok_envelope;
use support::fixtures::managed_config_fixture;

// Defends: integration-facts.compute returns the Rust-owned sidebar, editor-kind, and Yazi command payload directly.
// Contract: WSS-005, SOE-004
// Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
#[test]
fn integration_facts_compute_reports_sidebar_editor_and_yazi_payload() {
    let fixture = managed_config_fixture(
        r#"[editor]
command = "nvim"
enable_sidebar = false

[yazi]
command = "yy"
ya_command = "ya-test"
"#,
    );

    let output = yzx_core_command_in_fixture(&fixture, "integration-facts.compute")
        .output()
        .unwrap();
    let envelope: Value = ok_envelope(&output);

    assert_eq!(envelope["command"], "integration-facts.compute");
    assert_eq!(envelope["data"]["enable_sidebar"], false);
    assert_eq!(envelope["data"]["managed_editor_kind"], "neovim");
    assert_eq!(envelope["data"]["yazi_command"], "yy");
    assert_eq!(envelope["data"]["ya_command"], "ya-test");
}

// Defends: transient-pane-facts.compute keeps popup argv and geometry under one Rust-owned facts surface for popup/menu callers.
// Contract: POP-001, POP-002, POP-003
// Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
#[test]
fn transient_pane_facts_compute_reports_popup_program_and_geometry() {
    let fixture = managed_config_fixture(
        r#"[zellij]
popup_program = ["gitui", "--theme", "cyan"]
popup_width_percent = 82
popup_height_percent = 76
"#,
    );

    let output = yzx_core_command_in_fixture(&fixture, "transient-pane-facts.compute")
        .output()
        .unwrap();
    let envelope: Value = ok_envelope(&output);

    assert_eq!(envelope["command"], "transient-pane-facts.compute");
    assert_eq!(
        envelope["data"]["popup_program"],
        serde_json::json!(["gitui", "--theme", "cyan"])
    );
    assert_eq!(envelope["data"]["popup_width_percent"], 82);
    assert_eq!(envelope["data"]["popup_height_percent"], 76);
}

// Defends: startup-facts.compute returns the retained welcome, session, shell, and terminal facts without Nushell-side config parsing.
// Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
#[test]
fn startup_facts_compute_reports_retained_startup_and_session_fields() {
    let fixture = managed_config_fixture(
        r#"[core]
debug_mode = true
skip_welcome_screen = true
welcome_style = "static"
game_of_life_cell_style = "dotted"
welcome_duration_seconds = 2.5
show_macchina_on_welcome = false

[shell]
default_shell = "bash"

[terminal]
terminals = ["wezterm", "ghostty"]
config_mode = "user"

[zellij]
persistent_sessions = true
session_name = "demo-session"
"#,
    );

    let output = yzx_core_command_in_fixture(&fixture, "startup-facts.compute")
        .output()
        .unwrap();
    let envelope: Value = ok_envelope(&output);

    assert_eq!(envelope["command"], "startup-facts.compute");
    assert_eq!(envelope["data"]["default_shell"], "bash");
    assert_eq!(envelope["data"]["debug_mode"], true);
    assert_eq!(envelope["data"]["skip_welcome_screen"], true);
    assert_eq!(envelope["data"]["welcome_style"], "static");
    assert_eq!(envelope["data"]["game_of_life_cell_style"], "dotted");
    assert_eq!(envelope["data"]["welcome_duration_seconds"], 2.5);
    assert_eq!(envelope["data"]["show_macchina_on_welcome"], false);
    assert_eq!(envelope["data"]["persistent_sessions"], true);
    assert_eq!(envelope["data"]["session_name"], "demo-session");
    assert_eq!(
        envelope["data"]["terminals"],
        serde_json::json!(["wezterm", "ghostty"])
    );
    assert_eq!(envelope["data"]["terminal_config_mode"], "user");
}
