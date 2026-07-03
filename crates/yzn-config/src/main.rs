use std::{
    collections::BTreeMap,
    env, fs, io,
    path::{Path, PathBuf},
    process::{self, Command},
    time::{SystemTime, UNIX_EPOCH},
};

use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use ratconfig::toml_adapter::{get_toml_path, parse_toml_value, set_toml_value_text};
use ratconfig::{
    ConfigContract, ConfigUiApp, ConfigUiApplyStatus, ConfigUiDiagnostic, ConfigUiEditBehavior,
    ConfigUiFieldRowSpec, ConfigUiFileAction, ConfigUiIntent, ConfigUiKey, ConfigUiListColumn,
    ConfigUiListTable, ConfigUiModel, ConfigUiPathOwner, ConfigUiSource,
    ConfigUiStringListChoiceSpec, build_config_ui_field, build_string_list_choice_field,
    draw_config_ui, join_toml_contract_text_from_version, reconcile_joined_toml_contract_text,
    string_list_values_from_json,
};
use serde_json::{Value as JsonValue, json};

mod catalog;

use catalog::*;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

struct ConfigPaths {
    root: PathBuf,
    mars: PathBuf,
    zellij: PathBuf,
    helix_dir: PathBuf,
    helix_config: PathBuf,
    helix_languages: PathBuf,
    helix_module: PathBuf,
    helix_init: PathBuf,
    nu_env: PathBuf,
    nu_config: PathBuf,
    starship: PathBuf,
    yazi_init: PathBuf,
    yazi_keymap: PathBuf,
}

struct FileActionSpec {
    source_id: &'static str,
    action_id: &'static str,
    tab: &'static str,
    label: &'static str,
    description: &'static str,
    path: PathBuf,
    starter: &'static str,
}

impl FieldSpec {
    fn json_choice<'a>(&self, value: &'a JsonValue) -> Result<&'a str> {
        let Some(value) = value.as_str() else {
            return Err(error(format!("{} must be a string", self.path)));
        };
        if !self.allowed_values.is_empty() && !self.allowed_values.contains(&value) {
            return Err(error(format!(
                "{} must be one of: {}",
                self.path,
                self.allowed_values.join(", ")
            )));
        }
        Ok(value)
    }
}

struct ZellijSidecar {
    pane_frames: bool,
    mouse_mode: bool,
    scroll_buffer_size: i64,
    copy_on_select: bool,
    copy_clipboard: String,
    styled_underlines: bool,
    show_startup_tips: bool,
    rounded_corners: bool,
}

impl Default for ZellijSidecar {
    fn default() -> Self {
        Self {
            pane_frames: true,
            mouse_mode: true,
            scroll_buffer_size: 10000,
            copy_on_select: true,
            copy_clipboard: "system".to_string(),
            styled_underlines: true,
            show_startup_tips: true,
            rounded_corners: false,
        }
    }
}

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

fn run_ui() -> Result<()> {
    let paths = ensure_config_sources()?;
    let mut app = ConfigUiApp::new(build_model(&paths)?);
    let mut session = TerminalSession::enter()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;

    loop {
        terminal.draw(|frame| draw_config_ui(frame, &mut app))?;
        let Event::Key(key) = event::read()? else {
            continue;
        };
        let Some(key) = config_key(key) else {
            continue;
        };
        match app.handle_key(key) {
            ConfigUiIntent::Exit => break,
            ConfigUiIntent::None => {}
            ConfigUiIntent::BeginEdit { field_index, .. } => app.begin_edit_field(field_index),
            ConfigUiIntent::EditTextExternally {
                field_index, input, ..
            } => {
                let result = session.suspend(|| edit_text_externally(&input))?;
                match result {
                    Ok(edited) => {
                        if let Err(message) = app.apply_external_text_edit(field_index, edited) {
                            app.notice_error(message);
                        }
                    }
                    Err(error) => app.notice_error(error.to_string()),
                }
            }
            ConfigUiIntent::OpenFile {
                source_id,
                action_id,
                path,
                create_if_missing,
                ..
            } => {
                let result = session.suspend(|| {
                    open_file_action(&paths, &source_id, &action_id, &path, create_if_missing)
                })?;
                app.model = build_model(&paths)?;
                match result {
                    Ok(()) => app.notice_info(format!("Opened {}.", path.display())),
                    Err(error) => app.notice_error(error.to_string()),
                }
            }
            ConfigUiIntent::SetField {
                source_id,
                path: field_path,
                value,
                ..
            } => {
                if let Err(error) = write_source_field(&paths, &source_id, &field_path, &value) {
                    app.notice_error(error.to_string());
                    app.model = build_model(&paths)?;
                    continue;
                }
                app.model = build_model(&paths)?;
                app.notice_info(format!("Saved {field_path}."));
                app.finish_successful_write();
            }
            ConfigUiIntent::UnsetField {
                source_id,
                path: field_path,
                ..
            } => {
                if let Err(error) = write_source_default(&paths, &source_id, &field_path) {
                    app.notice_error(error.to_string());
                    app.model = build_model(&paths)?;
                    continue;
                }
                app.model = build_model(&paths)?;
                app.notice_info(format!("Restored default for {field_path}."));
            }
        }
    }

    Ok(())
}

struct TerminalSession;

impl TerminalSession {
    fn enter() -> Result<Self> {
        enable_raw_mode()?;
        let session = Self;
        execute!(io::stdout(), EnterAlternateScreen, cursor::Hide)?;
        Ok(session)
    }

    fn suspend<T>(&mut self, action: impl FnOnce() -> Result<T>) -> Result<Result<T>> {
        disable_raw_mode()?;
        execute!(io::stdout(), cursor::Show, LeaveAlternateScreen)?;
        let result = action();
        enable_raw_mode()?;
        execute!(io::stdout(), EnterAlternateScreen, cursor::Hide)?;
        Ok(result)
    }
}

impl Drop for TerminalSession {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), cursor::Show, LeaveAlternateScreen);
    }
}

fn config_key(key: KeyEvent) -> Option<ConfigUiKey> {
    if key.kind == KeyEventKind::Release {
        return None;
    }
    let unsupported =
        KeyModifiers::ALT | KeyModifiers::SUPER | KeyModifiers::HYPER | KeyModifiers::META;
    match key.code {
        KeyCode::Esc => Some(ConfigUiKey::Esc),
        KeyCode::Enter => Some(ConfigUiKey::Enter),
        KeyCode::Backspace => Some(ConfigUiKey::Backspace),
        KeyCode::Tab => Some(ConfigUiKey::Tab),
        KeyCode::BackTab => Some(ConfigUiKey::BackTab),
        KeyCode::Up => Some(ConfigUiKey::Up),
        KeyCode::Down => Some(ConfigUiKey::Down),
        KeyCode::Left => Some(ConfigUiKey::Left),
        KeyCode::Right => Some(ConfigUiKey::Right),
        KeyCode::Char(_) if key.modifiers.intersects(unsupported) => None,
        KeyCode::Char(ch) if key.modifiers.contains(KeyModifiers::CONTROL) => {
            Some(ConfigUiKey::Ctrl(ch))
        }
        KeyCode::Char(ch) => Some(ConfigUiKey::Char(ch)),
        _ => None,
    }
}

fn print_config_field(path: &str) -> Result<()> {
    if path == BAR_WIDGETS_PATH {
        let config = ensure_config_file_at(config_paths()?.root)?;
        println!("{}", read_bar_widgets_field(&config)?);
    } else {
        let spec = config_field(path)?;
        let config = ensure_config_file_at(config_paths()?.root)?;
        println!("{}", read_config_field(&config, spec)?);
    }
    Ok(())
}

fn config_field(path: &str) -> Result<&'static ConfigFieldSpec> {
    CONFIG_FIELDS
        .iter()
        .find(|spec| spec.field.path == path)
        .ok_or_else(|| error(format!("unknown config path: {path}")))
}

fn ensure_config_sources() -> Result<ConfigPaths> {
    let paths = config_paths()?;
    ensure_config_file_at(paths.root.clone())?;
    ensure_plain_config_file_at(&paths.mars, DEFAULT_MARS_CONFIG_TOML)?;
    ensure_plain_config_file_at(
        &paths.zellij,
        &render_zellij_sidecar(&ZellijSidecar::default()),
    )?;
    ensure_plain_config_file_at(&paths.starship, DEFAULT_STARSHIP_CONFIG_TOML)?;
    Ok(paths)
}

