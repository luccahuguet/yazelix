// Test lane: default

use std::fs;
use tempfile::tempdir;

mod support;

use support::commands::yzx_control_command;
use support::envelopes::stdout_text;

// Defends: the public `yzx tutor begin/list` flow exposes concrete lessons and the first workspace mini quest through the actual CLI binary.
#[test]
fn yzx_control_tutor_begin_and_list_expose_guided_lessons() {
    let list_output = yzx_control_command()
        .args(["tutor", "list"])
        .output()
        .unwrap();
    let list_stdout = stdout_text(list_output);
    assert!(list_stdout.contains("Yazelix tutor lessons"));
    assert!(list_stdout.contains("yzx tutor workspace"));
    assert!(list_stdout.contains("yzx tutor discovery"));

    let begin_output = yzx_control_command()
        .args(["tutor", "begin"])
        .output()
        .unwrap();
    let begin_stdout = stdout_text(begin_output);
    assert!(begin_stdout.contains("Mini quest"));
    assert!(begin_stdout.contains("yzx enter"));
    assert!(begin_stdout.contains("yzx launch --path <dir>"));
    assert!(begin_stdout.contains("yzx keys"));
    assert!(!begin_stdout.contains("yzx cwd"));
}

// Defends: the Rust-owned `yzx whats_new` command still renders the current-version summary and marks the version seen in Yazelix-managed state.
#[test]
fn yzx_control_whats_new_renders_current_summary_and_marks_seen() {
    let tmp = tempdir().unwrap();
    let runtime_dir = tmp.path().join("runtime");
    let state_dir = tmp.path().join("state");
    fs::create_dir_all(runtime_dir.join("docs")).unwrap();
    fs::create_dir_all(&state_dir).unwrap();
    fs::write(runtime_dir.join("CHANGELOG.md"), "# Changelog\n").unwrap();
    fs::write(
        runtime_dir.join("runtime_identity.json"),
        r#"{
          "schema_version": 1,
          "version": "v15.4",
          "runtime_variant": "ghostty",
          "source": {
            "revision": "0123456789abcdef0123456789abcdef01234567",
            "short_revision": "0123456"
          }
        }"#,
    )
    .unwrap();
    fs::write(
        runtime_dir.join("docs/upgrade_notes.toml"),
        r#"
[releases."v15.4"]
headline = "Config migration follow-up after the v15.4 upgrade"
summary = ["Retain the public workspace core while front-door owners move to Rust."]
upgrade_impact = "migration_available"
migration_ids = ["remove_zellij_widget_tray_layout"]
"#,
    )
    .unwrap();

    let output = yzx_control_command()
        .arg("whats_new")
        .env("YAZELIX_RUNTIME_DIR", &runtime_dir)
        .env("YAZELIX_STATE_DIR", &state_dir)
        .output()
        .unwrap();

    let stdout = stdout_text(output);
    assert!(stdout.contains("What's New In Yazelix v15.4"));
    assert!(stdout.contains("historical release included config-shape changes"));
    assert!(stdout.contains("yzx reset config"));

    let state_path = state_dir.join("state/upgrade_summary/last_seen_version.txt");
    assert_eq!(fs::read_to_string(state_path).unwrap().trim(), "v15.4");
}
