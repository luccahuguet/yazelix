// Test lane: default
use super::*;
use crate::ghostty_cursor_registry::DEFAULT_CURSOR_CONFIG_FILENAME;
use tempfile::tempdir;

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
    fs::write(runtime.join("runtime_variant"), "ghostty\n").expect("runtime variant");
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

fn test_request(runtime: &Path, config: &Path) -> ConfigUiRequest {
    ConfigUiRequest {
        runtime_dir: runtime.to_path_buf(),
        config_dir: config.to_path_buf(),
        config_override: None,
    }
}

fn write_main_settings(
    runtime: &Path,
    config: &Path,
    mutate: impl FnOnce(&mut JsonValue),
) -> PathBuf {
    write_main_settings_with_prefix(runtime, config, "", mutate)
}

fn write_main_settings_with_prefix(
    runtime: &Path,
    config: &Path,
    prefix: &str,
    mutate: impl FnOnce(&mut JsonValue),
) -> PathBuf {
    let mut value = read_settings_jsonc_value(&runtime.join("settings_default.jsonc"))
        .expect("default settings");
    mutate(&mut value);
    let path = config.join("settings.jsonc");
    fs::create_dir_all(config).expect("config dir");
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

fn model_field<'a>(model: &'a ConfigUiModel, path: &str) -> &'a ConfigUiField {
    model
        .fields
        .iter()
        .find(|field| field.path == path)
        .expect("field")
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

// Regression: summarized list displays must not become the edit buffer, because placeholders like `[7 items]` are not JSON.
#[test]
fn list_fields_edit_from_full_json_not_display_summary() {
    let runtime = tempdir().expect("runtime");
    let config = tempdir().expect("config");
    write_runtime_layout(runtime.path());
    let request = test_request(runtime.path(), config.path());
    let model = build_config_ui_model(&request).expect("model");
    let field = model_field(&model, "zellij.widget_tray");

    assert_eq!(field.current_value, "[6 items]");
    assert_eq!(field.apply_status.summary, "after Yazelix restart");
    let input = edit_input_for_field(field);
    assert!(input.starts_with("[\"editor\",\"shell\",\"term\""));
    assert_eq!(
        parse_edit_input(field, &input).expect("string list"),
        json!(["editor", "shell", "term", "codex_usage", "cpu", "ram"])
    );
}

// Defends: config UI does not expose cursor editor fields when the packaged runtime disables the cursor component.
#[test]
fn disabled_cursor_component_removes_cursor_editor_fields() {
    let runtime = tempdir().expect("runtime");
    let config = tempdir().expect("config");
    write_runtime_layout(runtime.path());
    fs::write(
        runtime.path().join("runtime_components.json"),
        r#"{
              "cursors": { "enabled": false, "disableable": true, "notes": [] },
              "screen": { "enabled": true, "disableable": true, "notes": [] }
            }"#,
    )
    .expect("runtime component manifest");
    fs::remove_file(runtime.path().join(DEFAULT_CURSOR_CONFIG_FILENAME))
        .expect("remove cursor defaults");

    let request = test_request(runtime.path(), config.path());
    let model = build_config_ui_model(&request).expect("model");

    assert!(!model.tabs.contains(&"cursors".to_string()));
    assert!(
        model
            .fields
            .iter()
            .all(|field| !field.path.starts_with("cursors."))
    );
}

