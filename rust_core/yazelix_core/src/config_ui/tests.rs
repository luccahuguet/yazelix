// Test lane: default
use super::*;
use crate::ghostty_cursor_registry::DEFAULT_CURSOR_CONFIG_FILENAME;
use tempfile::{TempDir, tempdir};

fn write_runtime_layout(runtime: &Path) {
    fs::create_dir_all(runtime.join("config_metadata")).expect("metadata dir");
    fs::write(
        runtime
            .join("config_metadata")
            .join("main_config_contract.toml"),
        include_str!("../../../../config_metadata/main_config_contract.toml"),
    )
    .expect("main config contract");
    fs::write(
        runtime
            .join("config_metadata")
            .join("yazelix_settings.schema.json"),
        include_str!("../../../../config_metadata/yazelix_settings.schema.json"),
    )
    .expect("settings schema");
    fs::write(
        runtime
            .join("config_metadata")
            .join("config_ui_metadata.toml"),
        include_str!("../../../../config_metadata/config_ui_metadata.toml"),
    )
    .expect("config ui metadata");
    fs::write(
        runtime.join("settings_default.jsonc"),
        include_str!("../../../../settings_default.jsonc"),
    )
    .expect("main defaults");
    fs::write(runtime.join("runtime_variant"), "mars\n").expect("runtime variant");
    let mars_config = runtime.join("share/mars/config.toml");
    fs::create_dir_all(mars_config.parent().unwrap()).expect("mars config dir");
    fs::write(
        mars_config,
        "confirm-before-quit = true\n[mars.appearance]\npreset = \"dark\"\n[window]\nopacity = 0.78\n",
    )
    .expect("mars config");
    fs::write(
        runtime.join(DEFAULT_CURSOR_CONFIG_FILENAME),
        include_str!("../../../../yazelix_cursors_default.toml"),
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

struct Fixture {
    runtime: TempDir,
    config: TempDir,
}

impl Fixture {
    fn new() -> Self {
        let runtime = tempdir().expect("runtime");
        let config = tempdir().expect("config");
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
        build_config_ui_model(&self.request()).expect("model")
    }

    fn app(&self) -> YazelixConfigUiApp {
        let request = self.request();
        let model = build_config_ui_model(&request).expect("model");
        YazelixConfigUiApp::new(request, model)
    }

    fn settings_path(&self) -> PathBuf {
        self.config.path().join("settings.jsonc")
    }

    fn cursor_path(&self) -> PathBuf {
        crate::user_config_paths::shared_cursor_config(self.config.path())
    }

    fn mars_path(&self) -> PathBuf {
        crate::user_config_paths::mars_config(self.config.path())
    }

    fn write_settings(&self, mutate: impl FnOnce(&mut JsonValue)) -> PathBuf {
        self.write_settings_with_prefix("", mutate)
    }

    fn write_settings_with_prefix(
        &self,
        prefix: &str,
        mutate: impl FnOnce(&mut JsonValue),
    ) -> PathBuf {
        let mut value =
            read_settings_jsonc_value(&self.runtime.path().join("settings_default.jsonc"))
                .expect("default settings");
        mutate(&mut value);
        let path = self.settings_path();
        fs::create_dir_all(self.config.path()).expect("config dir");
        fs::write(
            &path,
            format!(
                "{}{}\n",
                prefix,
                serde_json::to_string_pretty(&value).expect("settings json")
            ),
        )
        .expect("settings");
        path
    }

    #[cfg(unix)]
    fn write_home_manager_settings(&self) -> PathBuf {
        let hm_dir = self.config.path().join("profile-home-manager-files");
        fs::create_dir_all(&hm_dir).expect("home manager dir");
        let hm_settings = hm_dir.join("settings.jsonc");
        fs::write(
            &hm_settings,
            render_default_settings_jsonc(&self.runtime.path().join("settings_default.jsonc"))
                .unwrap(),
        )
        .expect("home manager settings");
        std::os::unix::fs::symlink(&hm_settings, self.settings_path()).expect("settings symlink");
        hm_settings
    }
}

fn model_field<'a>(model: &'a ConfigUiModel, path: &str) -> &'a ConfigUiField {
    model
        .fields
        .iter()
        .find(|field| field.path == path)
        .expect("field")
}

// Defends: Ratconfig exposes the packaged complete Mars TOML as generic defaulted rows without a main-owned field catalog.
#[test]
fn model_exposes_generic_mars_document_and_file_action() {
    let fixture = Fixture::new();
    let model = fixture.model();
    let opacity = model_field(&model, "window.opacity");

    assert_eq!(opacity.source_id, MARS_SOURCE_ID);
    assert_eq!(opacity.state, ConfigUiValueState::Defaulted);
    assert_eq!(opacity.current_value, "0.78");
    assert!(model.tab_list_tables.contains_key(MARS_TAB));
    assert_eq!(model.file_actions.len(), 1);
    assert_eq!(model.file_actions[0].path, fixture.mars_path());
    assert!(!model.file_actions[0].exists);
}

