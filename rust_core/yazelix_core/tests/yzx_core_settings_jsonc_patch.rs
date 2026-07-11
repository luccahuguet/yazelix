// Test lane: default

use serde_json::json;
use std::path::Path;
use yazelix_core::settings_jsonc_patch::{
    SettingsJsoncPatchError, SettingsJsoncPatchMutation, set_jsonc_value_text,
    set_settings_jsonc_value_text, unset_settings_jsonc_value_text,
};
use yazelix_core::settings_surface::parse_config_value;
fn settings_path() -> &'static Path {
    Path::new("settings.jsonc")
}

// Defends: scalar JSONC edits replace only the selected value while preserving surrounding comments and file shape.
#[test]
fn replaces_scalar_without_dropping_comments() {
    let raw = r#"{
  // General runtime switches
  "core": {
    // Keep this comment attached to the setting
    "debug_mode": false
  },
  "editor": {
    "command": "hx"
  }
}
"#;

    let outcome =
        set_settings_jsonc_value_text(settings_path(), raw, "core.debug_mode", &json!(true))
            .expect("patch");

    assert_eq!(outcome.mutation, SettingsJsoncPatchMutation::Replaced);
    assert!(outcome.text.contains("// General runtime switches"));
    assert!(
        outcome
            .text
            .contains("// Keep this comment attached to the setting")
    );
    assert!(outcome.text.contains(r#""debug_mode": true"#));
    let parsed = parse_config_value(settings_path(), &outcome.text).expect("parse");
    assert_eq!(parsed["core"]["debug_mode"], json!(true));
}

// Defends: defaulted settings can be materialized into an existing object without requiring a whole-file rewrite.
#[test]
fn inserts_absent_field_in_existing_section() {
    let raw = r#"{
  "editor": {
    // Editor command stays documented
    "command": "hx"
  }
}
"#;

    let outcome = set_settings_jsonc_value_text(
        settings_path(),
        raw,
        "editor.hide_sidebar_on_file_open",
        &json!(true),
    )
    .expect("patch");

    assert_eq!(outcome.mutation, SettingsJsoncPatchMutation::Inserted);
    assert!(outcome.text.contains("// Editor command stays documented"));
    assert!(
        outcome
            .text
            .contains(r#""hide_sidebar_on_file_open": true"#)
    );
    let parsed = parse_config_value(settings_path(), &outcome.text).expect("parse");
    assert_eq!(parsed["editor"]["hide_sidebar_on_file_open"], json!(true));
}

// Invariant: reset-style edits can remove an explicit value while preserving the rest of the JSONC document.
#[test]
fn unsets_existing_field_without_rewriting_document() {
    let raw = r#"{
  "core": {
    "debug_mode": true,
    "skip_welcome_screen": false
  }
}
"#;

    let outcome =
        unset_settings_jsonc_value_text(settings_path(), raw, "core.debug_mode").expect("patch");

    assert_eq!(outcome.mutation, SettingsJsoncPatchMutation::Removed);
    assert!(!outcome.text.contains("debug_mode"));
    assert!(outcome.text.contains("skip_welcome_screen"));
    let parsed = parse_config_value(settings_path(), &outcome.text).expect("parse");
    assert!(parsed["core"].get("debug_mode").is_none());
}

// Defends: parent/path shapes that ratconfig cannot patch safely still fail clearly.
#[test]
fn rejects_unsafe_patch_shapes() {
    let blocked_parent = set_settings_jsonc_value_text(
        settings_path(),
        r#"{ "editor": false }"#,
        "editor.hide_sidebar_on_file_open",
        &json!(true),
    )
    .expect_err("parent is not object");
    assert_eq!(blocked_parent.code(), "settings_jsonc_rewrite_required");

    let invalid_path =
        set_settings_jsonc_value_text(settings_path(), "{}", "editor[0]", &json!(true))
            .expect_err("invalid path");
    assert_eq!(invalid_path.code(), "invalid_settings_path");
}

// Defends: ratconfig-owned deterministic defaults can materialize structured JSON values without a whole-file rewrite.
#[test]
fn inserts_structured_values_without_rewriting_document() {
    let raw = r#"{
  // keep root comment
  "zellij": {}
}
"#;

    let outcome = set_settings_jsonc_value_text(
        settings_path(),
        raw,
        "zellij.custom_popups",
        &json!([
            {
                "id": "zenith",
                "command": ["zenith"],
                "keybindings": ["Alt Shift I"],
                "keep_alive": true
            }
        ]),
    )
    .expect("patch structured default");

    assert_eq!(outcome.mutation, SettingsJsoncPatchMutation::Inserted);
    assert!(outcome.text.contains("// keep root comment"));
    let parsed = parse_config_value(settings_path(), &outcome.text).expect("parse");
    assert_eq!(
        parsed["zellij"]["custom_popups"][0]["command"],
        json!(["zenith"])
    );
}

// Defends: reusable JSONC patch primitives report project-agnostic errors before Yazelix maps them into CoreError.
#[test]
fn generic_patch_uses_project_agnostic_error_type() {
    let error = set_jsonc_value_text("{}", "editor[0]", &json!(true)).expect_err("invalid path");

    assert_eq!(
        error,
        SettingsJsoncPatchError::InvalidPath {
            path: "editor[0]".to_string()
        }
    );
}
