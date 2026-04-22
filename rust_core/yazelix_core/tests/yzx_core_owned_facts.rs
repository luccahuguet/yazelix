// Test lane: default

use assert_cmd::Command;
use serde_json::Value;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repo root")
}

struct OwnedFactsFixture {
    _tmp: TempDir,
    home_dir: PathBuf,
    runtime_dir: PathBuf,
    config_dir: PathBuf,
}

fn prepare_owned_facts_fixture(raw_config: &str) -> OwnedFactsFixture {
    let repo = repo_root();
    let tmp = TempDir::new().unwrap();
    let home_dir = tmp.path().join("home");
    let runtime_dir = tmp.path().join("runtime");
    let config_dir = home_dir.join(".config").join("yazelix");
    let managed_config = config_dir.join("user_configs").join("yazelix.toml");

    fs::create_dir_all(runtime_dir.join("config_metadata")).unwrap();
    fs::create_dir_all(managed_config.parent().unwrap()).unwrap();
    fs::create_dir_all(&home_dir).unwrap();

    fs::copy(
        repo.join("yazelix_default.toml"),
        runtime_dir.join("yazelix_default.toml"),
    )
    .unwrap();
    fs::copy(
        repo.join("config_metadata/main_config_contract.toml"),
        runtime_dir.join("config_metadata/main_config_contract.toml"),
    )
    .unwrap();
    fs::write(runtime_dir.join(".taplo.toml"), "[format]\n").unwrap();
    fs::write(&managed_config, raw_config).unwrap();

    OwnedFactsFixture {
        _tmp: tmp,
        home_dir,
        runtime_dir,
        config_dir,
    }
}

fn owned_facts_command(fixture: &OwnedFactsFixture, helper_command: &str) -> Command {
    let mut command = Command::cargo_bin("yzx_core").unwrap();
    command
        .arg(helper_command)
        .env_clear()
        .env("HOME", &fixture.home_dir)
        .env("PATH", std::env::var("PATH").unwrap_or_default())
        .env("YAZELIX_RUNTIME_DIR", &fixture.runtime_dir)
        .env("YAZELIX_CONFIG_DIR", &fixture.config_dir);
    command
}

fn envelope_data(output: &std::process::Output) -> Value {
    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let envelope: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(envelope["status"], "ok");
    envelope
}

// Defends: integration-facts.compute returns the Rust-owned sidebar, editor-kind, and Yazi command payload directly.
// Contract: WSS-005, SOE-004
// Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
#[test]
fn integration_facts_compute_reports_sidebar_editor_and_yazi_payload() {
    let fixture = prepare_owned_facts_fixture(
        r#"[editor]
command = "nvim"
enable_sidebar = false

[yazi]
command = "yy"
ya_command = "ya-test"
"#,
    );

    let output = owned_facts_command(&fixture, "integration-facts.compute")
        .output()
        .unwrap();
    let envelope = envelope_data(&output);

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
    let fixture = prepare_owned_facts_fixture(
        r#"[zellij]
popup_program = ["gitui", "--theme", "cyan"]
popup_width_percent = 82
popup_height_percent = 76
"#,
    );

    let output = owned_facts_command(&fixture, "transient-pane-facts.compute")
        .output()
        .unwrap();
    let envelope = envelope_data(&output);

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
    let fixture = prepare_owned_facts_fixture(
        r#"[core]
debug_mode = true
skip_welcome_screen = true
welcome_style = "static"
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

    let output = owned_facts_command(&fixture, "startup-facts.compute")
        .output()
        .unwrap();
    let envelope = envelope_data(&output);

    assert_eq!(envelope["command"], "startup-facts.compute");
    assert_eq!(envelope["data"]["default_shell"], "bash");
    assert_eq!(envelope["data"]["debug_mode"], true);
    assert_eq!(envelope["data"]["skip_welcome_screen"], true);
    assert_eq!(envelope["data"]["welcome_style"], "static");
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
