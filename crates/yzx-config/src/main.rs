use std::{env, process};

mod catalog;
mod common;
mod custom_popups;
mod file_actions;
mod helix_config;
mod model;
mod native_config;
mod paths;
mod root_config;
mod ui;
mod yazi_config;
mod zellij_sidecar;

use catalog::*;
use common::*;
use custom_popups::*;
use helix_config::*;
use native_config::write_effective_starship_config;
use paths::*;
use root_config::*;
use ui::*;
use yazelix_cursors::initialize_cursor_config;

fn main() {
    if let Err(error) = run() {
        eprintln!("yzx-config: {error}");
        process::exit(1);
    }
}

fn run() -> Result<()> {
    let mut args = env::args().skip(1);
    match args.next().as_deref() {
        None => run_ui(),
        Some("--get") => {
            let path = args
                .next()
                .ok_or_else(|| error("--get requires a config path"))?;
            if args.next().is_some() {
                return Err(error("--get accepts exactly one config path"));
            }
            print_config_field(&path)
        }
        Some("--init-cursors") => {
            if args.next().is_some() {
                return Err(error("--init-cursors does not accept arguments"));
            }
            initialize_cursor_config(&config_paths()?.cursors)?;
            Ok(())
        }
        Some("--write-effective-helix-config") => {
            let packaged = args
                .next()
                .ok_or_else(|| error("--write-effective-helix-config requires a packaged path"))?;
            let user = args
                .next()
                .ok_or_else(|| error("--write-effective-helix-config requires a user path"))?;
            let output = args
                .next()
                .ok_or_else(|| error("--write-effective-helix-config requires an output path"))?;
            if args.next().is_some() {
                return Err(error(
                    "--write-effective-helix-config accepts exactly three paths",
                ));
            }
            write_effective_helix_config(
                std::path::Path::new(&packaged),
                std::path::Path::new(&user),
                std::path::Path::new(&output),
            )
        }
        Some("--write-effective-starship-config") => {
            let user = args
                .next()
                .ok_or_else(|| error("--write-effective-starship-config requires a user path"))?;
            let output = args.next().ok_or_else(|| {
                error("--write-effective-starship-config requires an output path")
            })?;
            if args.next().is_some() {
                return Err(error(
                    "--write-effective-starship-config accepts exactly two paths",
                ));
            }
            write_effective_starship_config(
                std::path::Path::new(&user),
                std::path::Path::new(&output),
            )
        }
        Some(arg) => Err(error(format!("unknown argument: {arg}"))),
    }
}

