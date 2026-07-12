// Test lane: default
use super::*;
use crate::ghostty_cursor_registry::DEFAULT_CURSOR_CONFIG_FILENAME;
use tempfile::{TempDir, tempdir};

fn write_runtime_layout(runtime: &Path) {
    fs::create_dir_all(runtime.join("config_metadata")).unwrap();
    for (name, contents) in [
        (
            "main_config_contract.toml",
            include_str!("../../../../config_metadata/main_config_contract.toml"),
        ),
        (
            "classic_main_config_contract.toml",
            include_str!("../../../../config_metadata/classic_main_config_contract.toml"),
        ),
        (
            "classic_config_default.toml",
            include_str!("../../../../config_metadata/classic_config_default.toml"),
        ),
    ] {
        fs::write(runtime.join("config_metadata").join(name), contents).unwrap();
    }
    fs::write(
        runtime.join("config_metadata/yazelix_settings.schema.json"),
        include_str!("../../../../config_metadata/yazelix_settings.schema.json"),
    )
    .unwrap();
    fs::write(
        runtime.join("config_metadata/config_ui_metadata.toml"),
        include_str!("../../../../config_metadata/config_ui_metadata.toml"),
    )
    .unwrap();
    fs::write(
        runtime.join("config_default.toml"),
        include_str!("../../../../config_default.toml"),
    )
    .unwrap();
    fs::write(runtime.join("runtime_variant"), "mars\n").unwrap();
    let mars = runtime.join("share/mars/config.toml");
    fs::create_dir_all(mars.parent().unwrap()).unwrap();
    fs::write(
        mars,
        "confirm-before-quit = true\n[window]\nopacity = 0.78\n",
    )
    .unwrap();
    fs::write(
        runtime.join(DEFAULT_CURSOR_CONFIG_FILENAME),
        yazelix_cursors::DEFAULT_CURSOR_CONFIG_TEMPLATE,
    )
    .unwrap();
    fs::write(
        runtime.join("runtime_components.json"),
        r#"{
          "cursors": { "enabled": true, "disableable": true, "notes": [] },
          "screen": { "enabled": true, "disableable": true, "notes": [] }
        }"#,
    )
    .unwrap();
}

struct Fixture {
    runtime: TempDir,
    config: TempDir,
}

impl Fixture {
    fn new() -> Self {
        let runtime = tempdir().unwrap();
        let config = tempdir().unwrap();
        write_runtime_layout(runtime.path());
        Self { runtime, config }
    }

    fn request(&self) -> ConfigUiRequest {
        ConfigUiRequest {
            runtime_dir: self.runtime.path().to_path_buf(),
            config_dir: self.config.path().to_path_buf(),
            config_override: None,
        }
    }

    fn model(&self) -> ConfigUiModel {
        build_config_ui_model(&self.request()).unwrap()
    }

    fn app(&self) -> YazelixConfigUiApp {
        let request = self.request();
        let model = build_config_ui_model(&request).unwrap();
        YazelixConfigUiApp::new(request, model)
    }

    fn root(&self) -> PathBuf {
        self.config.path().join("config.toml")
    }
}

fn field<'a>(model: &'a ConfigUiModel, path: &str) -> &'a ConfigUiField {
    model
        .fields
        .iter()
        .find(|field| field.source_id == SETTINGS_SOURCE_ID && field.path == path)
        .expect("field")
}

// Defends: Ratconfig exposes exactly the static Nova semantic inventory and no retired Classic writer.
#[test]
fn model_exposes_only_nova_root_fields() {
    let fixture = Fixture::new();
    let model = fixture.model();
    let paths = model
        .fields
        .iter()
        .filter(|field| field.source_id == SETTINGS_SOURCE_ID)
        .map(|field| field.path.as_str())
        .collect::<BTreeSet<_>>();

    assert_eq!(
        paths,
        BTreeSet::from([
            "agent.args",
            "agent.command",
            "bar.widgets",
            "editor.command",
            "keybindings.agent",
            "keybindings.config",
            "keybindings.git",
            "keybindings.menu",
            "open.log_level",
            "popup.side_margin",
            "popup.vertical_margin",
            "shell.program",
            "welcome.duration_seconds",
            "welcome.enabled",
            "welcome.style",
        ])
    );
    assert!(!paths.iter().any(|path| path.starts_with("workspace.")));
    assert!(!paths.iter().any(|path| path.starts_with("zellij.")));
}

// Defends: a static Nova save writes only the selected sparse field and reloads its explicit state.
#[test]
fn root_write_stays_sparse_and_nova_native() {
    let fixture = Fixture::new();
    let mut app = fixture.app();

    app.write_source_field_value(SETTINGS_SOURCE_ID, "welcome.enabled", &json!(false))
        .unwrap();

    let raw = fs::read_to_string(fixture.root()).unwrap();
    assert_eq!(raw, "[welcome]\nenabled = false\n");
    assert_eq!(
        field(&app.model, "welcome.enabled").state,
        ConfigUiValueState::Explicit
    );
    assert_eq!(
        field(&app.model, "shell.program").state,
        ConfigUiValueState::Defaulted
    );
}