// Defends: the keybinding tab renders Yazelix action registry labels, scoped ids, defaults, remaps, and disabled actions instead of an opaque JSON object.
#[test]
fn zellij_keybinding_details_use_action_registry_metadata() {
    let runtime = tempdir().expect("runtime");
    let config = tempdir().expect("config");
    write_runtime_layout(runtime.path());
    write_main_settings(runtime.path(), config.path(), |settings| {
        settings["zellij"]["keybindings"]["bottom_popup"] = json!(["Alt x"]);
        settings["zellij"]["keybindings"]["menu"] = json!([]);
        settings["zellij"]["keybindings"]["unknown_action"] = json!(["Alt z"]);
    });
    let request = test_request(runtime.path(), config.path());
    let model = build_config_ui_model(&request).expect("model");
    let mut app = YazelixConfigUiApp::new(request, model);

    select_field_path(&mut app, "zellij.keybindings");
    let details = lines_text(&render_details(
        &app.ui,
        app.visible_rows()[app.selected_row],
    ));

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
    let runtime = tempdir().expect("runtime");
    let config = tempdir().expect("config");
    write_runtime_layout(runtime.path());
    write_main_settings(runtime.path(), config.path(), |settings| {
        settings["zellij"]["native_keybindings"]["scroll_mode"] = json!(["Ctrl Alt x"]);
        settings["zellij"]["native_keybindings"]["scroll_mode_unbind"] = json!([]);
        settings["zellij"]["native_keybindings"]["unknown_policy"] = json!(["Alt z"]);
    });
    let request = test_request(runtime.path(), config.path());
    let model = build_config_ui_model(&request).expect("model");
    let mut app = YazelixConfigUiApp::new(request, model);

    select_field_path(&mut app, "zellij.native_keybindings");
    let details = lines_text(&render_details(
        &app.ui,
        app.visible_rows()[app.selected_row],
    ));

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
    let runtime = tempdir().expect("runtime");
    let config = tempdir().expect("config");
    write_runtime_layout(runtime.path());
    let request = test_request(runtime.path(), config.path());
    let model = build_config_ui_model(&request).expect("model");
    let mut app = YazelixConfigUiApp::new(request, model);

    select_field_path(&mut app, "zellij.keybindings");
    app.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));

    assert!(app.edit.is_none());
    assert_eq!(
        app.notice.as_ref().expect("notice").text,
        "Select an action row below to edit one binding list."
    );
}

// Defends: complex array/object fields without a dedicated structured editor do not fall back to an unreadable one-line JSON editor.
#[test]
fn complex_registry_field_does_not_open_raw_array_editor() {
    let runtime = tempdir().expect("runtime");
    let config = tempdir().expect("config");
    write_runtime_layout(runtime.path());
    let request = test_request(runtime.path(), config.path());
    let model = build_config_ui_model(&request).expect("model");
    let mut app = YazelixConfigUiApp::new(request, model);

    select_field_path(&mut app, "cursors.cursor");
    app.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));

    assert!(app.edit.is_none());
    assert_eq!(
        app.notice.as_ref().expect("notice").text,
        "Cursor registry definitions are edited in the source file; run `yzx edit cursors`."
    );
}

// Defends: cursor preset selection is a picker-backed string list, not a rejected generic JSON array.
#[test]
fn cursor_enabled_cursors_opens_multi_choice_picker_and_writes_cursor_config() {
    let runtime = tempdir().expect("runtime");
    let config = tempdir().expect("config");
    write_runtime_layout(runtime.path());
    let cursor_path = crate::user_config_paths::shared_cursor_config(config.path());
    let request = test_request(runtime.path(), config.path());
    let model = build_config_ui_model(&request).expect("model");
    let field = model_field(&model, "cursors.enabled_cursors");

    assert_eq!(field.kind, "string_list");
    assert!(field.allowed_values.contains(&"blaze".to_string()));
    assert!(field.allowed_values.contains(&"snow".to_string()));
    assert!(field.allowed_values.contains(&"ice".to_string()));
    assert!(field.allowed_values.contains(&"midnight".to_string()));

    let mut app = YazelixConfigUiApp::new(request, model);
    select_field_path(&mut app, "cursors.enabled_cursors");
    app.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));

    let edit = app.edit.clone().expect("edit");
    assert_eq!(edit.mode, ConfigUiEditMode::MultiChoice);
    let details = lines_text(&render_details(&app.ui, UiRowRef::Field(edit.field_index)));
    assert!(details.contains("> [x] blaze"));

    app.handle_key(KeyEvent::new(KeyCode::Char(' '), KeyModifiers::NONE));
    app.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));

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
}

