// Test lane: default

use serde_json::json;
use std::fs;
use tempfile::tempdir;
use yazelix_core::settings_surface::read_settings_jsonc_value;
use yazelix_core::user_config_paths::shared_cursor_config;

mod support;

use support::commands::yzx_control_command;
use support::fixtures::{
    prepend_path, repo_root, write_executable_script, write_runtime_contract_assets,
};

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

    let cursor_settings_path = shared_cursor_config(&config);
    let cursor_value =
        read_settings_jsonc_value(&cursor_settings_path).expect("cursor settings after cursor set");
    assert_eq!(cursor_value["settings"]["trail"], json!("magma"));
    let value = read_settings_jsonc_value(&settings_path).expect("settings after cursor set");
    assert!(value.get("cursors").is_none());

    let mut unset = yzx_control_command();
    with_config_env(&mut unset, &home, &runtime, &config);
    unset.args(["config", "unset", "editor.hide_sidebar_on_file_open"]);
    unset.assert().success();

    let value = read_settings_jsonc_value(&settings_path).expect("settings after unset");
    assert!(value["editor"].get("hide_sidebar_on_file_open").is_none());
}

// Regression: live-with-pane-refresh config saves emit a versioned pane-orchestrator reload payload instead of leaving the saved value silently inactive.
#[test]
fn config_set_live_zellij_field_reloads_pane_orchestrator_runtime_config() {
    let repo = repo_root();
    let temp = tempdir().expect("tempdir");
    let home = temp.path().join("home");
    let runtime = temp.path().join("runtime");
    let config = temp.path().join("config");
    let state = temp.path().join("state");
    let fake_bin = temp.path().join("fake-bin");
    let payload_log = temp.path().join("reload-payload.json");
    write_runtime_contract_assets(&repo, &runtime);
    fs::create_dir_all(state.join("configs/zellij")).unwrap();
    fs::write(
        state.join("configs/zellij/.yazelix_generation.json"),
        r#"{"fingerprint":"gen-a"}"#,
    )
    .unwrap();
    write_executable_script(
        &fake_bin.join("zellij"),
        &format!(
            "#!/bin/sh\nif [ \"$1\" = \"action\" ] && [ \"$2\" = \"pipe\" ] && [ \"$6\" = \"reload_runtime_config\" ]; then\n  printf '%s' \"$8\" > \"{}\"\n  printf '%s\\n' 'ok'\n  exit 0\nfi\nprintf 'unexpected zellij args: %s\\n' \"$*\" >&2\nexit 1\n",
            payload_log.display()
        ),
    );

    let output = yzx_control_command()
        .env_clear()
        .env("HOME", &home)
        .env("PATH", prepend_path(&fake_bin))
        .env("YAZELIX_RUNTIME_DIR", &runtime)
        .env("YAZELIX_CONFIG_DIR", &config)
        .env("YAZELIX_STATE_DIR", &state)
        .args(["config", "set", "zellij.popup_width_percent", "82"])
        .output()
        .unwrap();

    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Updated zellij.popup_width_percent."));
    assert!(stdout.contains("Refreshed pane-orchestrator runtime config."));
    let payload: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(payload_log).unwrap()).unwrap();
    assert_eq!(payload["schema_version"], json!(1));
    assert_eq!(payload["generation"], json!("gen-a"));
    assert_eq!(payload["runtime_config"]["popup_width_percent"], json!(82));
    assert_eq!(payload["runtime_config"]["popup_height_percent"], json!(90));
    assert_eq!(
        payload["runtime_config"]["screen_saver_enabled"],
        json!(false)
    );
}