fn root_config_field_paths() -> impl Iterator<Item = &'static str> {
    CONFIG_FIELDS
        .iter()
        .map(|spec| spec.field.path)
        .chain([BAR_WIDGETS_PATH])
}

fn ensure_config_file_at(path: PathBuf) -> Result<PathBuf> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let raw = if path.exists() {
        fs::read_to_string(&path)?
    } else {
        DEFAULT_CONFIG_TOML.to_string()
    };
    let reconciled = reconcile_contract(&raw)?;
    let completed = fill_missing_defaults(&reconciled)?;
    if completed != raw || !path.exists() {
        if path.exists() {
            if path_read_only(&path) && toml_semantically_equal(&raw, &completed)? {
                return Ok(path);
            }
            reject_read_only_source(&path, SOURCE_CONFIG)?;
        }
        atomic_write(&path, &completed)?;
    }
    Ok(path)
}

fn toml_semantically_equal(left: &str, right: &str) -> Result<bool> {
    Ok(
        parse_toml_value(left).map_err(|error| boxed_debug("invalid TOML", error))?
            == parse_toml_value(right).map_err(|error| boxed_debug("invalid TOML", error))?,
    )
}

fn ensure_plain_config_file_at(path: &Path, default: &str) -> Result<()> {
    if path.exists() {
        return Ok(());
    }
    atomic_write(path, default)
}

fn config_paths() -> Result<ConfigPaths> {
    let home = config_home()?;
    Ok(ConfigPaths {
        root: home.join("config.toml"),
        mars: home.join("mars/config.toml"),
        zellij: home.join("zellij/config.kdl"),
        helix_dir: home.join("helix"),
        helix_config: home.join("helix/config.toml"),
        helix_languages: home.join("helix/languages.toml"),
        helix_module: home.join("helix/helix.scm"),
        helix_init: home.join("helix/init.scm"),
        nu_env: home.join("nu/env.nu"),
        nu_config: home.join("nu/config.nu"),
        starship: home.join("starship.toml"),
        yazi_init: home.join("yazi/init.lua"),
        yazi_keymap: home.join("yazi/keymap.toml"),
    })
}

fn config_home() -> Result<PathBuf> {
    if let Some(path) = env::var_os("YAZELIX_NEXT_CONFIG_HOME") {
        return Ok(PathBuf::from(path));
    }
    if let Some(path) = env::var_os("XDG_CONFIG_HOME") {
        return Ok(PathBuf::from(path).join("yazelix-next"));
    }
    let home = env::var_os("HOME").ok_or_else(|| error("HOME is required"))?;
    Ok(PathBuf::from(home).join(".config/yazelix-next"))
}

fn reconcile_contract(raw: &str) -> Result<String> {
    let contract = ConfigContract {
        id: CONTRACT_ID.to_string(),
        baseline_version: CONTRACT_VERSION,
        current_version: CONTRACT_VERSION,
        changes: Vec::new(),
    };
    let joined =
        join_toml_contract_text_from_version(raw, &contract, CONTRACT_STATE_PATH, CONTRACT_VERSION)
            .or_else(|_| reconcile_joined_toml_contract_text(raw, &contract, CONTRACT_STATE_PATH))
            .map_err(|error| boxed_debug("could not reconcile config contract", error))?;
    Ok(joined.text)
}

fn fill_missing_defaults(raw: &str) -> Result<String> {
    let mut text = raw.to_string();
    let defaults = default_config()?;
    for field_path in root_config_field_paths() {
        let default = default_config_path_value(&defaults, field_path)?;
        let value = parse_toml_value(&text).map_err(|error| boxed_debug("invalid TOML", error))?;
        if get_toml_path(&value, field_path).is_none() {
            text = set_toml_value_text(&text, field_path, &default)
                .map_err(|error| boxed_debug("could not write missing default", error))?
                .text;
        }
    }
    Ok(text)
}

fn default_config() -> Result<JsonValue> {
    parse_toml_value(DEFAULT_CONFIG_TOML)
        .map_err(|error| boxed_debug("invalid default config.toml", error))
}

fn default_config_path_value(defaults: &JsonValue, field_path: &str) -> Result<JsonValue> {
    get_toml_path(defaults, field_path)
        .cloned()
        .ok_or_else(|| error(format!("default config.toml is missing {field_path}")))
}

fn read_toml_file_value(path: &Path, label: &'static str) -> Result<JsonValue> {
    let raw = fs::read_to_string(path)?;
    parse_toml_value(&raw).map_err(|error| boxed_debug(label, error))
}

fn read_config_field(path: &Path, spec: &ConfigFieldSpec) -> Result<String> {
    let value = read_toml_file_value(path, "config.toml")?;
    validate_popup_keybindings(&value)?;
    let Some(value) = get_toml_path(&value, spec.field.path) else {
        return Err(error(format!("unknown config path: {}", spec.field.path)));
    };
    validate_config_value(spec.field.path, value)?;
    match spec.field.kind {
        "string" => Ok(spec.field.json_choice(value)?.to_string()),
        "boolean" => Ok(json_bool(spec.field.path, value)?.to_string()),
        "integer" => Ok(json_i64(spec.field.path, value)?.to_string()),
        _ => Err(error(format!(
            "{} must be {}",
            spec.field.path, spec.field.validation
        ))),
    }
}

fn read_bar_widgets_field(path: &Path) -> Result<String> {
    let value = read_toml_file_value(path, "config.toml")?;
    let Some(value) = get_toml_path(&value, BAR_WIDGETS_PATH) else {
        return Err(error(format!("unknown config path: {BAR_WIDGETS_PATH}")));
    };
    Ok(serde_json::to_string(&bar_widgets(value)?)?)
}

fn bar_widgets(value: &JsonValue) -> Result<Vec<String>> {
    string_list_values_from_json(BAR_WIDGETS_PATH, value, &string_values(BAR_WIDGET_VALUES))
        .map_err(error)
}