// Defends: dynamic cursor trail selection is a single-select picker over none, random, and enabled cursor names.
#[test]
fn cursor_trail_uses_dynamic_single_choice_picker() {
    let runtime = tempdir().expect("runtime");
    let config = tempdir().expect("config");
    write_runtime_layout(runtime.path());
    let request = test_request(runtime.path(), config.path());
    let model = build_config_ui_model(&request).expect("model");
    let field = model_field(&model, "cursors.settings.trail");

    assert_eq!(field.kind, "string");
    assert_eq!(field.allowed_values[0], "none");
    assert_eq!(field.allowed_values[1], "random");
    assert!(field.allowed_values.contains(&"blaze".to_string()));

    let mut app = YazelixConfigUiApp::new(request, model);
    select_field_path(&mut app, "cursors.settings.trail");
    let details = lines_text(&render_details(
        &app.ui,
        app.visible_rows()[app.selected_row],
    ));
    assert!(details.contains("  ( ) none"));
    assert!(details.contains("  (x) random"));
    assert!(!details.contains("> (x) random"));

    app.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));

    let edit = app.edit.clone().expect("edit");
    assert_eq!(edit.mode, ConfigUiEditMode::Choice);
    let details = lines_text(&render_details(&app.ui, UiRowRef::Field(edit.field_index)));
    assert!(details.contains("  ( ) none"));
    assert!(details.contains("> (x) random"));
}

// Defends: keybinding actions are editable as one semantic action row with friendly key-list input instead of forcing a full object edit.
#[test]
fn keybinding_action_row_writes_single_binding_list() {
    let runtime = tempdir().expect("runtime");
    let config = tempdir().expect("config");
    write_runtime_layout(runtime.path());
    let settings_path = config.path().join("settings.jsonc");
    write_main_settings(runtime.path(), config.path(), |settings| {
        settings["zellij"]["keybindings"]["bottom_popup"] = json!(["Alt x"]);
    });
    let request = test_request(runtime.path(), config.path());
    let model = build_config_ui_model(&request).expect("model");
    let mut app = YazelixConfigUiApp::new(request, model);

    select_field_path(&mut app, "zellij.keybindings.bottom_popup");
    app.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));

    let edit = app.edit.as_mut().expect("edit");
    assert_eq!(edit.mode, ConfigUiEditMode::Text);
    assert_eq!(edit.input, "Alt x");
    edit.input = "Alt Shift X".to_string();
    app.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));

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
    let runtime = tempdir().expect("runtime");
    let config = tempdir().expect("config");
    write_runtime_layout(runtime.path());
    let request = test_request(runtime.path(), config.path());
    let model = build_config_ui_model(&request).expect("model");
    let parent = model_field(&model, CUSTOM_POPUPS_FIELD_PATH);
    let add = model_field(&model, "zellij.custom_popups.$add");
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
    assert_eq!(add.kind, "string");
    assert_eq!(overview.kind, "custom_popup");
    assert_eq!(
        overview.edit_behavior,
        ConfigUiEditBehavior::StructuredOnly {
            notice:
                "Select a custom popup child row to edit it, or press u on the popup row to remove it."
                    .to_string()
        }
    );
    assert_eq!(command.kind, "string_list");
    assert_eq!(command.current_value, "[\"zenith\"]");
    assert_eq!(edit_input_for_field(command), "zenith");
    assert_eq!(keybindings.kind, "string_list");
    assert_eq!(edit_input_for_field(keybindings), "Alt Shift I");
    assert_eq!(keep_alive.kind, "bool");
    assert_eq!(keep_alive.current_value, "true");
}