// Regression: a user Mars file is a full native replacement, so Ratconfig must not present omitted packaged rows as inherited values.
#[test]
fn model_does_not_merge_packaged_rows_into_user_mars_config() {
    let fixture = Fixture::new();
    fs::create_dir_all(fixture.mars_path().parent().unwrap()).unwrap();
    fs::write(fixture.mars_path(), "[window]\nopacity = 0.5\n").unwrap();

    let model = fixture.model();

    assert_eq!(
        model_field(&model, "window.opacity").state,
        ConfigUiValueState::Explicit
    );
    assert!(
        model
            .fields
            .iter()
            .all(|field| field.source_id != MARS_SOURCE_ID || field.path != "confirm-before-quit")
    );
}

// Defends: editing one generic Mars scalar creates the complete packaged config and patches it through Ratconfig TOML primitives.
#[test]
fn generic_mars_field_write_creates_complete_native_config() {
    let fixture = Fixture::new();
    let mut app = fixture.app();

    app.write_source_field_value(MARS_SOURCE_ID, "window.opacity", &json!(0.5))
        .unwrap();

    let raw = fs::read_to_string(fixture.mars_path()).unwrap();
    let value = ratconfig::toml_adapter::parse_toml_value(&raw).unwrap();
    assert_eq!(
        ratconfig::toml_adapter::get_toml_path(&value, "window.opacity"),
        Some(&json!(0.5))
    );
    assert!(raw.contains("confirm-before-quit = true"));
}

// Defends: the Mars file action creates the canonical file as an exact copy of the packaged complete config.
#[test]
fn mars_file_action_prepares_complete_packaged_config() {
    let fixture = Fixture::new();
    let request = fixture.request();

    prepare_mars_config_file(&request).unwrap();

    assert_eq!(
        fs::read_to_string(fixture.mars_path()).unwrap(),
        fs::read_to_string(user_config_paths::packaged_mars_config(
            fixture.runtime.path()
        ))
        .unwrap()
    );
}

// Regression: a dangling Home Manager config symlink is owned state, not a missing file that Ratconfig may replace.
#[cfg(unix)]
#[test]
fn mars_file_action_preserves_dangling_home_manager_symlink() {
    use std::os::unix::fs::symlink;

    let fixture = Fixture::new();
    let request = fixture.request();
    let mars = fixture.mars_path();
    fs::create_dir_all(mars.parent().unwrap()).unwrap();
    let target = fixture
        .config
        .path()
        .join("profile-home-manager-files/missing-mars.toml");
    symlink(&target, &mars).unwrap();

    let error = prepare_mars_config_file(&request).unwrap_err();

    assert_eq!(error.code(), "read_only_mars_config");
    assert_eq!(fs::read_link(&mars).unwrap(), target);
}

fn line_text(line: &Line<'_>) -> String {
    line.spans
        .iter()
        .map(|span| span.content.as_ref())
        .collect()
}

fn lines_text(lines: &[Line<'_>]) -> String {
    lines.iter().map(line_text).collect::<Vec<_>>().join("\n")
}

fn select_field_path(app: &mut ConfigUiApp, path: &str) {
    let field = app
        .model
        .fields
        .iter()
        .find(|field| field.path == path)
        .expect("field");
    app.selected_tab = app
        .model
        .tabs
        .iter()
        .position(|tab| tab == &field.tab)
        .expect("tab");
    app.selected_row = app
        .visible_rows()
        .iter()
        .position(|row| {
            matches!(
                row,
                UiRowRef::Field(index) if app.model.fields[*index].path == path
            )
        })
        .expect("row");
}

fn selected_details(app: &mut YazelixConfigUiApp, path: &str) -> String {
    select_field_path(&mut app.ui, path);
    lines_text(&render_details(
        &app.ui,
        app.ui.visible_rows()[app.ui.selected_row],
    ))
}

fn field_details(app: &YazelixConfigUiApp, field_index: usize) -> String {
    lines_text(&render_details(&app.ui, UiRowRef::Field(field_index)))
}

// Defends: list fields edit from their full JSON value instead of a lossy display label.
#[test]
fn list_fields_edit_from_full_json_value() {
    let fixture = Fixture::new();
    let model = fixture.model();
    let field = model_field(&model, "zellij.widget_tray");

    assert_eq!(field.current_value, "[5 items]");
    assert_eq!(field.apply_status.summary, "after Yazelix restart");
    let input = edit_input_for_field(field);
    assert_eq!(
        input,
        "[\"session\",\"editor\",\"shell\",\"term\",\"codex_usage\"]"
    );
    assert_eq!(
        parse_edit_input(field, &input).expect("string list"),
        json!(["session", "editor", "shell", "term", "codex_usage"])
    );
}

// Defends: config UI does not expose cursor editor fields when the packaged runtime disables the cursor component.
#[test]
fn disabled_cursor_component_removes_cursor_editor_fields() {
    let fixture = Fixture::new();
    fs::write(
        fixture.runtime.path().join("runtime_components.json"),
        r#"{
              "cursors": { "enabled": false, "disableable": true, "notes": [] },
              "screen": { "enabled": true, "disableable": true, "notes": [] }
            }"#,
    )
    .expect("runtime component manifest");
    fs::remove_file(fixture.runtime.path().join(DEFAULT_CURSOR_CONFIG_FILENAME))
        .expect("remove cursor defaults");

    let model = fixture.model();

    assert!(!model.tabs.contains(&"cursors".to_string()));
    assert!(
        model
            .fields
            .iter()
            .all(|field| !field.path.starts_with("cursors."))
    );
    assert!(
        model
            .sources
            .iter()
            .all(|source| source.id != CURSORS_SOURCE_ID)
    );
}