fn build_model(paths: &ConfigPaths) -> Result<ConfigUiModel> {
    let config_active = read_toml_file_value(&paths.root, "config.toml")?;
    let config_default = default_config()?;
    let mars_active = read_toml_file_value(&paths.mars, "invalid mars/config.toml")?;
    let mars_default = parse_toml_value(DEFAULT_MARS_CONFIG_TOML)
        .map_err(|error| boxed_debug("invalid default Mars config", error))?;
    let starship_active = read_toml_file_value(&paths.starship, "invalid starship.toml")?;
    let starship_default = parse_toml_value(DEFAULT_STARSHIP_CONFIG_TOML)
        .map_err(|error| boxed_debug("invalid default Starship config", error))?;
    let (zellij_active, diagnostics) = parse_zellij_sidecar(&fs::read_to_string(&paths.zellij)?);
    let zellij_default = ZellijSidecar::default();
    let zellij_blocking = diagnostics.iter().any(|diagnostic| diagnostic.blocking);

    let mut fields: Vec<_> = CONFIG_FIELDS
        .iter()
        .map(|spec| build_root_config_field(&config_active, &config_default, spec))
        .collect::<Result<_>>()?;
    fields.push(build_bar_widgets_field(&config_active, &config_default)?);
    fields.extend(KEY_BINDINGS.iter().map(build_key_binding_field));
    for spec in MARS_FIELDS {
        fields.push(build_config_field(
            SOURCE_MARS,
            TAB_MARS,
            spec,
            get_toml_path(&mars_active, spec.path),
            get_toml_path(&mars_default, spec.path),
            next_launch_apply_status("mars", "Saved values apply to newly launched Mars windows."),
            get_toml_path(&mars_active, spec.path)
                .is_some_and(|value| validate_mars_field(spec, value).is_err()),
        ));
    }
    for spec in STARSHIP_FIELDS {
        fields.push(build_config_field(
            SOURCE_STARSHIP,
            TAB_STARSHIP,
            spec,
            get_toml_path(&starship_active, spec.path),
            get_toml_path(&starship_default, spec.path),
            ConfigUiApplyStatus {
                summary: "new prompts".to_string(),
                label: "starship".to_string(),
                detail: "Saved values apply to newly rendered managed Nu prompts.".to_string(),
                pending: false,
            },
            get_toml_path(&starship_active, spec.path)
                .is_some_and(|value| validate_starship_field(spec, value).is_err()),
        ));
    }
    for spec in ZELLIJ_FIELDS {
        let current = zellij_field_value(&zellij_active, spec.path);
        let default = zellij_field_value(&zellij_default, spec.path);
        fields.push(build_config_field(
            SOURCE_ZELLIJ,
            TAB_ZELLIJ,
            spec,
            Some(&current),
            Some(&default),
            next_launch_apply_status(
                "zellij",
                "Saved values apply to newly launched Zellij sessions.",
            ),
            zellij_blocking,
        ));
    }

    Ok(ConfigUiModel {
        active_config_path: paths.root.clone(),
        cursor_config_path: PathBuf::new(),
        default_cursor_config_path: PathBuf::new(),
        active_config_exists: paths.root.exists(),
        config_owner: ConfigUiPathOwner::User,
        config_read_only: path_read_only(&paths.root),
        sources: vec![
            build_config_source(SOURCE_CONFIG, TAB_CONFIG, "config.toml", &paths.root),
            build_config_source(SOURCE_MARS, TAB_MARS, "mars/config.toml", &paths.mars),
            build_config_source(
                SOURCE_ZELLIJ,
                TAB_ZELLIJ,
                "zellij/config.kdl",
                &paths.zellij,
            ),
            build_config_source(
                SOURCE_STARSHIP,
                TAB_STARSHIP,
                "starship.toml",
                &paths.starship,
            ),
            build_config_source(SOURCE_HELIX, TAB_HELIX, "helix", &paths.helix_dir),
            ConfigUiSource {
                id: SOURCE_KEYS.to_string(),
                tab: TAB_KEYS.to_string(),
                label: "key bindings".to_string(),
                path: PathBuf::from("packaged-key-bindings"),
                exists: true,
                owner: ConfigUiPathOwner::Default,
                read_only: true,
            },
        ],
        tabs: vec![
            TAB_CONFIG.to_string(),
            TAB_MARS.to_string(),
            TAB_ZELLIJ.to_string(),
            TAB_STARSHIP.to_string(),
            TAB_HELIX.to_string(),
            TAB_KEYS.to_string(),
            TAB_ADVANCED.to_string(),
        ],
        tab_list_tables: BTreeMap::from([(
            TAB_KEYS.to_string(),
            ConfigUiListTable {
                columns: KEY_COLUMNS
                    .iter()
                    .map(|(title, width)| ConfigUiListColumn {
                        title: (*title).to_string(),
                        width: *width,
                    })
                    .collect(),
            },
        )]),
        fields,
        file_actions: build_file_actions(paths),
        sidecars: Vec::new(),
        native_config_statuses: Vec::new(),
        diagnostics,
    })
}

fn build_file_actions(paths: &ConfigPaths) -> Vec<ConfigUiFileAction> {
    file_action_specs(paths)
        .into_iter()
        .map(|spec| ConfigUiFileAction {
            source_id: spec.source_id.to_string(),
            action_id: spec.action_id.to_string(),
            tab: spec.tab.to_string(),
            label: spec.label.to_string(),
            description: spec.description.to_string(),
            exists: spec.path.exists(),
            read_only: path_read_only(&spec.path),
            create_if_missing: true,
            disabled_reason: None,
            path: spec.path,
        })
        .collect()
}

fn build_key_binding_field(
    [group, chord, action, owner, source]: &[&str; 5],
) -> ratconfig::ConfigUiField {
    ratconfig::ConfigUiField {
        source_id: SOURCE_KEYS.to_string(),
        path: chord.to_string(),
        tab: TAB_KEYS.to_string(),
        display_label: format!("{group}: {chord} - {action}"),
        list_cells: [*group, *chord, *action, *owner]
            .into_iter()
            .map(str::to_string)
            .collect(),
        kind: "string".to_string(),
        current_value: format!("{owner} / {source}"),
        edit_value: String::new(),
        default_value: ratconfig::NO_CONFIG_DEFAULT_VALUE_LABEL.to_string(),
        state: ratconfig::ConfigUiValueState::Explicit,
        description: format!("Group: {group}. Owner: {owner}. Source: {source}. Editable: no."),
        allowed_values: Vec::new(),
        validation: KEY_READ_ONLY_REASON.to_string(),
        rebuild_required: false,
        apply_status: ConfigUiApplyStatus {
            summary: "read-only".to_string(),
            label: "read-only".to_string(),
            detail: KEY_READ_ONLY_REASON.to_string(),
            pending: false,
        },
        edit_behavior: ConfigUiEditBehavior::StructuredOnly {
            notice: KEY_READ_ONLY_REASON.to_string(),
        },
    }
}

fn file_action_specs(paths: &ConfigPaths) -> [FileActionSpec; 8] {
    [
        FileActionSpec {
            source_id: SOURCE_HELIX,
            action_id: ACTION_HELIX_CONFIG,
            tab: TAB_HELIX,
            label: "helix/config.toml",
            description: "Open the managed Helix TOML config file.",
            path: paths.helix_config.clone(),
            starter: HELIX_CONFIG_STARTER,
        },
        FileActionSpec {
            source_id: SOURCE_HELIX,
            action_id: ACTION_HELIX_LANGUAGES,
            tab: TAB_HELIX,
            label: "helix/languages.toml",
            description: "Open the managed Helix language override file.",
            path: paths.helix_languages.clone(),
            starter: HELIX_LANGUAGES_STARTER,
        },
        FileActionSpec {
            source_id: SOURCE_HELIX,
            action_id: ACTION_HELIX_MODULE,
            tab: TAB_HELIX,
            label: "helix/helix.scm",
            description: "Open the managed Helix Steel module file.",
            path: paths.helix_module.clone(),
            starter: HELIX_MODULE_STARTER,
        },
        FileActionSpec {
            source_id: SOURCE_HELIX,
            action_id: ACTION_HELIX_INIT,
            tab: TAB_HELIX,
            label: "helix/init.scm",
            description: "Open the managed Helix Steel startup file.",
            path: paths.helix_init.clone(),
            starter: HELIX_INIT_STARTER,
        },
        FileActionSpec {
            source_id: SOURCE_ADVANCED,
            action_id: ACTION_NU_ENV,
            tab: TAB_ADVANCED,
            label: "nu/env.nu",
            description: "Open the user Nushell environment file.",
            path: paths.nu_env.clone(),
            starter: NU_ENV_STARTER,
        },
        FileActionSpec {
            source_id: SOURCE_ADVANCED,
            action_id: ACTION_NU_CONFIG,
            tab: TAB_ADVANCED,
            label: "nu/config.nu",
            description: "Open the user Nushell config file.",
            path: paths.nu_config.clone(),
            starter: NU_CONFIG_STARTER,
        },
        FileActionSpec {
            source_id: SOURCE_ADVANCED,
            action_id: ACTION_YAZI_INIT,
            tab: TAB_ADVANCED,
            label: "yazi/init.lua",
            description: "Open the managed Yazi user init.lua file.",
            path: paths.yazi_init.clone(),
            starter: YAZI_INIT_STARTER,
        },
        FileActionSpec {
            source_id: SOURCE_ADVANCED,
            action_id: ACTION_YAZI_KEYMAP,
            tab: TAB_ADVANCED,
            label: "yazi/keymap.toml",
            description: "Open the managed Yazi user keymap.toml file.",
            path: paths.yazi_keymap.clone(),
            starter: YAZI_KEYMAP_STARTER,
        },
    ]
}

fn build_config_source(id: &str, tab: &str, label: &str, path: &Path) -> ConfigUiSource {
    ConfigUiSource {
        id: id.to_string(),
        tab: tab.to_string(),
        label: label.to_string(),
        path: path.to_path_buf(),
        exists: path.exists(),
        owner: ConfigUiPathOwner::User,
        read_only: path_read_only(path),
    }
}