// Defends: editing one custom popup child row rewrites zellij.custom_popups as a validated list while preserving sibling popup definitions.
#[test]
fn custom_popup_child_rows_write_parent_popup_list() {
    let runtime = tempdir().expect("runtime");
    let config = tempdir().expect("config");
    write_runtime_layout(runtime.path());
    let settings_path = config.path().join("settings.jsonc");
    write_main_settings(runtime.path(), config.path(), |settings| {
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
    let request = test_request(runtime.path(), config.path());
    let model = build_config_ui_model(&request).expect("model");
    let mut app = YazelixConfigUiApp::new(request, model);

    app.write_field_value(
        "zellij.custom_popups.gitui.command",
        &json!(["gitui", "--watch"]),
    )
    .expect("write command");
    app.write_field_value(
        "zellij.custom_popups.gitui.keybindings",
        &json!(["Alt Shift G"]),
    )
    .expect("write keybindings");
    app.write_field_value("zellij.custom_popups.gitui.keep_alive", &json!(true))
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
    let runtime = tempdir().expect("runtime");
    let config = tempdir().expect("config");
    write_runtime_layout(runtime.path());
    let settings_path = config.path().join("settings.jsonc");
    let request = test_request(runtime.path(), config.path());
    let model = build_config_ui_model(&request).expect("model");
    let mut app = YazelixConfigUiApp::new(request, model);

    app.write_field_value("zellij.custom_popups.$add", &json!("gitui"))
        .expect("add popup");
    select_field_path(&mut app, "zellij.custom_popups.zenith");
    app.handle_key(KeyEvent::new(KeyCode::Char('u'), KeyModifiers::NONE));

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
    let runtime = tempdir().expect("runtime");
    let config = tempdir().expect("config");
    write_runtime_layout(runtime.path());
    let settings_path = config.path().join("settings.jsonc");
    let request = test_request(runtime.path(), config.path());
    let model = build_config_ui_model(&request).expect("model");
    let mut app = YazelixConfigUiApp::new(request, model);

    let error = app
        .write_field_value(
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
    let runtime = tempdir().expect("runtime");
    let config = tempdir().expect("config");
    write_runtime_layout(runtime.path());
    let settings_path = config.path().join("settings.jsonc");
    let request = test_request(runtime.path(), config.path());
    let model = build_config_ui_model(&request).expect("model");
    let mut app = YazelixConfigUiApp::new(request, model);

    let duplicate_id = app
        .write_field_value("zellij.custom_popups.$add", &json!("zenith"))
        .unwrap_err();
    assert_eq!(duplicate_id.code(), "duplicate_custom_popup_id");
    assert!(!settings_path.exists());

    let reserved_id = app
        .write_field_value("zellij.custom_popups.$add", &json!("bottom_popup"))
        .unwrap_err();
    assert_eq!(reserved_id.code(), "reserved_custom_popup_id");
    assert!(!settings_path.exists());

    let empty_command = app
        .write_field_value("zellij.custom_popups.zenith.command", &json!([]))
        .unwrap_err();
    assert_eq!(empty_command.code(), "empty_config_string_list");
    assert!(!settings_path.exists());
}

// Defends: custom popup pseudo-rows do not bypass the Home Manager read-only settings boundary.
#[cfg(unix)]
#[test]
fn home_manager_owned_custom_popup_rows_are_read_only() {
    let runtime = tempdir().expect("runtime");
    let config = tempdir().expect("config");
    write_runtime_layout(runtime.path());
    let hm_dir = config.path().join("profile-home-manager-files");
    fs::create_dir_all(&hm_dir).expect("home manager dir");
    let hm_settings = hm_dir.join("settings.jsonc");
    fs::write(
        &hm_settings,
        render_default_settings_jsonc(&runtime.path().join("settings_default.jsonc")).unwrap(),
    )
    .expect("home manager settings");
    std::os::unix::fs::symlink(&hm_settings, config.path().join("settings.jsonc"))
        .expect("settings symlink");
    let original = fs::read_to_string(&hm_settings).expect("hm settings raw");
    let request = test_request(runtime.path(), config.path());
    let model = build_config_ui_model(&request).expect("model");
    let mut app = YazelixConfigUiApp::new(request, model);

    let error = app
        .write_field_value("zellij.custom_popups.$add", &json!("gitui"))
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
    let runtime = tempdir().expect("runtime");
    let config = tempdir().expect("config");
    write_runtime_layout(runtime.path());
    write_main_settings(runtime.path(), config.path(), |settings| {
        settings["yazi"]["keybindings"]["open_zoxide_in_editor"] = json!([]);
    });
    let request = test_request(runtime.path(), config.path());
    let model = build_config_ui_model(&request).expect("model");
    let mut app = YazelixConfigUiApp::new(request, model);

    select_field_path(&mut app, "yazi.keybindings");
    let details = lines_text(&render_details(
        &app.ui,
        app.visible_rows()[app.selected_row],
    ));

    assert!(details.contains("Yazelix Yazi actions"));
    assert!(details.contains("Retarget the managed editor through the Yazi zoxide picker"));
    assert!(details.contains("yazi.open_zoxide_in_editor"));
    assert!(details.contains("disabled (disabled)"));
    assert!(details.contains("section"));
    assert!(details.contains("keymap"));
}

// Defends: machine-readable apply modes from main_config_contract.toml reach clear user-facing takes-effect labels.
#[test]
fn model_exposes_apply_statuses_from_contract() {
    let runtime = tempdir().expect("runtime");
    let config = tempdir().expect("config");
    write_runtime_layout(runtime.path());
    let request = test_request(runtime.path(), config.path());
    let model = build_config_ui_model(&request).expect("model");

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

    let terminal_config_mode = model_field(&model, "terminal.config_mode");
    assert_eq!(
        terminal_config_mode.apply_status.summary,
        "after Yazelix restart"
    );

    let widget_tray = model_field(&model, "zellij.widget_tray");
    assert_eq!(widget_tray.apply_status.summary, "after Yazelix restart");
    assert!(
        widget_tray
            .apply_status
            .detail
            .contains("regenerates managed config")
    );

    let popup_width = model_field(&model, "zellij.popup_width_percent");
    assert_eq!(popup_width.apply_status.summary, "after Yazelix restart");

    let yazi_theme = model_field(&model, "yazi.theme");
    assert_eq!(yazi_theme.apply_status.summary, "after pane reopen");
}

// Defends: Home Manager-owned settings are presented as activation-scoped even when the field's intrinsic apply mode is narrower.
#[cfg(unix)]
#[test]
fn home_manager_owned_settings_use_activation_apply_mode() {
    let runtime = tempdir().expect("runtime");
    let config = tempdir().expect("config");
    write_runtime_layout(runtime.path());
    let hm_dir = config.path().join("profile-home-manager-files");
    fs::create_dir_all(&hm_dir).expect("home manager dir");
    let hm_settings = hm_dir.join("settings.jsonc");
    fs::write(
        &hm_settings,
        render_default_settings_jsonc(&runtime.path().join("settings_default.jsonc")).unwrap(),
    )
    .expect("home manager settings");
    std::os::unix::fs::symlink(&hm_settings, config.path().join("settings.jsonc"))
        .expect("settings symlink");

    let request = test_request(runtime.path(), config.path());
    let model = build_config_ui_model(&request).expect("model");
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
    let runtime = tempdir().expect("runtime");
    let config = tempdir().expect("config");
    write_runtime_layout(runtime.path());
    let request = test_request(runtime.path(), config.path());

    let model = build_config_ui_model(&request).expect("model");
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
    let runtime = tempdir().expect("runtime");
    let config = tempdir().expect("config");
    write_runtime_layout(runtime.path());
    let settings_path = config.path().join("settings.jsonc");
    let request = test_request(runtime.path(), config.path());
    let model = build_config_ui_model(&request).expect("model");
    let mut app = YazelixConfigUiApp::new(request, model);

    select_field_path(&mut app, "zellij.widget_tray");
    app.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));

    let edit = app.edit.clone().expect("edit");
    assert_eq!(edit.mode, ConfigUiEditMode::MultiChoice);
    let details = lines_text(&render_details(&app.ui, UiRowRef::Field(edit.field_index)));
    assert!(details.contains("> [x] editor"));
    assert!(details.contains("  [ ] workspace"));
    assert!(!details.contains("cursor"));

    app.handle_key(KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE));
    app.handle_key(KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE));
    app.handle_key(KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE));
    app.handle_key(KeyEvent::new(KeyCode::Char(' '), KeyModifiers::NONE));

    let field = app.model.fields[edit.field_index].clone();
    let input = app.edit.as_ref().expect("edit").input.clone();
    assert_eq!(
        parse_string_list_values(&field, &input).expect("values"),
        vec![
            "editor",
            "shell",
            "term",
            "workspace",
            "codex_usage",
            "cpu",
            "ram"
        ]
    );

    app.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));

    assert!(app.edit.is_none());
    let value = read_settings_jsonc_value(&settings_path).expect("settings jsonc");
    assert_eq!(
        get_json_path(&value, "zellij.widget_tray"),
        Some(&json!([
            "editor",
            "shell",
            "term",
            "workspace",
            "codex_usage",
            "cpu",
            "ram"
        ]))
    );
}

