// Test lane: default

use serde_json::{Value, json};
use std::fs;

mod support;

use support::commands::{apply_managed_config_env, yzx_control_command};
use support::fixtures::{
    managed_config_fixture, prepend_path, write_executable_script, write_session_config_snapshot,
};

fn yzx_control_command_in_fixture(
    fixture: &support::fixtures::ManagedConfigFixture,
) -> assert_cmd::Command {
    let mut command = yzx_control_command();
    apply_managed_config_env(&mut command, fixture);
    command
}

// Defends: the Rust-owned legacy workspace-retarget route syncs the plugin-owned sidebar from the active-tab session snapshot once.
// Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
#[test]
fn yzx_control_cwd_retargets_workspace_and_syncs_sidebar() {
    let fixture = managed_config_fixture(
        r#"[terminal]
ghostty_trail_color = "random"

[yazi]
ya_command = "config-ya"
"#,
    );
    let snapshot = write_session_config_snapshot(
        &fixture,
        &[("editor_command", json!("hx")), ("ya_command", json!("ya"))],
    );
    let fake_bin = fixture.home_dir.join("fake-bin");
    let target_dir = fixture.home_dir.join("workspace");
    let zellij_commands_log = fixture.home_dir.join("zellij-commands.log");
    let retarget_payload_log = fixture.home_dir.join("retarget-payload.json");
    let ya_log = fixture.home_dir.join("ya.log");
    fs::create_dir_all(&fake_bin).unwrap();
    fs::create_dir_all(&target_dir).unwrap();

    write_executable_script(
        &fake_bin.join("zoxide"),
        &format!(
            "#!/bin/sh\nif [ \"$1\" = \"query\" ] && [ \"$2\" = \"--\" ]; then\n  printf '%s\\n' \"{}\"\n  exit 0\nfi\nprintf 'unexpected zoxide args: %s\\n' \"$*\" >&2\nexit 1\n",
            target_dir.display()
        ),
    );
    write_executable_script(
        &fake_bin.join("zellij"),
        &format!(
            "#!/bin/sh\nprintf '%s\\n' \"$6\" >> \"{}\"\nif [ \"$6\" = \"retarget_workspace\" ]; then\n  printf '%s' \"$8\" > \"{}\"\n  printf '%s\\n' '{{\"status\":\"ok\",\"editor_status\":\"ok\",\"sidebar_yazi_id\":\"plugin-sidebar-yazi-123\",\"sidebar_yazi_cwd\":\"/home/sidebar\"}}'\n  exit 0\nfi\nprintf 'unexpected zellij args: %s\\n' \"$*\" >&2\nexit 1\n",
            zellij_commands_log.display(),
            retarget_payload_log.display()
        ),
    );
    write_executable_script(
        &fake_bin.join("ya"),
        &format!(
            "#!/bin/sh\nprintf '%s\\n' \"$*\" >> \"{}\"\n",
            ya_log.display()
        ),
    );

    let output = yzx_control_command_in_fixture(&fixture)
        .env("PATH", prepend_path(&fake_bin))
        .env("ZELLIJ", "1")
        .env("YAZELIX_SESSION_CONFIG_PATH", snapshot)
        .arg("cwd")
        .arg("workspace-alias")
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(0));
    assert!(output.stderr.is_empty());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let payload: Value = serde_json::from_slice(&fs::read(retarget_payload_log).unwrap()).unwrap();
    assert_eq!(
        fs::read_to_string(zellij_commands_log).unwrap().trim(),
        "retarget_workspace"
    );
    assert_eq!(
        payload["workspace_root"],
        target_dir.to_string_lossy().to_string()
    );
    assert_eq!(payload["cd_focused_pane"], true);
    assert_eq!(payload["editor"], "helix");
    assert_eq!(
        fs::read_to_string(ya_log).unwrap().trim(),
        format!(
            "emit-to plugin-sidebar-yazi-123 cd {}",
            target_dir.display()
        )
    );
    assert!(stdout.contains("Updated current tab workspace directory to"));
    assert!(stdout.contains("Managed editor cwd synced"));
    assert!(stdout.contains("Sidebar Yazi synced"));
}

// Defends: consumers can obtain the current versioned pane-orchestrator status bus without parsing ad hoc logs or generated files.
// Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
#[test]
fn yzx_control_zellij_status_bus_json_reads_versioned_snapshot() {
    let fixture = managed_config_fixture("");
    let fake_bin = fixture.home_dir.join("fake-bin");
    fs::create_dir_all(&fake_bin).unwrap();
    write_executable_script(
        &fake_bin.join("zellij"),
        "#!/bin/sh\nname=''\nprevious=''\nfor arg in \"$@\"; do\n  if [ \"$previous\" = '--name' ]; then name=\"$arg\"; fi\n  previous=\"$arg\"\ndone\nif [ \"$name\" = 'get_active_tab_session_state' ]; then\n  printf '%s\\n' '{\"schema_version\":1,\"active_tab_position\":4,\"workspace\":{\"root\":\"/tmp/project\",\"source\":\"explicit\"},\"managed_panes\":{\"editor_pane_id\":\"terminal:7\",\"sidebar_pane_id\":\"terminal:8\"},\"focus_context\":\"editor\",\"layout\":{\"active_swap_layout_name\":\"single_open\",\"sidebar_collapsed\":false},\"sidebar_yazi\":{\"yazi_id\":\"yazi-123\",\"cwd\":\"/tmp/project\"},\"transient_panes\":{\"popup\":null,\"menu\":null},\"extensions\":{\"ai_pane_activity\":[]}}'\n  exit 0\nfi\nprintf 'unexpected pipe name: %s\\n' \"$name\" >&2\nexit 1\n",
    );

    let output = yzx_control_command_in_fixture(&fixture)
        .env("PATH", prepend_path(&fake_bin))
        .env("ZELLIJ", "1")
        .arg("zellij")
        .arg("status-bus")
        .arg("--json")
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(0));
    assert!(output.stderr.is_empty());
    let snapshot: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(snapshot["schema_version"], 1);
    assert_eq!(snapshot["active_tab_position"], 4);
    assert_eq!(snapshot["workspace"]["root"], "/tmp/project");
    assert_eq!(snapshot["managed_panes"]["editor_pane_id"], "terminal:7");
}

