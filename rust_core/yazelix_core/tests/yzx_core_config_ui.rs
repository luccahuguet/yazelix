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
        runtime
            .join("config_metadata")
            .join("config_ui_metadata.toml"),
        include_str!("../../../config_metadata/config_ui_metadata.toml"),
    )
    .expect("config ui metadata");
    fs::write(
        runtime.join("config_default.toml"),
        include_str!("../../../config_default.toml"),
    )
    .expect("main defaults");
    fs::write(runtime.join("runtime_variant"), "mars\n").expect("runtime variant");
    let mars_config = runtime.join("share/mars/config.toml");
    fs::create_dir_all(mars_config.parent().expect("Mars config parent")).expect("Mars config dir");
    fs::write(
        mars_config,
        "[mars.appearance]\npreset = \"dark\"\n[window]\nopacity = 0.78\n",
    )
    .expect("Mars config");
    fs::write(
        runtime.join(DEFAULT_CURSOR_CONFIG_FILENAME),
        include_str!("../../../yazelix_cursors_default.toml"),
    )
    .expect("cursor defaults");
    fs::write(
        runtime.join("runtime_components.json"),
        r#"{
          "cursors": { "enabled": true, "disableable": true, "notes": [] },
          "screen": { "enabled": true, "disableable": true, "notes": [] }
        }"#,
    )
    .expect("runtime component manifest");
}

fn request(runtime: PathBuf, config: PathBuf) -> ConfigUiRequest {
    ConfigUiRequest {
        runtime_dir: runtime,
        config_dir: config,
        config_override: None,
    }
}

// Defends: advanced config UI state exposes nested sidecar presence and the Home Manager/read-only ownership signal without mutating config files.
#[cfg(unix)]
#[test]
fn reports_sidecars_and_home_manager_read_only_state() {
    use std::os::unix::fs::symlink;

    let runtime = tempdir().expect("runtime");
    let config = tempdir().expect("config");
    write_runtime_layout(runtime.path());
    let hm_dir = config.path().join("profile-home-manager-files");
    fs::create_dir_all(&hm_dir).expect("hm dir");
    let hm_settings = hm_dir.join("config.toml");
    fs::write(&hm_settings, "[core]\ndebug_mode = false\n").expect("hm settings");
    let mut permissions = fs::metadata(&hm_settings).expect("metadata").permissions();
    permissions.set_readonly(true);
    fs::set_permissions(&hm_settings, permissions).expect("readonly");
    symlink(&hm_settings, config.path().join("config.toml")).expect("settings symlink");
    fs::create_dir_all(config.path().join("zellij")).expect("zellij dir");
    fs::write(
        config.path().join("zellij/config.kdl"),
        "mouse_mode false\n",
    )
    .expect("zellij sidecar");

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
        .find(|sidecar| sidecar.name == "zellij/config.kdl")
        .expect("zellij sidecar");
    assert!(zellij.present);
    assert_eq!(zellij.owner, ConfigUiPathOwner::User);
    assert!(
        model
            .sidecars
            .iter()
            .any(|sidecar| sidecar.name == "zellij/plugins.kdl")
    );
    let yazi_keymap = model
        .sidecars
        .iter()
        .find(|sidecar| sidecar.name == "yazi/keymap.toml")
        .expect("yazi keymap sidecar");
    assert!(!yazi_keymap.present);
}

// Defends: malformed config.toml stops the config UI before rendering stale or misleading values.
#[test]
fn rejects_invalid_config_toml() {
    let runtime = tempdir().expect("runtime");
    let config = tempdir().expect("config");
    write_runtime_layout(runtime.path());
    fs::write(
        config.path().join("config.toml"),
        "[core\ndebug_mode = false\n",
    )
    .expect("settings");

    let error = build_config_ui_model(&request(
        runtime.path().to_path_buf(),
        config.path().to_path_buf(),
    ))
    .expect_err("invalid toml");

    assert_eq!(error.code(), "invalid_main_config_toml");
}

// Defends: blocking config diagnostics are visible in the config UI model instead of making the read-only browser unusable for stale configs.
#[test]
fn marks_blocking_diagnostics_without_aborting_model_build() {
    let runtime = tempdir().expect("runtime");
    let config = tempdir().expect("config");
    write_runtime_layout(runtime.path());
    fs::write(
        config.path().join("config.toml"),
        "[core]\ndebug_mode = \"yes\"\n",
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