// Defends: the config UI model names the owning config source for each editable tab.
#[test]
fn model_exposes_source_metadata_and_field_source_ids() {
    let fixture = Fixture::new();
    let model = fixture.model();

    let general = model
        .sources
        .iter()
        .find(|source| source.tab == "general")
        .expect("general source");
    assert_eq!(general.id, SETTINGS_SOURCE_ID);
    assert_eq!(general.label, "settings.jsonc");
    assert_eq!(general.path, fixture.settings_path());

    let cursors = model
        .sources
        .iter()
        .find(|source| source.tab == "cursors")
        .expect("cursor source");
    assert_eq!(cursors.id, CURSORS_SOURCE_ID);
    assert_eq!(cursors.label, "yazelix_cursors/settings.jsonc");
    assert_eq!(cursors.path, fixture.cursor_path());
    assert!(!model.sources.iter().any(|source| source.tab == "advanced"));
    assert!(
        model
            .sidecars
            .iter()
            .all(|sidecar| sidecar.name != "yazelix_cursors/settings.jsonc")
    );

    assert_eq!(
        model_field(&model, "editor.hide_sidebar_on_file_open").source_id,
        SETTINGS_SOURCE_ID
    );
    assert_eq!(
        model_field(&model, "zellij.keybindings.bottom_popup").source_id,
        SETTINGS_SOURCE_ID
    );
    assert_eq!(
        model_field(&model, "zellij.custom_popups.zenith.command").source_id,
        SETTINGS_SOURCE_ID
    );
    assert_eq!(
        model_field(&model, "cursors.enabled_cursors").source_id,
        CURSORS_SOURCE_ID
    );
}

// Defends: source-aware routing rejects mismatched source/path pairs before either backing file is written.
#[test]
fn source_routing_rejects_wrong_source_before_write() {
    let fixture = Fixture::new();
    let mut app = fixture.app();

    let cursor_as_settings = app
        .write_source_field_value(
            SETTINGS_SOURCE_ID,
            "cursors.enabled_cursors",
            &json!(["snow"]),
        )
        .unwrap_err();
    assert_eq!(cursor_as_settings.code(), "config_source_mismatch");
    assert!(!fixture.settings_path().exists());
    assert!(!fixture.cursor_path().exists());

    let settings_as_cursor = app
        .write_source_field_value(
            CURSORS_SOURCE_ID,
            "editor.hide_sidebar_on_file_open",
            &json!(true),
        )
        .unwrap_err();
    assert_eq!(settings_as_cursor.code(), "config_source_mismatch");
    assert!(!fixture.settings_path().exists());
    assert!(!fixture.cursor_path().exists());
}

// Regression: arbitrary Mars keys may share a dotted path with main settings and must still route by source id.
#[test]
fn source_routing_allows_same_path_in_mars_and_main_documents() {
    let fixture = Fixture::new();
    fs::create_dir_all(fixture.mars_path().parent().unwrap()).unwrap();
    fs::write(fixture.mars_path(), "[zellij]\ntheme = \"native\"\n").unwrap();
    let mut app = fixture.app();

    app.write_source_field_value(MARS_SOURCE_ID, "zellij.theme", &json!("mars"))
        .unwrap();

    assert!(
        fs::read_to_string(fixture.mars_path())
            .unwrap()
            .contains("theme = \"mars\"")
    );
    assert!(!fixture.settings_path().exists());
}

// Defends: the keybinding tab renders Yazelix action registry labels, scoped ids, defaults, remaps, and disabled actions instead of an opaque JSON object.
#[test]
fn zellij_keybinding_details_use_action_registry_metadata() {
    let fixture = Fixture::new();
    fixture.write_settings(|settings| {
        settings["zellij"]["keybindings"]["bottom_popup"] = json!(["Alt x"]);
        settings["zellij"]["keybindings"]["menu"] = json!([]);
        settings["zellij"]["keybindings"]["unknown_action"] = json!(["Alt z"]);
    });
    let mut app = fixture.app();

    let details = selected_details(&mut app, "zellij.keybindings");

    assert!(details.contains("Toggle the bottom popup slot"));
    assert!(details.contains("zellij.bottom_popup"));
    assert!(details.contains("Alt x (remapped)"));
    assert!(details.contains("Alt Shift J"));
    assert!(details.contains("Alt Shift K"));
    assert!(details.contains("Open the Yazelix command palette popup"));
    assert!(details.contains("disabled (disabled)"));
    assert!(details.contains("empty list disables this action"));
    assert!(details.contains("unsupported"));
    assert!(details.contains("unknown_action"));
}