fn build_root_config_field(
    active: &JsonValue,
    defaults: &JsonValue,
    spec: &ConfigFieldSpec,
) -> Result<ratconfig::ConfigUiField> {
    let default = default_config_path_value(defaults, spec.field.path)?;
    let current = get_toml_path(active, spec.field.path);
    Ok(build_config_field(
        SOURCE_CONFIG,
        TAB_CONFIG,
        &spec.field,
        current,
        Some(&default),
        ConfigUiApplyStatus {
            summary: spec.apply_summary.to_string(),
            label: "runtime".to_string(),
            detail: spec.apply_detail.to_string(),
            pending: false,
        },
        current.is_some_and(|value| validate_config_value(spec.field.path, value).is_err())
            || (popup_keybinding_spec(spec.field.path).is_some()
                && validate_popup_keybindings(active).is_err()),
    ))
}

fn build_config_field(
    source_id: &'static str,
    tab: &'static str,
    spec: &FieldSpec,
    current: Option<&JsonValue>,
    default: Option<&JsonValue>,
    apply_status: ConfigUiApplyStatus,
    has_blocking_diagnostic: bool,
) -> ratconfig::ConfigUiField {
    build_config_ui_field(ConfigUiFieldRowSpec {
        source_id,
        path: spec.path,
        display_label: String::new(),
        list_cells: Vec::new(),
        tab,
        kind: spec.kind,
        current,
        default,
        description: spec.description.to_string(),
        allowed_values: string_values(spec.allowed_values),
        validation: spec.validation.to_string(),
        rebuild_required: false,
        apply_status,
        has_blocking_diagnostic,
        edit_behavior: ConfigUiEditBehavior::Default,
    })
}

fn build_bar_widgets_field(
    active: &JsonValue,
    defaults: &JsonValue,
) -> Result<ratconfig::ConfigUiField> {
    let current = get_toml_path(active, BAR_WIDGETS_PATH)
        .map(bar_widgets)
        .transpose();
    let has_blocking_diagnostic = current.is_err();
    let default = bar_widgets(&default_config_path_value(defaults, BAR_WIDGETS_PATH)?)?;
    build_string_list_choice_field(ConfigUiStringListChoiceSpec {
        source_id: SOURCE_CONFIG.to_string(),
        path: BAR_WIDGETS_PATH.to_string(),
        display_label: String::new(),
        list_cells: Vec::new(),
        tab: TAB_CONFIG.to_string(),
        current: current.ok().flatten(),
        default: Some(default),
        description: "Top bar widgets, left to right.".to_string(),
        allowed_values: string_values(BAR_WIDGET_VALUES),
        validation: "known widget ids".to_string(),
        rebuild_required: false,
        apply_status: ConfigUiApplyStatus {
            summary: "next launch".to_string(),
            label: "bar".to_string(),
            detail: "Saved widget order applies to newly launched Yazelix sessions.".to_string(),
            pending: false,
        },
        has_blocking_diagnostic,
        edit_behavior: ConfigUiEditBehavior::OrderedStringList,
    })
    .map_err(error)
}

fn next_launch_apply_status(label: &str, detail: &str) -> ConfigUiApplyStatus {
    ConfigUiApplyStatus {
        summary: "next launch".to_string(),
        label: label.to_string(),
        detail: detail.to_string(),
        pending: false,
    }
}

fn string_values(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| (*value).to_string()).collect()
}

fn write_source_field(
    paths: &ConfigPaths,
    source_id: &str,
    field_path: &str,
    value: &JsonValue,
) -> Result<()> {
    match source_id {
        SOURCE_CONFIG => {
            reject_read_only_source(&paths.root, source_id)?;
            write_config_field(&paths.root, field_path, value)
        }
        SOURCE_MARS => {
            reject_read_only_source(&paths.mars, source_id)?;
            write_mars_config_field(&paths.mars, field_path, value)
        }
        SOURCE_ZELLIJ => {
            reject_read_only_source(&paths.zellij, source_id)?;
            write_zellij_config_field(&paths.zellij, field_path, value)
        }
        SOURCE_STARSHIP => {
            reject_read_only_source(&paths.starship, source_id)?;
            write_starship_config_field(&paths.starship, field_path, value)
        }
        _ => Err(error(format!("unknown config source: {source_id}"))),
    }
}

fn write_source_default(paths: &ConfigPaths, source_id: &str, field_path: &str) -> Result<()> {
    let value = match source_id {
        SOURCE_CONFIG => default_config_value(field_path)?,
        SOURCE_MARS => {
            let default = parse_toml_value(DEFAULT_MARS_CONFIG_TOML)
                .map_err(|error| boxed_debug("invalid default Mars config", error))?;
            get_toml_path(&default, field_path)
                .cloned()
                .ok_or_else(|| error(format!("unknown Mars config path: {field_path}")))?
        }
        SOURCE_ZELLIJ => zellij_field_value(&ZellijSidecar::default(), field_path),
        SOURCE_STARSHIP => {
            let default = parse_toml_value(DEFAULT_STARSHIP_CONFIG_TOML)
                .map_err(|error| boxed_debug("invalid default Starship config", error))?;
            get_toml_path(&default, field_path)
                .cloned()
                .ok_or_else(|| error(format!("unknown Starship config path: {field_path}")))?
        }
        _ => return Err(error(format!("unknown config source: {source_id}"))),
    };
    write_source_field(paths, source_id, field_path, &value)
}

fn open_file_action(
    paths: &ConfigPaths,
    source_id: &str,
    action_id: &str,
    path: &Path,
    create_if_missing: bool,
) -> Result<()> {
    let editor = configured_editor()?;
    prepare_file_action(paths, source_id, action_id, path, create_if_missing)?;
    let status = Command::new(&editor).arg(path).status().map_err(|error| {
        io::Error::other(format!(
            "failed to launch editor `{}`: {error}",
            editor.display()
        ))
    })?;
    if !status.success() {
        return Err(error(format!(
            "editor `{}` exited with status {status}",
            editor.display()
        )));
    }
    Ok(())
}

fn prepare_file_action(
    paths: &ConfigPaths,
    source_id: &str,
    action_id: &str,
    path: &Path,
    create_if_missing: bool,
) -> Result<()> {
    let spec = file_action_spec(paths, source_id, action_id, path)?;
    let is_helix_steel_action = spec.source_id == SOURCE_HELIX
        && matches!(spec.action_id, ACTION_HELIX_MODULE | ACTION_HELIX_INIT);
    if spec.path.exists() {
        if is_helix_steel_action {
            ensure_helix_steel_pair(paths)?;
        }
        return Ok(());
    }
    if !create_if_missing {
        return Err(error(format!("config file is missing: {}", path.display())));
    }
    atomic_write(&spec.path, spec.starter)?;
    if is_helix_steel_action {
        ensure_helix_steel_pair(paths)?;
    }
    Ok(())
}

fn file_action_spec(
    paths: &ConfigPaths,
    source_id: &str,
    action_id: &str,
    path: &Path,
) -> Result<FileActionSpec> {
    let Some(spec) = file_action_specs(paths)
        .into_iter()
        .find(|spec| spec.source_id == source_id && spec.action_id == action_id)
    else {
        return Err(error(format!("unknown file action: {action_id}")));
    };
    if spec.path != path {
        return Err(error(format!(
            "file action `{action_id}` does not own {}",
            path.display()
        )));
    }
    Ok(spec)
}

fn ensure_helix_steel_pair(paths: &ConfigPaths) -> Result<()> {
    if !paths.helix_module.exists() {
        atomic_write(&paths.helix_module, HELIX_MODULE_STARTER)?;
    }
    if !paths.helix_init.exists() {
        atomic_write(&paths.helix_init, HELIX_INIT_STARTER)?;
    }
    Ok(())
}

fn configured_editor() -> Result<PathBuf> {
    ["YAZELIX_NEXT_EDITOR", "VISUAL", "EDITOR"]
        .into_iter()
        .find_map(|key| env::var_os(key).filter(|value| !value.is_empty()))
        .map(PathBuf::from)
        .ok_or_else(|| error("no editor configured; set YAZELIX_NEXT_EDITOR, VISUAL, or EDITOR"))
}

fn edit_text_externally(input: &str) -> Result<String> {
    edit_text_with_editor(input, &configured_editor()?)
}