// Defends: enum rows open a single-select picker that can be driven with hjkl and saved through the JSONC patcher.
#[test]
fn scalar_enum_enter_opens_single_select_picker() {
    let runtime = tempdir().expect("runtime");
    let config = tempdir().expect("config");
    write_runtime_layout(runtime.path());
    let settings_path = config.path().join("settings.jsonc");
    let request = test_request(runtime.path(), config.path());
    let model = build_config_ui_model(&request).expect("model");
    let mut app = YazelixConfigUiApp::new(request, model);

    select_field_path(&mut app, "terminal.config_mode");
    app.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));

    let edit = app.edit.clone().expect("edit");
    assert_eq!(edit.mode, ConfigUiEditMode::Choice);
    let details = lines_text(&render_details(&app.ui, UiRowRef::Field(edit.field_index)));
    assert!(details.contains("> (x) yazelix"));
    assert!(details.contains("  ( ) user"));

    app.handle_key(KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE));
    let details = lines_text(&render_details(&app.ui, UiRowRef::Field(edit.field_index)));
    assert!(details.contains("> ( ) user"));
    app.handle_key(KeyEvent::new(KeyCode::Char(' '), KeyModifiers::NONE));
    let details = lines_text(&render_details(&app.ui, UiRowRef::Field(edit.field_index)));
    assert!(details.contains("> (x) user"));

    app.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));

    assert!(app.edit.is_none());
    let value = read_settings_jsonc_value(&settings_path).expect("settings jsonc");
    assert_eq!(
        get_json_path(&value, "terminal.config_mode"),
        Some(&json!("user"))
    );
}

