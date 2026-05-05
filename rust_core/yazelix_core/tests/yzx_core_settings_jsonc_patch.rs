// Test lane: default

use serde_json::json;
use std::fs;
use std::path::Path;
use tempfile::tempdir;
use yazelix_core::config_normalize::{NormalizeConfigRequest, normalize_config};
use yazelix_core::settings_jsonc_patch::{
    SettingsJsoncPatchMutation, set_settings_jsonc_value_text, unset_settings_jsonc_value_text,
};
use yazelix_core::settings_surface::{parse_jsonc_value, render_default_settings_jsonc};

mod support;

use support::fixtures::{repo_root, write_runtime_contract_assets};

fn settings_path() -> &'static Path {
    Path::new("settings.jsonc")
}

// Defends: scalar JSONC edits replace only the selected value while preserving surrounding comments and file shape.
// Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
#[test]
fn replaces_scalar_without_dropping_comments() {
    let raw = r#"{
  // General runtime switches
  "core": {
    // Keep this comment attached to the setting
    "debug_mode": false
  },
  "editor": {
    "sidebar_width_percent": 20
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
    let parsed = parse_jsonc_value(settings_path(), &outcome.text).expect("parse");
    assert_eq!(parsed["core"]["debug_mode"], json!(true));
}

// Defends: defaulted settings can be materialized into an existing object without requiring a whole-file rewrite.
// Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
#[test]
fn inserts_absent_field_in_existing_section() {
    let raw = r#"{
  "editor": {
    // Width stays documented
    "sidebar_width_percent": 20
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
    assert!(outcome.text.contains("// Width stays documented"));
    assert!(
        outcome
            .text
            .contains(r#""hide_sidebar_on_file_open": true"#)
    );
    let parsed = parse_jsonc_value(settings_path(), &outcome.text).expect("parse");
    assert_eq!(parsed["editor"]["hide_sidebar_on_file_open"], json!(true));
}

// Defends: reset-style edits can remove an explicit value while preserving the rest of the JSONC document.
// Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
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
    let parsed = parse_jsonc_value(settings_path(), &outcome.text).expect("parse");
    assert!(parsed["core"].get("debug_mode").is_none());
}

// Defends: unsupported structures fail clearly instead of causing an implicit pretty-print rewrite.
// Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
#[test]
fn rejects_unsafe_patch_shapes() {
    let unsupported_value = set_settings_jsonc_value_text(
        settings_path(),
        "{}",
        "cursors.cursor",
        &json!([{ "name": "block" }]),
    )
    .expect_err("unsupported array shape");
    assert_eq!(
        unsupported_value.code(),
        "unsupported_settings_jsonc_patch_value"
    );

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

// Defends: patched settings JSONC remains compatible with the existing normalized config contract.
// Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
#[test]
fn patched_text_round_trips_through_config_normalization() {
    let repo = repo_root();
    let temp = tempdir().expect("tempdir");
    let runtime = temp.path().join("runtime");
    let config = temp.path().join("config");
    write_runtime_contract_assets(&repo, &runtime);
    fs::create_dir_all(&config).expect("config dir");

    let raw = render_default_settings_jsonc(
        &runtime.join("yazelix_default.toml"),
        &runtime.join("yazelix_cursors_default.toml"),
    )
    .expect("default settings");
    let patched = set_settings_jsonc_value_text(
        &config.join("settings.jsonc"),
        &raw,
        "terminal.terminals",
        &json!(["ghostty"]),
    )
    .expect("patch");
    let config_path = config.join("settings.jsonc");
    fs::write(&config_path, patched.text).expect("write settings");

    let data = normalize_config(&NormalizeConfigRequest {
        config_path,
        default_config_path: runtime.join("yazelix_default.toml"),
        contract_path: runtime
            .join("config_metadata")
            .join("main_config_contract.toml"),
        include_missing: true,
    })
    .expect("normalize");

    assert_eq!(data.normalized_config["terminals"], json!(["ghostty"]));
}