fn edit_text_with_editor(input: &str, editor: &Path) -> Result<String> {
    let path = external_text_edit_path();
    fs::write(&path, input)?;
    let status = Command::new(editor).arg(&path).status().map_err(|error| {
        io::Error::other(format!(
            "failed to launch editor `{}`: {error}",
            editor.display()
        ))
    })?;
    if !status.success() {
        let _ = fs::remove_file(&path);
        return Err(error(format!(
            "editor `{}` exited with status {status}",
            editor.display()
        )));
    }

    let read_result = fs::read_to_string(&path);
    let _ = fs::remove_file(&path);
    let mut text = read_result?;
    if text.ends_with('\n') {
        text.pop();
        if text.ends_with('\r') {
            text.pop();
        }
    }
    Ok(text)
}

fn external_text_edit_path() -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);
    env::temp_dir().join(format!("yzn-config-edit-{}-{nonce}.txt", process::id()))
}

fn reject_read_only_source(path: &Path, source_id: &str) -> Result<()> {
    if path_read_only(path) {
        return Err(error(format!(
            "config source `{source_id}` is read-only: {}",
            path.display()
        )));
    }
    Ok(())
}

fn path_read_only(path: &Path) -> bool {
    fs::metadata(path)
        .map(|metadata| metadata.permissions().readonly())
        .unwrap_or(false)
}

fn write_config_field(path: &Path, field_path: &str, value: &JsonValue) -> Result<()> {
    validate_config_value(field_path, value)?;
    let raw = fs::read_to_string(path)?;
    let text = set_toml_value_text(&raw, field_path, value)
        .map_err(|error| boxed_debug("could not update config.toml", error))?
        .text;
    let text = fill_missing_defaults(&reconcile_contract(&text)?)?;
    validate_popup_keybindings(
        &parse_toml_value(&text).map_err(|error| boxed_debug("invalid config.toml", error))?,
    )?;
    atomic_write(path, &text)
}

fn default_config_value(field_path: &str) -> Result<JsonValue> {
    if field_path != BAR_WIDGETS_PATH {
        config_field(field_path)?;
    }
    default_config_path_value(&default_config()?, field_path)
}

fn validate_config_value(field_path: &str, value: &JsonValue) -> Result<()> {
    if field_path == BAR_WIDGETS_PATH {
        return bar_widgets(value).map(|_| ());
    }

    let spec = &config_field(field_path)?.field;
    match spec.kind {
        "boolean" => json_bool(field_path, value).map(|_| ()),
        "string" => {
            let value = spec.json_choice(value)?;
            if field_path == EDITOR_COMMAND_PATH {
                validate_editor_command(value)?;
            } else if popup_keybinding_spec(field_path).is_some() {
                validate_popup_keybinding(field_path, value)?;
            }
            Ok(())
        }
        "integer" => {
            let value = json_i64(field_path, value)?;
            if matches!(
                field_path,
                POPUP_SIDE_MARGIN_PATH | POPUP_VERTICAL_MARGIN_PATH
            ) && value < 0
            {
                return Err(error(format!("{field_path} must be zero or greater")));
            }
            if field_path == WELCOME_DURATION_SECONDS_PATH && !(1..=60).contains(&value) {
                return Err(error(format!("{field_path} must be between 1 and 60")));
            }
            Ok(())
        }
        _ => Err(error(format!("{field_path} must be {}", spec.validation))),
    }
}

fn validate_editor_command(value: &str) -> Result<()> {
    if value.is_empty() {
        return Err(error("editor.command must not be empty"));
    }
    if value.chars().any(char::is_whitespace) {
        return Err(error(
            "editor.command must be one executable command without arguments",
        ));
    }
    Ok(())
}

fn validate_popup_keybindings(value: &JsonValue) -> Result<()> {
    let mut used = BTreeMap::new();
    for spec in POPUP_KEYBINDINGS {
        let Some(value) = get_toml_path(value, spec.path) else {
            continue;
        };
        let chord = config_field(spec.path)?.field.json_choice(value)?;
        validate_popup_keybinding(spec.path, chord)?;
        if let Some(existing) = used.insert(chord.to_ascii_lowercase(), spec.path) {
            return Err(error(format!(
                "{} conflicts with {existing}: {chord}",
                spec.path
            )));
        }
    }
    Ok(())
}

fn popup_keybinding_spec(field_path: &str) -> Option<&'static PopupKeybindingSpec> {
    POPUP_KEYBINDINGS
        .iter()
        .find(|spec| spec.path == field_path)
}

fn validate_popup_keybinding(field_path: &str, value: &str) -> Result<()> {
    let spec = popup_keybinding_spec(field_path).ok_or_else(|| error("unknown keybinding role"))?;
    validate_key_chord(field_path, value)?;
    let conflicts = value != spec.default
        && KEY_BINDINGS
            .iter()
            .any(|[_group, chord, _action, _owner, _source]| {
                packaged_chord_matches(chord, value) && !popup_default_chord_matches(value)
            });
    if conflicts {
        return Err(error(format!(
            "{field_path} conflicts with packaged key {value}"
        )));
    }
    Ok(())
}

fn packaged_chord_matches(pattern: &str, value: &str) -> bool {
    pattern.split(" / ").any(|chord| {
        chord.eq_ignore_ascii_case(value)
            || matches!(
                (chord, value.strip_prefix("Alt ")),
                (
                    "Alt 1-9",
                    Some("1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9")
                )
            )
    })
}

fn popup_default_chord_matches(value: &str) -> bool {
    POPUP_KEYBINDINGS
        .iter()
        .any(|spec| spec.default.eq_ignore_ascii_case(value))
}

fn validate_key_chord(field_path: &str, value: &str) -> Result<()> {
    value
        .rsplit_once(' ')
        .filter(|(modifiers, key)| {
            matches!(
                *modifiers,
                "Ctrl"
                    | "Alt"
                    | "Shift"
                    | "Ctrl Alt"
                    | "Ctrl Shift"
                    | "Alt Shift"
                    | "Ctrl Alt Shift"
            ) && valid_key_token(key)
        })
        .map(|_| ())
        .ok_or_else(|| keybinding_syntax_error(field_path))
}

fn valid_key_token(key: &str) -> bool {
    matches!(key.as_bytes(), [ch] if ch.is_ascii_alphanumeric())
        || matches!(
            key,
            "Left"
                | "Right"
                | "Up"
                | "Down"
                | "Enter"
                | "Esc"
                | "Tab"
                | "Backspace"
                | "Space"
                | "Home"
                | "End"
                | "PageUp"
                | "PageDown"
        )
}

fn keybinding_syntax_error(field_path: &str) -> Box<dyn std::error::Error> {
    error(format!("{field_path} must be a key chord like Alt Shift A"))
}

fn write_mars_config_field(path: &Path, field_path: &str, value: &JsonValue) -> Result<()> {
    let spec = MARS_FIELDS
        .iter()
        .find(|spec| spec.path == field_path)
        .ok_or_else(|| error(format!("unknown Mars config path: {field_path}")))?;
    validate_mars_field(spec, value)?;
    let raw = fs::read_to_string(path)?;
    let text = set_toml_value_text(&raw, field_path, value)
        .map_err(|error| boxed_debug("could not update mars/config.toml", error))?
        .text;
    atomic_write(path, &text)
}

fn validate_mars_field(spec: &FieldSpec, value: &JsonValue) -> Result<()> {
    match spec.kind {
        "boolean" if value.is_boolean() => Ok(()),
        "integer" => {
            let value = json_i64(spec.path, value)?;
            if matches!(spec.path, "window.width" | "window.height") && value <= 0 {
                return Err(error(format!("{} must be positive", spec.path)));
            }
            Ok(())
        }
        "float" => {
            let value = value
                .as_f64()
                .ok_or_else(|| error(format!("{} must be {}", spec.path, spec.validation)))?;
            match spec.path {
                "window.opacity" if !(0.0..=1.0).contains(&value) => {
                    Err(error("window.opacity must be between 0.0 and 1.0"))
                }
                "fonts.size" | "line-height" if value <= 0.0 => {
                    Err(error(format!("{} must be positive", spec.path)))
                }
                _ => Ok(()),
            }
        }
        "string" => spec.json_choice(value).map(|_| ()),
        _ => Err(error(format!("{} must be {}", spec.path, spec.validation))),
    }
}