// Defends: Space remains a direct toggle for bools, but scalar selects open the picker instead of cycling blindly.
#[test]
fn scalar_enum_space_opens_picker_without_writing() {
    let runtime = tempdir().expect("runtime");
    let config = tempdir().expect("config");
    write_runtime_layout(runtime.path());
    let settings_path = config.path().join("settings.jsonc");
    let request = test_request(runtime.path(), config.path());
    let model = build_config_ui_model(&request).expect("model");
    let mut app = YazelixConfigUiApp::new(request, model);

    select_field_path(&mut app, "terminal.config_mode");
    app.handle_key(KeyEvent::new(KeyCode::Char(' '), KeyModifiers::NONE));

    let edit = app.edit.clone().expect("edit");
    assert_eq!(edit.mode, ConfigUiEditMode::Choice);
    let details = lines_text(&render_details(&app.ui, UiRowRef::Field(edit.field_index)));
    assert!(details.contains("> (x) yazelix"));
    assert!(!settings_path.exists());
}

// Defends: Enter on bool rows performs the direct control action instead of opening an edit session.
#[test]
fn enter_directly_applies_bool_field_without_edit_mode() {
    let runtime = tempdir().expect("runtime");
    let config = tempdir().expect("config");
    write_runtime_layout(runtime.path());
    let settings_path = config.path().join("settings.jsonc");
    write_main_settings(runtime.path(), config.path(), |settings| {
        settings["editor"]["hide_sidebar_on_file_open"] = json!(false);
    });
    let request = test_request(runtime.path(), config.path());
    let model = build_config_ui_model(&request).expect("model");
    let mut app = YazelixConfigUiApp::new(request, model);

    select_field_path(&mut app, "editor.hide_sidebar_on_file_open");
    app.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));

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
    let runtime = tempdir().expect("runtime");
    let config = tempdir().expect("config");
    write_runtime_layout(runtime.path());
    let settings_path = config.path().join("settings.jsonc");
    write_main_settings_with_prefix(
        runtime.path(),
        config.path(),
        "// keep this comment\n",
        |settings| {
            settings["editor"]["hide_sidebar_on_file_open"] = json!(false);
        },
    );
    let request = test_request(runtime.path(), config.path());
    let model = build_config_ui_model(&request).expect("model");
    let mut app = YazelixConfigUiApp::new(request, model);

    let outcome = app
        .write_field_value("editor.hide_sidebar_on_file_open", &json!(true))
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
}

// Regression: a save-time refresh failure remains visible as pending apply work instead of hiding the fact that the setting was already persisted.
#[test]
fn write_notice_keeps_saved_setting_visible_when_apply_fails() {
    let outcome = ConfigUiWriteOutcome {
        mutation: SettingsJsoncPatchMutation::Replaced,
        apply_status: None,
        apply_error: Some(
            "Apply pending: Saved yazi.theme, but generated config refresh failed.".to_string(),
        ),
    };

    let notice = write_notice_text("Saved", "yazi.theme", &outcome);

    assert!(notice.contains("Saved yazi.theme."));
    assert!(notice.contains("Apply pending"));
    assert!(notice.contains("generated config refresh failed"));
}