// Defends: native Zellij policy keybindings use the same structured action-row editor as semantic Yazelix bindings.
#[test]
fn zellij_native_keybinding_details_use_policy_registry_metadata() {
    let fixture = Fixture::new();
    fixture.write_settings(|settings| {
        settings["zellij"]["native_keybindings"]["scroll_mode"] = json!(["Ctrl Alt x"]);
        settings["zellij"]["native_keybindings"]["scroll_mode_unbind"] = json!([]);
        settings["zellij"]["native_keybindings"]["unknown_policy"] = json!(["Alt z"]);
    });
    let mut app = fixture.app();

    let details = selected_details(&mut app, "zellij.native_keybindings");

    assert!(details.contains("Yazelix native Zellij policy"));
    assert!(details.contains("Toggle scroll mode"));
    assert!(details.contains("Ctrl Alt x (remapped)"));
    assert!(details.contains("Ctrl Alt s"));
    assert!(details.contains("Unbind default scroll-mode key"));
    assert!(details.contains("disabled (disabled)"));
    assert!(details.contains("unsupported"));
    assert!(details.contains("unknown_policy"));
}

// Regression: keybinding map parents are structured overviews; pressing Enter must not open the whole map as one raw JSON editing line.
#[test]
fn keybinding_map_parent_does_not_open_raw_object_editor() {
    let fixture = Fixture::new();
    let mut app = fixture.app();

    select_field_path(&mut app, "zellij.keybindings");
    app.handle_key(ConfigUiKey::Enter);

    assert!(app.edit.is_none());
    assert_eq!(
        app.notice.as_ref().expect("notice").text,
        "Select an action row below to edit one binding list."
    );
}

// Defends: complex array/object fields without a dedicated structured editor do not fall back to an unreadable one-line JSON editor.
#[test]
fn complex_registry_field_does_not_open_raw_array_editor() {
    let fixture = Fixture::new();
    let mut app = fixture.app();

    select_field_path(&mut app, "cursors.cursor");
    app.handle_key(ConfigUiKey::Enter);

    assert!(app.edit.is_none());
    assert_eq!(
        app.notice.as_ref().expect("notice").text,
        "Cursor registry definitions are edited in the source file; run `yzx edit cursors`."
    );
}

// Defends: cursor preset selection is a picker-backed string list, not a rejected generic JSON array.
#[test]
fn cursor_enabled_cursors_opens_multi_choice_picker_and_writes_cursor_config() {
    let fixture = Fixture::new();
    let settings_path = fixture.settings_path();
    let cursor_path = fixture.cursor_path();
    let mut app = fixture.app();
    let field = model_field(&app.model, "cursors.enabled_cursors");

    assert_eq!(field.kind, "string_list");
    assert!(field.allowed_values.contains(&"blaze".to_string()));
    assert!(field.allowed_values.contains(&"snow".to_string()));
    assert!(field.allowed_values.contains(&"ice".to_string()));
    assert!(field.allowed_values.contains(&"midnight".to_string()));

    select_field_path(&mut app, "cursors.enabled_cursors");
    app.handle_key(ConfigUiKey::Enter);

    let edit = app.edit.clone().expect("edit");
    assert_eq!(edit.mode, ConfigUiEditMode::MultiChoice);
    let details = field_details(&app, edit.field_index);
    assert!(details.contains("> [x] blaze"));

    app.handle_key(ConfigUiKey::Char(' '));
    app.handle_key(ConfigUiKey::Enter);

    assert!(app.edit.is_none());
    let value = read_settings_jsonc_value(&cursor_path).expect("cursor settings jsonc");
    let enabled = get_json_path(&value, "enabled_cursors")
        .and_then(JsonValue::as_array)
        .expect("enabled cursors");
    assert!(!enabled.iter().any(|value| value.as_str() == Some("blaze")));
    assert!(enabled.iter().any(|value| value.as_str() == Some("snow")));
    assert!(enabled.iter().any(|value| value.as_str() == Some("ice")));
    assert!(
        enabled
            .iter()
            .any(|value| value.as_str() == Some("midnight"))
    );
    assert!(!settings_path.exists());
}