// Defends: hide-on-file-open keeps `yzx reveal` on the managed-sidebar path instead of reviving no-sidebar guidance.
// Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
#[test]
fn yzx_control_reveal_treats_hide_on_file_open_as_managed_sidebar_available() {
    let fixture = managed_config_fixture(
        r#"[editor]
hide_sidebar_on_file_open = true
"#,
    );
    let target_path = fixture.home_dir.join("target.txt");
    fs::write(&target_path, "").unwrap();

    let output = yzx_control_command_in_fixture(&fixture)
        .arg("reveal")
        .arg(&target_path)
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(0));
    assert!(output.stderr.is_empty());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Reveal in Yazi only works inside a Yazelix/Zellij session"));
    assert!(!stdout.contains("no-sidebar mode"));
}

// Defends: the public Rust-owned `yzx reveal` route uses the pane-orchestrator session snapshot as the only sidebar identity source and then focuses the sidebar.
// Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
#[test]
fn yzx_control_reveal_uses_session_snapshot_and_focuses_sidebar() {
    let fixture = managed_config_fixture(
        r#"[editor]
hide_sidebar_on_file_open = false

[yazi]
ya_command = "ya"
"#,
    );
    let fake_bin = fixture.home_dir.join("fake-bin");
    let target_path = fixture.home_dir.join("target.txt");
    let zellij_commands_log = fixture.home_dir.join("zellij-commands.log");
    let ya_log = fixture.home_dir.join("ya.log");
    fs::create_dir_all(&fake_bin).unwrap();
    fs::write(&target_path, "").unwrap();

    write_executable_script(
        &fake_bin.join("zellij"),
        &format!(
            "#!/bin/sh\nprintf '%s\\n' \"$6\" >> \"{}\"\ncase \"$6\" in\n  get_active_tab_session_state)\n    printf '%s\\n' '{{\"schema_version\":1,\"active_tab_position\":0,\"focus_context\":\"editor\",\"managed_panes\":{{\"editor_pane_id\":\"terminal:1\",\"sidebar_pane_id\":\"terminal:2\"}},\"layout\":{{\"active_swap_layout_name\":null,\"sidebar_collapsed\":false}},\"sidebar_yazi\":{{\"yazi_id\":\"plugin-yazi-id\",\"cwd\":\"/home/plugin\"}}}}'\n    exit 0\n    ;;\n  focus_sidebar)\n    printf '%s\\n' 'ok'\n    exit 0\n    ;;\nesac\nprintf 'unexpected zellij args: %s\\n' \"$*\" >&2\nexit 1\n",
            zellij_commands_log.display()
        ),
    );
    write_executable_script(
        &fake_bin.join("ya"),
        &format!(
            "#!/bin/sh\nprintf '%s\\n' \"$*\" >> \"{}\"\n",
            ya_log.display()
        ),
    );

    let output = yzx_control_command_in_fixture(&fixture)
        .env("PATH", prepend_path(&fake_bin))
        .env("ZELLIJ", "1")
        .arg("reveal")
        .arg(&target_path)
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(0));
    assert!(output.stderr.is_empty());
    assert!(output.stdout.is_empty());
    assert_eq!(
        fs::read_to_string(zellij_commands_log)
            .unwrap()
            .lines()
            .collect::<Vec<_>>(),
        vec!["get_active_tab_session_state", "focus_sidebar"]
    );
    assert_eq!(
        fs::read_to_string(ya_log).unwrap().trim(),
        format!("emit-to plugin-yazi-id reveal {}", target_path.display())
    );
}

// Defends: the default Rust-owned `yzx popup` route uses the pane-orchestrator toggle command so repeated invocations manage one popup instead of spawning duplicates.
// Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
#[test]
fn yzx_control_popup_without_override_uses_transient_toggle_contract() {
    let fixture = managed_config_fixture("");
    let fake_bin = fixture.home_dir.join("fake-bin");
    let zellij_commands_log = fixture.home_dir.join("zellij-commands.log");
    let toggle_payload_log = fixture.home_dir.join("toggle-payload.log");
    fs::create_dir_all(&fake_bin).unwrap();

    write_executable_script(
        &fake_bin.join("zellij"),
        &format!(
            "#!/bin/sh\nprintf '%s\\n' \"$6\" >> \"{}\"\nprintf '%s' \"$8\" > \"{}\"\nif [ \"$6\" = \"toggle_transient_pane\" ]; then\n  printf '%s\\n' 'closed'\n  exit 0\nfi\nprintf 'unexpected zellij args: %s\\n' \"$*\" >&2\nexit 1\n",
            zellij_commands_log.display(),
            toggle_payload_log.display()
        ),
    );

    let output = yzx_control_command_in_fixture(&fixture)
        .env("PATH", prepend_path(&fake_bin))
        .env("ZELLIJ", "1")
        .arg("popup")
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(0));
    assert!(output.stderr.is_empty());
    assert!(output.stdout.is_empty());
    assert_eq!(
        fs::read_to_string(zellij_commands_log).unwrap().trim(),
        "toggle_transient_pane"
    );
    assert_eq!(fs::read_to_string(toggle_payload_log).unwrap(), "popup");
}

// Defends: popup command overrides still use an explicit open request so the pane orchestrator receives the one-shot argv, cwd, and runtime contract.
// Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
#[test]
fn yzx_control_popup_override_opens_transient_pane_with_explicit_payload() {
    let fixture = managed_config_fixture("");
    let fake_bin = fixture.home_dir.join("fake-bin");
    let workspace_dir = fixture.home_dir.join("workspace");
    let zellij_commands_log = fixture.home_dir.join("zellij-commands.log");
    let open_payload_log = fixture.home_dir.join("open-payload.json");
    fs::create_dir_all(&fake_bin).unwrap();
    fs::create_dir_all(&workspace_dir).unwrap();

    write_executable_script(
        &fake_bin.join("zellij"),
        &format!(
            "#!/bin/sh\nprintf '%s\\n' \"$6\" >> \"{}\"\ncase \"$6\" in\n  get_active_tab_session_state)\n    printf '%s\\n' '{{\"workspace\":{{\"root\":\"{}\",\"source\":\"explicit\"}}}}'\n    exit 0\n    ;;\n  open_transient_pane)\n    printf '%s' \"$8\" > \"{}\"\n    printf '%s\\n' 'opened'\n    exit 0\n    ;;\nesac\nprintf 'unexpected zellij args: %s\\n' \"$*\" >&2\nexit 1\n",
            zellij_commands_log.display(),
            workspace_dir.display(),
            open_payload_log.display()
        ),
    );

    let output = yzx_control_command_in_fixture(&fixture)
        .env("PATH", prepend_path(&fake_bin))
        .env("ZELLIJ", "1")
        .arg("popup")
        .arg("lazygit")
        .arg("status")
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(0));
    assert!(output.stderr.is_empty());
    assert!(output.stdout.is_empty());
    assert_eq!(
        fs::read_to_string(zellij_commands_log)
            .unwrap()
            .lines()
            .collect::<Vec<_>>(),
        vec!["get_active_tab_session_state", "open_transient_pane"]
    );

    let payload: Value = serde_json::from_slice(&fs::read(open_payload_log).unwrap()).unwrap();
    assert_eq!(payload["kind"], "popup");
    assert_eq!(payload["args"], serde_json::json!(["lazygit", "status"]));
    assert_eq!(payload["cwd"], workspace_dir.to_string_lossy().to_string());
    assert_eq!(
        payload["runtime_dir"],
        fixture.runtime_dir.to_string_lossy().to_string()
    );
}

