// Test lane: default

use serde_json::json;
use std::fs;
use tempfile::tempdir;
use yazelix_core::settings_surface::read_config_value;
use yazelix_core::user_config_paths::cursor_config;

mod support;

use support::commands::yzx_control_command;
use support::fixtures::{repo_root, write_runtime_contract_assets};

fn with_config_env(
    command: &mut assert_cmd::Command,
    home: &std::path::Path,
    runtime: &std::path::Path,
    config: &std::path::Path,
) {
    command
        .env_clear()
        .env("HOME", home)
        .env("PATH", std::env::var("PATH").unwrap_or_default())
        .env("YAZELIX_RUNTIME_DIR", runtime)
        .env("YAZELIX_CONFIG_DIR", config);
}

// Defends: the public config edit command uses the lossless TOML adapter and validates the result before writing.
#[test]
fn config_set_and_unset_edit_config_toml() {
    let repo = repo_root();
    let temp = tempdir().expect("tempdir");
    let home = temp.path().join("home");
    let runtime = temp.path().join("runtime");
    let config = temp.path().join("config");
    write_runtime_contract_assets(&repo, &runtime);

    let mut set = yzx_control_command();
    with_config_env(&mut set, &home, &runtime, &config);
    set.args(["config", "set", "welcome.enabled", "true"]);
    set.assert().success();

    let settings_path = config.join("config.toml");
    let value = read_config_value(&settings_path).expect("settings after set");
    assert_eq!(value["welcome"]["enabled"], json!(true));
    assert_eq!(value.as_object().unwrap().len(), 1);

    let mut set_cursor = yzx_control_command();
    with_config_env(&mut set_cursor, &home, &runtime, &config);
    set_cursor.args(["config", "set", "cursors.settings.trail", "\"magma\""]);
    let expected = "Updated cursors.settings.trail.\nApply: New shell or terminal.\n";
    set_cursor.assert().success().stdout(expected);

    let cursors = read_config_value(&cursor_config(&config)).unwrap();
    assert_eq!(cursors["settings"]["trail"], json!("magma"));
    let settings = read_config_value(&settings_path).unwrap();
    assert!(settings.get("cursors").is_none());

    let mut unset = yzx_control_command();
    with_config_env(&mut unset, &home, &runtime, &config);
    unset.args(["config", "unset", "welcome.enabled"]);
    unset.assert().success();

    assert!(!settings_path.exists());
}

// Defends: an inherited root has no explicit document content to print and remains absent.
#[test]
fn config_show_is_empty_when_every_value_is_inherited() {
    let repo = repo_root();
    let temp = tempdir().expect("tempdir");
    let home = temp.path().join("home");
    let runtime = temp.path().join("runtime");
    let config = temp.path().join("config");
    write_runtime_contract_assets(&repo, &runtime);

    let mut show = yzx_control_command();
    with_config_env(&mut show, &home, &runtime, &config);
    let output = show.args(["config"]).output().unwrap();

    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(!config.join("config.toml").exists());
}

// Regression: unset removes a semantically empty root even when the selected key was already absent.
#[test]
fn config_unset_removes_preexisting_empty_root() {
    let repo = repo_root();
    let temp = tempdir().expect("tempdir");
    let home = temp.path().join("home");
    let runtime = temp.path().join("runtime");
    let config = temp.path().join("config");
    write_runtime_contract_assets(&repo, &runtime);
    fs::create_dir_all(&config).unwrap();
    fs::write(config.join("config.toml"), "[welcome]\n").unwrap();

    let mut unset = yzx_control_command();
    with_config_env(&mut unset, &home, &runtime, &config);
    unset
        .args(["config", "unset", "welcome.enabled"])
        .assert()
        .success();

    assert!(!config.join("config.toml").exists());
}