// Defends: dynamic cursor trail selection is a single-select picker over none, random, and enabled cursor names.
#[test]
fn cursor_trail_uses_dynamic_single_choice_picker() {
    let fixture = Fixture::new();
    let mut app = fixture.app();
    let field = model_field(&app.model, "cursors.settings.trail");

    assert_eq!(field.kind, "string");
    assert_eq!(field.allowed_values[0], "none");
    assert_eq!(field.allowed_values[1], "random");
    assert!(field.allowed_values.contains(&"blaze".to_string()));

    let details = selected_details(&mut app, "cursors.settings.trail");
    assert!(details.contains("  ( ) none"));
    assert!(details.contains("  (x) random"));
    assert!(!details.contains("> (x) random"));

    app.handle_key(ConfigUiKey::Enter);

    let edit = app.edit.clone().expect("edit");
    assert_eq!(edit.mode, ConfigUiEditMode::Choice);
    let details = field_details(&app, edit.field_index);
    assert!(details.contains("  ( ) none"));
    assert!(details.contains("> (x) random"));
}

// Defends: keybinding actions are editable as one semantic action row with friendly key-list input instead of forcing a full object edit.
#[test]
fn keybinding_action_row_writes_single_binding_list() {
    let fixture = Fixture::new();
    let settings_path = fixture.settings_path();
    fixture.write_settings(|settings| {
        settings["zellij"]["keybindings"]["bottom_popup"] = json!(["Alt x"]);
    });
    let mut app = fixture.app();

    select_field_path(&mut app, "zellij.keybindings.bottom_popup");
    app.handle_key(ConfigUiKey::Enter);

    let edit = app.edit.as_mut().expect("edit");
    assert_eq!(edit.mode, ConfigUiEditMode::Text);
    assert_eq!(edit.input, "Alt x");
    edit.input = "Alt Shift X".to_string();
    app.handle_key(ConfigUiKey::Enter);

    assert!(app.edit.is_none());
    let value = read_settings_jsonc_value(&settings_path).expect("settings jsonc");
    assert_eq!(
        get_json_path(&value, "zellij.keybindings.bottom_popup"),
        Some(&json!(["Alt Shift X"]))
    );
}

// Defends: custom popup definitions have a structured config UI instead of forcing the whole JSON array into one edit line.
#[test]
fn custom_popup_rows_expose_structured_editor() {
    let fixture = Fixture::new();
    let model = fixture.model();
    let parent = model_field(&model, CUSTOM_POPUPS_FIELD_PATH);
    let overview = model_field(&model, "zellij.custom_popups.zenith");
    let command = model_field(&model, "zellij.custom_popups.zenith.command");
    let keybindings = model_field(&model, "zellij.custom_popups.zenith.keybindings");
    let keep_alive = model_field(&model, "zellij.custom_popups.zenith.keep_alive");

    assert_eq!(
        parent.edit_behavior,
        ConfigUiEditBehavior::StructuredOnly {
            notice: "Select a custom popup row below to edit one popup definition.".to_string()
        }
    );
    assert_eq!(overview.kind, "custom_popup");
    assert_eq!(command.kind, "string_list");
    assert_eq!(command.current_value, "[\"zenith\"]");
    assert_eq!(edit_input_for_field(command), "zenith");
    assert_eq!(keybindings.kind, "string_list");
    assert_eq!(edit_input_for_field(keybindings), "Alt Shift I");
    assert_eq!(keep_alive.kind, "bool");
}

// Defends: editing one custom popup child row rewrites zellij.custom_popups as a validated list while preserving sibling popup definitions.
#[test]
fn custom_popup_child_rows_write_parent_popup_list() {
    let fixture = Fixture::new();
    let settings_path = fixture.settings_path();
    fixture.write_settings(|settings| {
        settings["zellij"]["custom_popups"] = json!([
            {
                "id": "zenith",
                "command": ["zenith"],
                "keybindings": ["Alt Shift I"],
                "keep_alive": true
            },
            {
                "id": "gitui",
                "command": ["gitui"],
                "keybindings": [],
                "keep_alive": false
            }
        ]);
    });
    let mut app = fixture.app();

    app.write_source_field_value(
        SETTINGS_SOURCE_ID,
        "zellij.custom_popups.gitui.command",
        &json!(["gitui", "--watch"]),
    )
    .expect("write command");
    app.write_source_field_value(
        SETTINGS_SOURCE_ID,
        "zellij.custom_popups.gitui.keybindings",
        &json!(["Alt Shift G"]),
    )
    .expect("write keybindings");
    app.write_source_field_value(
        SETTINGS_SOURCE_ID,
        "zellij.custom_popups.gitui.keep_alive",
        &json!(true),
    )
    .expect("write keep alive");

    let value = read_settings_jsonc_value(&settings_path).expect("settings jsonc");
    assert_eq!(
        get_json_path(&value, "zellij.custom_popups"),
        Some(&json!([
            {
                "id": "zenith",
                "command": ["zenith"],
                "keybindings": ["Alt Shift I"],
                "keep_alive": true
            },
            {
                "id": "gitui",
                "command": ["gitui", "--watch"],
                "keybindings": ["Alt Shift G"],
                "keep_alive": true
            }
        ]))
    );
}

