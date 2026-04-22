// Test lane: default

use serde_json::Value;
use std::fs;

mod support;

use support::commands::{apply_managed_config_env, yzx_control_command};
use support::fixtures::{managed_config_fixture, prepend_path, repo_root, write_executable_script};

fn yzx_control_command_in_fixture(
    fixture: &support::fixtures::ManagedConfigFixture,
) -> assert_cmd::Command {
    let mut command = yzx_control_command();
    apply_managed_config_env(&mut command, fixture);
    command
}

// Defends: the public Rust-owned `yzx config --path` route still bootstraps the managed config surface and returns its canonical path.
// Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
#[test]
fn yzx_control_config_path_bootstraps_missing_managed_config() {
    let fixture = managed_config_fixture("");
    fs::remove_file(&fixture.managed_config).unwrap();

    let output = yzx_control_command_in_fixture(&fixture)
        .arg("config")
        .arg("--path")
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(0));
    assert!(fixture.managed_config.is_file());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap().trim(),
        fixture.managed_config.to_string_lossy()
    );

    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("Creating yazelix.toml from yazelix_default.toml"));
    assert!(stderr.contains("yazelix.toml created"));
}

// Defends: the public Rust-owned `yzx status --json` surface keeps the typed runtime summary instead of a wrapper-shaped blob.
// Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
#[test]
fn yzx_control_status_json_reports_typed_summary() {
    let fixture = managed_config_fixture(
        r#"[shell]
default_shell = "nu"

[terminal]
terminals = ["ghostty"]
"#,
    );
    let repo = repo_root();

    let output = yzx_control_command_in_fixture(&fixture)
        .env("YAZELIX_RUNTIME_DIR", &repo)
        .arg("status")
        .arg("--json")
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(0));
    assert!(output.stderr.is_empty());

    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    let summary = &report["summary"];
    assert_eq!(report["title"], "Yazelix status");
    assert!(
        summary["config_file"]
            .as_str()
            .unwrap()
            .ends_with("yazelix.toml")
    );
    assert_eq!(summary["default_shell"], "nu");
    assert_eq!(summary["terminals"], serde_json::json!(["ghostty"]));
    assert!(summary["generated_state_repair_needed"].is_boolean());
    assert!(summary["generated_state_materialization_status"].is_string());
}

// Defends: the public Rust-owned `yzx status --json --versions` surface still attaches the optional tool matrix under one machine-readable report.
// Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
#[test]
fn yzx_control_status_json_versions_includes_tool_matrix() {
    let fixture = managed_config_fixture(
        r#"[terminal]
terminals = ["ghostty"]
"#,
    );
    let repo = repo_root();
    let fake_bin = fixture.home_dir.join("fake-bin");
    fs::create_dir_all(&fake_bin).unwrap();
    write_executable_script(
        &fake_bin.join("nix"),
        "#!/bin/sh\nprintf 'nix (Nix) 2.28.3\\n'\n",
    );
    write_executable_script(
        &fake_bin.join("nu"),
        "#!/bin/sh\nprintf '0.105.1\\n'\n",
    );

    let output = yzx_control_command_in_fixture(&fixture)
        .env("YAZELIX_RUNTIME_DIR", &repo)
        .env("PATH", prepend_path(&fake_bin))
        .arg("status")
        .arg("--json")
        .arg("--versions")
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(0));
    assert!(output.stderr.is_empty());

    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    let versions = report["versions"].as_object().unwrap();
    let tools = versions["tools"].as_array().unwrap();
    let nix_entry = tools
        .iter()
        .find(|entry| entry["tool"] == "nix")
        .expect("nix entry");

    assert_eq!(report["title"], "Yazelix status");
    assert_eq!(versions["title"], "Yazelix Tool Versions");
    assert_eq!(nix_entry["runtime"], "2.28.3");
}

// Defends: the Rust-owned `yzx update upstream` route still fails early for Home Manager-owned installs instead of probing the profile path.
// Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
#[test]
fn yzx_control_update_upstream_rejects_home_manager_owned_install() {
    let fixture = managed_config_fixture("");
    let hm_store_config = fixture
        .home_dir
        .join("hm-store")
        .join("abc-home-manager-files")
        .join("yazelix.toml");
    fs::create_dir_all(hm_store_config.parent().unwrap()).unwrap();
    fs::write(&hm_store_config, "[core]\nwelcome_style = \"random\"\n").unwrap();
    fs::remove_file(&fixture.managed_config).unwrap();
    std::os::unix::fs::symlink(&hm_store_config, &fixture.managed_config).unwrap();

    let output = yzx_control_command_in_fixture(&fixture)
        .arg("update")
        .arg("upstream")
        .output()
        .unwrap();

    assert_ne!(output.status.code(), Some(0));
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("appears to be Home Manager-owned"));
    assert!(stdout.contains("yzx update home_manager"));
    assert!(stdout.contains("home-manager switch"));
}

// Defends: the public Rust-owned `yzx run` route preserves child dash flags end to end instead of stealing them as wrapper options.
// Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
#[test]
fn yzx_control_run_preserves_child_dash_flags_end_to_end() {
    let fixture = managed_config_fixture("");
    let fake_bin = fixture.home_dir.join("fake-bin");
    let command_log = fixture.home_dir.join("child-command.log");
    fs::create_dir_all(&fake_bin).unwrap();
    write_executable_script(
        &fake_bin.join("cargo"),
        &format!(
            "#!/bin/sh\nprintf '%s\\n' \"$*\" > \"{}\"\n",
            command_log.display()
        ),
    );

    let output = yzx_control_command_in_fixture(&fixture)
        .env("PATH", prepend_path(&fake_bin))
        .arg("run")
        .arg("cargo")
        .arg("--verbose")
        .arg("check")
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(0));
    assert!(output.stderr.is_empty());
    assert_eq!(
        fs::read_to_string(command_log).unwrap().trim(),
        "--verbose check"
    );
}
