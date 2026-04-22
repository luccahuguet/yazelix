// Test lane: default

use serde_json::Value;
use std::fs;

mod support;

use support::commands::{apply_managed_config_env, yzx_control_command};
use support::fixtures::{managed_config_fixture, prepend_path, write_executable_script};

fn yzx_control_command_in_fixture(
    fixture: &support::fixtures::ManagedConfigFixture,
) -> assert_cmd::Command {
    let mut command = yzx_control_command();
    apply_managed_config_env(&mut command, fixture);
    command
}

// Defends: the public Rust-owned `yzx cwd` route retargets the active tab through the pane orchestrator and syncs the plugin-owned sidebar once.
// Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
#[test]
fn yzx_control_cwd_retargets_workspace_and_syncs_sidebar() {
    let fixture = managed_config_fixture(
        r#"[editor]
command = "hx"
enable_sidebar = true

[yazi]
ya_command = "ya"
"#,
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

// Defends: the public Rust-owned `yzx reveal` route keeps the sidebar-disabled guidance instead of failing through missing session state.
// Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
#[test]
fn yzx_control_reveal_keeps_sidebar_disabled_guidance() {
    let fixture = managed_config_fixture(
        r#"[editor]
enable_sidebar = false
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
    assert!(stdout.contains("Reveal in Yazi only works in sidebar mode"));
    assert!(stdout.contains("enable sidebar mode in yazelix.toml"));
}

// Defends: the public Rust-owned `yzx reveal` route uses the pane-orchestrator session snapshot as the only sidebar identity source and then focuses the sidebar.
// Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
#[test]
fn yzx_control_reveal_uses_session_snapshot_and_focuses_sidebar() {
    let fixture = managed_config_fixture(
        r#"[editor]
enable_sidebar = true

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