// Regression: a popup pane already running inside an older session does not strictly reload all user config before starting its program.
// Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
#[test]
fn yzx_control_popup_pane_tolerates_unrelated_unknown_config_fields() {
    let fixture = managed_config_fixture(
        r#"[zellij]
popup_program = ["popup-probe", "hello"]

[terminal]
ghostty_trail_color = "random"
"#,
    );
    let fake_bin = fixture.home_dir.join("fake-bin");
    let popup_log = fixture.home_dir.join("popup.log");
    let zellij_log = fixture.home_dir.join("zellij.log");
    fs::create_dir_all(&fake_bin).unwrap();

    write_executable_script(
        &fake_bin.join("popup-probe"),
        &format!(
            "#!/bin/sh\nprintf 'args=%s\\n' \"$*\" > \"{}\"\nprintf 'editor=%s\\n' \"$EDITOR\" >> \"{}\"\n",
            popup_log.display(),
            popup_log.display()
        ),
    );
    write_executable_script(
        &fake_bin.join("zellij"),
        &format!(
            "#!/bin/sh\nprintf '%s\\n' \"$*\" >> \"{}\"\nexit 0\n",
            zellij_log.display()
        ),
    );

    let output = yzx_control_command_in_fixture(&fixture)
        .env("PATH", prepend_path(&fake_bin))
        .env("ZELLIJ", "1")
        .env("YAZELIX_POPUP_PANE", "true")
        .env("EDITOR", "runtime-editor")
        .arg("popup")
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(0));
    assert!(output.stderr.is_empty());
    assert!(output.stdout.is_empty());
    assert_eq!(
        fs::read_to_string(popup_log).unwrap(),
        "args=hello\neditor=runtime-editor\n"
    );

    let zellij_log = fs::read_to_string(zellij_log).unwrap();
    assert!(zellij_log.contains("action rename-pane yzx_popup"));
    assert!(zellij_log.contains("action close-pane"));
}

// Defends: the Rust-owned Yazi file-open route carries all selected files into the managed editor, then retargets the workspace and syncs the plugin-owned sidebar.
// Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
#[test]
fn yzx_control_zellij_open_editor_reuses_managed_editor_and_syncs_sidebar() {
    let fixture = managed_config_fixture(
        r#"[editor]
command = "nvim"
hide_sidebar_on_file_open = false

[yazi]
ya_command = "ya"
"#,
    );
    let fake_bin = fixture.home_dir.join("fake-bin");
    let target_dir = fixture.home_dir.join("workspace");
    let target_file = target_dir.join("notes.txt");
    let second_target_file = target_dir.join("tasks.txt");
    let zellij_commands_log = fixture.home_dir.join("zellij-commands.log");
    let open_file_payload_log = fixture.home_dir.join("open-file-payload.json");
    let retarget_payload_log = fixture.home_dir.join("retarget-payload.json");
    let ya_log = fixture.home_dir.join("ya.log");
    fs::create_dir_all(&fake_bin).unwrap();
    fs::create_dir_all(&target_dir).unwrap();
    fs::write(&target_file, "").unwrap();
    fs::write(&second_target_file, "").unwrap();

    write_executable_script(
        &fake_bin.join("zellij"),
        &format!(
            "#!/bin/sh\nif [ \"$1\" = \"action\" ] && [ \"$2\" = \"pipe\" ]; then\n  printf '%s\\n' \"$6\" >> \"{}\"\n  case \"$6\" in\n    open_file)\n      printf '%s' \"$8\" > \"{}\"\n      printf '%s\\n' 'ok'\n      exit 0\n      ;;\n    retarget_workspace)\n      printf '%s' \"$8\" > \"{}\"\n      printf '%s\\n' '{{\"status\":\"ok\",\"editor_status\":\"skipped\",\"sidebar_yazi_id\":\"plugin-sidebar-yazi-123\",\"sidebar_yazi_cwd\":\"/home/sidebar\"}}'\n      exit 0\n      ;;\n  esac\nfi\nprintf 'unexpected zellij args: %s\\n' \"$*\" >&2\nexit 1\n",
            zellij_commands_log.display(),
            open_file_payload_log.display(),
            retarget_payload_log.display()
        ),
    );
    write_executable_script(
        &fake_bin.join("ya"),
        &format!(
            "#!/bin/sh\nprintf '%s\\n' \"$*\" >> \"{}\"\n",
            ya_log.display()
        ),
    );

    let output = yzx_control_command_in_fixture(&fixture)
        .env("PATH", prepend_path(&fake_bin))
        .env("ZELLIJ", "1")
        .env("YAZI_ID", "current-yazi")
        .arg("zellij")
        .arg("open-editor")
        .arg(&target_file)
        .arg(&second_target_file)
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(0));
    assert!(output.stderr.is_empty());
    assert!(output.stdout.is_empty());
    assert_eq!(
        fs::read_to_string(zellij_commands_log)
            .unwrap()
            .lines()
            .collect::<Vec<_>>(),
        vec!["open_file", "retarget_workspace"]
    );

    let open_file_payload: Value =
        serde_json::from_slice(&fs::read(open_file_payload_log).unwrap()).unwrap();
    assert_eq!(open_file_payload["editor"], "neovim");
    assert_eq!(
        open_file_payload["file_path"],
        target_file.to_string_lossy().to_string()
    );
    assert_eq!(
        open_file_payload["file_paths"],
        serde_json::json!([
            target_file.to_string_lossy().to_string(),
            second_target_file.to_string_lossy().to_string()
        ])
    );
    assert_eq!(
        open_file_payload["working_dir"],
        target_dir.to_string_lossy().to_string()
    );

    let retarget_payload: Value =
        serde_json::from_slice(&fs::read(retarget_payload_log).unwrap()).unwrap();
    assert_eq!(
        retarget_payload["workspace_root"],
        target_dir.to_string_lossy().to_string()
    );
    assert_eq!(retarget_payload["cd_focused_pane"], false);
    assert!(retarget_payload["editor"].is_null());
    assert_eq!(
        fs::read_to_string(ya_log).unwrap().trim(),
        format!(
            "emit-to plugin-sidebar-yazi-123 cd {}",
            target_dir.display()
        )
    );
}

