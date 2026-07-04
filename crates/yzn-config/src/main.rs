use std::{env, process};

mod catalog;
mod common;
mod custom_popups;
mod file_actions;
mod model;
mod native_config;
mod paths;
mod root_config;
mod ui;
mod zellij_sidecar;

use catalog::*;
use common::*;
use custom_popups::*;
use paths::*;
use root_config::*;
use ui::*;

fn main() {
    if let Err(error) = run() {
        eprintln!("yzn-config: {error}");
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
        Some(arg) => Err(error(format!("unknown argument: {arg}"))),
    }
}

fn print_config_field(path: &str) -> Result<()> {
    if path == BAR_WIDGETS_PATH {
        let config = ensure_config_file_at(config_paths()?.root)?;
        println!("{}", read_bar_widgets_field(&config)?);
    } else if path == CUSTOM_POPUPS_KDL_PATH {
        let config = ensure_config_file_at(config_paths()?.root)?;
        print!("{}", read_custom_popups_kdl(&config)?);
    } else if path == CUSTOM_POPUP_KEYBINDINGS_KDL_PATH {
        let config = ensure_config_file_at(config_paths()?.root)?;
        print!("{}", read_custom_popup_keybindings_kdl(&config)?);
    } else {
        let spec = config_field(path)?;
        let config = ensure_config_file_at(config_paths()?.root)?;
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
    use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
    use ratconfig::toml_adapter::{get_toml_path, set_toml_value_text};
    use ratconfig::{ConfigUiDiagnostic, ConfigUiEditBehavior, ConfigUiModel, ConfigUiValueState};
    use serde_json::{Value as JsonValue, json};

    struct TempHome {
        path: PathBuf,
    }

    impl TempHome {
        fn new() -> Self {
            let path = env::temp_dir().join(format!(
                "yzn-config-test-{}-{}",
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
        ConfigPaths {
            root: temp.path.join("config.toml"),
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
            yazi_init: temp.path.join("yazi/init.lua"),
            yazi_keymap: temp.path.join("yazi/keymap.toml"),
            zellij_plugins: temp.path.join("zellij/plugins.kdl"),
        }
    }

    fn ensure_temp_sources(paths: &ConfigPaths) {
        ensure_config_file_at(paths.root.clone()).unwrap();
        ensure_plain_config_file_at(&paths.mars, DEFAULT_MARS_CONFIG_TOML).unwrap();
        ensure_plain_config_file_at(
            &paths.zellij,
            &render_zellij_sidecar(&ZellijSidecar::default()),
        )
        .unwrap();
        ensure_plain_config_file_at(&paths.starship, DEFAULT_STARSHIP_CONFIG_TOML).unwrap();
    }

    fn temp_sources() -> (TempHome, ConfigPaths) {
        let temp = TempHome::new();
        let paths = temp_paths(&temp);
        ensure_temp_sources(&paths);
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

    fn write_toml_value(path: &Path, field_path: &str, value: &JsonValue) {
        let raw = fs::read_to_string(path).unwrap();
        let updated = set_toml_value_text(&raw, field_path, value).unwrap().text;
        fs::write(path, updated).unwrap();
    }

    fn write_config_text(path: &Path, text: &str) {
        fs::write(path, text).unwrap();
        ensure_config_file_at(path.to_path_buf()).unwrap();
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

    #[cfg(unix)]
    #[test]
    fn external_text_editor_round_trips_staged_input() {
        use std::os::unix::fs::PermissionsExt;

        let temp = TempHome::new();
        let editor = temp.path.join("editor.sh");
        fs::write(
            &editor,
            "#!/bin/sh\ncat > \"$1\" <<'EOF'\nline one\nline two\nEOF\n",
        )
        .unwrap();
        let mut permissions = fs::metadata(&editor).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&editor, permissions).unwrap();

        assert_eq!(
            edit_text_with_editor("original", &editor).unwrap(),
            "line one\nline two"
        );
    }

    fn model_field<'a>(model: &'a ConfigUiModel, path: &str) -> &'a ratconfig::ConfigUiField {
        model
            .fields
            .iter()
            .find(|field| field.path == path)
            .unwrap_or_else(|| panic!("missing config field {path}"))
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

    fn assert_config_field(model: &ConfigUiModel, path: &str, kind: &str, summary: &str) {
        let field = model_field(model, path);
        assert_eq!(field.tab, TAB_CONFIG);
        assert_eq!(field.kind, kind);
        assert_eq!(
            field.current_value,
            default_config_value(path).unwrap().to_string()
        );
        assert_eq!(field.apply_status.summary, summary);
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

        for field_path in root_config_field_paths() {
            let value = default_config_path_value(&defaults, field_path).unwrap();
            assert_eq!(default_config_value(field_path).unwrap(), value);
            validate_config_value(field_path, &value).unwrap();
        }
        for spec in POPUP_KEYBINDINGS {
            assert_eq!(
                default_config_value(spec.path).unwrap(),
                json!(spec.default),
                "{}",
                spec.path
            );
        }
    }

    #[test]
    fn ensure_config_creates_defaults_and_contract_state() {
        let temp = TempHome::new();
        let path = ensure_config_file_at(temp.path.join("config.toml")).unwrap();
        let value = read_toml_file_value(&path, "config.toml").unwrap();
        let defaults = default_config().unwrap();

        for field_path in root_config_field_paths() {
            assert_eq!(
                get_toml_path(&value, field_path),
                get_toml_path(&defaults, field_path),
                "{field_path}"
            );
        }
        assert_eq!(
            get_toml_path(&value, "ratconfig.contract.contract_id"),
            Some(&json!(CONTRACT_ID))
        );
        assert_eq!(
            get_toml_path(&value, "ratconfig.contract.version"),
            Some(&json!(CONTRACT_VERSION))
        );
    }

    #[test]
    fn write_config_field_persists_valid_values_and_rejects_bad_values() {
        let temp = TempHome::new();
        let path = ensure_config_file_at(temp.path.join("config.toml")).unwrap();

        for (field_path, value, read_back) in [
            (OPEN_LOG_LEVEL_PATH, json!("debug"), None),
            (SHELL_PROGRAM_PATH, json!("fish"), None),
            (EDITOR_COMMAND_PATH, json!("nvim"), Some("nvim")),
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
            (POPUP_SIDE_MARGIN_PATH, json!(-1), "zero or greater"),
            (
                KEYBINDINGS_AGENT_PATH,
                json!("Alt+Shift+A"),
                "keybindings.agent must be a key chord",
            ),
        ] {
            assert_write_config_error(&path, field_path, value, expected);
        }
        for value in ["Alt Shift h", "Alt z"] {
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
        assert_toml_value(
            &path,
            BAR_WIDGETS_PATH,
            &default_config_value(BAR_WIDGETS_PATH).unwrap(),
        );

        let error = write_config_field(&path, BAR_WIDGETS_PATH, &json!(["weather"]))
            .unwrap_err()
            .to_string();
        assert!(error.contains("bar.widgets must be one of"));
        assert!(error.contains("claude_usage"));
    }

    #[test]
    fn bar_widgets_are_read_as_json_array_and_validated() {
        let temp = TempHome::new();
        let path = ensure_config_file_at(temp.path.join("config.toml")).unwrap();

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
    fn custom_popups_validate_semantic_surface() {
        // Defends: Custom popup specs stay semantic and cannot shadow packaged popup ownership.
        for (text, expected) in [
            (
                "[popups.btm]\ncommand = \"btm --basic\"\nkeybinding = \"Alt Shift B\"\n",
                "without arguments",
            ),
            (
                "[popups.config]\ncommand = \"btm\"\nkeybinding = \"Alt Shift B\"\n",
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
                "[popups.btm]\ncommand = \"btm\"\ntitle = \"git_popup\"\nkeybinding = \"Alt Shift B\"\n",
                "popups.btm.title conflicts with packaged popup title git_popup",
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
                " mars",
                " zellij",
                " starship",
                " helix",
                " keys",
                " advanced",
            ]
        );
        assert!(!model.tabs.contains(&"shell".to_string()));
        assert_config_field(&model, SHELL_PROGRAM_PATH, "string", "new panes");
        let editor = model_field(&model, EDITOR_COMMAND_PATH);
        assert_config_field(&model, EDITOR_COMMAND_PATH, "string", "new opens");
        assert!(editor.allowed_values.is_empty());
        assert_config_field(&model, POPUP_SIDE_MARGIN_PATH, "integer", "next launch");
        assert_config_field(&model, POPUP_VERTICAL_MARGIN_PATH, "integer", "next launch");

        for (path, kind) in [
            ("force-theme", "string"),
            ("colors.background", "string"),
            ("colors.foreground", "string"),
            ("colors.dim-foreground", "string"),
            ("yazelix.cursor.divider", "string"),
            ("yazelix.cursor.colors", "string_list"),
            ("yazelix.cursor.cursor_color", "string"),
        ] {
            let field = model_field(&model, path);
            assert_eq!(field.source_id, SOURCE_MARS);
            assert_eq!(field.tab, TAB_MARS);
            assert_eq!(field.kind, kind);
            assert_eq!(field.apply_status.summary, "next launch");
            assert_eq!(field.apply_status.label, "mars");
        }
        assert_eq!(
            model_field(&model, "yazelix.cursor.divider").allowed_values,
            string_values(&["vertical", "horizontal"])
        );
        assert_eq!(
            model_field(&model, "yazelix.cursor.colors").edit_value,
            r##"["#00e6ff","#00ff66"]"##
        );

        for spec in POPUP_KEYBINDINGS {
            assert_config_field(&model, spec.path, "string", "next launch");
        }

        let field = model_field(&model, BAR_WIDGETS_PATH);

        assert_eq!(field.tab, TAB_CONFIG);
        assert_eq!(field.kind, "string_list");
        assert_eq!(field.edit_behavior, ConfigUiEditBehavior::OrderedStringList);
        assert_eq!(field.allowed_values, string_values(BAR_WIDGET_VALUES));
        assert_eq!(
            field.edit_value,
            r#"["editor","shell","term","codex_usage","cpu","ram"]"#
        );
        assert!(field.allowed_values.contains(&"claude_usage".to_string()));
    }

    #[test]
    fn config_model_exposes_structured_starship_tab() {
        let (_temp, paths) = temp_sources();

        let model = build_model(&paths).unwrap();
        let format = model_field(&model, "format");
        let right_format = model_field(&model, "right_format");

        assert!(model.tabs.contains(&TAB_STARSHIP.to_string()));
        assert!(model.sources.iter().any(|source| {
            source.id == SOURCE_STARSHIP
                && source.tab == TAB_STARSHIP
                && source.path == paths.starship
        }));
        assert_eq!(format.source_id, SOURCE_STARSHIP);
        assert_eq!(format.tab, TAB_STARSHIP);
        assert_eq!(format.kind, "string");
        assert_eq!(format.current_value, r#"":: ""#);
        assert_eq!(format.apply_status.summary, "new prompts");
        assert_eq!(right_format.current_value, r#""""#);
        assert_eq!(model_field(&model, "add_newline").current_value, "true");
        assert_eq!(
            model
                .fields
                .iter()
                .filter(|field| field.source_id == SOURCE_STARSHIP)
                .count(),
            STARSHIP_FIELDS.len()
        );
    }

    #[test]
    fn config_model_marks_invalid_bar_widgets() {
        let (_temp, paths) = temp_sources();
        write_toml_value(&paths.root, BAR_WIDGETS_PATH, &json!(["weather"]));

        let model = build_model(&paths).unwrap();
        assert_eq!(
            model_field(&model, BAR_WIDGETS_PATH).state,
            ConfigUiValueState::Invalid
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
                && matches!(
                    field.edit_behavior,
                    ConfigUiEditBehavior::StructuredOnly { .. }
                )
                && field.list_cells.len() == KEY_COLUMNS.len()
        }));

        let config_popup = key_field(&model, "Alt Shift K");
        assert_eq!(
            config_popup.display_label,
            "Popups: Alt Shift K - Toggle config popup"
        );
        assert_eq!(config_popup.current_value, "Yazelix / config.kdl");
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
        assert_eq!(yazi_zoxide.current_value, "Yazi / yazi/keymap.toml");
    }

    #[test]
    fn read_only_existing_sources_are_not_replaced() {
        let (_temp, paths) = temp_sources();

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

        let error = ensure_config_file_at(paths.root.clone())
            .unwrap_err()
            .to_string();
        assert!(error.contains("read-only"));
        assert_eq!(fs::read_to_string(&paths.root).unwrap(), before_root);
    }

    #[test]
    fn read_only_complete_root_config_accepts_format_only_drift() {
        let (_temp, paths) = temp_sources();
        let text = concat!(
            "\n[bar]\nwidgets = [\"editor\", \"shell\", \"term\", \"codex_usage\", \"cpu\", \"ram\"]\n\n",
            "[editor]\ncommand = \"yzn-hx\"\n\n",
            "[open]\nlog_level = \"info\"\n\n",
            "[popup]\nside_margin = 1\nvertical_margin = 0\n\n",
            "[keybindings]\nconfig = \"Alt Shift K\"\nagent = \"Alt Shift L\"\ngit = \"Alt Shift J\"\nmenu = \"Alt Shift M\"\n\n",
            "[ratconfig.contract]\napplied_change_ids = []\ncontract_id = \"yazelix-next.config\"\nschema_version = 1\nversion = 1\n\n",
            "[shell]\nprogram = \"fish\"\n\n",
            "[welcome]\nduration_seconds = 3\nenabled = false\nstyle = \"random\"\n",
        );

        fs::write(&paths.root, text).unwrap();
        set_read_only(&paths.root);

        ensure_config_file_at(paths.root.clone()).unwrap();
        assert_eq!(fs::read_to_string(&paths.root).unwrap(), text);
    }

    #[test]
    fn manual_invalid_log_level_is_rejected_on_read_and_marked_invalid() {
        let temp = TempHome::new();
        let path = temp.path.join("config.toml");
        fs::write(&path, "[open]\nlog_level = \"loud\"\n").unwrap();

        let error =
            read_config_field(&path, config_field(OPEN_LOG_LEVEL_PATH).unwrap()).unwrap_err();
        assert!(error.to_string().contains("off, error, info, debug"));

        let paths = temp_paths(&temp);
        ensure_temp_sources(&paths);

        let model = build_model(&paths).unwrap();
        assert_eq!(model.fields[0].state, ConfigUiValueState::Invalid);
    }

    #[test]
    fn ensure_config_sources_creates_source_backed_files() {
        let (_temp, paths) = temp_sources();

        assert_exists(&[&paths.root, &paths.mars, &paths.zellij, &paths.starship]);
        assert_missing(&[
            &paths.helix_config,
            &paths.helix_languages,
            &paths.helix_module,
            &paths.helix_init,
            &paths.nu_env,
            &paths.nu_config,
            &paths.yazi_init,
            &paths.yazi_keymap,
        ]);
        assert!(
            !fs::read_to_string(paths.mars)
                .unwrap()
                .contains("ratconfig.contract")
        );
        assert!(
            fs::read_to_string(paths.zellij)
                .unwrap()
                .contains("rounded_corners false")
        );
        assert_eq!(
            fs::read_to_string(paths.starship).unwrap(),
            DEFAULT_STARSHIP_CONFIG_TOML
        );
    }

    #[test]
    fn native_file_tabs_list_owned_file_actions() {
        let (_temp, paths) = temp_sources();

        let model = build_model(&paths).unwrap();
        assert!(model.tabs.contains(&TAB_STARSHIP.to_string()));
        assert!(model.tabs.contains(&TAB_HELIX.to_string()));
        assert!(model.sources.iter().any(|source| {
            source.id == SOURCE_HELIX && source.tab == TAB_HELIX && source.path == paths.helix_dir
        }));
        let summaries: Vec<_> = model
            .file_actions
            .iter()
            .map(|action| {
                [
                    action.source_id.as_str(),
                    action.action_id.as_str(),
                    action.tab.as_str(),
                    action.label.as_str(),
                ]
            })
            .collect();
        assert_eq!(
            summaries,
            [
                [
                    SOURCE_HELIX,
                    ACTION_HELIX_CONFIG,
                    TAB_HELIX,
                    "helix/config.toml"
                ],
                [
                    SOURCE_HELIX,
                    ACTION_HELIX_LANGUAGES,
                    TAB_HELIX,
                    "helix/languages.toml",
                ],
                [
                    SOURCE_HELIX,
                    ACTION_HELIX_MODULE,
                    TAB_HELIX,
                    "helix/helix.scm"
                ],
                [SOURCE_HELIX, ACTION_HELIX_INIT, TAB_HELIX, "helix/init.scm"],
                [SOURCE_ADVANCED, ACTION_NU_ENV, TAB_ADVANCED, "nu/env.nu"],
                [
                    SOURCE_ADVANCED,
                    ACTION_NU_CONFIG,
                    TAB_ADVANCED,
                    "nu/config.nu"
                ],
                [
                    SOURCE_ADVANCED,
                    ACTION_YAZI_INIT,
                    TAB_ADVANCED,
                    "yazi/init.lua"
                ],
                [
                    SOURCE_ADVANCED,
                    ACTION_YAZI_KEYMAP,
                    TAB_ADVANCED,
                    "yazi/keymap.toml",
                ],
                [
                    SOURCE_ADVANCED,
                    ACTION_ZELLIJ_PLUGINS,
                    TAB_ADVANCED,
                    "zellij/plugins.kdl",
                ],
            ]
        );
        assert_eq!(
            model
                .file_actions
                .iter()
                .map(|action| action.path.as_path())
                .collect::<Vec<_>>(),
            [
                paths.helix_config.as_path(),
                paths.helix_languages.as_path(),
                paths.helix_module.as_path(),
                paths.helix_init.as_path(),
                paths.nu_env.as_path(),
                paths.nu_config.as_path(),
                paths.yazi_init.as_path(),
                paths.yazi_keymap.as_path(),
                paths.zellij_plugins.as_path(),
            ]
        );
        assert!(
            model
                .file_actions
                .iter()
                .all(|action| !action.exists && action.create_if_missing)
        );
    }

    #[test]
    fn prepare_file_action_creates_owned_missing_file() {
        let (_temp, paths) = temp_sources();

        prepare_file_action(&paths, SOURCE_ADVANCED, ACTION_NU_ENV, &paths.nu_env, true).unwrap();

        assert_file_text(&paths.nu_env, NU_ENV_STARTER);
        assert!(paths.starship.exists());
        assert_missing(&[
            &paths.nu_config,
            &paths.helix_config,
            &paths.helix_languages,
            &paths.helix_module,
            &paths.helix_init,
            &paths.yazi_init,
            &paths.yazi_keymap,
            &paths.zellij_plugins,
        ]);
    }

    #[test]
    fn prepare_file_action_creates_managed_helix_toml_independently() {
        let (_temp, paths) = temp_sources();

        prepare_file_action(
            &paths,
            SOURCE_HELIX,
            ACTION_HELIX_CONFIG,
            &paths.helix_config,
            true,
        )
        .unwrap();

        assert_file_text(&paths.helix_config, HELIX_CONFIG_STARTER);
        assert_missing(&[
            &paths.helix_languages,
            &paths.helix_module,
            &paths.helix_init,
            &paths.nu_env,
            &paths.yazi_init,
        ]);
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
    fn prepare_file_action_creates_managed_yazi_init_only() {
        let (_temp, paths) = temp_sources();
        let yazi_toml = paths.yazi_init.with_file_name("yazi.toml");
        let yazi_plugins = paths.yazi_init.with_file_name("plugins");

        prepare_file_action(
            &paths,
            SOURCE_ADVANCED,
            ACTION_YAZI_INIT,
            &paths.yazi_init,
            true,
        )
        .unwrap();

        assert_file_text(&paths.yazi_init, YAZI_INIT_STARTER);
        assert_missing(&[
            &yazi_toml,
            &paths.yazi_keymap,
            &yazi_plugins,
            &paths.zellij_plugins,
        ]);
    }

    #[test]
    fn prepare_file_action_creates_managed_yazi_keymap_only() {
        let (_temp, paths) = temp_sources();
        let yazi_toml = paths.yazi_keymap.with_file_name("yazi.toml");
        let yazi_plugins = paths.yazi_keymap.with_file_name("plugins");

        prepare_file_action(
            &paths,
            SOURCE_ADVANCED,
            ACTION_YAZI_KEYMAP,
            &paths.yazi_keymap,
            true,
        )
        .unwrap();

        assert_file_text(&paths.yazi_keymap, YAZI_KEYMAP_STARTER);
        assert_missing(&[
            &paths.yazi_init,
            &yazi_toml,
            &yazi_plugins,
            &paths.zellij_plugins,
        ]);
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
    fn source_routing_writes_mars_without_touching_config_toml() {
        let (_temp, paths) = temp_sources();
        let before_root = fs::read_to_string(&paths.root).unwrap();

        write_source_field(&paths, SOURCE_MARS, "window.width", &json!(1200)).unwrap();
        write_source_field(&paths, SOURCE_MARS, "force-theme", &json!("light")).unwrap();
        write_source_field(&paths, SOURCE_MARS, "colors.background", &json!("#f5f3ef")).unwrap();
        write_source_field(
            &paths,
            SOURCE_MARS,
            "yazelix.cursor.divider",
            &json!("horizontal"),
        )
        .unwrap();
        write_source_field(
            &paths,
            SOURCE_MARS,
            "yazelix.cursor.colors",
            &json!(["#ff1600", "#20242f"]),
        )
        .unwrap();
        write_source_field(
            &paths,
            SOURCE_MARS,
            "yazelix.cursor.cursor_color",
            &json!("#0077cc"),
        )
        .unwrap();

        assert_eq!(fs::read_to_string(&paths.root).unwrap(), before_root);
        let mars = read_toml_file_value(&paths.mars, "mars").unwrap();
        assert_eq!(get_toml_path(&mars, "window.width"), Some(&json!(1200)));
        assert_eq!(get_toml_path(&mars, "force-theme"), Some(&json!("light")));
        assert_eq!(
            get_toml_path(&mars, "colors.background"),
            Some(&json!("#f5f3ef"))
        );
        assert_eq!(
            get_toml_path(&mars, "yazelix.cursor.divider"),
            Some(&json!("horizontal"))
        );
        assert_eq!(
            get_toml_path(&mars, "yazelix.cursor.colors"),
            Some(&json!(["#ff1600", "#20242f"]))
        );
        assert_eq!(
            get_toml_path(&mars, "yazelix.cursor.cursor_color"),
            Some(&json!("#0077cc"))
        );

        let error = write_source_field(&paths, SOURCE_MARS, "force-theme", &json!("auto"))
            .unwrap_err()
            .to_string();
        assert!(error.contains("dark, light"), "{error}");

        let error = write_source_field(&paths, SOURCE_MARS, "colors.background", &json!("light"))
            .unwrap_err()
            .to_string();
        assert!(error.contains("hex color"), "{error}");

        let error = write_source_field(
            &paths,
            SOURCE_MARS,
            "yazelix.cursor.divider",
            &json!("diagonal"),
        )
        .unwrap_err()
        .to_string();
        assert!(error.contains("vertical, horizontal"), "{error}");

        let error = write_source_field(
            &paths,
            SOURCE_MARS,
            "yazelix.cursor.colors",
            &json!(["#ff1600"]),
        )
        .unwrap_err()
        .to_string();
        assert!(error.contains("exactly two"), "{error}");

        let error = write_source_field(
            &paths,
            SOURCE_MARS,
            "yazelix.cursor.colors",
            &json!(["#ff1600", "blue"]),
        )
        .unwrap_err()
        .to_string();
        assert!(error.contains("hex color"), "{error}");
    }

    #[test]
    fn source_routing_writes_starship_without_touching_config_toml() {
        let (_temp, paths) = temp_sources();
        let before_root = fs::read_to_string(&paths.root).unwrap();

        write_source_field(&paths, SOURCE_STARSHIP, "right_format", &json!("$time")).unwrap();
        write_source_field(&paths, SOURCE_STARSHIP, "add_newline", &json!(false)).unwrap();
        write_source_default(&paths, SOURCE_STARSHIP, "format").unwrap();

        assert_eq!(fs::read_to_string(&paths.root).unwrap(), before_root);
        let starship = read_toml_file_value(&paths.starship, "starship").unwrap();
        assert_eq!(
            get_toml_path(&starship, "right_format"),
            Some(&json!("$time"))
        );
        assert_eq!(get_toml_path(&starship, "add_newline"), Some(&json!(false)));
        assert_eq!(get_toml_path(&starship, "format"), Some(&json!(":: ")));

        let error = write_source_field(&paths, SOURCE_STARSHIP, "add_newline", &json!("nope"))
            .unwrap_err()
            .to_string();
        assert!(error.contains("true or false"));
    }

    #[test]
    fn zellij_source_renders_nested_rounded_corners() {
        let (_temp, paths) = temp_sources();

        write_source_field(
            &paths,
            SOURCE_ZELLIJ,
            "ui.pane_frames.rounded_corners",
            &json!(true),
        )
        .unwrap();

        let raw = fs::read_to_string(paths.zellij).unwrap();
        assert!(raw.contains("ui {"));
        assert!(raw.contains("rounded_corners true"));
    }

    #[test]
    fn zellij_source_blocks_guarded_sidecar_nodes() {
        let temp = TempHome::new();
        let path = temp.path.join("zellij/config.kdl");
        atomic_write(&path, "keybinds {}\npane_frames true\n").unwrap();

        let (_config, diagnostics) = parse_zellij_sidecar(&fs::read_to_string(&path).unwrap());
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.blocking));

        let error = write_zellij_config_field(&path, "pane_frames", &json!(false)).unwrap_err();
        assert!(error.to_string().contains("guarded Zellij node"));
    }

    #[test]
    fn zellij_sidecar_skips_hash_comments_and_blocks_compact_guarded_nodes() {
        let (config, diagnostics) = parse_zellij_sidecar("# note\npane_frames false;\n");
        assert!(diagnostics.is_empty());
        assert!(!config.pane_frames);

        let (_config, diagnostics) = parse_zellij_sidecar("# note\nkeybinds{}\n");
        assert!(has_diagnostic(&diagnostics, "guarded Zellij node"));
    }

    #[test]
    fn zellij_sidecar_rejects_non_positive_scrollback_and_unclosed_blocks() {
        let temp = TempHome::new();
        let path = temp.path.join("zellij/config.kdl");
        atomic_write(&path, &render_zellij_sidecar(&ZellijSidecar::default())).unwrap();

        let error = write_zellij_config_field(&path, "scroll_buffer_size", &json!(-1)).unwrap_err();
        assert!(error.to_string().contains("positive integer"));

        let (_config, diagnostics) = parse_zellij_sidecar("scroll_buffer_size -1\n");
        assert!(has_diagnostic(&diagnostics, "scroll_buffer_size"));

        let (_config, diagnostics) = parse_zellij_sidecar("ui {\n");
        assert!(has_diagnostic(&diagnostics, "unterminated"));
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
            assert_eq!(config_key(key), None);
        }
    }
}