// Regression: adding or removing custom popups must use the parent list patch, not synthetic JSON paths that do not exist in settings.jsonc.
#[test]
fn custom_popup_add_and_remove_rows_patch_parent_popup_list() {
    let fixture = Fixture::new();
    let settings_path = fixture.settings_path();
    let mut app = fixture.app();

    app.write_source_field_value(
        SETTINGS_SOURCE_ID,
        "zellij.custom_popups.$add",
        &json!("gitui"),
    )
    .expect("add popup");
    select_field_path(&mut app, "zellij.custom_popups.zenith");
    app.handle_key(ConfigUiKey::Char('u'));

    let value = read_settings_jsonc_value(&settings_path).expect("settings jsonc");
    assert_eq!(
        get_json_path(&value, "zellij.custom_popups"),
        Some(&json!([
            {
                "id": "gitui",
                "command": ["gitui"],
                "keybindings": [],
                "keep_alive": false
            }
        ]))
    );
    assert_eq!(
        model_field(&app.model, "zellij.custom_popups.gitui.command").current_value,
        "[\"gitui\"]"
    );
    assert!(
        app.model
            .fields
            .iter()
            .all(|field| field.path != "zellij.custom_popups.zenith")
    );
}

// Regression: the config UI must reject custom popup keybinding conflicts through the same materialization rule before it writes settings.jsonc.
#[test]
fn custom_popup_duplicate_keybinding_fails_before_write() {
    let fixture = Fixture::new();
    let settings_path = fixture.settings_path();
    let mut app = fixture.app();

    let error = app
        .write_source_field_value(
            SETTINGS_SOURCE_ID,
            "zellij.custom_popups.zenith.keybindings",
            &json!(["Alt Shift J"]),
        )
        .unwrap_err();

    assert_eq!(error.code(), "duplicate_custom_popup_keybinding");
    assert!(!settings_path.exists());
}

// Regression: custom popup id and command validation must run before the config UI persists the parent list rewrite.
#[test]
fn custom_popup_invalid_identity_and_command_fail_before_write() {
    let fixture = Fixture::new();
    let settings_path = fixture.settings_path();
    let mut app = fixture.app();

    let duplicate_id = app
        .write_source_field_value(
            SETTINGS_SOURCE_ID,
            "zellij.custom_popups.$add",
            &json!("zenith"),
        )
        .unwrap_err();
    assert_eq!(duplicate_id.code(), "duplicate_custom_popup_id");

    let reserved_id = app
        .write_source_field_value(
            SETTINGS_SOURCE_ID,
            "zellij.custom_popups.$add",
            &json!("bottom_popup"),
        )
        .unwrap_err();
    assert_eq!(reserved_id.code(), "reserved_custom_popup_id");

    let empty_command = app
        .write_source_field_value(
            SETTINGS_SOURCE_ID,
            "zellij.custom_popups.zenith.command",
            &json!([]),
        )
        .unwrap_err();
    assert_eq!(empty_command.code(), "empty_config_string_list");
    assert!(!settings_path.exists());
}

// Defends: custom popup pseudo-rows do not bypass the Home Manager read-only settings boundary.
#[cfg(unix)]
#[test]
fn home_manager_owned_custom_popup_rows_are_read_only() {
    let fixture = Fixture::new();
    let hm_settings = fixture.write_home_manager_settings();
    let original = fs::read_to_string(&hm_settings).expect("hm settings raw");
    let mut app = fixture.app();

    let error = app
        .write_source_field_value(
            SETTINGS_SOURCE_ID,
            "zellij.custom_popups.$add",
            &json!("gitui"),
        )
        .unwrap_err();

    assert_eq!(error.code(), "home_manager_owned_config");
    assert_eq!(
        fs::read_to_string(&hm_settings).expect("hm settings raw"),
        original
    );
}

// Defends: the same structured keybinding map treatment covers Yazi actions instead of leaving a second raw object editor in the keybindings tab.
#[test]
fn yazi_keybinding_details_use_action_registry_metadata() {
    let fixture = Fixture::new();
    fixture.write_settings(|settings| {
        settings["yazi"]["keybindings"]["open_zoxide_in_editor"] = json!([]);
    });
    let mut app = fixture.app();

    let details = selected_details(&mut app, "yazi.keybindings");

    assert!(details.contains("Yazelix Yazi actions"));
    assert!(details.contains("Retarget the managed editor through the Yazi zoxide picker"));
    assert!(details.contains("yazi.open_zoxide_in_editor"));
    assert!(details.contains("disabled (disabled)"));
}