// Regression: early Yazi file opens carry the active sidebar Yazi identity through the existing retarget pipe instead of waiting for startup sidebar-state registration.
// Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
#[test]
fn yzx_control_zellij_open_editor_passes_current_yazi_state_to_retarget() {
    let fixture = managed_config_fixture(
        r#"[editor]
command = "nvim"
hide_sidebar_on_file_open = false
"#,
    );
    let fake_bin = fixture.home_dir.join("fake-bin");
    let sidebar_dir = fixture.home_dir.join("sidebar-cwd");
    let target_dir = fixture.home_dir.join("workspace");
    let target_file = target_dir.join("notes.txt");
    let retarget_payload_log = fixture.home_dir.join("retarget-payload.json");
    fs::create_dir_all(&fake_bin).unwrap();
    fs::create_dir_all(&sidebar_dir).unwrap();
    fs::create_dir_all(&target_dir).unwrap();
    fs::write(&target_file, "").unwrap();

    write_executable_script(
        &fake_bin.join("zellij"),
        &format!(
            "#!/bin/sh\nif [ \"$1\" = \"action\" ] && [ \"$2\" = \"pipe\" ]; then\n  case \"$6\" in\n    open_file)\n      printf '%s\\n' 'ok'\n      exit 0\n      ;;\n    retarget_workspace)\n      printf '%s' \"$8\" > \"{}\"\n      printf '%s\\n' '{{\"status\":\"ok\",\"editor_status\":\"skipped\"}}'\n      exit 0\n      ;;\n  esac\nfi\nprintf 'unexpected zellij args: %s\\n' \"$*\" >&2\nexit 1\n",
            retarget_payload_log.display(),
        ),
    );

    let output = yzx_control_command_in_fixture(&fixture)
        .current_dir(&sidebar_dir)
        .env("PATH", prepend_path(&fake_bin))
        .env("ZELLIJ", "1")
        .env("ZELLIJ_PANE_ID", "42")
        .env("YAZI_ID", "current-yazi")
        .arg("zellij")
        .arg("open-editor")
        .arg(&target_file)
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(0));
    assert!(output.stderr.is_empty());
    let retarget_payload: Value =
        serde_json::from_slice(&fs::read(retarget_payload_log).unwrap()).unwrap();
    assert_eq!(retarget_payload["sidebar_yazi"]["pane_id"], "terminal:42");
    assert_eq!(retarget_payload["sidebar_yazi"]["yazi_id"], "current-yazi");
    assert_eq!(
        retarget_payload["sidebar_yazi"]["cwd"],
        sidebar_dir.to_string_lossy().to_string()
    );
}

// Defends: hide_sidebar_on_file_open hides the managed sidebar before opening files so the editor is not visibly resized after focus.
// Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
#[test]
fn yzx_control_zellij_open_editor_hides_sidebar_when_configured() {
    let fixture = managed_config_fixture(
        r#"[editor]
command = "nvim"
hide_sidebar_on_file_open = true
"#,
    );
    let fake_bin = fixture.home_dir.join("fake-bin");
    let target_file = fixture.home_dir.join("notes.txt");
    let zellij_commands_log = fixture.home_dir.join("zellij-commands.log");
    fs::create_dir_all(&fake_bin).unwrap();
    fs::write(&target_file, "").unwrap();

    write_executable_script(
        &fake_bin.join("zellij"),
        &format!(
            "#!/bin/sh\nif [ \"$1\" = \"action\" ] && [ \"$2\" = \"pipe\" ]; then\n  printf '%s\\n' \"$6\" >> \"{}\"\n  case \"$6\" in\n    open_file)\n      printf '%s\\n' 'ok'\n      exit 0\n      ;;\n    retarget_workspace)\n      printf '%s\\n' '{{\"status\":\"ok\",\"editor_status\":\"skipped\"}}'\n      exit 0\n      ;;\n    get_active_tab_session_state)\n      printf '%s\\n' '{{\"schema_version\":1,\"layout\":{{\"sidebar_collapsed\":false}}}}'\n      exit 0\n      ;;\n    hide_sidebar)\n      printf '%s\\n' 'ok'\n      exit 0\n      ;;\n  esac\nfi\nprintf 'unexpected zellij args: %s\\n' \"$*\" >&2\nexit 1\n",
            zellij_commands_log.display(),
        ),
    );

    let output = yzx_control_command_in_fixture(&fixture)
        .env("PATH", prepend_path(&fake_bin))
        .env("ZELLIJ", "1")
        .env("YAZI_ID", "current-yazi")
        .arg("zellij")
        .arg("open-editor")
        .arg(&target_file)
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(0));
    assert!(output.stderr.is_empty());
    assert_eq!(
        fs::read_to_string(zellij_commands_log)
            .unwrap()
            .lines()
            .collect::<Vec<_>>(),
        vec![
            "get_active_tab_session_state",
            "hide_sidebar",
            "open_file",
            "retarget_workspace",
        ]
    );
}

// Regression: the first single-Yazi pane cannot visibly hide until the editor pane exists, so a missing-editor open needs one post-create hide pass.
// Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
#[test]
fn yzx_control_zellij_open_editor_hides_sidebar_after_creating_first_editor_pane() {
    let fixture = managed_config_fixture(
        r#"[editor]
command = "nvim"
hide_sidebar_on_file_open = true
"#,
    );
    let fake_bin = fixture.home_dir.join("fake-bin");
    let target_file = fixture.home_dir.join("notes.txt");
    let zellij_commands_log = fixture.home_dir.join("zellij-commands.log");
    fs::create_dir_all(&fake_bin).unwrap();
    fs::write(&target_file, "").unwrap();

    write_executable_script(
        &fake_bin.join("zellij"),
        &format!(
            "#!/bin/sh\nif [ \"$1\" = \"action\" ] && [ \"$2\" = \"pipe\" ]; then\n  printf '%s\\n' \"$6\" >> \"{}\"\n  case \"$6\" in\n    get_active_tab_session_state)\n      printf '%s\\n' '{{\"schema_version\":1,\"layout\":{{\"sidebar_collapsed\":false}}}}'\n      exit 0\n      ;;\n    hide_sidebar)\n      printf '%s\\n' 'ok'\n      exit 0\n      ;;\n    open_file)\n      printf '%s\\n' 'missing'\n      exit 0\n      ;;\n    retarget_workspace)\n      printf '%s\\n' '{{\"status\":\"ok\",\"editor_status\":\"skipped\"}}'\n      exit 0\n      ;;\n  esac\nfi\nif [ \"$1\" = \"run\" ]; then\n  printf '%s\\n' 'run_editor' >> \"{}\"\n  exit 0\nfi\nprintf 'unexpected zellij args: %s\\n' \"$*\" >&2\nexit 1\n",
            zellij_commands_log.display(),
            zellij_commands_log.display(),
        ),
    );

    let output = yzx_control_command_in_fixture(&fixture)
        .env("PATH", prepend_path(&fake_bin))
        .env("ZELLIJ", "1")
        .env("YAZI_ID", "current-yazi")
        .arg("zellij")
        .arg("open-editor")
        .arg(&target_file)
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(0));
    assert!(output.stderr.is_empty());
    assert_eq!(
        fs::read_to_string(zellij_commands_log)
            .unwrap()
            .lines()
            .collect::<Vec<_>>(),
        vec![
            "get_active_tab_session_state",
            "hide_sidebar",
            "open_file",
            "run_editor",
            "retarget_workspace",
            "get_active_tab_session_state",
            "hide_sidebar",
        ]
    );
}