// Regression: opening Ratconfig activates the backup-first Classic root migration before reading fields.
#[test]
fn model_migrates_classic_root_before_reading() {
    let fixture = Fixture::new();
    fs::write(
        fixture.root(),
        "[core]\nskip_welcome_screen = true\n[workspace.right_sidebar]\ncommand = \"codex\"\nargs = []\n",
    )
    .unwrap();

    let model = fixture.model();
    let migrated = read_config_value(&fixture.root()).unwrap();

    assert_eq!(migrated["welcome"]["enabled"], json!(false));
    assert_eq!(migrated["agent"]["command"], json!("codex"));
    assert_eq!(field(&model, "welcome.enabled").current_value, "false");
    assert!(
        fs::read_dir(fixture.config.path())
            .unwrap()
            .filter_map(Result::ok)
            .any(|entry| entry.file_name().to_string_lossy().contains(".backup-"))
    );
}

// Defends: generic Mars rows remain child-shaped and sparse after the semantic cutover.
#[test]
fn mars_rows_and_writes_remain_native() {
    let fixture = Fixture::new();
    let mut app = fixture.app();
    let model = &app.model;

    assert_eq!(
        model
            .fields
            .iter()
            .find(|field| field.source_id == MARS_SOURCE_ID && field.path == "window.opacity")
            .unwrap()
            .current_value,
        "0.78"
    );

    app.write_source_field_value(MARS_SOURCE_ID, "window.opacity", &json!(0.5))
        .unwrap();
    let raw = fs::read_to_string(user_config_paths::mars_config(fixture.config.path())).unwrap();
    assert_eq!(raw, "[window]\nopacity = 0.5\n");
}

// Defends: cursor rows remain independently owned after the semantic root cutover.
#[test]
fn cursor_source_remains_separate_from_config_toml() {
    let fixture = Fixture::new();
    let mut app = fixture.app();

    app.write_source_field_value(CURSORS_SOURCE_ID, "cursors.settings.trail", &json!("magma"))
        .unwrap();

    assert!(!fixture.root().exists());
    let cursors =
        read_config_value(&user_config_paths::cursor_config(fixture.config.path())).unwrap();
    assert_eq!(cursors["settings"]["trail"], json!("magma"));
}

// Defends: a Nova root owned by Home Manager is visible but cannot be mutated through Ratconfig.
#[cfg(unix)]
#[test]
fn home_manager_nova_root_is_read_only() {
    use std::os::unix::fs::symlink;

    let fixture = Fixture::new();
    let hm_dir = fixture.config.path().join("profile-home-manager-files");
    fs::create_dir_all(&hm_dir).unwrap();
    let target = hm_dir.join("config.toml");
    fs::write(&target, "[welcome]\nenabled = false\n").unwrap();
    symlink(&target, fixture.root()).unwrap();

    let mut app = fixture.app();
    assert_eq!(
        app.model
            .sources
            .iter()
            .find(|source| source.id == SETTINGS_SOURCE_ID)
            .unwrap()
            .owner,
        ConfigUiPathOwner::HomeManager
    );
    let error = app
        .write_source_field_value(SETTINGS_SOURCE_ID, "welcome.enabled", &json!(true))
        .unwrap_err();
    assert_eq!(error.code(), "home_manager_owned_config");
    assert_eq!(
        fs::read_to_string(target).unwrap(),
        "[welcome]\nenabled = false\n"
    );
}

// Regression: read-only labels alone did not stop Ratconfig from opening Home Manager-owned native files in an editor.
#[cfg(unix)]
#[test]
fn home_manager_native_file_actions_are_disabled_with_exact_remediation() {
    use std::os::unix::fs::symlink;

    let fixture = Fixture::new();
    let hm_dir = fixture.config.path().join("profile-home-manager-files");
    fs::create_dir_all(&hm_dir).unwrap();

    let cursor_target = hm_dir.join("cursors.toml");
    fs::write(
        &cursor_target,
        yazelix_cursors::DEFAULT_CURSOR_CONFIG_TEMPLATE,
    )
    .unwrap();
    symlink(
        &cursor_target,
        user_config_paths::cursor_config(fixture.config.path()),
    )
    .unwrap();

    let mars_target = hm_dir.join("mars.toml");
    fs::write(&mars_target, "[window]\nopacity = 0.9\n").unwrap();
    let mars_path = user_config_paths::mars_config(fixture.config.path());
    fs::create_dir_all(mars_path.parent().unwrap()).unwrap();
    symlink(&mars_target, &mars_path).unwrap();

    let model = fixture.model();
    for (action_id, option) in [
        (CURSORS_CONFIG_ACTION_ID, "programs.yazelix.config.cursors"),
        (MARS_CONFIG_ACTION_ID, "programs.yazelix.config.mars"),
    ] {
        let action = model
            .file_actions
            .iter()
            .find(|action| action.action_id == action_id)
            .unwrap();
        assert!(action.read_only);
        assert!(
            action
                .disabled_reason
                .as_deref()
                .is_some_and(|reason| reason.contains(option))
        );
    }

    let mut app = fixture.app();
    let error = app
        .write_source_field_value(CURSORS_SOURCE_ID, "cursors.settings.trail", &json!("reef"))
        .unwrap_err();
    assert_eq!(
        error.remediation(),
        "Edit programs.yazelix.config.cursors, then run home-manager switch."
    );
}