// Defends: machine-readable apply modes from main_config_contract.toml reach clear user-facing takes-effect labels.
#[test]
fn model_exposes_apply_statuses_from_contract() {
    let fixture = Fixture::new();
    let model = fixture.model();

    let screen_saver = model_field(&model, "zellij.screen_saver_enabled");
    assert_eq!(screen_saver.apply_status.summary, "now");
    assert!(!screen_saver.apply_status.pending);
    assert!(
        screen_saver
            .apply_status
            .detail
            .contains("active pane owner")
    );

    let editor_command = model_field(&model, "editor.command");
    assert_eq!(editor_command.apply_status.summary, "after Yazelix restart");

    let widget_tray = model_field(&model, "zellij.widget_tray");
    assert_eq!(widget_tray.apply_status.summary, "after Yazelix restart");
    assert!(
        widget_tray
            .apply_status
            .detail
            .contains("regenerates managed config")
    );

    let yazi_theme = model_field(&model, "yazi.theme");
    assert_eq!(yazi_theme.apply_status.summary, "after pane reopen");
}

// Defends: Home Manager-owned settings are presented as activation-scoped even when the field's intrinsic apply mode is narrower.
#[cfg(unix)]
#[test]
fn home_manager_owned_settings_use_activation_apply_mode() {
    let fixture = Fixture::new();
    fixture.write_home_manager_settings();

    let model = fixture.model();
    let popup_width = model_field(&model, "zellij.popup_width_percent");

    assert_eq!(model.config_owner, ConfigUiPathOwner::HomeManager);
    assert_eq!(
        popup_width.apply_status.summary,
        "after Home Manager switch"
    );
    assert!(
        popup_width
            .apply_status
            .detail
            .contains("home-manager switch")
    );
}

// Defends: the config UI consumes the shared native-config status labels instead of maintaining separate sidecar wording.
#[test]
fn model_includes_native_config_status_entries() {
    let fixture = Fixture::new();

    let model = fixture.model();
    let settings = model
        .native_config_statuses
        .iter()
        .find(|status| status.surface == "settings.main")
        .expect("settings status");

    assert_eq!(settings.status, "canonical_settings");
    assert_eq!(settings.label, "Canonical Yazelix settings");
    assert!(
        model
            .native_config_statuses
            .iter()
            .any(|status| status.surface == "zellij.generated"
                && status.status == "generated_runtime")
    );
}

// Defends: enum-backed string lists use an enable/disable picker instead of forcing users to edit JSON arrays.
#[test]
fn enum_string_list_picker_toggles_subvalues_with_space() {
    let fixture = Fixture::new();
    let settings_path = fixture.settings_path();
    let mut app = fixture.app();

    select_field_path(&mut app, "zellij.widget_tray");
    app.handle_key(ConfigUiKey::Enter);

    let edit = app.edit.clone().expect("edit");
    assert_eq!(edit.mode, ConfigUiEditMode::MultiChoice);
    let details = field_details(&app, edit.field_index);
    assert!(details.contains("> [x] session"));
    assert!(details.contains("  [x] editor"));
    assert!(details.contains("  [ ] workspace"));
    assert!(!details.contains("cursor"));

    app.handle_key(ConfigUiKey::Char('j'));
    app.handle_key(ConfigUiKey::Char('j'));
    app.handle_key(ConfigUiKey::Char('j'));
    app.handle_key(ConfigUiKey::Char('j'));
    app.handle_key(ConfigUiKey::Char(' '));

    let field = app.model.fields[edit.field_index].clone();
    let input = app.edit.as_ref().expect("edit").input.clone();
    assert_eq!(
        parse_string_list_values(&field, &input).expect("values"),
        vec![
            "session",
            "editor",
            "shell",
            "term",
            "workspace",
            "codex_usage"
        ]
    );

    app.handle_key(ConfigUiKey::Enter);

    assert!(app.edit.is_none());
    let value = read_settings_jsonc_value(&settings_path).expect("settings jsonc");
    assert_eq!(
        get_json_path(&value, "zellij.widget_tray"),
        Some(&json!([
            "session",
            "editor",
            "shell",
            "term",
            "workspace",
            "codex_usage"
        ]))
    );
}

// Defends: enum rows open a single-select picker that can be driven with hjkl and saved through the JSONC patcher.
#[test]
fn scalar_enum_enter_opens_single_select_picker() {
    let fixture = Fixture::new();
    let settings_path = fixture.settings_path();
    let mut app = fixture.app();

    select_field_path(&mut app, "appearance.mode");
    app.handle_key(ConfigUiKey::Enter);

    let edit = app.edit.clone().expect("edit");
    assert_eq!(edit.mode, ConfigUiEditMode::Choice);
    let details = field_details(&app, edit.field_index);
    assert!(details.contains("> (x) dark"));
    assert!(details.contains("  ( ) light"));

    app.handle_key(ConfigUiKey::Char('j'));
    let details = field_details(&app, edit.field_index);
    assert!(details.contains("> ( ) light"));
    app.handle_key(ConfigUiKey::Char(' '));
    let details = field_details(&app, edit.field_index);
    assert!(details.contains("> (x) light"));

    app.handle_key(ConfigUiKey::Enter);

    assert!(app.edit.is_none());
    let value = read_settings_jsonc_value(&settings_path).expect("settings jsonc");
    assert_eq!(
        get_json_path(&value, "appearance.mode"),
        Some(&json!("light"))
    );
}