// Regression: nested Yazi-to-Helix file opens keep the Git workspace root instead of retargeting the tab to the file parent.
// Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
#[test]
fn yzx_control_zellij_open_editor_keeps_repo_root_for_nested_helix_file() {
    let fixture = managed_config_fixture(
        r#"[editor]
command = "hx"
hide_sidebar_on_file_open = false

[yazi]
ya_command = "ya"
"#,
    );
    let fake_bin = fixture.home_dir.join("fake-bin");
    let repo_dir = fixture.home_dir.join("workspace");
    let nested_dir = repo_dir.join("crates").join("app").join("src");
    let target_file = nested_dir.join("main.rs");
    let zellij_commands_log = fixture.home_dir.join("zellij-commands.log");
    let open_file_payload_log = fixture.home_dir.join("open-file-payload.json");
    let retarget_payload_log = fixture.home_dir.join("retarget-payload.json");
    let ya_log = fixture.home_dir.join("ya.log");
    fs::create_dir_all(&fake_bin).unwrap();
    fs::create_dir_all(&nested_dir).unwrap();
    fs::write(&target_file, "").unwrap();

    write_executable_script(
        &fake_bin.join("git"),
        &format!(
            "#!/bin/sh\nif [ \"$1\" = \"-C\" ] && [ \"$3\" = \"rev-parse\" ] && [ \"$4\" = \"--show-toplevel\" ]; then\n  printf '%s\\n' \"{}\"\n  exit 0\nfi\nprintf 'unexpected git args: %s\\n' \"$*\" >&2\nexit 1\n",
            repo_dir.display()
        ),
    );
    write_executable_script(
        &fake_bin.join("zellij"),
        &format!(
            "#!/bin/sh\nif [ \"$1\" = \"action\" ] && [ \"$2\" = \"pipe\" ]; then\n  printf '%s\\n' \"$6\" >> \"{}\"\n  case \"$6\" in\n    open_file)\n      printf '%s' \"$8\" > \"{}\"\n      printf '%s\\n' 'ok'\n      exit 0\n      ;;\n    retarget_workspace)\n      printf '%s' \"$8\" > \"{}\"\n      printf '%s\\n' '{{\"status\":\"ok\",\"editor_status\":\"skipped\",\"sidebar_yazi_id\":\"plugin-sidebar-yazi-123\",\"sidebar_yazi_cwd\":\"/home/sidebar\"}}'\n      exit 0\n      ;;\n  esac\nfi\nprintf 'unexpected zellij args: %s\\n' \"$*\" >&2\nexit 1\n",
            zellij_commands_log.display(),
            open_file_payload_log.display(),
            retarget_payload_log.display()
        ),
    );
    write_executable_script(
        &fake_bin.join("ya"),
        &format!(
            "#!/bin/sh\nprintf '%s\\n' \"$*\" >> \"{}\"\n",
            ya_log.display()
        ),
    );

    let output = yzx_control_command_in_fixture(&fixture)
        .env("PATH", prepend_path(&fake_bin))
        .env("ZELLIJ", "1")
        .env("YAZI_ID", "current-yazi")
        .arg("zellij")
        .arg("open-editor")
        .arg(&target_file)
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(0));
    assert!(output.stderr.is_empty());
    assert!(output.stdout.is_empty());
    assert_eq!(
        fs::read_to_string(zellij_commands_log)
            .unwrap()
            .lines()
            .collect::<Vec<_>>(),
        vec!["open_file", "retarget_workspace"]
    );

    let open_file_payload: Value =
        serde_json::from_slice(&fs::read(open_file_payload_log).unwrap()).unwrap();
    assert_eq!(open_file_payload["editor"], "helix");
    assert_eq!(
        open_file_payload["working_dir"],
        repo_dir.to_string_lossy().to_string()
    );

    let retarget_payload: Value =
        serde_json::from_slice(&fs::read(retarget_payload_log).unwrap()).unwrap();
    assert_eq!(
        retarget_payload["workspace_root"],
        repo_dir.to_string_lossy().to_string()
    );
    assert_eq!(retarget_payload["cd_focused_pane"], false);
    assert!(retarget_payload["editor"].is_null());
    assert_eq!(
        fs::read_to_string(ya_log).unwrap().trim(),
        format!(
            "emit-to plugin-sidebar-yazi-123 cd {}",
            nested_dir.display()
        )
    );
}

