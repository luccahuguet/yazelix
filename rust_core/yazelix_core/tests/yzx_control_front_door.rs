// Test lane: default

use std::fs;
use tempfile::tempdir;

mod support;

use support::commands::yzx_control_command;
use support::envelopes::stdout_text;

// Defends: the Rust-owned `yzx tutor` root still exposes the managed-workspace guided overview instead of regressing to a thin wrapper.
// Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
#[test]
fn yzx_control_tutor_root_keeps_guided_overview() {
    let output = yzx_control_command().arg("tutor").output().unwrap();
    let stdout = stdout_text(output);
    assert!(stdout.contains("Yazelix tutor"));
    assert!(stdout.contains("yzx tutor begin"));
    assert!(stdout.contains("yzx tutor list"));
    assert!(stdout.contains("yzx launch"));
    assert!(stdout.contains("yzx menu"));
    assert!(stdout.contains("yzx doctor"));
}

// Defends: the public `yzx tutor begin/list` flow exposes concrete lessons and the first workspace mini quest through the actual CLI binary.
// Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
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
    assert!(begin_stdout.contains("yzx warp ."));
    assert!(begin_stdout.contains("yzx keys yazi"));
}

// Defends: the Rust-owned `yzx whats_new` command still renders the current-version summary and marks the version seen in Yazelix-managed state.
// Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
#[test]
fn yzx_control_whats_new_renders_current_summary_and_marks_seen() {
    let tmp = tempdir().unwrap();
    let runtime_dir = tmp.path().join("runtime");
    let state_dir = tmp.path().join("state");
    fs::create_dir_all(runtime_dir.join("docs")).unwrap();
    fs::create_dir_all(runtime_dir.join("nushell/scripts/utils")).unwrap();
    fs::create_dir_all(&state_dir).unwrap();
    fs::write(runtime_dir.join("CHANGELOG.md"), "# Changelog\n").unwrap();
    fs::write(
        runtime_dir.join("nushell/scripts/utils/constants.nu"),
        "export const YAZELIX_VERSION = \"v15.4\"\n",
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