// Defends: Space remains a direct toggle for bools, but scalar selects open the picker instead of cycling blindly.
#[test]
fn scalar_enum_space_opens_picker_without_writing() {
    let fixture = Fixture::new();
    let settings_path = fixture.settings_path();
    let mut app = fixture.app();

    select_field_path(&mut app, "appearance.mode");
    app.handle_key(ConfigUiKey::Char(' '));

    let edit = app.edit.clone().expect("edit");
    assert_eq!(edit.mode, ConfigUiEditMode::Choice);
    let details = field_details(&app, edit.field_index);
    assert!(details.contains("> (x) dark"));
    assert!(!settings_path.exists());
}

// Defends: Space stages a bool until Enter saves it, while normal Enter is inert and Escape cancels without touching host-owned config.
#[test]
fn bool_field_stages_before_persisting_and_can_cancel() {
    let fixture = Fixture::new();
    let settings_path = fixture.settings_path();
    fixture.write_settings(|settings| {
        settings["editor"]["hide_sidebar_on_file_open"] = json!(false);
    });
    let mut app = fixture.app();

    select_field_path(&mut app, "editor.hide_sidebar_on_file_open");
    app.handle_key(ConfigUiKey::Enter);

    assert!(app.edit.is_none());
    let value = read_settings_jsonc_value(&settings_path).expect("settings jsonc");
    assert_eq!(
        get_json_path(&value, "editor.hide_sidebar_on_file_open"),
        Some(&json!(false))
    );

    app.handle_key(ConfigUiKey::Char(' '));

    assert_eq!(app.edit.as_ref().expect("staged bool").input, "true");
    let value = read_settings_jsonc_value(&settings_path).expect("settings jsonc");
    assert_eq!(
        get_json_path(&value, "editor.hide_sidebar_on_file_open"),
        Some(&json!(false))
    );

    app.handle_key(ConfigUiKey::Esc);
    assert!(app.edit.is_none());
    let value = read_settings_jsonc_value(&settings_path).expect("settings jsonc");
    assert_eq!(
        get_json_path(&value, "editor.hide_sidebar_on_file_open"),
        Some(&json!(false))
    );

    app.handle_key(ConfigUiKey::Char(' '));
    app.handle_key(ConfigUiKey::Enter);

    assert!(app.edit.is_none());
    let value = read_settings_jsonc_value(&settings_path).expect("settings jsonc");
    assert_eq!(
        get_json_path(&value, "editor.hide_sidebar_on_file_open"),
        Some(&json!(true))
    );
}

// Defends: UI edits use the same comment-preserving settings.jsonc patcher and validation path as `yzx config set`.
#[test]
fn write_field_value_patches_settings_jsonc_and_reloads_model() {
    let fixture = Fixture::new();
    let settings_path = fixture.settings_path();
    fixture.write_settings_with_prefix("// keep this comment\n", |settings| {
        settings["editor"]["hide_sidebar_on_file_open"] = json!(false);
    });
    let mut app = fixture.app();

    let outcome = app
        .write_source_field_value(
            SETTINGS_SOURCE_ID,
            "editor.hide_sidebar_on_file_open",
            &json!(true),
        )
        .expect("write");

    assert_eq!(outcome.mutation, SettingsJsoncPatchMutation::Replaced);
    let raw = fs::read_to_string(&settings_path).expect("settings raw");
    assert!(raw.contains("// keep this comment"));
    let value = read_settings_jsonc_value(&settings_path).expect("settings jsonc");
    assert_eq!(
        get_json_path(&value, "editor.hide_sidebar_on_file_open"),
        Some(&json!(true))
    );
    assert_eq!(
        get_json_path(&value, "ratconfig.contract.contract_id"),
        Some(&json!("yazelix.settings"))
    );
    let field = model_field(&app.model, "editor.hide_sidebar_on_file_open");
    assert_eq!(field.state, ConfigUiValueState::Explicit);
    assert_eq!(field.current_value, "true");
    assert!(!fixture.cursor_path().exists());
}

// Regression: a save-time refresh failure remains visible as pending apply work instead of hiding the fact that the setting was already persisted.
#[test]
fn write_notice_keeps_saved_setting_visible_when_apply_fails() {
    let outcome = ConfigUiWriteOutcome {
        mutation: SettingsJsoncPatchMutation::Replaced,
        apply_notice: Some(
            "Apply pending: Saved yazi.theme, but generated config refresh failed.".to_string(),
        ),
    };

    let notice = write_notice_text("Saved", "yazi.theme", &outcome);

    assert!(notice.contains("Saved yazi.theme."));
    assert!(notice.contains("Apply pending"));
    assert!(notice.contains("generated config refresh failed"));
}