// Regression: when the managed editor pane is absent, multi-file Yazi open uses the immutable session snapshot even if the live config has newer stale fields.
// Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
#[test]
fn yzx_control_zellij_open_editor_opens_missing_editor_with_all_selected_files() {
    let fixture = managed_config_fixture(
        r#"[editor]
command = "vim"

[yazi]
ya_command = "config-ya"

[terminal]
ghostty_trail_color = "random"
"#,
    );
    let snapshot = write_session_config_snapshot(
        &fixture,
        &[("editor_command", json!("hx")), ("ya_command", json!("ya"))],
    );
    let fake_bin = fixture.home_dir.join("fake-bin");
    let target_dir = fixture.home_dir.join("workspace");
    let target_file = target_dir.join("notes.txt");
    let second_target_file = target_dir.join("tasks.txt");
    let zellij_commands_log = fixture.home_dir.join("zellij-commands.log");
    let open_file_payload_log = fixture.home_dir.join("open-file-payload.json");
    let retarget_payload_log = fixture.home_dir.join("retarget-payload.json");
    let zellij_run_log = fixture.home_dir.join("zellij-run.log");
    let ya_log = fixture.home_dir.join("ya.log");
    fs::create_dir_all(&fake_bin).unwrap();
    fs::create_dir_all(&target_dir).unwrap();
    fs::write(&target_file, "").unwrap();
    fs::write(&second_target_file, "").unwrap();

    write_executable_script(
        &fake_bin.join("zellij"),
        &format!(
            "#!/bin/sh\nif [ \"$1\" = \"action\" ] && [ \"$2\" = \"pipe\" ]; then\n  printf '%s\\n' \"$6\" >> \"{}\"\n  case \"$6\" in\n    open_file)\n      printf '%s' \"$8\" > \"{}\"\n      printf '%s\\n' 'missing'\n      exit 0\n      ;;\n    retarget_workspace)\n      printf '%s' \"$8\" > \"{}\"\n      printf '%s\\n' '{{\"status\":\"ok\",\"editor_status\":\"skipped\",\"sidebar_yazi_id\":\"plugin-sidebar-yazi-123\",\"sidebar_yazi_cwd\":\"/home/sidebar\"}}'\n      exit 0\n      ;;\n  esac\nfi\nif [ \"$1\" = \"run\" ]; then\n  printf '%s\\n' \"$*\" >> \"{}\"\n  exit 0\nfi\nprintf 'unexpected zellij args: %s\\n' \"$*\" >&2\nexit 1\n",
            zellij_commands_log.display(),
            open_file_payload_log.display(),
            retarget_payload_log.display(),
            zellij_run_log.display()
        ),
    );
    write_executable_script(
        &fake_bin.join("ya"),
        &format!(
            "#!/bin/sh\nprintf '%s\\n' \"$*\" >> \"{}\"\n",
            ya_log.display()
        ),
    );

    let output = yzx_control_command_in_fixture(&fixture)
        .env("PATH", prepend_path(&fake_bin))
        .env("ZELLIJ", "1")
        .env("YAZI_ID", "current-yazi")
        .env("YAZELIX_SESSION_CONFIG_PATH", snapshot)
        .arg("zellij")
        .arg("open-editor")
        .arg(&target_file)
        .arg(&second_target_file)
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(0));
    assert!(output.stderr.is_empty());
    assert!(output.stdout.is_empty());
    assert_eq!(
        fs::read_to_string(zellij_commands_log)
            .unwrap()
            .lines()
            .collect::<Vec<_>>(),
        vec!["open_file", "retarget_workspace"]
    );

    let open_file_payload: Value =
        serde_json::from_slice(&fs::read(open_file_payload_log).unwrap()).unwrap();
    assert_eq!(open_file_payload["editor"], "helix");
    assert_eq!(
        open_file_payload["file_paths"],
        serde_json::json!([
            target_file.to_string_lossy().to_string(),
            second_target_file.to_string_lossy().to_string()
        ])
    );

    let run_log = fs::read_to_string(zellij_run_log).unwrap();
    assert!(run_log.contains("--name editor"));
    assert!(run_log.contains(&format!("--cwd {}", target_dir.display())));
    assert!(run_log.contains("YAZI_ID=current-yazi"));
    assert!(run_log.contains("shells/posix/yazelix_hx.sh"));
    assert!(run_log.contains(target_file.to_string_lossy().as_ref()));
    assert!(run_log.contains(second_target_file.to_string_lossy().as_ref()));

    let retarget_payload: Value =
        serde_json::from_slice(&fs::read(retarget_payload_log).unwrap()).unwrap();
    assert_eq!(
        retarget_payload["workspace_root"],
        target_dir.to_string_lossy().to_string()
    );
    assert_eq!(
        fs::read_to_string(ya_log).unwrap().trim(),
        format!(
            "emit-to plugin-sidebar-yazi-123 cd {}",
            target_dir.display()
        )
    );
}

// Defends: the Rust-owned Yazi zoxide route retargets the managed editor cwd and opens a new managed pane when the editor pane is missing.
// Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
#[test]
fn yzx_control_zellij_open_editor_cwd_opens_missing_managed_editor_pane() {
    let fixture = managed_config_fixture(
        r#"[editor]
command = "hx"
hide_sidebar_on_file_open = false

[yazi]
ya_command = "ya"
"#,
    );
    let fake_bin = fixture.home_dir.join("fake-bin");
    let target_dir = fixture.home_dir.join("workspace");
    let zellij_commands_log = fixture.home_dir.join("zellij-commands.log");
    let retarget_payload_log = fixture.home_dir.join("retarget-payload.json");
    let zellij_run_log = fixture.home_dir.join("zellij-run.log");
    let ya_log = fixture.home_dir.join("ya.log");
    fs::create_dir_all(&fake_bin).unwrap();
    fs::create_dir_all(&target_dir).unwrap();

    write_executable_script(
        &fake_bin.join("zellij"),
        &format!(
            "#!/bin/sh\nif [ \"$1\" = \"action\" ] && [ \"$2\" = \"pipe\" ]; then\n  printf '%s\\n' \"$6\" >> \"{}\"\n  if [ \"$6\" = \"retarget_workspace\" ]; then\n    printf '%s' \"$8\" > \"{}\"\n    printf '%s\\n' '{{\"status\":\"ok\",\"editor_status\":\"missing\",\"sidebar_yazi_id\":\"plugin-sidebar-yazi-123\",\"sidebar_yazi_cwd\":\"/home/sidebar\"}}'\n    exit 0\n  fi\nfi\nif [ \"$1\" = \"run\" ]; then\n  printf '%s\\n' \"$*\" >> \"{}\"\n  exit 0\nfi\nprintf 'unexpected zellij args: %s\\n' \"$*\" >&2\nexit 1\n",
            zellij_commands_log.display(),
            retarget_payload_log.display(),
            zellij_run_log.display()
        ),
    );
    write_executable_script(
        &fake_bin.join("ya"),
        &format!(
            "#!/bin/sh\nprintf '%s\\n' \"$*\" >> \"{}\"\n",
            ya_log.display()
        ),
    );

    let output = yzx_control_command_in_fixture(&fixture)
        .env("PATH", prepend_path(&fake_bin))
        .env("ZELLIJ", "1")
        .env("YAZI_ID", "current-yazi")
        .arg("zellij")
        .arg("open-editor-cwd")
        .arg(&target_dir)
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(0));
    assert!(output.stderr.is_empty());
    assert!(output.stdout.is_empty());
    assert_eq!(
        fs::read_to_string(zellij_commands_log).unwrap().trim(),
        "retarget_workspace"
    );

    let retarget_payload: Value =
        serde_json::from_slice(&fs::read(retarget_payload_log).unwrap()).unwrap();
    assert_eq!(
        retarget_payload["workspace_root"],
        target_dir.to_string_lossy().to_string()
    );
    assert_eq!(retarget_payload["cd_focused_pane"], false);
    assert_eq!(retarget_payload["editor"], "helix");

    let run_log = fs::read_to_string(zellij_run_log).unwrap();
    assert!(run_log.contains("--name editor"));
    assert!(run_log.contains(&format!("--cwd {}", target_dir.display())));
    assert!(run_log.contains("YAZI_ID=current-yazi"));
    assert!(run_log.contains("shells/posix/yazelix_hx.sh"));
    assert!(run_log.contains(target_dir.to_string_lossy().as_ref()));
    assert_eq!(
        fs::read_to_string(ya_log).unwrap().trim(),
        format!(
            "emit-to plugin-sidebar-yazi-123 cd {}",
            target_dir.display()
        )
    );
}