fn write_starship_config_field(path: &Path, field_path: &str, value: &JsonValue) -> Result<()> {
    let spec = STARSHIP_FIELDS
        .iter()
        .find(|spec| spec.path == field_path)
        .ok_or_else(|| error(format!("unknown Starship config path: {field_path}")))?;
    validate_starship_field(spec, value)?;
    let raw = fs::read_to_string(path)?;
    let text = set_toml_value_text(&raw, field_path, value)
        .map_err(|error| boxed_debug("could not update starship.toml", error))?
        .text;
    atomic_write(path, &text)
}

fn validate_starship_field(spec: &FieldSpec, value: &JsonValue) -> Result<()> {
    match spec.kind {
        "boolean" => json_bool(spec.path, value).map(|_| ()),
        "string" => spec.json_choice(value).map(|_| ()),
        _ => Err(error(format!("{} must be {}", spec.path, spec.validation))),
    }
}

fn write_zellij_config_field(path: &Path, field_path: &str, value: &JsonValue) -> Result<()> {
    if !ZELLIJ_FIELDS.iter().any(|spec| spec.path == field_path) {
        return Err(error(format!("unknown Zellij config path: {field_path}")));
    }
    let raw = fs::read_to_string(path)?;
    let (mut config, diagnostics) = parse_zellij_sidecar(&raw);
    if let Some(diagnostic) = diagnostics.iter().find(|diagnostic| diagnostic.blocking) {
        return Err(error(format!(
            "cannot update zellij/config.kdl: {}",
            diagnostic.headline
        )));
    }
    set_zellij_field_value(&mut config, field_path, value)?;
    atomic_write(path, &render_zellij_sidecar(&config))
}

fn parse_zellij_sidecar(raw: &str) -> (ZellijSidecar, Vec<ConfigUiDiagnostic>) {
    let mut config = ZellijSidecar::default();
    let mut diagnostics = Vec::new();
    let mut stack: Vec<&str> = Vec::new();

    for (index, raw_line) in raw.lines().enumerate() {
        let line_number = index + 1;
        let mut line = raw_line
            .split_once("//")
            .map_or(raw_line, |(content, _)| content)
            .trim();
        while let Some(rest) = line.strip_prefix('}') {
            if stack.pop().is_none() {
                diagnostics.push(zellij_diagnostic(
                    line_number,
                    "unmatched Zellij block close".to_string(),
                    "Remove the extra closing brace before editing from yzn config.",
                ));
            }
            line = rest.trim_start();
        }
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some(token) = line
            .split(|ch: char| ch.is_whitespace() || ch == '{' || ch == ';')
            .next()
            .filter(|token| !token.is_empty())
        else {
            continue;
        };
        match stack.as_slice() {
            [] => {
                parse_zellij_top_level_line(&mut config, &mut diagnostics, line, token, line_number)
            }
            ["ui"] => {
                if token == "pane_frames" && line.contains('{') {
                    stack.push("pane_frames");
                } else {
                    diagnostics.push(zellij_diagnostic(
                        line_number,
                        format!("unsupported Zellij ui node `{token}`"),
                        "The managed editor only supports ui.pane_frames.rounded_corners.",
                    ));
                }
            }
            ["ui", "pane_frames"] => {
                if token == "rounded_corners" {
                    parse_zellij_config_value(
                        &mut config,
                        zellij_field("ui.pane_frames.rounded_corners").expect("known field"),
                        line,
                        token,
                        line_number,
                        &mut diagnostics,
                    );
                } else {
                    diagnostics.push(zellij_diagnostic(
                        line_number,
                        format!("unsupported Zellij pane frame node `{token}`"),
                        "The managed editor only supports rounded_corners in this block.",
                    ));
                }
            }
            _ => unreachable!("Zellij parser stack only contains ui and pane_frames"),
        }
        if stack.is_empty() && token == "ui" && line.contains('{') {
            stack.push("ui");
        }
    }

    if !stack.is_empty() {
        diagnostics.push(zellij_diagnostic(
            raw.lines().count().max(1),
            "unterminated Zellij block".to_string(),
            "The managed editor only supports complete multiline ui.pane_frames blocks.",
        ));
    }

    (config, diagnostics)
}

fn parse_zellij_top_level_line(
    config: &mut ZellijSidecar,
    diagnostics: &mut Vec<ConfigUiDiagnostic>,
    line: &str,
    token: &str,
    line_number: usize,
) {
    if token == "ui" {
        if !line.contains('{') {
            diagnostics.push(zellij_diagnostic(
                line_number,
                "unsupported Zellij ui form".to_string(),
                "The managed editor expects ui as a block.",
            ));
        }
        return;
    }
    if ZELLIJ_FORBIDDEN_TOP_LEVEL.contains(&token) {
        diagnostics.push(zellij_diagnostic(
            line_number,
            format!("guarded Zellij node `{token}`"),
            "This node belongs to the managed runtime and cannot live in the editable sidecar.",
        ));
        return;
    }
    let Some(spec) = top_level_zellij_field(token) else {
        diagnostics.push(zellij_diagnostic(
            line_number,
            format!("unsupported Zellij node `{token}`"),
            "Remove this node before editing from yzn config, or keep editing the sidecar by hand.",
        ));
        return;
    };

    parse_zellij_config_value(config, spec, line, token, line_number, diagnostics);
}

fn top_level_zellij_field(token: &str) -> Option<&'static FieldSpec> {
    zellij_field(token).filter(|spec| !spec.path.contains('.'))
}

fn zellij_field(path: &str) -> Option<&'static FieldSpec> {
    ZELLIJ_FIELDS.iter().find(|spec| spec.path == path)
}

fn parse_zellij_config_value(
    config: &mut ZellijSidecar,
    spec: &FieldSpec,
    line: &str,
    token: &str,
    line_number: usize,
    diagnostics: &mut Vec<ConfigUiDiagnostic>,
) {
    let Some(value) = parse_kdl_json_value(line, token, spec, line_number, diagnostics) else {
        return;
    };
    if let Err(error) = set_zellij_field_value(config, spec.path, &value) {
        diagnostics.push(zellij_diagnostic(
            line_number,
            format!("invalid Zellij value for `{}`", spec.path),
            error.to_string(),
        ));
    }
}

fn parse_kdl_json_value(
    line: &str,
    token: &str,
    spec: &FieldSpec,
    line_number: usize,
    diagnostics: &mut Vec<ConfigUiDiagnostic>,
) -> Option<JsonValue> {
    let Some(value) = first_kdl_value(line, token) else {
        diagnostics.push(zellij_diagnostic(
            line_number,
            format!("missing Zellij value for `{}`", spec.path),
            format!("Expected {}.", zellij_expected_value(spec.kind)),
        ));
        return None;
    };
    match spec.kind {
        "boolean" => match value {
            "true" => Some(json!(true)),
            "false" => Some(json!(false)),
            _ => {
                diagnostics.push(zellij_diagnostic(
                    line_number,
                    format!("invalid Zellij value for `{}`", spec.path),
                    "Expected true or false.",
                ));
                None
            }
        },
        "integer" => value.parse::<i64>().map(JsonValue::from).map_or_else(
            |_| {
                diagnostics.push(zellij_diagnostic(
                    line_number,
                    format!("invalid Zellij value for `{}`", spec.path),
                    "Expected an integer.",
                ));
                None
            },
            Some,
        ),
        "string" => Some(json!(value)),
        _ => {
            diagnostics.push(zellij_diagnostic(
                line_number,
                format!("unsupported Zellij value kind `{}`", spec.kind),
                "The managed editor only supports boolean, integer, and string KDL values.",
            ));
            None
        }
    }
}

fn zellij_expected_value(kind: &str) -> &'static str {
    match kind {
        "boolean" => "true or false",
        "integer" => "an integer",
        "string" => "a string",
        _ => "a supported scalar",
    }
}

fn first_kdl_value<'a>(line: &'a str, token: &str) -> Option<&'a str> {
    line.strip_prefix(token)?
        .split_whitespace()
        .next()
        .map(|value| {
            value
                .trim_end_matches(';')
                .trim_end_matches('{')
                .trim_matches('"')
        })
}

