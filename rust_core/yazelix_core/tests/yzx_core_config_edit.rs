// Test lane: default

use serde_json::json;
use tempfile::tempdir;
use yazelix_core::settings_surface::read_settings_jsonc_value;

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

// Defends: the public config edit command uses the lossless settings.jsonc patcher and validates the result before writing.
// Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
#[test]
fn config_set_and_unset_edit_settings_jsonc() {
    let repo = repo_root();
    let temp = tempdir().expect("tempdir");
    let home = temp.path().join("home");
    let runtime = temp.path().join("runtime");
    let config = temp.path().join("config");
    write_runtime_contract_assets(&repo, &runtime);

    let mut set = yzx_control_command();
    with_config_env(&mut set, &home, &runtime, &config);
    set.args(["config", "set", "editor.hide_sidebar_on_file_open", "true"]);
    set.assert().success();

    let settings_path = config.join("settings.jsonc");
    let value = read_settings_jsonc_value(&settings_path).expect("settings after set");
    assert_eq!(value["editor"]["hide_sidebar_on_file_open"], json!(true));

    let mut set_cursor = yzx_control_command();
    with_config_env(&mut set_cursor, &home, &runtime, &config);
    set_cursor.args(["config", "set", "cursors.settings.trail", "\"magma\""]);
    set_cursor.assert().success();

    let value = read_settings_jsonc_value(&settings_path).expect("settings after cursor set");
    assert_eq!(value["cursors"]["settings"]["trail"], json!("magma"));

    let mut unset = yzx_control_command();
    with_config_env(&mut unset, &home, &runtime, &config);
    unset.args(["config", "unset", "editor.hide_sidebar_on_file_open"]);
    unset.assert().success();

    let value = read_settings_jsonc_value(&settings_path).expect("settings after unset");
    assert!(value["editor"].get("hide_sidebar_on_file_open").is_none());
}
