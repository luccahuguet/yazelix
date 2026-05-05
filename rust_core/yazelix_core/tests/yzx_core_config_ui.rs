// Test lane: default

use std::fs;
use std::path::{Path, PathBuf};
use tempfile::tempdir;
use yazelix_core::config_ui::{
    ConfigUiPathOwner, ConfigUiRequest, ConfigUiValueState, build_config_ui_model,
};
use yazelix_core::ghostty_cursor_registry::DEFAULT_CURSOR_CONFIG_FILENAME;

fn write_runtime_layout(runtime: &Path) {
    fs::create_dir_all(runtime.join("config_metadata")).expect("metadata dir");
    fs::write(
        runtime
            .join("config_metadata")
            .join("main_config_contract.toml"),
        include_str!("../../../config_metadata/main_config_contract.toml"),
    )
    .expect("main config contract");
    fs::write(
        runtime
            .join("config_metadata")
            .join("yazelix_settings.schema.json"),
        include_str!("../../../config_metadata/yazelix_settings.schema.json"),
    )
    .expect("settings schema");
    fs::write(
        runtime.join("yazelix_default.toml"),
        include_str!("../../../yazelix_default.toml"),
    )
    .expect("main defaults");
    fs::write(
        runtime.join(DEFAULT_CURSOR_CONFIG_FILENAME),
        include_str!("../../../yazelix_cursors_default.toml"),
    )
    .expect("cursor defaults");
}

fn request(runtime: PathBuf, config: PathBuf) -> ConfigUiRequest {
    ConfigUiRequest {
        runtime_dir: runtime,
        config_dir: config,
        config_override: None,
    }
}

// Defends: the read-only config UI inventory uses the main contract and cursor schema together, with user-intent tabs and explicit/defaulted states.
// Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
#[test]
fn builds_inventory_tabs_and_value_states() {
    let runtime = tempdir().expect("runtime");
    let config = tempdir().expect("config");
    write_runtime_layout(runtime.path());
    fs::write(
        config.path().join("settings.jsonc"),
        r##"{
          "core": { "debug_mode": true },
          "editor": { "hide_sidebar_on_file_open": true },
          "cursors": { "settings": { "trail": "magma" } }
        }"##,
    )
    .expect("settings");

    let model = build_config_ui_model(&request(
        runtime.path().to_path_buf(),
        config.path().to_path_buf(),
    ))
    .expect("model");

    assert_eq!(
        model.tabs,
        vec![
            "general", "editor", "terminal", "zellij", "yazi", "cursors", "advanced"
        ]
    );
    let debug_mode = model
        .fields
        .iter()
        .find(|field| field.path == "core.debug_mode")
        .expect("debug mode field");
    assert_eq!(debug_mode.tab, "general");
    assert_eq!(debug_mode.state, ConfigUiValueState::Explicit);
    assert_eq!(debug_mode.current_value, "true");

    let zellij_theme = model
        .fields
        .iter()
        .find(|field| field.path == "zellij.theme")
        .expect("zellij theme field");
    assert_eq!(zellij_theme.tab, "zellij");
    assert_eq!(zellij_theme.state, ConfigUiValueState::Defaulted);

    let cursor_trail = model
        .fields
        .iter()
        .find(|field| field.path == "cursors.settings.trail")
        .expect("cursor trail field");
    assert_eq!(cursor_trail.tab, "cursors");
    assert_eq!(cursor_trail.state, ConfigUiValueState::Explicit);
    assert_eq!(cursor_trail.current_value, "\"magma\"");
}

// Defends: advanced config UI state exposes sidecar presence and the Home Manager/read-only ownership signal without mutating config files.
// Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
#[test]
#[cfg(unix)]
fn reports_sidecars_and_home_manager_read_only_state() {
    use std::os::unix::fs::symlink;

    let runtime = tempdir().expect("runtime");
    let config = tempdir().expect("config");
    write_runtime_layout(runtime.path());
    let hm_dir = config.path().join("profile-home-manager-files");
    fs::create_dir_all(&hm_dir).expect("hm dir");
    let hm_settings = hm_dir.join("settings.jsonc");
    fs::write(&hm_settings, "{}").expect("hm settings");
    let mut permissions = fs::metadata(&hm_settings).expect("metadata").permissions();
    permissions.set_readonly(true);
    fs::set_permissions(&hm_settings, permissions).expect("readonly");
    symlink(&hm_settings, config.path().join("settings.jsonc")).expect("settings symlink");
    fs::write(config.path().join("zellij.kdl"), "layout {}\n").expect("zellij sidecar");

    let model = build_config_ui_model(&request(
        runtime.path().to_path_buf(),
        config.path().to_path_buf(),
    ))
    .expect("model");

    assert_eq!(model.config_owner, ConfigUiPathOwner::HomeManager);
    assert!(model.config_read_only);
    let zellij = model
        .sidecars
        .iter()
        .find(|sidecar| sidecar.name == "zellij.kdl")
        .expect("zellij sidecar");
    assert!(zellij.present);
    assert_eq!(zellij.owner, ConfigUiPathOwner::User);
    let yazi_keymap = model
        .sidecars
        .iter()
        .find(|sidecar| sidecar.name == "yazi_keymap.toml")
        .expect("yazi keymap sidecar");
    assert!(!yazi_keymap.present);
}

// Defends: malformed settings.jsonc stops the config UI before rendering stale or misleading values.
// Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
#[test]
fn rejects_invalid_settings_jsonc() {
    let runtime = tempdir().expect("runtime");
    let config = tempdir().expect("config");
    write_runtime_layout(runtime.path());
    fs::write(config.path().join("settings.jsonc"), r#"{ "core": "#).expect("settings");

    let error = build_config_ui_model(&request(
        runtime.path().to_path_buf(),
        config.path().to_path_buf(),
    ))
    .expect_err("invalid jsonc");

    assert_eq!(error.code(), "invalid_settings_jsonc");
}

// Defends: blocking config diagnostics are visible in the config UI model instead of making the read-only browser unusable for stale configs.
// Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
#[test]
fn marks_blocking_diagnostics_without_aborting_model_build() {
    let runtime = tempdir().expect("runtime");
    let config = tempdir().expect("config");
    write_runtime_layout(runtime.path());
    fs::write(
        config.path().join("settings.jsonc"),
        r#"{ "core": { "debug_mode": "yes" } }"#,
    )
    .expect("settings");

    let model = build_config_ui_model(&request(
        runtime.path().to_path_buf(),
        config.path().to_path_buf(),
    ))
    .expect("model");

    let debug_mode = model
        .fields
        .iter()
        .find(|field| field.path == "core.debug_mode")
        .expect("debug mode field");
    assert_eq!(debug_mode.state, ConfigUiValueState::Invalid);
    assert!(
        model
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.path == "core.debug_mode" && diagnostic.blocking)
    );
}