fn zellij_diagnostic(
    line_number: usize,
    headline: String,
    detail: impl Into<String>,
) -> ConfigUiDiagnostic {
    ConfigUiDiagnostic {
        path: format!("zellij/config.kdl:{line_number}"),
        status: "blocked".to_string(),
        headline,
        blocking: true,
        detail_lines: vec![detail.into()],
    }
}

fn zellij_field_value(config: &ZellijSidecar, path: &str) -> JsonValue {
    match path {
        "pane_frames" => json!(config.pane_frames),
        "mouse_mode" => json!(config.mouse_mode),
        "scroll_buffer_size" => json!(config.scroll_buffer_size),
        "copy_on_select" => json!(config.copy_on_select),
        "copy_clipboard" => json!(config.copy_clipboard),
        "styled_underlines" => json!(config.styled_underlines),
        "show_startup_tips" => json!(config.show_startup_tips),
        "ui.pane_frames.rounded_corners" => json!(config.rounded_corners),
        _ => JsonValue::Null,
    }
}

fn set_zellij_field_value(config: &mut ZellijSidecar, path: &str, value: &JsonValue) -> Result<()> {
    match path {
        "pane_frames" => config.pane_frames = json_bool(path, value)?,
        "mouse_mode" => config.mouse_mode = json_bool(path, value)?,
        "scroll_buffer_size" => config.scroll_buffer_size = json_positive_i64(path, value)?,
        "copy_on_select" => config.copy_on_select = json_bool(path, value)?,
        "copy_clipboard" => {
            config.copy_clipboard = zellij_field(path)
                .expect("known field")
                .json_choice(value)?
                .to_string();
        }
        "styled_underlines" => config.styled_underlines = json_bool(path, value)?,
        "show_startup_tips" => config.show_startup_tips = json_bool(path, value)?,
        "ui.pane_frames.rounded_corners" => config.rounded_corners = json_bool(path, value)?,
        _ => return Err(error(format!("unknown Zellij config path: {path}"))),
    }
    Ok(())
}

fn render_zellij_sidecar(config: &ZellijSidecar) -> String {
    format!(
        "\
pane_frames {}
mouse_mode {}
scroll_buffer_size {}
copy_on_select {}
copy_clipboard \"{}\"
styled_underlines {}
show_startup_tips {}

ui {{
    pane_frames {{
        rounded_corners {}
    }}
}}
",
        kdl_bool(config.pane_frames),
        kdl_bool(config.mouse_mode),
        config.scroll_buffer_size,
        kdl_bool(config.copy_on_select),
        config.copy_clipboard,
        kdl_bool(config.styled_underlines),
        kdl_bool(config.show_startup_tips),
        kdl_bool(config.rounded_corners),
    )
}

fn kdl_bool(value: bool) -> &'static str {
    if value { "true" } else { "false" }
}

fn json_bool(path: &str, value: &JsonValue) -> Result<bool> {
    value
        .as_bool()
        .ok_or_else(|| error(format!("{path} must be true or false")))
}

fn json_i64(path: &str, value: &JsonValue) -> Result<i64> {
    value
        .as_i64()
        .or_else(|| value.as_u64().and_then(|value| i64::try_from(value).ok()))
        .ok_or_else(|| error(format!("{path} must be an integer")))
}

fn json_positive_i64(path: &str, value: &JsonValue) -> Result<i64> {
    let value = json_i64(path, value)?;
    if value <= 0 {
        return Err(error(format!("{path} must be a positive integer")));
    }
    Ok(value)
}

fn atomic_write(path: &Path, text: &str) -> Result<()> {
    let parent = path
        .parent()
        .ok_or_else(|| error(format!("{} has no parent directory", path.display())))?;
    fs::create_dir_all(parent)?;
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);
    let tmp = parent.join(format!(
        ".{}.tmp-{}-{nonce}",
        path.file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("config.toml"),
        process::id()
    ));
    fs::write(&tmp, text)?;
    fs::rename(&tmp, path)?;
    Ok(())
}

fn error(message: impl Into<String>) -> Box<dyn std::error::Error> {
    Box::new(io::Error::other(message.into()))
}