fn print_config_field(path: &str) -> Result<()> {
    if path == BAR_WIDGETS_PATH {
        let config = validate_config_file_at(config_paths()?.root)?;
        println!("{}", read_bar_widgets_field(&config)?);
    } else if path == CUSTOM_POPUPS_KDL_PATH {
        let config = validate_config_file_at(config_paths()?.root)?;
        print!("{}", read_custom_popups_kdl(&config)?);
    } else if path == CUSTOM_POPUP_KEYBINDINGS_KDL_PATH {
        let config = validate_config_file_at(config_paths()?.root)?;
        print!("{}", read_custom_popup_keybindings_kdl(&config)?);
    } else if path == AGENT_POPUP_KDL_PATH {
        let config = validate_config_file_at(config_paths()?.root)?;
        print!("{}", read_agent_popup_kdl(&config)?);
    } else {
        let spec = config_field(path)?;
        let config = validate_config_file_at(config_paths()?.root)?;
        println!("{}", read_config_field(&config, spec)?);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        env, fs,
        path::{Path, PathBuf},
        process,
        time::{SystemTime, UNIX_EPOCH},
    };

    use crate::file_actions::*;
    use crate::model::*;
    use crate::zellij_sidecar::*;
    use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
    use ratatui::style::{Color, Style};
    use ratconfig::toml_adapter::{get_toml_path, parse_toml_value, set_toml_value_text};
    use ratconfig::{
        ConfigUiApp, ConfigUiCapability, ConfigUiDiagnostic, ConfigUiDiagnosticScope,
        ConfigUiField, ConfigUiFieldId, ConfigUiKey, ConfigUiModel, ConfigUiOverride,
        ConfigUiSettingsView, ConfigUiTextEncoding, ConfigUiTheme, UiRowRef,
        file_action_status_label, file_action_status_style,
    };
    use serde_json::{Value as JsonValue, json};
    use yazelix_cursors::{DEFAULT_CURSOR_CONFIG_TEMPLATE, load_cursor_config};

    struct TempHome {
        path: PathBuf,
    }

    impl TempHome {
        fn new() -> Self {
            let path = env::temp_dir().join(format!(
                "yzx-config-test-{}-{}",
                process::id(),
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            ));
            fs::create_dir_all(&path).unwrap();
            Self { path }
        }
    }

    impl Drop for TempHome {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    fn temp_paths(temp: &TempHome) -> ConfigPaths {
        let packaged_yazi = temp.path.join("packaged-yazi");
        fs::create_dir_all(&packaged_yazi).unwrap();
        fs::write(packaged_yazi.join("yazi.toml"), "").unwrap();
        ConfigPaths {
            store_root: temp.path.join("store"),
            root: temp.path.join("config.toml"),
            cursors: temp.path.join("cursors.toml"),
            mars: temp.path.join("mars/config.toml"),
            zellij: temp.path.join("zellij/config.kdl"),
            helix_dir: temp.path.join("helix"),
            helix_config: temp.path.join("helix/config.toml"),
            helix_languages: temp.path.join("helix/languages.toml"),
            helix_module: temp.path.join("helix/helix.scm"),
            helix_init: temp.path.join("helix/init.scm"),
            nu_env: temp.path.join("nu/env.nu"),
            nu_config: temp.path.join("nu/config.nu"),
            starship: temp.path.join("starship.toml"),
            yazi_config: temp.path.join("yazi/yazi.toml"),
            yazi_init: temp.path.join("yazi/init.lua"),
            yazi_keymap: temp.path.join("yazi/keymap.toml"),
            yazi_package: temp.path.join("yazi/package.toml"),
            yazi_theme: temp.path.join("yazi/theme.toml"),
            packaged_yazi,
            zellij_plugins: temp.path.join("zellij/plugins.kdl"),
        }
    }

    fn temp_sources() -> (TempHome, ConfigPaths) {
        let temp = TempHome::new();
        let paths = ensure_config_sources_at(temp_paths(&temp)).unwrap();
        (temp, paths)
    }

    fn has_diagnostic(diagnostics: &[ConfigUiDiagnostic], text: &str) -> bool {
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.headline.contains(text))
    }

    fn set_read_only(path: &Path) {
        let mut permissions = fs::metadata(path).unwrap().permissions();
        permissions.set_readonly(true);
        fs::set_permissions(path, permissions).unwrap();
    }

    #[cfg(unix)]
    fn link_from_store(paths: &ConfigPaths, path: &Path, text: &str) {
        use std::os::unix::fs::symlink;

        fs::create_dir_all(&paths.store_root).unwrap();
        let target = paths.store_root.join(path.file_name().unwrap());
        fs::write(&target, text).unwrap();
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        symlink(target, path).unwrap();
    }

    fn write_toml_value(path: &Path, field_path: &str, value: &JsonValue) {
        let raw = if path.is_file() {
            fs::read_to_string(path).unwrap()
        } else {
            String::new()
        };
        let updated = set_toml_value_text(&raw, field_path, value).unwrap().text;
        atomic_write(path, &updated).unwrap();
    }

    fn write_config_text(path: &Path, text: &str) {
        fs::write(path, text).unwrap();
    }

    fn assert_toml_value(path: &Path, field_path: &str, expected: &JsonValue) {
        let value = read_toml_file_value(path, "config.toml").unwrap();
        assert_eq!(
            get_toml_path(&value, field_path),
            Some(expected),
            "{field_path}"
        );
    }

    fn assert_write_config_error(path: &Path, field_path: &str, value: JsonValue, expected: &str) {
        let error = write_config_field(path, field_path, &value).unwrap_err();
        assert!(
            error.to_string().contains(expected),
            "expected `{expected}` in `{error}`"
        );
    }

    fn assert_write_round_trip(
        path: &Path,
        field_path: &str,
        value: JsonValue,
        read_back: Option<&str>,
    ) {
        write_config_field(path, field_path, &value).unwrap();
        assert_toml_value(path, field_path, &value);
        if let Some(expected) = read_back {
            assert_eq!(
                read_config_field(path, config_field(field_path).unwrap()).unwrap(),
                expected
            );
        }
    }

    fn assert_custom_popup_error(text: &str, expected: &str) {
        let temp = TempHome::new();
        let path = temp.path.join("config.toml");
        write_config_text(&path, text);

        let error = read_custom_popups_kdl(&path).unwrap_err().to_string();
        assert!(
            error.contains(expected),
            "expected `{expected}` in `{error}`"
        );
    }

    fn assert_agent_popup_error(text: &str, expected: &str) {
        let temp = TempHome::new();
        let path = temp.path.join("config.toml");
        write_config_text(&path, text);

        let error = read_agent_popup_kdl(&path).unwrap_err().to_string();
        assert!(
            error.contains(expected),
            "expected `{expected}` in `{error}`"
        );
    }

    #[cfg(unix)]
    #[test]
    fn external_text_editor_round_trips_staged_input() {
        use std::os::unix::fs::PermissionsExt;

        let temp = TempHome::new();
        let editor = temp.path.join("editor.sh");
        fs::write(
            &editor,
            "#!/bin/sh\n[ \"${YAZELIX_HELIX_BRIDGE:-}\" = 0 ] || exit 20\ncase \"${1##*/}\" in *ui.title*) ;; *) exit 21 ;; esac\ncat > \"$1\" <<'EOF'\nline one\nline two\nEOF\n",
        )
        .unwrap();
        let mut permissions = fs::metadata(&editor).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&editor, permissions).unwrap();

        assert_eq!(
            edit_text_with_editor("ui.title", "original", &editor).unwrap(),
            "line one\nline two"
        );
    }

    #[test]
    fn external_text_editor_removes_buffer_when_launch_fails() {
        let temp = TempHome::new();
        let prefix = format!("yzx-config-cleanup.failure.test-{}-", process::id());

        let error = edit_text_with_editor(
            "cleanup.failure.test",
            "sensitive staged value",
            &temp.path.join("missing-editor"),
        )
        .unwrap_err();
        assert!(error.to_string().contains("failed to launch editor"));

        let leftovers = fs::read_dir(env::temp_dir())
            .unwrap()
            .filter_map(std::result::Result::ok)
            .map(|entry| entry.path())
            .filter(|path| {
                path.file_name()
                    .and_then(|name| name.to_str())
                    .is_some_and(|name| name.starts_with(&prefix))
            })
            .collect::<Vec<_>>();
        for path in &leftovers {
            let _ = fs::remove_file(path);
        }
        assert!(leftovers.is_empty(), "temporary edit buffer leaked");
    }

    fn model_field<'a>(model: &'a ConfigUiModel, path: &str) -> &'a ratconfig::ConfigUiField {
        model
            .fields
            .iter()
            .find(|field| field.path == path)
            .unwrap_or_else(|| panic!("missing config field {path}"))
    }

    fn effective_value(field: &ConfigUiField) -> Option<&JsonValue> {
        field
            .snapshot
            .effective
            .as_ref()
            .map(|resolved| &resolved.value)
    }

    fn baseline_value(field: &ConfigUiField) -> Option<&JsonValue> {
        field
            .snapshot
            .baseline
            .as_ref()
            .map(|resolved| &resolved.value)
    }

    fn choice_values(field: &ConfigUiField) -> Vec<&JsonValue> {
        match &field.capability {
            ConfigUiCapability::Choice { choices }
            | ConfigUiCapability::MultiChoice { choices, .. } => {
                choices.iter().map(|choice| &choice.value).collect()
            }
            ConfigUiCapability::Toggle { off, on } => vec![&off.value, &on.value],
            ConfigUiCapability::ReadOnly { .. } | ConfigUiCapability::FreeText { .. } => Vec::new(),
        }
    }

    fn assert_inherited(field: &ConfigUiField, value: &JsonValue) {
        assert_eq!(field.snapshot.intent, ConfigUiOverride::Absent);
        assert_eq!(effective_value(field), Some(value));
        assert_eq!(baseline_value(field), Some(value));
    }

    fn assert_explicit(field: &ConfigUiField, value: &JsonValue) {
        assert_eq!(
            field.snapshot.intent,
            ConfigUiOverride::Explicit(value.clone())
        );
        assert_eq!(effective_value(field), Some(value));
    }

    fn select_field(app: &mut ConfigUiApp, path: &str) {
        for _ in 0..app.visible_rows().len() {
            if app.selected_field().is_some_and(|field| field.path == path) {
                return;
            }
            app.move_down();
        }
        panic!("missing visible field {path}");
    }

    fn add_flavor(directory: &Path, name: &str) {
        let flavor = directory.join("flavors").join(format!("{name}.yazi"));
        fs::create_dir_all(&flavor).unwrap();
        fs::write(flavor.join("flavor.toml"), "").unwrap();
    }

    fn key_field<'a>(model: &'a ConfigUiModel, label: &str) -> &'a ratconfig::ConfigUiField {
        model
            .fields
            .iter()
            .find(|field| field.source_id == SOURCE_KEYS && field.display_label.contains(label))
            .unwrap_or_else(|| panic!("missing key action {label}"))
    }

    fn assert_missing(paths: &[&Path]) {
        for path in paths {
            assert!(!path.exists(), "{} should not exist", path.display());
        }
    }

    fn assert_exists(paths: &[&Path]) {
        for path in paths {
            assert!(path.exists(), "{} should exist", path.display());
        }
    }

    fn assert_file_text(path: &Path, expected: &str) {
        assert_eq!(
            fs::read_to_string(path).unwrap(),
            expected,
            "{}",
            path.display()
        );
    }

    fn assert_config_field_on_tab(
        model: &ConfigUiModel,
        path: &str,
        tab: &str,
        kind: &str,
        summary: &str,
    ) {
        let field = model_field(model, path);
        assert_eq!(field.tab, tab);
        assert_eq!(field.type_label.as_deref(), Some(kind));
        assert_inherited(field, &default_config_value(path).unwrap());
        assert_eq!(field.apply_status.summary, summary);
    }
    fn assert_config_field(model: &ConfigUiModel, path: &str, kind: &str, summary: &str) {
        assert_config_field_on_tab(model, path, TAB_CONFIG, kind, summary);
    }

    #[test]
    fn config_field_rejects_unknown_paths_before_io() {
        assert!(
            config_field("shell.typo")
                .unwrap_err()
                .to_string()
                .contains("unknown config path")
        );
    }

    #[test]
    fn root_config_catalog_defaults_come_from_config_toml_and_validate() {
        let defaults = default_config().unwrap();
        validate_root_config(&defaults).unwrap();

        for field_path in CONFIG_FIELDS
            .iter()
            .map(|spec| spec.field.path)
            .chain([BAR_WIDGETS_PATH])
        {
            let value = default_config_path_value(&defaults, field_path).unwrap();
            assert_eq!(default_config_value(field_path).unwrap(), value);
            validate_config_value(field_path, &value).unwrap();
        }
        for &(path, default) in MANAGED_KEYBINDINGS {
            assert_eq!(
                default_config_value(path).unwrap(),
                json!(default),
                "{path}"
            );
        }
    }

    #[test]
    fn root_schema_rejects_unknown_paths_and_accepts_sparse_dynamic_popups() {
        for (raw, expected) in [
            (
                "mystery = true\n",
                "mystery is not supported; use a documented Nova config path",
            ),
            (
                "[welcome]\nextra = true\n",
                "welcome.extra is not supported; use a documented Nova config path",
            ),
            (
                "[welcome]\nenabld = true\n",
                "welcome.enabld is not supported; use a documented Nova config path",
            ),
            (
                "\"welcome.enabled\" = true\n",
                "welcome.enabled must use nested TOML tables, not a quoted dotted key",
            ),
            ("welcome = 1\n", "welcome must be a table"),
            (
                "[welcome]\nenabled = \"yes\"\n",
                "welcome.enabled must be true or false",
            ),
            (
                "[popups.build]\ncommand = \"btm\"\nkeybinding = \"Alt B\"\ncolor = \"blue\"\n",
                "popups.build.color is not supported; use command, args, title, keybinding, or keep_alive",
            ),
        ] {
            let value = parse_toml_value(raw).unwrap();
            let error = validate_root_config(&value).unwrap_err().to_string();
            assert_eq!(error, expected);
        }

        for raw in [
            "",
            "[welcome]\nenabled = false\n",
            "[popups.build]\ncommand = \"btm\"\nkeybinding = \"Alt B\"\n\n[popups.logs]\ncommand = \"lnav\"\nargs = [\"app.log\"]\nkeybinding = \"Alt Shift P\"\nkeep_alive = true\n",
        ] {
            validate_root_config(&parse_toml_value(raw).unwrap()).unwrap();
        }
    }

    #[test]
    fn root_config_stays_sparse_and_inherits_packaged_defaults() {
        let temp = TempHome::new();
        let path = validate_config_file_at(temp.path.join("config.toml")).unwrap();
        assert!(!path.exists());
        assert_eq!(
            read_config_field(&path, config_field(OPEN_LOG_LEVEL_PATH).unwrap()).unwrap(),
            "info"
        );
        assert!(!path.exists());

        write_config_field(&path, OPEN_LOG_LEVEL_PATH, &json!("info")).unwrap();
        let value = read_toml_file_value(&path, "config.toml").unwrap();
        assert_eq!(
            get_toml_path(&value, OPEN_LOG_LEVEL_PATH),
            Some(&json!("info"))
        );
        assert_eq!(get_toml_path(&value, SHELL_PROGRAM_PATH), None);
        let paths = ensure_config_sources_at(temp_paths(&temp)).unwrap();
        assert_explicit(
            model_field(&build_model(&paths).unwrap(), OPEN_LOG_LEVEL_PATH),
            &json!("info"),
        );

        unset_config_field(&path, OPEN_LOG_LEVEL_PATH).unwrap();
        assert!(!path.exists());
        assert_eq!(
            read_config_field(&path, config_field(OPEN_LOG_LEVEL_PATH).unwrap()).unwrap(),
            "info"
        );

        let changed_defaults = parse_toml_value("[open]\nlog_level = \"debug\"\n").unwrap();
        let inherited = JsonValue::Object(Default::default());
        assert_eq!(
            config_path_value(&inherited, &changed_defaults, OPEN_LOG_LEVEL_PATH).unwrap(),
            json!("debug")
        );
        assert_eq!(
            config_path_value(
                &parse_toml_value("[open]\nlog_level = \"info\"\n").unwrap(),
                &changed_defaults,
                OPEN_LOG_LEVEL_PATH,
            )
            .unwrap(),
            json!("info")
        );

        std::os::unix::fs::symlink(temp.path.join("missing"), &path).unwrap();
        assert!(write_config_field(&path, OPEN_LOG_LEVEL_PATH, &json!("debug")).is_err());
        assert!(
            fs::symlink_metadata(&path)
                .unwrap()
                .file_type()
                .is_symlink()
        );
    }

    #[test]
    fn write_config_field_persists_valid_values_and_rejects_bad_values() {
        let temp = TempHome::new();
        let path = validate_config_file_at(temp.path.join("config.toml")).unwrap();

        for (field_path, value, read_back) in [
            (OPEN_LOG_LEVEL_PATH, json!("debug"), None),
            (SHELL_PROGRAM_PATH, json!("fish"), None),
            (EDITOR_COMMAND_PATH, json!("nvim"), Some("nvim")),
            (AGENT_COMMAND_PATH, json!("codex"), Some("codex")),
            (
                AGENT_ARGS_PATH,
                json!(["resume", "--dangerously-bypass-approvals-and-sandbox"]),
                Some(r#"["resume","--dangerously-bypass-approvals-and-sandbox"]"#),
            ),
            (POPUP_SIDE_MARGIN_PATH, json!(2), Some("2")),
            (POPUP_VERTICAL_MARGIN_PATH, json!(1), None),
        ] {
            assert_write_round_trip(&path, field_path, value, read_back);
        }

        for (field_path, value) in [
            (KEYBINDINGS_CONFIG_PATH, "Alt Shift C"),
            (KEYBINDINGS_AGENT_PATH, "Alt Shift A"),
            (KEYBINDINGS_GIT_PATH, "Alt Shift G"),
            (KEYBINDINGS_MENU_PATH, "Alt Shift U"),
            (KEYBINDINGS_SCREEN_PATH, "Ctrl Shift S"),
            (KEYBINDINGS_SIDEBAR_PATH, "Ctrl Shift B"),
            (KEYBINDINGS_SIDEBAR_FOCUS_PATH, "Ctrl Shift E"),
        ] {
            assert_write_round_trip(&path, field_path, json!(value), Some(value));
        }
        write_config_field(&path, KEYBINDINGS_AGENT_PATH, &json!("Alt Shift M")).unwrap();
        assert_toml_value(&path, KEYBINDINGS_AGENT_PATH, &json!("Alt Shift M"));
        write_config_field(&path, KEYBINDINGS_AGENT_PATH, &json!("Alt Shift A")).unwrap();

        for (field_path, value, expected) in [
            (
                OPEN_LOG_LEVEL_PATH,
                json!("loud"),
                "off, error, info, debug",
            ),
            (SHELL_PROGRAM_PATH, json!("tcsh"), "nu, bash, zsh, fish"),
            (EDITOR_COMMAND_PATH, json!(""), "must not be empty"),
            (
                EDITOR_COMMAND_PATH,
                json!("nvim --clean"),
                "without arguments",
            ),
            (AGENT_COMMAND_PATH, json!(""), "must not be empty"),
            (
                AGENT_COMMAND_PATH,
                json!("codex resume"),
                "without arguments",
            ),
            (AGENT_ARGS_PATH, json!("resume"), "JSON string array"),
            (AGENT_ARGS_PATH, json!([1]), "contain only strings"),
            (POPUP_SIDE_MARGIN_PATH, json!(-1), "zero or greater"),
            (
                KEYBINDINGS_AGENT_PATH,
                json!("Alt+Shift+A"),
                "keybindings.agent must be a key chord",
            ),
        ] {
            assert_write_config_error(&path, field_path, value, expected);
        }
        for value in ["Alt Shift f", "Alt Shift Y", "Alt z"] {
            assert_write_config_error(
                &path,
                KEYBINDINGS_AGENT_PATH,
                json!(value),
                &format!("conflicts with packaged key {value}"),
            );
        }
        assert_write_config_error(
            &path,
            KEYBINDINGS_AGENT_PATH,
            json!("Alt Shift U"),
            "keybindings.menu conflicts with keybindings.agent: Alt Shift U",
        );
        assert_write_config_error(
            &path,
            KEYBINDINGS_AGENT_PATH,
            json!("Ctrl Shift S"),
            "keybindings.screen conflicts with keybindings.agent: Ctrl Shift S",
        );

        write_config_field(
            &path,
            BAR_WIDGETS_PATH,
            &json!(["editor", "claude_usage", "cpu"]),
        )
        .unwrap();
        assert_toml_value(
            &path,
            BAR_WIDGETS_PATH,
            &json!(["editor", "claude_usage", "cpu"]),
        );

        write_source_default(&temp_paths(&temp), SOURCE_CONFIG, BAR_WIDGETS_PATH).unwrap();
        let value = read_toml_file_value(&path, "config.toml").unwrap();
        assert_eq!(get_toml_path(&value, BAR_WIDGETS_PATH), None);

        let error = write_config_field(&path, BAR_WIDGETS_PATH, &json!(["weather"]))
            .unwrap_err()
            .to_string();
        assert!(error.contains("bar.widgets must be one of"));
        assert!(error.contains("claude_usage"));

        write_config_field(&path, AGENT_COMMAND_PATH, &json!(AGENT_AUTO_COMMAND)).unwrap();
        let value = read_toml_file_value(&path, "config.toml").unwrap();
        assert_eq!(get_toml_path(&value, AGENT_ARGS_PATH), None);
        assert_write_config_error(
            &path,
            AGENT_ARGS_PATH,
            json!(["resume"]),
            "requires agent.command to be a custom command",
        );
    }

    #[test]
    fn bar_widgets_are_read_as_json_array_and_validated() {
        let temp = TempHome::new();
        let path = validate_config_file_at(temp.path.join("config.toml")).unwrap();

        assert_eq!(
            read_bar_widgets_field(&path).unwrap(),
            r#"["editor","shell","term","codex_usage","cpu","ram"]"#
        );

        write_toml_value(
            &path,
            BAR_WIDGETS_PATH,
            &json!(["editor", "claude_usage", "cpu"]),
        );
        assert_eq!(
            read_bar_widgets_field(&path).unwrap(),
            r#"["editor","claude_usage","cpu"]"#
        );

        write_toml_value(&path, BAR_WIDGETS_PATH, &json!(["editor", "weather"]));
        let error = read_bar_widgets_field(&path).unwrap_err().to_string();
        assert!(error.contains("bar.widgets must be one of"));
        assert!(error.contains("claude_usage"));
    }

    #[test]
    fn custom_popups_render_popup_and_keybinding_kdl() {
        // Defends: Custom popups render only popup-specific KDL and inherit runtime popup defaults.
        let temp = TempHome::new();
        let path = temp.path.join("config.toml");
        write_config_text(
            &path,
            "[popups.btm]\ncommand = \"btm\"\nargs = [\"--basic\", \"--battery\"]\ntitle = \"btm_popup\"\nkeybinding = \"Alt Shift B\"\nkeep_alive = true\n",
        );

        assert_eq!(
            read_custom_popups_kdl(&path).unwrap(),
            concat!(
                "            btm {\n",
                "                command \"btm\"\n",
                "                arg_1 \"--basic\"\n",
                "                arg_2 \"--battery\"\n",
                "                pane_title \"btm_popup\"\n",
                "                command_marker \"btm_popup\"\n",
                "                width_percent 100\n",
                "                height_percent 100\n",
                "                toggle_close_behavior \"hide\"\n",
                "            }\n",
            )
        );
        assert_eq!(
            read_custom_popup_keybindings_kdl(&path).unwrap(),
            concat!(
                "        bind \"Alt Shift B\" {\n",
                "            MessagePlugin \"yzpp\" {\n",
                "                name \"toggle\"\n",
                "                payload \"btm\"\n",
                "            }\n",
                "        }\n",
            )
        );
    }

    #[test]
    fn agent_popup_kdl_renders_custom_command_override() {
        let temp = TempHome::new();
        let path = validate_config_file_at(temp.path.join("config.toml")).unwrap();

        assert_agent_popup_error(
            "[agent]\ncommand = \"auto\"\nargs = [\"resume\"]\n",
            "requires agent.command to be a custom command",
        );

        write_config_field(&path, AGENT_COMMAND_PATH, &json!("codex")).unwrap();
        write_config_field(
            &path,
            AGENT_ARGS_PATH,
            &json!(["resume", "--dangerously-bypass-approvals-and-sandbox"]),
        )
        .unwrap();

        assert_eq!(
            read_agent_popup_kdl(&path).unwrap(),
            format!(
                concat!(
                    "            agent {{\n",
                    "                command \"{}\"\n",
                    "                arg_1 \"codex\"\n",
                    "                arg_2 \"resume\"\n",
                    "                arg_3 \"--dangerously-bypass-approvals-and-sandbox\"\n",
                    "                pane_title \"agent_popup\"\n",
                    "                width_percent 100\n",
                    "                height_percent 100\n",
                    "                preserve_terminal_title true\n",
                    "                toggle_close_behavior \"hide\"\n",
                    "            }}",
                ),
                PACKAGED_AGENT_LAUNCHER,
            )
        );
    }

    #[test]
    fn custom_popups_validate_semantic_surface() {
        // Defends: Custom popup specs stay semantic and cannot shadow packaged popup ownership.
        for (text, expected) in [
            (
                "[popups.btm]\ncommand = \"btm --basic\"\nkeybinding = \"Alt Shift B\"\n",
                "without arguments",
            ),
            (
                "[popups.yazi]\ncommand = \"btm\"\nkeybinding = \"Alt Shift B\"\n",
                "conflicts with packaged popup id",
            ),
            (
                "[popups.screen]\ncommand = \"btm\"\nkeybinding = \"Alt Shift B\"\n",
                "conflicts with packaged popup id",
            ),
            (
                "[popups.btm]\ncommand = \"btm\"\n",
                "popups.btm.keybinding is required",
            ),
            (
                "[popups.btm]\ncommand = \"btm\"\nkeybinding = \"Alt r\"\n",
                "conflicts with packaged key Alt r",
            ),
            (
                "[popups.btm]\ncommand = \"btm\"\nkeybinding = \"Alt Shift K\"\n",
                "popups.btm.keybinding conflicts with keybindings.config: Alt Shift K",
            ),
            (
                "[popups.btm]\ncommand = \"btm\"\nkeybinding = \"Alt Shift B\"\n\n[popups.htop]\ncommand = \"htop\"\nkeybinding = \"Alt Shift B\"\n",
                "popups.htop.keybinding conflicts with popups.btm.keybinding: Alt Shift B",
            ),
            (
                "[popups.btm]\ncommand = \"btm\"\ntitle = \" \"\nkeybinding = \"Alt Shift B\"\n",
                "popups.btm.title must not be empty",
            ),
            (
                "[popups.btm]\ncommand = \"btm\"\ntitle = \"yazi_popup\"\nkeybinding = \"Alt Shift B\"\n",
                "popups.btm.title conflicts with packaged popup title yazi_popup",
            ),
            (
                "[popups.btm]\ncommand = \"btm\"\ntitle = \"screen_popup\"\nkeybinding = \"Alt Shift B\"\n",
                "popups.btm.title conflicts with packaged popup title screen_popup",
            ),
            (
                "[popups.btm]\ncommand = \"btm\"\ntitle = \"shared_popup\"\nkeybinding = \"Alt Shift B\"\n\n[popups.htop]\ncommand = \"htop\"\ntitle = \"shared_popup\"\nkeybinding = \"Alt Shift U\"\n",
                "popups.htop.title conflicts with popups.btm.title: shared_popup",
            ),
        ] {
            assert_custom_popup_error(text, expected);
        }
    }

    #[test]
    fn config_model_exposes_root_config_fields() {
        let (_temp, paths) = temp_sources();

        let model = build_model(&paths).unwrap();
        assert_eq!(
            model.tabs,
            [
                " main",
                " popups",
                " mars",
                "󰇀 cursors",
                " zellij",
                " starship",
                " helix",
                "󰇥 yazi",
                " keys",
                "advanced",
            ]
        );
        assert!(!model.tabs.contains(&"shell".to_string()));
        assert_config_field(&model, SHELL_PROGRAM_PATH, "string", "new panes");
        let editor = model_field(&model, EDITOR_COMMAND_PATH);
        assert_config_field(&model, EDITOR_COMMAND_PATH, "string", "new opens");
        assert!(matches!(
            editor.capability,
            ConfigUiCapability::FreeText {
                encoding: ConfigUiTextEncoding::String
            }
        ));

        let appearance = model_field(&model, MARS_APPEARANCE_PRESET_PATH);
        assert_eq!(appearance.source_id, SOURCE_MARS);
        assert_eq!(appearance.tab, TAB_MARS);
        assert_eq!(appearance.type_label.as_deref(), Some("string"));
        assert_eq!(choice_values(appearance), [&json!("dark"), &json!("light")]);
        assert_eq!(appearance.apply_status.summary, "live");
        assert_eq!(appearance.apply_status.label, "mars/ui");
        let theme_switcher = model.theme_switcher.as_ref().expect("theme switcher");
        assert_eq!(theme_switcher.field.source_id, SOURCE_MARS);
        assert_eq!(theme_switcher.field.path, MARS_APPEARANCE_PRESET_PATH);
        assert_eq!(
            ConfigUiApp::try_new(model.clone()).unwrap().active_theme(),
            ConfigUiTheme::Dark
        );
        assert_eq!(
            theme_switcher.theme_for_value(&JsonValue::String("light".to_string())),
            Some(ConfigUiTheme::Light)
        );

        for hidden in [
            "force-theme",
            "colors.background",
            "colors.foreground",
            "colors.dim-foreground",
        ] {
            assert!(
                model.fields.iter().all(|field| field.path != hidden),
                "{hidden} should stay native TOML only"
            );
        }
        assert_eq!(
            model_field(&model, CURSOR_TRAIL_PATH).source_id,
            SOURCE_CURSORS
        );

        for &(path, _) in MANAGED_KEYBINDINGS {
            let tab = if matches!(
                path,
                KEYBINDINGS_SIDEBAR_PATH | KEYBINDINGS_SIDEBAR_FOCUS_PATH
            ) {
                TAB_CONFIG
            } else {
                TAB_POPUPS
            };
            assert_config_field_on_tab(&model, path, tab, "string", "next launch");
        }

        let field = model_field(&model, BAR_WIDGETS_PATH);

        assert_eq!(field.tab, TAB_CONFIG);
        assert_eq!(
            field.capability,
            ConfigUiCapability::MultiChoice {
                choices: string_values(BAR_WIDGET_VALUES)
                    .into_iter()
                    .map(|value| ratconfig::ConfigUiChoice::new(json!(value)))
                    .collect(),
                ordered: true,
            }
        );
        assert_inherited(
            field,
            &json!(["editor", "shell", "term", "codex_usage", "cpu", "ram"]),
        );
    }

    #[test]
    fn config_model_recommends_root_fields_without_hiding_explicit_values_or_all_search() {
        let (_temp, paths) = temp_sources();
        let model = build_model(&paths).unwrap();
        let recommended_fields = model
            .recommended_fields
            .as_ref()
            .expect("recommended fields");
        let root_recommended = recommended_fields
            .iter()
            .filter(|field| field.source_id == SOURCE_CONFIG)
            .map(|field| field.path.as_str())
            .collect::<Vec<_>>();
        assert_eq!(
            root_recommended,
            [
                SHELL_PROGRAM_PATH,
                EDITOR_COMMAND_PATH,
                AGENT_COMMAND_PATH,
                WELCOME_ENABLED_PATH,
                WELCOME_STYLE_PATH,
                KEYBINDINGS_CONFIG_PATH,
                KEYBINDINGS_AGENT_PATH,
                KEYBINDINGS_GIT_PATH,
                KEYBINDINGS_MENU_PATH,
                KEYBINDINGS_SCREEN_PATH,
                KEYBINDINGS_SIDEBAR_PATH,
                KEYBINDINGS_SIDEBAR_FOCUS_PATH,
                BAR_WIDGETS_PATH,
            ]
        );
        assert!(
            model
                .fields
                .iter()
                .filter(|field| field.source_id != SOURCE_CONFIG)
                .all(|field| {
                    recommended_fields.iter().any(|recommended| {
                        recommended.source_id == field.source_id && recommended.path == field.path
                    })
                })
        );

        let field_index = |path| {
            model
                .fields
                .iter()
                .position(|field| field.source_id == SOURCE_CONFIG && field.path == path)
                .unwrap()
        };
        let main_all_only = field_index(OPEN_LOG_LEVEL_PATH);
        let mut app = ConfigUiApp::try_new(model).unwrap();
        assert_eq!(app.settings_view(), ConfigUiSettingsView::Overview);
        assert!(
            app.visible_rows().contains(&UiRowRef::Field(main_all_only)),
            "a one-row reduction is negligible and projects All"
        );
        app.handle_key(ConfigUiKey::Char('a'));
        assert_eq!(app.settings_view(), ConfigUiSettingsView::Overview);

        app.next_tab();
        let popup_all_only = app
            .model()
            .fields
            .iter()
            .position(|field| field.path == AGENT_ARGS_PATH)
            .unwrap();
        assert!(
            !app.visible_rows()
                .contains(&UiRowRef::Field(popup_all_only))
        );
        app.handle_key(ConfigUiKey::Char('a'));
        assert_eq!(app.settings_view(), ConfigUiSettingsView::All);
        assert!(
            app.visible_rows()
                .contains(&UiRowRef::Field(popup_all_only))
        );
        app.handle_key(ConfigUiKey::Char('a'));
        assert_eq!(app.settings_view(), ConfigUiSettingsView::Overview);
        app.handle_key(ConfigUiKey::Char('/'));
        for ch in AGENT_ARGS_PATH.chars() {
            app.handle_key(ConfigUiKey::Char(ch));
        }
        assert_eq!(app.settings_view(), ConfigUiSettingsView::Overview);
        assert!(
            app.visible_rows()
                .contains(&UiRowRef::Field(popup_all_only))
        );

        write_config_field(&paths.root, AGENT_COMMAND_PATH, &json!("codex")).unwrap();
        write_config_field(&paths.root, AGENT_ARGS_PATH, &json!(["resume"])).unwrap();
        let explicit_model = build_model(&paths).unwrap();
        let explicit = explicit_model
            .fields
            .iter()
            .position(|field| field.source_id == SOURCE_CONFIG && field.path == AGENT_ARGS_PATH)
            .unwrap();
        let mut explicit_app = ConfigUiApp::try_new(explicit_model).unwrap();
        explicit_app.next_tab();
        assert!(
            explicit_app
                .visible_rows()
                .contains(&UiRowRef::Field(explicit))
        );
    }

    #[test]
    fn host_reload_preserves_failed_edits_and_completes_success_by_field_identity() {
        let (_temp, paths) = temp_sources();
        let mut app = ConfigUiApp::try_new(build_model(&paths).unwrap()).unwrap();
        select_field(&mut app, EDITOR_COMMAND_PATH);
        assert_eq!(
            app.handle_key(ConfigUiKey::Enter),
            ratconfig::ConfigUiIntent::None
        );
        let active = app.edit().cloned().expect("active editor command edit");
        let identity = active.field_id.clone();

        let mut reordered = build_model(&paths).unwrap();
        reordered.fields.rotate_left(3);
        reload_after_failed_write(&mut app, reordered, "host rejected write".to_string()).unwrap();
        assert_eq!(app.edit(), Some(&active));
        assert_eq!(
            app.selected_field().map(ConfigUiField::id),
            Some(identity.clone())
        );
        assert_eq!(
            app.notice().map(|notice| (&*notice.text, notice.is_error)),
            Some(("host rejected write", true))
        );

        let mut changed_capability = build_model(&paths).unwrap();
        let field = changed_capability
            .fields
            .iter_mut()
            .find(|field| field.id() == identity)
            .unwrap();
        field.capability = ConfigUiCapability::ReadOnly {
            reason: "Host authority changed after persistence.".to_string(),
            file_action_id: None,
        };
        reload_after_successful_write(
            &mut app,
            changed_capability,
            &identity,
            "saved by identity".to_string(),
        )
        .unwrap();
        assert!(app.edit().is_none());
        assert_eq!(app.selected_field().map(ConfigUiField::id), Some(identity));
        assert_eq!(
            app.notice().map(|notice| (&*notice.text, notice.is_error)),
            Some(("saved by identity", false))
        );
    }

    #[test]
    fn cursor_config_is_child_owned_preserved_and_structurally_editable() {
        let (_temp, paths) = temp_sources();
        let custom = r##"schema_version = 1
enabled_cursors = ["custom_test"]
[settings]
trail = "custom_test"
trail_effect = "tail"
mode_effect = "none"
glow = "none"
duration = 1.0
# user cursor must survive structured edits
[[cursor]]
name = "custom_test"
family = "mono"
color = "#123456"
"##;
        fs::write(&paths.cursors, custom).unwrap();

        let model = build_model(&paths).unwrap();
        let enabled = model_field(&model, CURSOR_ENABLED_PATH);
        let trail = model_field(&model, CURSOR_TRAIL_PATH);
        let mode = model_field(&model, "settings.mode_effect");
        assert_eq!(choice_values(enabled), [&json!("custom_test")]);
        assert_eq!(baseline_value(enabled), Some(&json!(["custom_test"])));
        assert!(choice_values(trail).contains(&&json!("random")));
        assert_eq!(trail.apply_status.summary, "next launch");
        assert_eq!(mode.apply_status.summary, "stored");
        assert!(
            !trail.can_unset,
            "cursor fields have no sparse inheritance contract"
        );
        write_source_field(&paths, SOURCE_CURSORS, CURSOR_TRAIL_PATH, &json!("none")).unwrap();
        let changed = fs::read_to_string(&paths.cursors).unwrap();
        assert!(changed.contains("# user cursor must survive structured edits"));
        assert_eq!(
            load_cursor_config(&paths.cursors).unwrap().settings.trail,
            "none"
        );

        write_source_field(
            &paths,
            SOURCE_CURSORS,
            CURSOR_TRAIL_PATH,
            &json!("missing_cursor"),
        )
        .unwrap_err();
        assert_eq!(fs::read_to_string(&paths.cursors).unwrap(), changed);

        assert!(
            write_source_default(&paths, SOURCE_CURSORS, CURSOR_TRAIL_PATH)
                .unwrap_err()
                .to_string()
                .contains("no inherited reset")
        );
    }

    #[test]
    fn config_model_exposes_popup_settings_tab() {
        let (_temp, paths) = temp_sources();

        let model = build_model(&paths).unwrap();

        assert!(model.tabs.contains(&TAB_POPUPS.to_string()));
        let popup_source = model
            .sources
            .iter()
            .find(|source| source.id == SOURCE_CONFIG)
            .expect("popup source");
        assert_eq!(popup_source.id, SOURCE_CONFIG);
        assert_eq!(popup_source.path, paths.root);
        for path in [
            AGENT_COMMAND_PATH,
            AGENT_ARGS_PATH,
            POPUP_SIDE_MARGIN_PATH,
            POPUP_VERTICAL_MARGIN_PATH,
            KEYBINDINGS_CONFIG_PATH,
            KEYBINDINGS_AGENT_PATH,
            KEYBINDINGS_GIT_PATH,
            KEYBINDINGS_MENU_PATH,
            KEYBINDINGS_SCREEN_PATH,
        ] {
            let field = model_field(&model, path);
            assert_eq!(field.source_id, SOURCE_CONFIG);
            assert_eq!(field.tab, TAB_POPUPS);
            assert_eq!(field.apply_status.summary, "next launch");
        }
    }

    #[test]
    fn configured_custom_popup_fields_use_generic_discovery_and_root_validation() {
        let (_temp, paths) = temp_sources();
        write_config_text(
            &paths.root,
            "[popups.btm]\ncommand = \"btm\"\nargs = [\"--basic\"]\nkeybinding = \"Alt Shift B\"\n",
        );

        let model = build_model(&paths).unwrap();
        let discovered = model
            .fields
            .iter()
            .filter(|field| field.path.starts_with("popups.btm."))
            .map(|field| {
                assert_eq!(field.tab, TAB_POPUPS);
                assert_eq!(field.apply_status.summary, "next launch");
                assert!(field.list_cells.is_empty());
                (field.path.as_str(), field.type_label.as_deref().unwrap())
            })
            .collect::<Vec<_>>();
        assert_eq!(
            discovered,
            [
                ("popups.btm.args", "string list"),
                ("popups.btm.command", "string"),
                ("popups.btm.keybinding", "string"),
            ]
        );
        assert!(
            model
                .recommended_fields
                .as_ref()
                .unwrap()
                .iter()
                .all(|field| {
                    field.source_id != SOURCE_CONFIG || !field.path.starts_with("popups.")
                })
        );

        write_source_field(
            &paths,
            SOURCE_CONFIG,
            "popups.btm.args",
            &json!(["--battery"]),
        )
        .unwrap();
        assert_toml_value(&paths.root, "popups.btm.args", &json!(["--battery"]));

        let unchanged = fs::read_to_string(&paths.root).unwrap();
        write_source_field(&paths, SOURCE_CONFIG, "popups.btm.unknown", &json!(true)).unwrap_err();
        assert_eq!(fs::read_to_string(&paths.root).unwrap(), unchanged);
    }

    #[test]
    fn config_model_uses_mars_appearance_as_initial_theme_source() {
        let (_temp, paths) = temp_sources();
        write_toml_value(&paths.mars, MARS_APPEARANCE_PRESET_PATH, &json!("light"));

        let model = build_model(&paths).unwrap();
        assert_eq!(
            ConfigUiApp::try_new(model).unwrap().active_theme(),
            ConfigUiTheme::Light
        );
    }

    #[test]
    fn mars_fields_have_complete_apply_timing() {
        let (_temp, paths) = temp_sources();
        let model = build_model(&paths).unwrap();
        let expected = [
            (MARS_APPEARANCE_PRESET_PATH, "live"),
            ("window.width", "new windows"),
            ("window.height", "new windows"),
            ("window.opacity", "live"),
            ("fonts.size", "live"),
            ("line-height", "live"),
            ("enable-scroll-bar", "live"),
            ("bell.audio", "live"),
            ("bell.visual", "live"),
        ];

        assert_eq!(MARS_FIELDS.len(), expected.len());
        for (path, summary) in expected {
            assert_eq!(model_field(&model, path).apply_status.summary, summary);
        }
    }

    #[test]
    fn config_model_exposes_structured_starship_tab() {
        let (_temp, paths) = temp_sources();

        let model = build_model(&paths).unwrap();
        let prompt = model_field(&model, "character.format");

        assert!(model.tabs.contains(&TAB_STARSHIP.to_string()));
        assert!(
            model
                .sources
                .iter()
                .any(|source| { source.id == SOURCE_STARSHIP && source.path == paths.starship })
        );
        assert_eq!(prompt.source_id, SOURCE_STARSHIP);
        assert_eq!(prompt.tab, TAB_STARSHIP);
        assert_eq!(prompt.type_label.as_deref(), Some("string"));
        assert_inherited(prompt, &json!(":: "));
        assert_eq!(prompt.apply_status.summary, "new prompts");
        assert_eq!(
            model
                .fields
                .iter()
                .filter(|field| field.source_id == SOURCE_STARSHIP)
                .map(|field| field.path.as_str())
                .collect::<Vec<_>>(),
            vec!["character.format"]
        );
    }

    // Defends: the Keys tab is a read-only discovery surface for current packaged bindings.
    #[test]
    fn config_model_exposes_read_only_key_bindings() {
        let (_temp, paths) = temp_sources();

        let model = build_model(&paths).unwrap();
        let rows: Vec<_> = model
            .fields
            .iter()
            .filter(|field| field.tab == TAB_KEYS)
            .collect();

        assert!(model.tabs.contains(&TAB_KEYS.to_string()));
        assert!(
            model
                .file_actions
                .iter()
                .all(|action| action.tab != TAB_KEYS)
        );
        assert_eq!(
            model
                .tab_list_tables
                .get(TAB_KEYS)
                .unwrap()
                .columns
                .iter()
                .map(|column| (column.title.as_str(), column.width))
                .collect::<Vec<_>>(),
            KEY_COLUMNS
        );
        assert_eq!(rows.len(), KEY_BINDINGS.len());
        assert!(rows.iter().all(|field| {
            field.apply_status.summary == "read-only"
                && matches!(field.capability, ConfigUiCapability::ReadOnly { .. })
                && field.list_cells.len() == KEY_COLUMNS.len()
        }));

        let config_popup = key_field(&model, "Alt Shift K");
        assert_eq!(
            config_popup.display_label,
            "Popups: Alt Shift K - Toggle config popup"
        );
        assert_explicit(config_popup, &json!("Yazelix / config.kdl"));
        assert_eq!(
            config_popup.list_cells,
            ["Popups", "Alt Shift K", "Toggle config popup", "Yazelix"].map(str::to_string)
        );
        assert!(config_popup.description.contains("Owner: Yazelix"));
        assert_eq!(config_popup.validation, KEY_READ_ONLY_REASON);

        let pane_mode = key_field(&model, "Ctrl p");
        assert!(pane_mode.display_label.contains("Ctrl p"));
        assert!(pane_mode.description.contains("Owner: Zellij"));

        let tab_jump = key_field(&model, "Alt 1-9");
        assert_eq!(
            tab_jump.display_label,
            "Tabs: Alt 1-9 - Go directly to tab 1-9"
        );
        assert!(tab_jump.description.contains("Owner: Zellij"));

        let reveal = key_field(&model, "Alt r");
        assert_eq!(
            reveal.display_label,
            "Sidebar: Alt r - Reveal editor file in Yazi"
        );
        assert!(reveal.description.contains("Owner: Yazelix"));

        let yazi_zoxide = key_field(&model, "Alt z");
        assert!(yazi_zoxide.display_label.contains("Alt z"));
        assert!(yazi_zoxide.description.contains("Owner: Yazi"));
        assert_explicit(yazi_zoxide, &json!("Yazi / yazi/keymap.toml"));

        let yazi_popup = key_field(&model, "Alt Shift Y");
        assert_eq!(
            yazi_popup.display_label,
            "Popups: Alt Shift Y - Hide or show Yazi popup"
        );
        assert!(yazi_popup.description.contains("Owner: Yazelix"));

        let screen_popup = key_field(&model, "Alt Shift S");
        assert_eq!(
            screen_popup.display_label,
            "Popups: Alt Shift S - Show a random full-screen visual"
        );
        assert!(screen_popup.description.contains("Owner: Yazelix"));
    }

    #[test]
    fn read_only_existing_sources_are_not_replaced() {
        let (_temp, paths) = temp_sources();

        atomic_write(&paths.mars, "[window]\nwidth = 960\n").unwrap();
        let before_mars = fs::read_to_string(&paths.mars).unwrap();
        set_read_only(&paths.mars);

        let error = write_source_field(&paths, SOURCE_MARS, "window.width", &json!(1200))
            .unwrap_err()
            .to_string();
        assert!(error.contains("read-only"));
        assert_eq!(fs::read_to_string(&paths.mars).unwrap(), before_mars);

        fs::write(&paths.root, "[open]\nlog_level = \"info\"\n").unwrap();
        let before_root = fs::read_to_string(&paths.root).unwrap();
        set_read_only(&paths.root);

        let error = write_source_field(&paths, SOURCE_CONFIG, OPEN_LOG_LEVEL_PATH, &json!("debug"))
            .unwrap_err()
            .to_string();
        assert!(error.contains("read-only"));
        assert_eq!(fs::read_to_string(&paths.root).unwrap(), before_root);
    }

    // Defends: only store-backed config is declarative, and every mutation route stops before IO.
    #[cfg(unix)]
    #[test]
    fn home_manager_sources_are_explicit_and_non_mutating() {
        let temp = TempHome::new();
        let paths = temp_paths(&temp);
        link_from_store(&paths, &paths.root, "");
        link_from_store(&paths, &paths.cursors, DEFAULT_CURSOR_CONFIG_TEMPLATE);
        link_from_store(&paths, &paths.starship, "[character]\nformat = \"::\"\n");
        link_from_store(&paths, &paths.nu_env, "# managed\n");
        atomic_write(&paths.mars, "[window]\nwidth = 960\n").unwrap();
        set_read_only(&paths.mars);
        let paths = ensure_config_sources_at(paths).unwrap();

        let model = build_model(&paths).unwrap();
        let source = |id| model.sources.iter().find(|source| source.id == id).unwrap();
        assert_eq!(
            source(SOURCE_CONFIG).owner_label.as_deref(),
            Some("Home Manager")
        );
        assert_eq!(
            source(SOURCE_STARSHIP).owner_label.as_deref(),
            Some("Home Manager")
        );
        assert_eq!(source(SOURCE_MARS).owner_label.as_deref(), Some("User"));
        assert!(source(SOURCE_MARS).read_only);
        assert_eq!(
            source(SOURCE_CURSORS).owner_label.as_deref(),
            Some("Home Manager")
        );
        assert!(source(SOURCE_CURSORS).read_only);
        assert!(
            model
                .fields
                .iter()
                .filter(|field| {
                    matches!(
                        field.source_id.as_str(),
                        SOURCE_CONFIG | SOURCE_CURSORS | SOURCE_STARSHIP
                    )
                })
                .all(|field| {
                    field.snapshot.external_manager.as_deref() == Some("Home Manager")
                        && matches!(field.capability, ConfigUiCapability::ReadOnly { .. })
                        && !field.can_unset
                })
        );

        let rejects = |result: Result<()>, option: &str| {
            let error = result.unwrap_err().to_string();
            assert!(error.contains(option), "{error}");
            assert!(error.contains("Home Manager switch"), "{error}");
        };
        rejects(
            write_source_field(&paths, SOURCE_CONFIG, OPEN_LOG_LEVEL_PATH, &json!("debug")),
            "programs.yazelix.config.settings",
        );
        rejects(
            write_source_default(&paths, SOURCE_STARSHIP, "character.format"),
            "programs.yazelix.config.starship",
        );
        rejects(
            write_source_field(&paths, SOURCE_CURSORS, CURSOR_TRAIL_PATH, &json!("none")),
            "programs.yazelix.config.cursors",
        );
        link_from_store(&paths, &paths.yazi_config, "[mgr]\nshow_hidden = true\n");
        link_from_store(
            &paths,
            &paths.yazi_theme,
            "[flavor]\ndark = \"catppuccin-mocha\"\n",
        );
        rejects(
            write_source_field(&paths, SOURCE_YAZI_CONFIG, "mgr.show_hidden", &json!(false)),
            "programs.yazelix.config.yazi.config",
        );
        rejects(
            write_source_default(&paths, SOURCE_YAZI_THEME, "flavor.dark"),
            "programs.yazelix.config.yazi.theme",
        );
        rejects(
            prepare_file_action(&paths, SOURCE_ADVANCED, ACTION_NU_ENV, &paths.nu_env, true),
            "programs.yazelix.config.nu.env",
        );
        assert_file_text(&paths.root, "");
        assert_file_text(&paths.cursors, DEFAULT_CURSOR_CONFIG_TEMPLATE);
        assert_file_text(&paths.starship, "[character]\nformat = \"::\"\n");
        assert_file_text(&paths.nu_env, "# managed\n");

        let action = build_file_actions(&paths)
            .into_iter()
            .find(|action| action.action_id == ACTION_NU_ENV)
            .unwrap();
        assert!(action.read_only);
        assert!(
            action
                .disabled_reason
                .unwrap()
                .contains("programs.yazelix.config.nu.env")
        );
    }

    #[test]
    fn invalid_root_values_remain_visible_while_unparseable_text_routes_to_the_file() {
        let temp = TempHome::new();
        let path = temp.path.join("config.toml");
        fs::write(&path, "[open]\nlog_level = \"loud\"\n").unwrap();

        let runtime_error =
            read_config_field(&path, config_field(OPEN_LOG_LEVEL_PATH).unwrap()).unwrap_err();
        assert!(
            runtime_error
                .to_string()
                .contains("off, error, info, debug")
        );
        let paths = ensure_config_sources_at(temp_paths(&temp)).unwrap();
        let model = build_model(&paths).unwrap();
        let field = model_field(&model, OPEN_LOG_LEVEL_PATH);
        assert_eq!(
            field.snapshot.intent,
            ConfigUiOverride::Invalid {
                input: "\"loud\"".to_string()
            }
        );
        assert_eq!(effective_value(field), None);
        assert_eq!(baseline_value(field), Some(&json!("info")));
        assert!(field.can_unset);
        assert!(model.diagnostics.iter().any(|diagnostic| {
            diagnostic.scope
                == ConfigUiDiagnosticScope::Field(ConfigUiFieldId::new(
                    SOURCE_CONFIG,
                    OPEN_LOG_LEVEL_PATH,
                ))
                && diagnostic
                    .detail_lines
                    .iter()
                    .any(|detail| detail == &runtime_error.to_string())
        }));
        ConfigUiApp::try_new(model).unwrap();

        fs::write(&path, "[open\nlog_level = \"loud\"\n").unwrap();
        let model = build_model(&paths).unwrap();
        assert!(model.diagnostics.iter().any(|diagnostic| {
            diagnostic.scope
                == ConfigUiDiagnosticScope::Source {
                    source_id: SOURCE_CONFIG.to_string(),
                }
        }));
        assert!(model.file_actions.iter().any(|action| {
            action.source_id == SOURCE_CONFIG
                && action.action_id == ACTION_ROOT_CONFIG
                && action.path == path
        }));
        assert!(
            model
                .fields
                .iter()
                .filter(|field| field.source_id == SOURCE_CONFIG)
                .all(|field| matches!(
                    field.capability,
                    ConfigUiCapability::ReadOnly {
                        file_action_id: Some(ref action),
                        ..
                    } if action == ACTION_ROOT_CONFIG
                ))
        );
        ConfigUiApp::try_new(model).unwrap();
    }

    #[test]
    fn ensure_config_sources_creates_source_backed_files() {
        let (_temp, paths) = temp_sources();

        assert_exists(&[&paths.cursors]);
        assert_missing(&[
            &paths.root,
            &paths.mars,
            &paths.starship,
            &paths.helix_config,
            &paths.helix_languages,
            &paths.helix_module,
            &paths.helix_init,
            &paths.nu_env,
            &paths.nu_config,
            &paths.yazi_config,
            &paths.yazi_init,
            &paths.yazi_keymap,
        ]);
    }

    #[test]
    fn native_file_tabs_list_owned_file_actions() {
        let (_temp, paths) = temp_sources();
        atomic_write(&paths.zellij, "keybinds{}\n").unwrap();

        let model = build_model(&paths).unwrap();
        assert!(model.tabs.contains(&TAB_STARSHIP.to_string()));
        assert!(model.tabs.contains(&TAB_HELIX.to_string()));
        assert!(model.tabs.contains(&TAB_YAZI.to_string()));
        assert!(model.tabs.contains(&TAB_CURSORS.to_string()));
        let mut app = ConfigUiApp::try_new(model.clone()).unwrap();
        for _ in 0..ratconfig::tab_index(&model.tabs, TAB_ADVANCED) {
            app.next_tab();
        }
        assert!(
            app.visible_rows()
                .contains(&ratconfig::UiRowRef::Diagnostic(0))
        );
        assert!(
            model
                .sources
                .iter()
                .any(|source| { source.id == SOURCE_HELIX && source.path == paths.helix_dir })
        );
        let yazi_sources = model
            .sources
            .iter()
            .filter(|source| {
                matches!(
                    source.id.as_str(),
                    SOURCE_YAZI | SOURCE_YAZI_CONFIG | SOURCE_YAZI_THEME
                )
            })
            .collect::<Vec<_>>();
        assert_eq!(yazi_sources.len(), 3);
        assert!(
            yazi_sources.iter().any(|source| {
                source.id == SOURCE_YAZI_CONFIG && source.path == paths.yazi_config
            })
        );
        assert!(
            yazi_sources.iter().any(|source| {
                source.id == SOURCE_YAZI_THEME && source.path == paths.yazi_theme
            })
        );
        assert!(model.file_actions.iter().all(|action| {
            let expected = match action.label.as_str() {
                "cursors.toml" => (SOURCE_CURSORS, TAB_CURSORS),
                "config.toml" => (SOURCE_CONFIG, TAB_ADVANCED),
                label if label.starts_with("helix/") => (SOURCE_HELIX, TAB_HELIX),
                "yazi/yazi.toml" => (SOURCE_YAZI_CONFIG, TAB_YAZI),
                "yazi/theme.toml" => (SOURCE_YAZI_THEME, TAB_YAZI),
                label if label.starts_with("yazi/") => (SOURCE_YAZI, TAB_YAZI),
                _ => (SOURCE_ADVANCED, TAB_ADVANCED),
            };
            (action.source_id.as_str(), action.tab.as_str()) == expected
        }));
        let summaries: Vec<_> = model
            .file_actions
            .iter()
            .map(|action| (action.action_id.as_str(), action.label.as_str()))
            .collect();
        assert_eq!(
            summaries,
            [
                (ACTION_ROOT_CONFIG, "config.toml"),
                (ACTION_CURSORS_CONFIG, "cursors.toml"),
                (ACTION_HELIX_CONFIG, "helix/config.toml"),
                (ACTION_HELIX_LANGUAGES, "helix/languages.toml"),
                (ACTION_HELIX_MODULE, "helix/helix.scm"),
                (ACTION_HELIX_INIT, "helix/init.scm"),
                (ACTION_NU_ENV, "nu/env.nu"),
                (ACTION_NU_CONFIG, "nu/config.nu"),
                (ACTION_YAZI_CONFIG, "yazi/yazi.toml"),
                (ACTION_YAZI_INIT, "yazi/init.lua"),
                (ACTION_YAZI_KEYMAP, "yazi/keymap.toml"),
                (ACTION_YAZI_PACKAGE, "yazi/package.toml"),
                (ACTION_YAZI_THEME, "yazi/theme.toml"),
                (ACTION_ZELLIJ_PLUGINS, "zellij/plugins.kdl"),
            ]
        );
        assert!(
            model
                .file_actions
                .iter()
                .all(|action| action.path.ends_with(&action.label))
        );
        assert!(model.file_actions.iter().all(|action| {
            action.create_if_missing
                && (action.exists == (action.action_id == ACTION_CURSORS_CONFIG))
        }));
        assert!(model.file_actions.iter().all(|action| {
            if action.action_id == ACTION_CURSORS_CONFIG {
                file_action_status_label(action) == "existing"
                    && file_action_status_style(action) == Style::default().fg(Color::Green)
            } else {
                file_action_status_label(action) == "absent"
                    && file_action_status_style(action) == Style::default().fg(Color::Gray)
            }
        }));
    }

    #[test]
    fn yazi_tab_renders_and_writes_native_config_with_discovered_flavors() {
        let (_temp, paths) = temp_sources();
        fs::write(
            paths.packaged_yazi.join("yazi.toml"),
            "[mgr]\nshow_hidden = false\nratio = [1, 2, 3]\n\n[preview]\nmax_width = 600\n",
        )
        .unwrap();
        add_flavor(&paths.packaged_yazi, "catppuccin-mocha");
        add_flavor(paths.yazi_config.parent().unwrap(), "custom");
        add_flavor(paths.yazi_config.parent().unwrap(), "");
        fs::write(
            &paths.yazi_config,
            "# keep config\n[mgr]\nshow_hidden = false\nratio = [1, 4, 0]\n\n[preview]\nmax_width = 800\n",
        )
        .unwrap();
        fs::write(
            &paths.yazi_theme,
            "# keep theme\n[flavor]\ndark = 42\nlight = \"custom\"\n\n[mgr]\ncwd = { fg = \"blue\" }\n",
        )
        .unwrap();

        let model = build_model(&paths).unwrap();
        assert!(!model.tab_list_tables.contains_key(TAB_YAZI));
        let dark = model_field(&model, "flavor.dark");
        assert_eq!(dark.source_id, SOURCE_YAZI_THEME);
        assert_eq!(dark.display_label, "Dark flavor");
        assert_eq!(
            choice_values(dark),
            [&json!("catppuccin-mocha"), &json!("custom")]
        );
        assert_eq!(
            dark.snapshot.intent,
            ConfigUiOverride::Invalid {
                input: "42".to_string()
            }
        );
        assert_eq!(effective_value(dark), None);
        let show_hidden = model_field(&model, "mgr.show_hidden");
        assert_eq!(show_hidden.source_id, SOURCE_YAZI_CONFIG);
        assert_eq!(
            (
                show_hidden.type_label.as_deref(),
                &show_hidden.snapshot.intent
            ),
            (Some("boolean"), &ConfigUiOverride::Explicit(json!(false)))
        );
        assert_eq!(show_hidden.apply_status.summary, "next Yazi");
        assert!(matches!(
            model_field(&model, "mgr.ratio").capability,
            ConfigUiCapability::ReadOnly { .. }
        ));
        assert_eq!(
            model_field(&model, "mgr.ratio").snapshot.intent,
            ConfigUiOverride::Explicit(json!([1, 4, 0]))
        );
        let flavor = model_field(&model, "flavor");
        assert_eq!(
            flavor.snapshot.intent,
            ConfigUiOverride::Explicit(json!({"dark": 42, "light": "custom"}))
        );
        assert_eq!(
            baseline_value(flavor),
            Some(&json!({"dark": "", "light": ""}))
        );

        write_source_field(&paths, SOURCE_YAZI_CONFIG, "mgr.show_hidden", &json!(true)).unwrap();
        write_source_field(
            &paths,
            SOURCE_YAZI_CONFIG,
            "preview.max_width",
            &json!(1200),
        )
        .unwrap();
        write_source_field(&paths, SOURCE_YAZI_THEME, "flavor.dark", &json!("custom")).unwrap();

        let config = fs::read_to_string(&paths.yazi_config).unwrap();
        assert!(config.starts_with("# keep config\n"));
        assert!(config.contains("show_hidden = true"));
        assert!(config.contains("ratio = [1, 4, 0]"));
        let theme = fs::read_to_string(&paths.yazi_theme).unwrap();
        assert!(theme.starts_with("# keep theme\n"));
        assert!(theme.contains("dark = \"custom\""));

        let error = write_source_field(&paths, SOURCE_YAZI_THEME, "flavor.dark", &json!("missing"))
            .unwrap_err()
            .to_string();
        assert!(error.contains("must name an installed flavor"), "{error}");

        write_source_default(&paths, SOURCE_YAZI_CONFIG, "mgr.show_hidden").unwrap();
        write_source_default(&paths, SOURCE_YAZI_THEME, "flavor.dark").unwrap();
        let config = read_toml_file_value(&paths.yazi_config, "Yazi config").unwrap();
        let theme = read_toml_file_value(&paths.yazi_theme, "Yazi theme").unwrap();
        assert_eq!(get_toml_path(&config, "mgr.show_hidden"), None);
        assert_eq!(
            get_toml_path(&config, "preview.max_width"),
            Some(&json!(1200))
        );
        assert_eq!(get_toml_path(&theme, "flavor.dark"), None);
        assert_eq!(
            get_toml_path(&theme, "flavor.light"),
            Some(&json!("custom"))
        );
        assert_eq!(get_toml_path(&theme, "mgr.cwd.fg"), Some(&json!("blue")));
    }

    #[test]
    fn prepare_file_action_creates_owned_missing_file() {
        let (_temp, paths) = temp_sources();

        prepare_file_action(&paths, SOURCE_ADVANCED, ACTION_NU_ENV, &paths.nu_env, true).unwrap();

        assert_file_text(&paths.nu_env, NU_ENV_STARTER);
        assert_missing(&[
            &paths.nu_config,
            &paths.helix_config,
            &paths.helix_languages,
            &paths.helix_module,
            &paths.helix_init,
            &paths.yazi_config,
            &paths.yazi_init,
            &paths.yazi_keymap,
            &paths.yazi_package,
            &paths.yazi_theme,
            &paths.zellij_plugins,
        ]);
    }

    #[test]
    fn helix_override_stays_sparse_and_merges_over_packaged_config() {
        let (temp, paths) = temp_sources();
        let packaged = temp.path.join("packaged.toml");
        let output = temp.path.join("state/helix/config.toml");
        prepare_file_action(
            &paths,
            SOURCE_HELIX,
            ACTION_HELIX_CONFIG,
            &paths.helix_config,
            true,
        )
        .unwrap();
        let starter = read_toml_file_value(&paths.helix_config, "Helix starter").unwrap();
        assert_eq!(starter, json!({}));

        fs::write(
            &packaged,
            concat!(
                "theme = \"ayu_evolve\"\n\n",
                "[editor]\nbufferline = \"always\"\n\n",
                "[keys.normal]\n",
                "A-r = ':sh yzx reveal \"%{buffer_name}\"'\n",
                "C-r = [\":config-reload\", \":reload\"]\n",
            ),
        )
        .unwrap();
        fs::write(
            &paths.helix_config,
            concat!(
                "[editor]\n",
                "bufferline = \"never\"\n",
                "line-number = \"relative\"\n\n",
                "[keys.normal]\n",
                "A-r = \":noop\"\n",
                "C-r = \":noop\"\n",
            ),
        )
        .unwrap();

        write_effective_helix_config(&packaged, &paths.helix_config, &output).unwrap();

        let value = read_toml_file_value(&output, "effective Helix config").unwrap();
        assert_eq!(get_toml_path(&value, "theme"), Some(&json!("ayu_evolve")));
        assert_eq!(
            get_toml_path(&value, "editor.bufferline"),
            Some(&json!("never"))
        );
        assert_eq!(
            get_toml_path(&value, "editor.line-number"),
            Some(&json!("relative"))
        );
        assert_eq!(
            get_toml_path(&value, "keys.normal.A-r"),
            Some(&json!(r#":sh yzx reveal "%{buffer_name}""#))
        );
        assert_eq!(
            get_toml_path(&value, "keys.normal.C-r"),
            Some(&json!(":noop"))
        );
    }

    #[test]
    fn effective_helix_config_rejects_non_table_keys_override() {
        let temp = TempHome::new();
        let packaged = temp.path.join("packaged.toml");
        let user = temp.path.join("user.toml");
        let output = temp.path.join("state/helix/config.toml");
        fs::write(&packaged, "[keys.normal]\nA-r = \":noop\"\n").unwrap();
        fs::write(&user, "keys = \"not a table\"\n").unwrap();

        let error = write_effective_helix_config(&packaged, &user, &output)
            .unwrap_err()
            .to_string();

        assert!(error.contains("[keys] must be a TOML table"));
        assert!(!output.exists());
    }

    #[test]
    fn prepare_file_action_creates_managed_helix_steel_pair() {
        let (_temp, paths) = temp_sources();

        prepare_file_action(
            &paths,
            SOURCE_HELIX,
            ACTION_HELIX_INIT,
            &paths.helix_init,
            true,
        )
        .unwrap();

        assert_file_text(&paths.helix_init, HELIX_INIT_STARTER);
        assert_file_text(&paths.helix_module, HELIX_MODULE_STARTER);
        assert_missing(&[
            &paths.helix_config,
            &paths.helix_languages,
            &paths.nu_env,
            &paths.yazi_init,
        ]);
    }

    #[test]
    fn prepare_existing_managed_helix_steel_row_creates_missing_pair_file() {
        let (_temp, paths) = temp_sources();
        atomic_write(&paths.helix_init, HELIX_INIT_STARTER).unwrap();

        prepare_file_action(
            &paths,
            SOURCE_HELIX,
            ACTION_HELIX_INIT,
            &paths.helix_init,
            true,
        )
        .unwrap();

        assert_eq!(
            fs::read_to_string(&paths.helix_module).unwrap(),
            HELIX_MODULE_STARTER
        );
    }

    #[test]
    fn prepare_file_action_creates_managed_yazi_files_independently() {
        let (_temp, paths) = temp_sources();
        let specs = [
            (ACTION_YAZI_CONFIG, &paths.yazi_config, YAZI_CONFIG_STARTER),
            (ACTION_YAZI_INIT, &paths.yazi_init, YAZI_INIT_STARTER),
            (ACTION_YAZI_KEYMAP, &paths.yazi_keymap, YAZI_KEYMAP_STARTER),
            (
                ACTION_YAZI_PACKAGE,
                &paths.yazi_package,
                YAZI_PACKAGE_STARTER,
            ),
            (ACTION_YAZI_THEME, &paths.yazi_theme, YAZI_THEME_STARTER),
        ];
        for &(action, target, starter) in &specs {
            let source = match action {
                ACTION_YAZI_CONFIG => SOURCE_YAZI_CONFIG,
                ACTION_YAZI_THEME => SOURCE_YAZI_THEME,
                _ => SOURCE_YAZI,
            };
            prepare_file_action(&paths, source, action, target, true).unwrap();
            assert_file_text(target, starter);
            assert_eq!(specs.iter().filter(|(_, path, _)| path.exists()).count(), 1);
            fs::remove_file(target).unwrap();
        }
        assert!(!paths.yazi_config.with_file_name("plugins").exists());
    }

    #[test]
    fn prepare_file_action_creates_zellij_plugins_sidecar_only() {
        let (_temp, paths) = temp_sources();

        prepare_file_action(
            &paths,
            SOURCE_ADVANCED,
            ACTION_ZELLIJ_PLUGINS,
            &paths.zellij_plugins,
            true,
        )
        .unwrap();

        assert_file_text(&paths.zellij_plugins, ZELLIJ_PLUGINS_STARTER);
        assert_missing(&[&paths.yazi_init, &paths.yazi_keymap]);
    }

    #[test]
    fn prepare_file_action_rejects_unowned_or_missing_paths() {
        let (_temp, paths) = temp_sources();

        for (source_id, action_id, path, create, expected) in [
            (
                SOURCE_ADVANCED,
                ACTION_NU_ENV,
                &paths.nu_config,
                true,
                "does not own",
            ),
            (
                SOURCE_ADVANCED,
                ACTION_HELIX_CONFIG,
                &paths.helix_config,
                true,
                "unknown file action",
            ),
            (
                SOURCE_ADVANCED,
                ACTION_NU_CONFIG,
                &paths.nu_config,
                false,
                "config file is missing",
            ),
        ] {
            let error = prepare_file_action(&paths, source_id, action_id, path, create)
                .unwrap_err()
                .to_string();
            assert!(error.contains(expected), "{error}");
        }
    }

    #[test]
    fn mars_source_stays_sparse_and_inherits_packaged_defaults() {
        let (_temp, paths) = temp_sources();
        let model = build_model(&paths).unwrap();
        let mars_fields: Vec<_> = model
            .fields
            .iter()
            .filter(|field| field.source_id == SOURCE_MARS)
            .collect();
        assert_eq!(mars_fields.len(), MARS_FIELDS.len());
        assert!(mars_fields.iter().all(|field| matches!(
            field.snapshot.intent,
            ConfigUiOverride::Absent
        ) && field.snapshot.effective
            == field.snapshot.baseline));

        write_source_field(&paths, SOURCE_MARS, "window.opacity", &json!(0.5)).unwrap();

        assert!(!paths.root.exists());
        let raw = fs::read_to_string(&paths.mars).unwrap();
        assert!(raw.contains("opacity = 0.5"));
        assert!(!raw.contains("width ="));
        assert!(!raw.contains("/nix/store"));

        fs::write(
            &paths.mars,
            format!("{raw}\n[colors]\nbackground = \"#010203\"\n"),
        )
        .unwrap();
        write_source_field(
            &paths,
            SOURCE_MARS,
            "mars.appearance.preset",
            &json!("light"),
        )
        .unwrap();

        let mars = read_toml_file_value(&paths.mars, "mars").unwrap();
        assert_eq!(get_toml_path(&mars, "window.opacity"), Some(&json!(0.5)));
        assert_eq!(
            get_toml_path(&mars, "mars.appearance.preset"),
            Some(&json!("light"))
        );
        assert_eq!(
            get_toml_path(&mars, "colors.background"),
            Some(&json!("#010203"))
        );
        let model = build_model(&paths).unwrap();
        assert_explicit(model_field(&model, "window.opacity"), &json!(0.5));
        assert_inherited(model_field(&model, "window.width"), &json!(960));

        write_source_default(&paths, SOURCE_MARS, "window.opacity").unwrap();
        let mars = read_toml_file_value(&paths.mars, "mars").unwrap();
        assert_eq!(get_toml_path(&mars, "window.opacity"), None);
        assert_eq!(
            get_toml_path(&mars, "colors.background"),
            Some(&json!("#010203"))
        );

        let error = write_source_field(
            &paths,
            SOURCE_MARS,
            "mars.appearance.preset",
            &json!("auto"),
        )
        .unwrap_err()
        .to_string();
        assert!(error.contains("dark, light"), "{error}");

        let error = write_source_field(&paths, SOURCE_MARS, "force-theme", &json!("light"))
            .unwrap_err()
            .to_string();
        assert!(error.contains("unknown Mars config path"), "{error}");

        let error = write_source_field(&paths, SOURCE_MARS, "colors.background", &json!("#f5f3ef"))
            .unwrap_err()
            .to_string();
        assert!(error.contains("unknown Mars config path"), "{error}");
    }

    #[test]
    fn source_routing_writes_starship_without_touching_config_toml() {
        let (_temp, paths) = temp_sources();

        write_source_field(&paths, SOURCE_STARSHIP, "character.format", &json!(">> ")).unwrap();

        assert!(!paths.root.exists());
        let starship = read_toml_file_value(&paths.starship, "starship").unwrap();
        assert_eq!(
            get_toml_path(&starship, "character.format"),
            Some(&json!(">> "))
        );
        assert_explicit(
            model_field(&build_model(&paths).unwrap(), "character.format"),
            &json!(">> "),
        );
        write_source_default(&paths, SOURCE_STARSHIP, "character.format").unwrap();
        assert!(!paths.starship.exists());

        fs::write(&paths.starship, "right_format = \"$time\"\n").unwrap();
        write_source_field(&paths, SOURCE_STARSHIP, "character.format", &json!(">> ")).unwrap();
        write_source_default(&paths, SOURCE_STARSHIP, "character.format").unwrap();
        let starship = read_toml_file_value(&paths.starship, "starship").unwrap();
        assert_eq!(
            get_toml_path(&starship, "right_format"),
            Some(&json!("$time"))
        );

        let error = write_source_field(&paths, SOURCE_STARSHIP, "format", &json!("$all"))
            .unwrap_err()
            .to_string();
        assert!(error.contains("unknown Starship config path"));
    }

    #[test]
    fn effective_starship_config_layers_sparse_user_values_over_defaults() {
        let temp = TempHome::new();
        let user = temp.path.join("config/starship.toml");
        let output = temp.path.join("state/starship.toml");
        fs::create_dir_all(user.parent().unwrap()).unwrap();
        fs::write(
            &user,
            "right_format = \"$time\"\n\n[character]\nformat = \">> \"\n\n[time]\ndisabled = false\n",
        )
        .unwrap();

        write_effective_starship_config(&user, &output).unwrap();

        let value = read_toml_file_value(&output, "effective Starship config").unwrap();
        assert_eq!(get_toml_path(&value, "format"), None);
        assert_eq!(get_toml_path(&value, "right_format"), Some(&json!("$time")));
        assert_eq!(get_toml_path(&value, "add_newline"), None);
        assert_eq!(
            get_toml_path(&value, "character.format"),
            Some(&json!(">> "))
        );
        assert_eq!(get_toml_path(&value, "time.disabled"), Some(&json!(false)));
    }

    #[test]
    fn zellij_sidecar_stays_sparse_and_inherits_packaged_defaults() {
        let (_temp, paths) = temp_sources();
        assert!(!paths.zellij.exists());

        let model = build_model(&paths).unwrap();
        let zellij_fields: Vec<_> = model
            .fields
            .iter()
            .filter(|field| field.source_id == SOURCE_ZELLIJ)
            .collect();
        assert_eq!(zellij_fields.len(), ZELLIJ_FIELDS.len());
        assert!(zellij_fields.iter().all(|field| matches!(
            field.snapshot.intent,
            ConfigUiOverride::Absent
        ) && field.snapshot.effective
            == field.snapshot.baseline));
        assert_inherited(model_field(&model, "pane_frames"), &json!(true));

        write_source_field(&paths, SOURCE_ZELLIJ, "pane_frames", &json!(true)).unwrap();
        assert_eq!(
            fs::read_to_string(&paths.zellij).unwrap(),
            "pane_frames true\n"
        );
        assert_explicit(
            model_field(&build_model(&paths).unwrap(), "pane_frames"),
            &json!(true),
        );

        write_source_field(
            &paths,
            SOURCE_ZELLIJ,
            "ui.pane_frames.rounded_corners",
            &json!(true),
        )
        .unwrap();

        let raw = fs::read_to_string(&paths.zellij).unwrap();
        assert_eq!(
            raw,
            "pane_frames true\n\nui {\n    pane_frames {\n        rounded_corners true\n    }\n}\n"
        );
        atomic_write(
            &paths.zellij,
            &format!(
                "# keep\n{}",
                raw.replacen("pane_frames true", "pane_frames true // { reset me", 1)
            ),
        )
        .unwrap();
        write_source_default(&paths, SOURCE_ZELLIJ, "pane_frames").unwrap();
        let raw = fs::read_to_string(&paths.zellij).unwrap();
        assert!(raw.starts_with("# keep\n"));
        assert!(!raw.contains("reset me"));
        assert!(raw.contains("rounded_corners true"));
        write_source_field(&paths, SOURCE_ZELLIJ, "mouse_mode", &json!(false)).unwrap();
        write_source_default(&paths, SOURCE_ZELLIJ, "ui.pane_frames.rounded_corners").unwrap();
        let raw = fs::read_to_string(&paths.zellij).unwrap();
        assert!(raw.starts_with("# keep\n"));
        assert!(raw.contains("mouse_mode false"));
        assert!(!raw.contains("ui {"));
        write_source_default(&paths, SOURCE_ZELLIJ, "mouse_mode").unwrap();
        assert!(!paths.zellij.exists());
    }

    #[test]
    fn zellij_theme_picker_preserves_custom_names_and_resets_sparsely() {
        let (_temp, paths) = temp_sources();
        let model = build_model(&paths).unwrap();
        let theme = model_field(&model, "theme");
        assert_inherited(theme, &json!("default"));
        assert_eq!(choice_values(theme).first(), Some(&&json!("default")));
        assert!(choice_values(theme).contains(&&json!("atelier-sulphurpool")));
        assert!(!choice_values(theme).contains(&&json!("atelier")));
        assert_eq!(theme.apply_status.summary, "live");

        atomic_write(
            &paths.zellij,
            "# keep\ntheme \"custom {ocean}\"\npane_frames false\n",
        )
        .unwrap();
        let model = build_model(&paths).unwrap();
        let theme = model_field(&model, "theme");
        assert_explicit(theme, &json!("custom {ocean}"));
        assert!(choice_values(theme).contains(&&json!("custom {ocean}")));

        write_source_field(&paths, SOURCE_ZELLIJ, "theme", &json!("dracula")).unwrap();
        let raw = fs::read_to_string(&paths.zellij).unwrap();
        assert!(raw.starts_with("# keep\n"));
        assert!(raw.contains("theme \"dracula\""));
        assert!(raw.contains("pane_frames false"));
        assert!(!raw.contains("custom {ocean}"));

        write_source_field(&paths, SOURCE_ZELLIJ, "theme", &json!("default")).unwrap();
        let raw = fs::read_to_string(&paths.zellij).unwrap();
        assert!(raw.starts_with("# keep\n"));
        assert!(raw.contains("pane_frames false"));
        assert!(!raw.contains("theme "));
    }

    #[test]
    fn zellij_theme_edits_preserve_opaque_native_leaf_nodes() {
        let (_temp, paths) = temp_sources();
        let raw = "// keep exactly\ndefault_mode \"normal\"\nfuture_flag;\nfuture_multi 1 2\nfuture_property mode=\"fast\"\nfuture_label \"two words // exact\"\nfuture_shape \"{opaque}\"\n";
        atomic_write(&paths.zellij, raw).unwrap();

        let model = build_model(&paths).unwrap();
        for path in [
            "default_mode",
            "future_flag",
            "future_multi",
            "future_property",
            "future_label",
            "future_shape",
        ] {
            let diagnostic = model
                .diagnostics
                .iter()
                .find(|diagnostic| diagnostic.headline.contains(path))
                .expect("unvalidated native setting diagnostic");
            assert!(!diagnostic.blocking);
            assert_eq!(diagnostic.status, "unvalidated");
            assert_eq!(
                diagnostic.scope,
                ConfigUiDiagnosticScope::Source {
                    source_id: SOURCE_ZELLIJ.to_string()
                }
            );
        }
        let theme = model_field(&model, "theme");
        assert_inherited(theme, &json!("default"));

        write_source_field(&paths, SOURCE_ZELLIJ, "theme", &json!("ansi")).unwrap();
        assert_eq!(
            fs::read_to_string(&paths.zellij).unwrap(),
            format!("{raw}theme \"ansi\"\n")
        );

        write_source_default(&paths, SOURCE_ZELLIJ, "theme").unwrap();
        assert_eq!(fs::read_to_string(&paths.zellij).unwrap(), raw);
    }

    #[test]
    fn zellij_runtime_field_patch_preserves_surrounding_config() {
        let runtime = "\
keybinds {}\n\
pane_frames true\n\
mouse_mode true\n\
plugins {}\n\
ui {\n\
    pane_frames {\n\
        rounded_corners false\n\
    }\n\
}\n";
        let patched = patch_zellij_field_in_text(runtime, "pane_frames", &json!(false)).unwrap();
        assert!(patched.contains("keybinds {}"));
        assert!(patched.contains("pane_frames false"));
        assert!(patched.contains("plugins {}"));
        assert!(!patched.contains("pane_frames true"));

        let rounded =
            patch_zellij_field_in_text(runtime, "ui.pane_frames.rounded_corners", &json!(true))
                .unwrap();
        assert!(rounded.contains("rounded_corners true"));
        assert!(!rounded.contains("rounded_corners false"));
        assert!(rounded.contains("keybinds {}"));

        let appended = patch_zellij_field_in_text(
            "keybinds {}\n",
            "ui.pane_frames.rounded_corners",
            &json!(true),
        )
        .unwrap();
        assert!(appended.contains("ui {"));
        assert!(appended.contains("        rounded_corners true"));

        let themed = patch_zellij_field_in_text(runtime, "theme", &json!("dracula")).unwrap();
        assert!(themed.contains("theme \"dracula\""));
        let reset = unset_zellij_field_in_text(&themed, "theme").unwrap();
        assert!(!reset.contains("theme "));
        assert!(reset.contains("keybinds {}"));
        assert!(reset.contains("plugins {}"));
    }

    #[test]
    fn zellij_source_blocks_guarded_sidecar_nodes() {
        let (_temp, paths) = temp_sources();
        let path = &paths.zellij;
        atomic_write(path, "keybinds {}\npane_frames true\n").unwrap();

        let (_config, _invalid, diagnostics) =
            parse_zellij_sidecar(&fs::read_to_string(path).unwrap());
        assert!(diagnostics.iter().any(|diagnostic| {
            diagnostic.blocking
                && diagnostic.scope
                    == ConfigUiDiagnosticScope::Source {
                        source_id: SOURCE_ZELLIJ.to_string(),
                    }
        }));
        let model = build_model(&paths).unwrap();
        assert_eq!(
            model_field(&model, "theme").snapshot.intent,
            ConfigUiOverride::Absent
        );
        assert_inherited(model_field(&model, "window.width"), &json!(960));

        let error = write_zellij_config_field(path, "pane_frames", &json!(false)).unwrap_err();
        assert!(error.to_string().contains("guarded Zellij node"));
        let error = unset_zellij_config_field(path, "pane_frames").unwrap_err();
        assert!(error.to_string().contains("guarded Zellij node"));
    }

    #[test]
    fn zellij_field_diagnostics_do_not_poison_unrelated_fields_or_sources() {
        let (_temp, paths) = temp_sources();
        atomic_write(&paths.zellij, "scroll_buffer_size \"100\"\n").unwrap();

        let model = build_model(&paths).unwrap();
        assert_eq!(
            model_field(&model, "scroll_buffer_size").snapshot.intent,
            ConfigUiOverride::Invalid {
                input: "\"100\"".to_string()
            }
        );
        assert_inherited(model_field(&model, "theme"), &json!("default"));
        assert_inherited(model_field(&model, "window.width"), &json!(960));

        write_source_field(&paths, SOURCE_ZELLIJ, "theme", &json!("ansi")).unwrap();
        assert_eq!(
            fs::read_to_string(&paths.zellij).unwrap(),
            "scroll_buffer_size \"100\"\ntheme \"ansi\"\n"
        );
        write_source_field(&paths, SOURCE_ZELLIJ, "scroll_buffer_size", &json!(5000)).unwrap();
        assert_eq!(
            fs::read_to_string(&paths.zellij).unwrap(),
            "scroll_buffer_size 5000\ntheme \"ansi\"\n"
        );
    }

    #[test]
    fn zellij_sidecar_skips_hash_comments_and_blocks_compact_guarded_nodes() {
        let (config, invalid, diagnostics) = parse_zellij_sidecar("# note\npane_frames false;\n");
        assert!(invalid.is_empty());
        assert!(diagnostics.is_empty());
        assert_eq!(config.get("pane_frames").cloned(), Some(json!(false)));

        let (_config, _invalid, diagnostics) = parse_zellij_sidecar("# note\nkeybinds{}\n");
        assert!(has_diagnostic(&diagnostics, "guarded Zellij node"));
    }

    #[test]
    fn zellij_sidecar_rejects_unsupported_scalars_and_unclosed_blocks() {
        let temp = TempHome::new();
        let path = temp.path.join("zellij/config.kdl");
        atomic_write(&path, "pane_frames true\n").unwrap();

        let error = write_zellij_config_field(&path, "scroll_buffer_size", &json!(-1)).unwrap_err();
        assert!(error.to_string().contains("positive integer"));
        let error = write_zellij_config_field(&path, "theme", &json!("custom\\name")).unwrap_err();
        assert!(error.to_string().contains("without escapes"));

        let (_config, invalid, diagnostics) = parse_zellij_sidecar("scroll_buffer_size -1\n");
        assert_eq!(invalid.get("scroll_buffer_size"), Some(&"-1".to_string()));
        let diagnostic = diagnostics
            .iter()
            .find(|diagnostic| diagnostic.headline.contains("scroll_buffer_size"))
            .unwrap();
        assert_eq!(
            diagnostic.scope,
            ConfigUiDiagnosticScope::Field(ConfigUiFieldId::new(
                SOURCE_ZELLIJ,
                "scroll_buffer_size"
            ))
        );

        for raw in ["theme ansi\n", "theme \"custom\\\\name\"\n"] {
            let (config, invalid, diagnostics) = parse_zellij_sidecar(raw);
            assert!(!config.contains_key("theme"));
            assert!(invalid.contains_key("theme"));
            assert!(diagnostics.iter().any(|diagnostic| {
                diagnostic.scope
                    == ConfigUiDiagnosticScope::Field(ConfigUiFieldId::new(SOURCE_ZELLIJ, "theme"))
            }));
        }

        let (_config, _invalid, diagnostics) = parse_zellij_sidecar("ui {\n");
        assert!(has_diagnostic(&diagnostics, "unterminated"));
    }

    #[test]
    fn zellij_sidecar_blocks_structurally_unsafe_native_nodes() {
        for raw in [
            "future_block {\n    value 1\n}\n",
            "future_label \"unterminated\n",
            "future_flag; theme \"ansi\"\n",
            "/-\ntheme \"ansi\"\n",
            "/*\ntheme \"ansi\"\n*/\n",
            "theme \\\n\"ansi\"\n",
            "ui mode=\"future\" {\n}\n",
            "ui {\n    pane_frames mode=\"future\" {\n    }\n}\n",
            "ui {\n}; theme \"ansi\"\n",
            "theme \"ansi\" {\n}\n",
            "pane_frames true false\n",
            "pane_frames true\npane_frames false\n",
            "rounded_corners true\n",
        ] {
            let (_config, _invalid, diagnostics) = parse_zellij_sidecar(raw);
            assert!(
                diagnostics.iter().any(|diagnostic| {
                    diagnostic.blocking
                        && matches!(diagnostic.scope, ConfigUiDiagnosticScope::Source { .. })
                }),
                "expected source blocker for {raw:?}"
            );
        }
    }

    #[test]
    fn unsupported_terminal_keys_are_ignored() {
        for key in [
            KeyEvent::new_with_kind(
                KeyCode::Char('q'),
                KeyModifiers::NONE,
                KeyEventKind::Release,
            ),
            KeyEvent::new(KeyCode::Char('u'), KeyModifiers::ALT),
            KeyEvent::new(KeyCode::F(1), KeyModifiers::NONE),
        ] {
            assert_eq!(config_event(Event::Key(key)), None);
        }
    }

    #[test]
    fn terminal_events_translate_inline_editor_navigation_and_paste() {
        for (code, expected) in [
            (KeyCode::Delete, ConfigUiKey::Delete),
            (KeyCode::Home, ConfigUiKey::Home),
            (KeyCode::End, ConfigUiKey::End),
        ] {
            assert_eq!(
                config_event(Event::Key(KeyEvent::new(code, KeyModifiers::NONE))),
                Some(expected)
            );
        }
        assert_eq!(
            config_event(Event::Paste("middle 👩‍💻".to_string())),
            Some(ConfigUiKey::Paste("middle 👩‍💻".to_string()))
        );
    }
}