// Regression: the Alt+z Yazi zoxide route must honor hide_sidebar_on_file_open before retargeting the editor cwd.
// Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
#[test]
fn yzx_control_zellij_open_editor_cwd_hides_sidebar_when_configured() {
    let fixture = managed_config_fixture(
        r#"[editor]
command = "hx"
hide_sidebar_on_file_open = true
"#,
    );
    let fake_bin = fixture.home_dir.join("fake-bin");
    let target_dir = fixture.home_dir.join("workspace");
    let zellij_commands_log = fixture.home_dir.join("zellij-commands.log");
    fs::create_dir_all(&fake_bin).unwrap();
    fs::create_dir_all(&target_dir).unwrap();

    write_executable_script(
        &fake_bin.join("zellij"),
        &format!(
            "#!/bin/sh\nif [ \"$1\" = \"action\" ] && [ \"$2\" = \"pipe\" ]; then\n  printf '%s\\n' \"$6\" >> \"{}\"\n  case \"$6\" in\n    get_active_tab_session_state)\n      printf '%s\\n' '{{\"schema_version\":1,\"layout\":{{\"sidebar_collapsed\":false}}}}'\n      exit 0\n      ;;\n    hide_sidebar)\n      printf '%s\\n' 'ok'\n      exit 0\n      ;;\n    retarget_workspace)\n      printf '%s\\n' '{{\"status\":\"ok\",\"editor_status\":\"ok\"}}'\n      exit 0\n      ;;\n  esac\nfi\nprintf 'unexpected zellij args: %s\\n' \"$*\" >&2\nexit 1\n",
            zellij_commands_log.display(),
        ),
    );

    let output = yzx_control_command_in_fixture(&fixture)
        .env("PATH", prepend_path(&fake_bin))
        .env("ZELLIJ", "1")
        .arg("zellij")
        .arg("open-editor-cwd")
        .arg(&target_dir)
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(0));
    assert!(output.stderr.is_empty());
    assert_eq!(
        fs::read_to_string(zellij_commands_log)
            .unwrap()
            .lines()
            .collect::<Vec<_>>(),
        vec![
            "get_active_tab_session_state",
            "hide_sidebar",
            "retarget_workspace"
        ]
    );
}

// Regression: Alt+z from the initial single-Yazi pane needs a post-create hide because the closed layout is not applicable before the editor pane exists.
// Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
#[test]
fn yzx_control_zellij_open_editor_cwd_hides_sidebar_after_creating_first_editor_pane() {
    let fixture = managed_config_fixture(
        r#"[editor]
command = "hx"
hide_sidebar_on_file_open = true
"#,
    );
    let fake_bin = fixture.home_dir.join("fake-bin");
    let target_dir = fixture.home_dir.join("workspace");
    let zellij_commands_log = fixture.home_dir.join("zellij-commands.log");
    fs::create_dir_all(&fake_bin).unwrap();
    fs::create_dir_all(&target_dir).unwrap();

    write_executable_script(
        &fake_bin.join("zellij"),
        &format!(
            "#!/bin/sh\nif [ \"$1\" = \"action\" ] && [ \"$2\" = \"pipe\" ]; then\n  printf '%s\\n' \"$6\" >> \"{}\"\n  case \"$6\" in\n    get_active_tab_session_state)\n      printf '%s\\n' '{{\"schema_version\":1,\"layout\":{{\"sidebar_collapsed\":false}}}}'\n      exit 0\n      ;;\n    hide_sidebar)\n      printf '%s\\n' 'ok'\n      exit 0\n      ;;\n    retarget_workspace)\n      printf '%s\\n' '{{\"status\":\"ok\",\"editor_status\":\"missing\"}}'\n      exit 0\n      ;;\n  esac\nfi\nif [ \"$1\" = \"run\" ]; then\n  printf '%s\\n' 'run_editor' >> \"{}\"\n  exit 0\nfi\nprintf 'unexpected zellij args: %s\\n' \"$*\" >&2\nexit 1\n",
            zellij_commands_log.display(),
            zellij_commands_log.display(),
        ),
    );

    let output = yzx_control_command_in_fixture(&fixture)
        .env("PATH", prepend_path(&fake_bin))
        .env("ZELLIJ", "1")
        .env("YAZI_ID", "current-yazi")
        .arg("zellij")
        .arg("open-editor-cwd")
        .arg(&target_dir)
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(0));
    assert!(output.stderr.is_empty());
    assert_eq!(
        fs::read_to_string(zellij_commands_log)
            .unwrap()
            .lines()
            .collect::<Vec<_>>(),
        vec![
            "get_active_tab_session_state",
            "hide_sidebar",
            "retarget_workspace",
            "run_editor",
            "get_active_tab_session_state",
            "hide_sidebar",
        ]
    );
}