fn boxed_debug(message: &'static str, error: impl std::fmt::Debug) -> Box<dyn std::error::Error> {
    Box::new(io::Error::other(format!("{message}: {error:?}")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratconfig::ConfigUiValueState;

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

        write_config_field(&path, OPEN_LOG_LEVEL_PATH, &json!("debug")).unwrap();
        assert_toml_value(&path, OPEN_LOG_LEVEL_PATH, &json!("debug"));

        write_config_field(&path, SHELL_PROGRAM_PATH, &json!("fish")).unwrap();
        assert_toml_value(&path, SHELL_PROGRAM_PATH, &json!("fish"));

        write_config_field(&path, EDITOR_COMMAND_PATH, &json!("nvim")).unwrap();
        assert_toml_value(&path, EDITOR_COMMAND_PATH, &json!("nvim"));
        assert_eq!(
            read_config_field(&path, config_field(EDITOR_COMMAND_PATH).unwrap()).unwrap(),
            "nvim"
        );

        write_config_field(&path, POPUP_SIDE_MARGIN_PATH, &json!(2)).unwrap();
        assert_toml_value(&path, POPUP_SIDE_MARGIN_PATH, &json!(2));
        assert_eq!(
            read_config_field(&path, config_field(POPUP_SIDE_MARGIN_PATH).unwrap()).unwrap(),
            "2"
        );

        write_config_field(&path, POPUP_VERTICAL_MARGIN_PATH, &json!(1)).unwrap();
        assert_toml_value(&path, POPUP_VERTICAL_MARGIN_PATH, &json!(1));

        for (field_path, value) in [
            (KEYBINDINGS_CONFIG_PATH, "Alt Shift C"),
            (KEYBINDINGS_AGENT_PATH, "Alt Shift A"),
            (KEYBINDINGS_LAZYGIT_PATH, "Alt Shift G"),
            (KEYBINDINGS_MENU_PATH, "Alt Shift U"),
        ] {
            write_config_field(&path, field_path, &json!(value)).unwrap();
            assert_toml_value(&path, field_path, &json!(value));
            assert_eq!(
                read_config_field(&path, config_field(field_path).unwrap()).unwrap(),
                value
            );
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
    fn config_model_exposes_root_config_fields() {
        let (_temp, paths) = temp_sources();

        let model = build_model(&paths).unwrap();
        assert!(!model.tabs.contains(&"shell".to_string()));
        assert_eq!(model_field(&model, SHELL_PROGRAM_PATH).tab, TAB_CONFIG);
        let editor = model_field(&model, EDITOR_COMMAND_PATH);
        assert_eq!(editor.tab, TAB_CONFIG);
        assert_eq!(editor.kind, "string");
        assert_eq!(
            editor.current_value,
            default_config_value(EDITOR_COMMAND_PATH)
                .unwrap()
                .to_string()
        );
        assert!(editor.allowed_values.is_empty());
        assert_eq!(editor.apply_status.summary, "new opens");

        let popup = model_field(&model, POPUP_SIDE_MARGIN_PATH);
        assert_eq!(popup.tab, TAB_CONFIG);
        assert_eq!(popup.kind, "integer");
        assert_eq!(
            popup.current_value,
            default_config_value(POPUP_SIDE_MARGIN_PATH)
                .unwrap()
                .to_string()
        );
        assert_eq!(popup.apply_status.summary, "next launch");
        assert_eq!(
            model_field(&model, POPUP_VERTICAL_MARGIN_PATH).current_value,
            default_config_value(POPUP_VERTICAL_MARGIN_PATH)
                .unwrap()
                .to_string()
        );

        for spec in POPUP_KEYBINDINGS {
            let field = model_field(&model, spec.path);
            assert_eq!(field.tab, TAB_CONFIG);
            assert_eq!(field.kind, "string");
            assert_eq!(
                field.current_value,
                default_config_value(spec.path).unwrap().to_string()
            );
            assert_eq!(field.apply_status.summary, "next launch");
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
        let text = r#"
[bar]
widgets = ["editor", "shell", "term", "codex_usage", "cpu", "ram"]

[editor]
command = "yzn-hx"

[open]
log_level = "info"

[popup]
side_margin = 1
vertical_margin = 0

[keybindings]
config = "Alt Shift K"
agent = "Alt Shift L"
lazygit = "Alt Shift J"
menu = "Alt Shift M"

[ratconfig.contract]
applied_change_ids = []
contract_id = "yazelix-next.config"
schema_version = 1
version = 1

[shell]
program = "fish"

[welcome]
duration_seconds = 3
enabled = false
style = "random"
"#;

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

        assert!(paths.root.exists());
        assert!(paths.mars.exists());
        assert!(paths.zellij.exists());
        assert!(paths.starship.exists());
        assert!(!paths.helix_config.exists());
        assert!(!paths.helix_languages.exists());
        assert!(!paths.helix_module.exists());
        assert!(!paths.helix_init.exists());
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
        assert!(!paths.nu_env.exists());
        assert!(!paths.nu_config.exists());
        assert!(!paths.yazi_init.exists());
        assert!(!paths.yazi_keymap.exists());
    }

    #[test]
    fn native_file_tabs_list_owned_file_actions() {
        let (_temp, paths) = temp_sources();

        let model = build_model(&paths).unwrap();
        let rows: Vec<_> = model
            .file_actions
            .iter()
            .map(|action| {
                (
                    action.source_id.as_str(),
                    action.action_id.as_str(),
                    action.tab.as_str(),
                    action.label.as_str(),
                    action.path.clone(),
                    action.exists,
                    action.create_if_missing,
                )
            })
            .collect();

        assert!(model.tabs.contains(&TAB_STARSHIP.to_string()));
        assert!(model.tabs.contains(&TAB_HELIX.to_string()));
        assert!(model.sources.iter().any(|source| {
            source.id == SOURCE_HELIX && source.tab == TAB_HELIX && source.path == paths.helix_dir
        }));
        assert_eq!(
            rows,
            vec![
                (
                    SOURCE_HELIX,
                    ACTION_HELIX_CONFIG,
                    TAB_HELIX,
                    "helix/config.toml",
                    paths.helix_config.clone(),
                    false,
                    true,
                ),
                (
                    SOURCE_HELIX,
                    ACTION_HELIX_LANGUAGES,
                    TAB_HELIX,
                    "helix/languages.toml",
                    paths.helix_languages.clone(),
                    false,
                    true,
                ),
                (
                    SOURCE_HELIX,
                    ACTION_HELIX_MODULE,
                    TAB_HELIX,
                    "helix/helix.scm",
                    paths.helix_module.clone(),
                    false,
                    true,
                ),
                (
                    SOURCE_HELIX,
                    ACTION_HELIX_INIT,
                    TAB_HELIX,
                    "helix/init.scm",
                    paths.helix_init.clone(),
                    false,
                    true,
                ),
                (
                    SOURCE_ADVANCED,
                    ACTION_NU_ENV,
                    TAB_ADVANCED,
                    "nu/env.nu",
                    paths.nu_env.clone(),
                    false,
                    true,
                ),
                (
                    SOURCE_ADVANCED,
                    ACTION_NU_CONFIG,
                    TAB_ADVANCED,
                    "nu/config.nu",
                    paths.nu_config.clone(),
                    false,
                    true,
                ),
                (
                    SOURCE_ADVANCED,
                    ACTION_YAZI_INIT,
                    TAB_ADVANCED,
                    "yazi/init.lua",
                    paths.yazi_init.clone(),
                    false,
                    true,
                ),
                (
                    SOURCE_ADVANCED,
                    ACTION_YAZI_KEYMAP,
                    TAB_ADVANCED,
                    "yazi/keymap.toml",
                    paths.yazi_keymap.clone(),
                    false,
                    true,
                ),
            ]
        );
    }

    #[test]
    fn prepare_file_action_creates_owned_missing_file() {
        let (_temp, paths) = temp_sources();

        prepare_file_action(&paths, SOURCE_ADVANCED, ACTION_NU_ENV, &paths.nu_env, true).unwrap();

        assert_eq!(fs::read_to_string(&paths.nu_env).unwrap(), NU_ENV_STARTER);
        assert!(!paths.nu_config.exists());
        assert!(paths.starship.exists());
        assert!(!paths.helix_config.exists());
        assert!(!paths.helix_languages.exists());
        assert!(!paths.helix_module.exists());
        assert!(!paths.helix_init.exists());
        assert!(!paths.yazi_init.exists());
        assert!(!paths.yazi_keymap.exists());
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

        assert_eq!(
            fs::read_to_string(&paths.helix_config).unwrap(),
            HELIX_CONFIG_STARTER
        );
        assert!(!paths.helix_languages.exists());
        assert!(!paths.helix_module.exists());
        assert!(!paths.helix_init.exists());
        assert!(!paths.nu_env.exists());
        assert!(!paths.yazi_init.exists());
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

        assert_eq!(
            fs::read_to_string(&paths.helix_init).unwrap(),
            HELIX_INIT_STARTER
        );
        assert_eq!(
            fs::read_to_string(&paths.helix_module).unwrap(),
            HELIX_MODULE_STARTER
        );
        assert!(!paths.helix_config.exists());
        assert!(!paths.helix_languages.exists());
        assert!(!paths.nu_env.exists());
        assert!(!paths.yazi_init.exists());
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
        let (temp, paths) = temp_sources();

        prepare_file_action(
            &paths,
            SOURCE_ADVANCED,
            ACTION_YAZI_INIT,
            &paths.yazi_init,
            true,
        )
        .unwrap();

        assert_eq!(
            fs::read_to_string(&paths.yazi_init).unwrap(),
            YAZI_INIT_STARTER
        );
        assert!(!temp.path.join("yazi/yazi.toml").exists());
        assert!(!temp.path.join("yazi/keymap.toml").exists());
        assert!(!temp.path.join("yazi/plugins").exists());
    }

    #[test]
    fn prepare_file_action_creates_managed_yazi_keymap_only() {
        let (temp, paths) = temp_sources();

        prepare_file_action(
            &paths,
            SOURCE_ADVANCED,
            ACTION_YAZI_KEYMAP,
            &paths.yazi_keymap,
            true,
        )
        .unwrap();

        assert_eq!(
            fs::read_to_string(&paths.yazi_keymap).unwrap(),
            YAZI_KEYMAP_STARTER
        );
        assert!(!temp.path.join("yazi/init.lua").exists());
        assert!(!temp.path.join("yazi/yazi.toml").exists());
        assert!(!temp.path.join("yazi/plugins").exists());
    }

    #[test]
    fn prepare_file_action_rejects_unowned_or_missing_paths() {
        let (_temp, paths) = temp_sources();

        let error = prepare_file_action(
            &paths,
            SOURCE_ADVANCED,
            ACTION_NU_ENV,
            &paths.nu_config,
            true,
        )
        .unwrap_err()
        .to_string();
        assert!(error.contains("does not own"));

        let error = prepare_file_action(
            &paths,
            SOURCE_ADVANCED,
            ACTION_HELIX_CONFIG,
            &paths.helix_config,
            true,
        )
        .unwrap_err()
        .to_string();
        assert!(error.contains("unknown file action"));

        let error = prepare_file_action(
            &paths,
            SOURCE_ADVANCED,
            ACTION_NU_CONFIG,
            &paths.nu_config,
            false,
        )
        .unwrap_err()
        .to_string();
        assert!(error.contains("config file is missing"));
    }

    #[test]
    fn source_routing_writes_mars_without_touching_config_toml() {
        let (_temp, paths) = temp_sources();
        let before_root = fs::read_to_string(&paths.root).unwrap();

        write_source_field(&paths, SOURCE_MARS, "window.width", &json!(1200)).unwrap();

        assert_eq!(fs::read_to_string(&paths.root).unwrap(), before_root);
        let mars = read_toml_file_value(&paths.mars, "mars").unwrap();
        assert_eq!(get_toml_path(&mars, "window.width"), Some(&json!(1200)));
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
        assert_eq!(
            config_key(KeyEvent::new_with_kind(
                KeyCode::Char('q'),
                KeyModifiers::NONE,
                KeyEventKind::Release,
            )),
            None
        );
        assert_eq!(
            config_key(KeyEvent::new(KeyCode::Char('u'), KeyModifiers::ALT)),
            None
        );
        assert_eq!(
            config_key(KeyEvent::new(KeyCode::F(1), KeyModifiers::NONE)),
            None
        );
    }
}