// Regression: Alt+z should still open the editor when a live/stale pane-orchestrator reports no sidebar collapsed fact.
// Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
#[test]
fn yzx_control_zellij_open_editor_cwd_continues_when_sidebar_state_unknown() {
    let fixture = managed_config_fixture(
        r#"[editor]
command = "hx"
hide_sidebar_on_file_open = true
"#,
    );
    let fake_bin = fixture.home_dir.join("fake-bin");
    let target_dir = fixture.home_dir.join("workspace");
    let zellij_commands_log = fixture.home_dir.join("zellij-commands.log");
    fs::create_dir_all(&fake_bin).unwrap();
    fs::create_dir_all(&target_dir).unwrap();

    write_executable_script(
        &fake_bin.join("zellij"),
        &format!(
            "#!/bin/sh\nif [ \"$1\" = \"action\" ] && [ \"$2\" = \"pipe\" ]; then\n  printf '%s\\n' \"$6\" >> \"{}\"\n  case \"$6\" in\n    get_active_tab_session_state)\n      printf '%s\\n' '{{\"schema_version\":1,\"layout\":{{\"active_swap_layout_name\":null,\"sidebar_collapsed\":null}}}}'\n      exit 0\n      ;;\n    hide_sidebar)\n      printf '%s\\n' 'unknown_layout'\n      exit 0\n      ;;\n    retarget_workspace)\n      printf '%s\\n' '{{\"status\":\"ok\",\"editor_status\":\"ok\"}}'\n      exit 0\n      ;;\n  esac\nfi\nprintf 'unexpected zellij args: %s\\n' \"$*\" >&2\nexit 1\n",
            zellij_commands_log.display(),
        ),
    );

    let output = yzx_control_command_in_fixture(&fixture)
        .env("PATH", prepend_path(&fake_bin))
        .env("ZELLIJ", "1")
        .arg("zellij")
        .arg("open-editor-cwd")
        .arg(&target_dir)
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(0));
    assert!(output.stderr.is_empty());
    assert_eq!(
        fs::read_to_string(zellij_commands_log)
            .unwrap()
            .lines()
            .collect::<Vec<_>>(),
        vec![
            "get_active_tab_session_state",
            "hide_sidebar",
            "retarget_workspace"
        ]
    );
}

// Defends: the public Rust-owned `yzx doctor --json` surface exposes structured findings without depending on the removed Nu omnibus suite.
// Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
#[test]
fn yzx_control_doctor_json_reports_structured_findings() {
    let fixture = managed_config_fixture("");
    fs::write(
        &fixture.managed_config,
        "[core]\nstale_field = true\nwelcome_style = \"random\"\n",
    )
    .unwrap();

    let output = yzx_control_command_in_fixture(&fixture)
        .arg("doctor")
        .arg("--json")
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(0));
    assert!(output.stderr.is_empty());
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    let stale_config_result = report["results"]
        .as_array()
        .unwrap()
        .iter()
        .find(|result| {
            result["message"]
                .as_str()
                .unwrap_or("")
                .contains("Stale or unsupported yazelix.toml entries detected")
        })
        .expect("stale config result");

    assert_eq!(report["title"], "Yazelix Health Checks");
    assert!(report["summary"]["warning_count"].as_u64().unwrap() >= 1);
    assert!(
        stale_config_result["config_diagnostic_report"]["issue_count"]
            .as_u64()
            .unwrap()
            >= 1
    );
}

// Regression: `yzx doctor --json` must surface mixed Home Manager/default-profile Yazelix ownership before Home Manager activation trips over the package collision.
// Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
#[test]
fn yzx_control_doctor_json_reports_home_manager_profile_collision() {
    let fixture = managed_config_fixture("");
    let hm_store_config = fixture
        .home_dir
        .join("hm-store")
        .join("abc-home-manager-files")
        .join("yazelix.toml");
    let manifest_path = fixture.home_dir.join(".nix-profile").join("manifest.json");
    fs::create_dir_all(hm_store_config.parent().unwrap()).unwrap();
    fs::create_dir_all(manifest_path.parent().unwrap()).unwrap();
    fs::write(&hm_store_config, "[core]\nwelcome_style = \"random\"\n").unwrap();
    fs::write(
        &manifest_path,
        r#"{"elements":{"yazelix":{"active":true,"storePaths":["/nix/store/test-yazelix"]}},"version":3}"#,
    )
    .unwrap();
    fs::remove_file(&fixture.managed_config).unwrap();
    std::os::unix::fs::symlink(&hm_store_config, &fixture.managed_config).unwrap();

    let output = yzx_control_command_in_fixture(&fixture)
        .arg("doctor")
        .arg("--json")
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(0));
    assert!(output.stderr.is_empty());
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    let ownership_result = report["results"]
        .as_array()
        .unwrap()
        .iter()
        .find(|result| {
            result["message"]
                .as_str()
                .unwrap_or("")
                .contains("default Nix profile still contains standalone Yazelix packages")
        })
        .expect("mixed ownership warning");

    assert_eq!(ownership_result["status"], "warn");
    assert!(
        ownership_result["details"]
            .as_str()
            .unwrap_or("")
            .contains("yzx home_manager prepare --apply")
    );
}

// Defends: `yzx doctor --fix-plan --json` exposes exact recovery commands without running the mutating fix flow.
// Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
#[test]
fn yzx_control_doctor_fix_plan_json_reports_recovery_commands() {
    let fixture = managed_config_fixture("");
    let hm_store_config = fixture
        .home_dir
        .join("hm-store")
        .join("abc-home-manager-files")
        .join("yazelix.toml");
    let manifest_path = fixture.home_dir.join(".nix-profile").join("manifest.json");
    fs::create_dir_all(hm_store_config.parent().unwrap()).unwrap();
    fs::create_dir_all(manifest_path.parent().unwrap()).unwrap();
    fs::write(&hm_store_config, "[core]\nwelcome_style = \"random\"\n").unwrap();
    fs::write(
        &manifest_path,
        r#"{"elements":{"yazelix":{"active":true,"storePaths":["/nix/store/test-yazelix"]}},"version":3}"#,
    )
    .unwrap();
    fs::remove_file(&fixture.managed_config).unwrap();
    std::os::unix::fs::symlink(&hm_store_config, &fixture.managed_config).unwrap();

    let output = yzx_control_command_in_fixture(&fixture)
        .arg("doctor")
        .arg("--fix-plan")
        .arg("--json")
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(0));
    assert!(output.stderr.is_empty());
    let plan: Value = serde_json::from_slice(&output.stdout).unwrap();
    let action = plan["actions"]
        .as_array()
        .unwrap()
        .iter()
        .find(|action| action["id"] == "resolve_home_manager_profile_collision")
        .expect("home manager recovery action");

    assert_eq!(plan["title"], "Yazelix Recovery Fix Plan");
    assert_eq!(plan["inspect_command"], "yzx inspect --json");
    assert_eq!(action["safe_to_run_automatically"], false);
    assert!(
        action["commands"]
            .as_array()
            .unwrap()
            .iter()
            .any(|command| command == "yzx home_manager prepare --apply")
    );
    assert!(action["evidence"].as_array().unwrap().iter().any(|line| {
        line.as_str()
            .unwrap_or("")
            .contains("Home Manager now owns")
    }));
}
