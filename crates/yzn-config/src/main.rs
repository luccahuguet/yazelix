// Test lane: default

use std::{
    env, fs, io,
    path::{Path, PathBuf},
    process,
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
    ConfigUiFieldRowSpec, ConfigUiIntent, ConfigUiKey, ConfigUiModel, ConfigUiPathOwner,
    ConfigUiSource, DEFAULT_CONFIG_SOURCE_ID, build_config_ui_field, draw_config_ui,
    join_toml_contract_text_from_version, reconcile_joined_toml_contract_text,
};
use serde_json::{Value as JsonValue, json};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

const DEFAULT_CONFIG_TOML: &str = include_str!("../../../config.toml");
const CONTRACT_ID: &str = "yazelix-next.config";
const CONTRACT_STATE_PATH: &str = "ratconfig.contract";
const CONTRACT_VERSION: u64 = 1;

const OPEN_LOG_LEVEL_PATH: &str = "open.log_level";
const OPEN_LOG_LEVEL_DEFAULT: &str = "info";
const OPEN_LOG_LEVELS: &[&str] = &["off", "error", "info", "debug"];
const DEFAULT_MARS_CONFIG_TOML: &str = include_str!("../../../mars.toml");

const SOURCE_CONFIG: &str = DEFAULT_CONFIG_SOURCE_ID;
const SOURCE_MARS: &str = "mars";
const SOURCE_ZELLIJ: &str = "zellij";
const TAB_CONFIG: &str = "config";
const TAB_MARS: &str = "mars";
const TAB_ZELLIJ: &str = "zellij";
const TAB_ADVANCED: &str = "advanced";

const MARS_FIELDS: &[FieldSpec] = &[
    FieldSpec::integer("window.width", "Initial Mars window width.", "pixels"),
    FieldSpec::integer("window.height", "Initial Mars window height.", "pixels"),
    FieldSpec::float("window.opacity", "Mars window opacity.", "0.0 to 1.0"),
    FieldSpec::float("fonts.size", "Mars font size.", "points"),
    FieldSpec::float("line-height", "Mars line height multiplier.", "multiplier"),
    FieldSpec::boolean("enable-scroll-bar", "Show the Mars scrollbar."),
    FieldSpec::boolean("bell.audio", "Play the Mars terminal bell."),
    FieldSpec::boolean("bell.visual", "Flash the Mars visual bell."),
    FieldSpec::boolean("effects.trail-cursor", "Draw the Mars cursor trail."),
];

const ZELLIJ_COPY_CLIPBOARD_VALUES: &[&str] = &["system", "primary"];
const ZELLIJ_FORBIDDEN_TOP_LEVEL: &[&str] = &[
    "keybinds",
    "plugins",
    "load_plugins",
    "default_shell",
    "default_layout",
    "layout",
    "support_kitty_keyboard_protocol",
    "env",
    "session_name",
    "attach_to_session",
];
const ZELLIJ_FIELDS: &[FieldSpec] = &[
    FieldSpec::boolean("pane_frames", "Show Zellij pane frames."),
    FieldSpec::boolean("mouse_mode", "Enable mouse support in Zellij."),
    FieldSpec::integer(
        "scroll_buffer_size",
        "Lines kept in Zellij scrollback.",
        "positive integer",
    ),
    FieldSpec::boolean("copy_on_select", "Copy selected text automatically."),
    FieldSpec::string_choice(
        "copy_clipboard",
        "Clipboard target for Zellij copy operations.",
        ZELLIJ_COPY_CLIPBOARD_VALUES,
        "system or primary",
    ),
    FieldSpec::boolean(
        "styled_underlines",
        "Render styled underlines in Zellij panes.",
    ),
    FieldSpec::boolean("show_startup_tips", "Show Zellij startup tips."),
    FieldSpec::boolean(
        "ui.pane_frames.rounded_corners",
        "Use rounded Zellij pane frame corners.",
    ),
];

#[derive(Debug, Clone)]
struct ConfigPaths {
    root: PathBuf,
    mars: PathBuf,
    zellij: PathBuf,
}

#[derive(Debug, Clone, Copy)]
struct FieldSpec {
    path: &'static str,
    kind: &'static str,
    description: &'static str,
    allowed_values: &'static [&'static str],
    validation: &'static str,
}

impl FieldSpec {
    const fn boolean(path: &'static str, description: &'static str) -> Self {
        Self::new(path, "boolean", description, &[], "true or false")
    }

    const fn integer(
        path: &'static str,
        description: &'static str,
        validation: &'static str,
    ) -> Self {
        Self::new(path, "integer", description, &[], validation)
    }

    const fn float(
        path: &'static str,
        description: &'static str,
        validation: &'static str,
    ) -> Self {
        Self::new(path, "float", description, &[], validation)
    }

    const fn string_choice(
        path: &'static str,
        description: &'static str,
        allowed_values: &'static [&'static str],
        validation: &'static str,
    ) -> Self {
        Self::new(path, "string", description, allowed_values, validation)
    }

    const fn new(
        path: &'static str,
        kind: &'static str,
        description: &'static str,
        allowed_values: &'static [&'static str],
        validation: &'static str,
    ) -> Self {
        Self {
            path,
            kind,
            description,
            allowed_values,
            validation,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
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
    let _terminal = TerminalSession::enter()?;
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
    if path != OPEN_LOG_LEVEL_PATH {
        return Err(error(format!("unknown config path: {path}")));
    }
    println!("{}", read_open_log_level(&ensure_config_file()?)?);
    Ok(())
}

fn ensure_config_file() -> Result<PathBuf> {
    ensure_config_file_at(config_paths()?.root)
}

fn ensure_config_sources() -> Result<ConfigPaths> {
    let paths = config_paths()?;
    ensure_config_file_at(paths.root.clone())?;
    ensure_plain_config_file_at(&paths.mars, DEFAULT_MARS_CONFIG_TOML)?;
    ensure_plain_config_file_at(&paths.zellij, &default_zellij_config_kdl())?;
    Ok(paths)
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
            reject_read_only_source(&path, SOURCE_CONFIG)?;
        }
        atomic_write(&path, &completed)?;
    }
    Ok(path)
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
    let value = parse_toml_value(raw).map_err(|error| boxed_debug("invalid TOML", error))?;
    if get_toml_path(&value, OPEN_LOG_LEVEL_PATH).is_some() {
        return Ok(raw.to_string());
    }
    set_toml_value_text(raw, OPEN_LOG_LEVEL_PATH, &open_log_level_default())
        .map(|patch| patch.text)
        .map_err(|error| boxed_debug("could not write missing default", error))
}

fn read_config_value(path: &Path) -> Result<JsonValue> {
    read_toml_file_value(path, "config.toml")
}

fn read_toml_file_value(path: &Path, label: &'static str) -> Result<JsonValue> {
    let raw = fs::read_to_string(path)?;
    parse_toml_value(&raw).map_err(|error| boxed_debug(label, error))
}

fn read_open_log_level(path: &Path) -> Result<String> {
    let value = read_config_value(path)?;
    let Some(value) = get_toml_path(&value, OPEN_LOG_LEVEL_PATH) else {
        return Err(error(format!("unknown config path: {OPEN_LOG_LEVEL_PATH}")));
    };
    Ok(open_log_level_from_json(value)?.to_string())
}

fn build_model(paths: &ConfigPaths) -> Result<ConfigUiModel> {
    let config_active = read_config_value(&paths.root)?;
    let mars_active = read_toml_file_value(&paths.mars, "invalid mars/config.toml")?;
    let mars_default = parse_toml_value(DEFAULT_MARS_CONFIG_TOML)
        .map_err(|error| boxed_debug("invalid default Mars config", error))?;
    let (zellij_active, diagnostics) = read_zellij_sidecar(&paths.zellij)?;
    let zellij_default = ZellijSidecar::default();
    let zellij_blocking = diagnostics.iter().any(|diagnostic| diagnostic.blocking);

    let mut fields = vec![build_open_log_level_field(&config_active)];
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
        ],
        tabs: vec![
            TAB_CONFIG.to_string(),
            TAB_MARS.to_string(),
            TAB_ZELLIJ.to_string(),
            TAB_ADVANCED.to_string(),
        ],
        fields,
        sidecars: Vec::new(),
        native_config_statuses: Vec::new(),
        diagnostics,
    })
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

fn build_open_log_level_field(active: &JsonValue) -> ratconfig::ConfigUiField {
    let default = open_log_level_default();
    let current = get_toml_path(active, OPEN_LOG_LEVEL_PATH);
    build_config_ui_field(ConfigUiFieldRowSpec {
        source_id: SOURCE_CONFIG,
        path: OPEN_LOG_LEVEL_PATH,
        display_label: String::new(),
        tab: TAB_CONFIG,
        kind: "string",
        current,
        default: Some(&default),
        description: "Diagnostics written by yzn-open for Yazi-to-Helix open requests.".to_string(),
        allowed_values: string_values(OPEN_LOG_LEVELS),
        validation: "off, error, info, or debug".to_string(),
        rebuild_required: false,
        apply_status: ConfigUiApplyStatus {
            summary: "new opens".to_string(),
            label: "runtime".to_string(),
            detail: "Saved values are exported as YZN_OPEN_LOG for managed Yazi opens.".to_string(),
            pending: false,
        },
        has_blocking_diagnostic: current
            .is_some_and(|value| open_log_level_from_json(value).is_err()),
        edit_behavior: ConfigUiEditBehavior::Default,
    })
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

fn next_launch_apply_status(label: &str, detail: &str) -> ConfigUiApplyStatus {
    ConfigUiApplyStatus {
        summary: "next launch".to_string(),
        label: label.to_string(),
        detail: detail.to_string(),
        pending: false,
    }
}

fn open_log_level_default() -> JsonValue {
    json!(OPEN_LOG_LEVEL_DEFAULT)
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
        _ => Err(error(format!("unknown config source: {source_id}"))),
    }
}

fn write_source_default(paths: &ConfigPaths, source_id: &str, field_path: &str) -> Result<()> {
    let value = match source_id {
        SOURCE_CONFIG => {
            if field_path != OPEN_LOG_LEVEL_PATH {
                return Err(error(format!("unknown config path: {field_path}")));
            }
            open_log_level_default()
        }
        SOURCE_MARS => {
            let default = parse_toml_value(DEFAULT_MARS_CONFIG_TOML)
                .map_err(|error| boxed_debug("invalid default Mars config", error))?;
            get_toml_path(&default, field_path)
                .cloned()
                .ok_or_else(|| error(format!("unknown Mars config path: {field_path}")))?
        }
        SOURCE_ZELLIJ => zellij_field_value(&ZellijSidecar::default(), field_path),
        _ => return Err(error(format!("unknown config source: {source_id}"))),
    };
    write_source_field(paths, source_id, field_path, &value)
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
    if field_path != OPEN_LOG_LEVEL_PATH {
        return Err(error(format!("unknown config path: {field_path}")));
    }
    open_log_level_from_json(value)?;
    let raw = fs::read_to_string(path)?;
    let text = set_toml_value_text(&raw, field_path, value)
        .map_err(|error| boxed_debug("could not update config.toml", error))?
        .text;
    atomic_write(path, &fill_missing_defaults(&reconcile_contract(&text)?)?)
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
        "string" => validate_choice(value, spec.path, spec.allowed_values),
        _ => Err(error(format!("{} must be {}", spec.path, spec.validation))),
    }
}

fn open_log_level_from_json(value: &JsonValue) -> Result<&str> {
    let Some(value) = value.as_str() else {
        return Err(error("open.log_level must be a string"));
    };
    if !OPEN_LOG_LEVELS.contains(&value) {
        return Err(error(format!(
            "open.log_level must be one of: {}",
            OPEN_LOG_LEVELS.join(", ")
        )));
    }
    Ok(value)
}

fn read_zellij_sidecar(path: &Path) -> Result<(ZellijSidecar, Vec<ConfigUiDiagnostic>)> {
    let raw = fs::read_to_string(path)?;
    Ok(parse_zellij_sidecar(&raw))
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
        let mut line = strip_kdl_comment(raw_line).trim();
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
        let Some(token) = first_kdl_token(line) else {
            continue;
        };
        match stack.as_slice() {
            [] => parse_zellij_top_level_line(&mut config, &mut diagnostics, line, token, line_number),
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
            _ => diagnostics.push(zellij_diagnostic(
                line_number,
                "unsupported nested Zellij block".to_string(),
                "The managed editor only supports scalar sidecar preferences and ui.pane_frames.rounded_corners.",
            )),
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
    ZELLIJ_FIELDS
        .iter()
        .find(|spec| spec.path == token && !spec.path.contains('.'))
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

fn first_kdl_token(line: &str) -> Option<&str> {
    line.split(|ch: char| ch.is_whitespace() || ch == '{' || ch == ';')
        .next()
        .filter(|token| !token.is_empty())
}

fn strip_kdl_comment(line: &str) -> &str {
    line.split_once("//").map_or(line, |(content, _)| content)
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
            validate_choice(value, path, ZELLIJ_COPY_CLIPBOARD_VALUES)?;
            config.copy_clipboard = value.as_str().expect("validated string").to_string();
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

fn default_zellij_config_kdl() -> String {
    render_zellij_sidecar(&ZellijSidecar::default())
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

fn validate_choice(value: &JsonValue, path: &str, allowed_values: &[&str]) -> Result<()> {
    let Some(value) = value.as_str() else {
        return Err(error(format!("{path} must be a string")));
    };
    if !allowed_values.is_empty() && !allowed_values.contains(&value) {
        return Err(error(format!(
            "{path} must be one of: {}",
            allowed_values.join(", ")
        )));
    }
    Ok(())
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
        }
    }

    fn ensure_temp_sources(paths: &ConfigPaths) {
        ensure_config_file_at(paths.root.clone()).unwrap();
        ensure_plain_config_file_at(&paths.mars, DEFAULT_MARS_CONFIG_TOML).unwrap();
        ensure_plain_config_file_at(&paths.zellij, &default_zellij_config_kdl()).unwrap();
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

    // Defends: yzn config creates the owned TOML config file with defaults and joined contract state.
    #[test]
    fn ensure_config_creates_defaults_and_contract_state() {
        let temp = TempHome::new();
        let path = ensure_config_file_at(temp.path.join("config.toml")).unwrap();
        let value = read_config_value(&path).unwrap();

        assert_eq!(
            get_toml_path(&value, OPEN_LOG_LEVEL_PATH),
            Some(&json!("info"))
        );
        assert_eq!(
            get_toml_path(&value, "ratconfig.contract.contract_id"),
            Some(&json!(CONTRACT_ID))
        );
        assert_eq!(
            get_toml_path(&value, "ratconfig.contract.version"),
            Some(&json!(CONTRACT_VERSION))
        );
    }

    // Defends: config edits are validated and persisted through ratconfig's TOML patch path.
    #[test]
    fn write_config_field_persists_valid_log_level_and_rejects_bad_values() {
        let temp = TempHome::new();
        let path = ensure_config_file_at(temp.path.join("config.toml")).unwrap();

        write_config_field(&path, OPEN_LOG_LEVEL_PATH, &json!("debug")).unwrap();
        let value = read_config_value(&path).unwrap();
        assert_eq!(
            get_toml_path(&value, OPEN_LOG_LEVEL_PATH),
            Some(&json!("debug"))
        );

        let error = write_config_field(&path, OPEN_LOG_LEVEL_PATH, &json!("loud")).unwrap_err();
        assert!(error.to_string().contains("off, error, info, debug"));
    }

    // Defends: atomic writes/reconciliation must not replace existing read-only config sources.
    #[test]
    fn read_only_existing_sources_are_not_replaced() {
        let temp = TempHome::new();
        let paths = temp_paths(&temp);
        ensure_temp_sources(&paths);

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

    // Defends: hand-edited config is validated before managed runtime exports it.
    #[test]
    fn manual_invalid_log_level_is_rejected_on_read_and_marked_invalid() {
        let temp = TempHome::new();
        let path = temp.path.join("config.toml");
        fs::write(&path, "[open]\nlog_level = \"loud\"\n").unwrap();

        let error = read_open_log_level(&path).unwrap_err();
        assert!(error.to_string().contains("off, error, info, debug"));

        let paths = temp_paths(&temp);
        ensure_temp_sources(&paths);

        let model = build_model(&paths).unwrap();
        assert_eq!(model.fields[0].state, ConfigUiValueState::Invalid);
    }

    // Defends: yzn config creates each owned source file without adding contracts to sidecars.
    #[test]
    fn ensure_config_sources_creates_root_mars_and_zellij_files() {
        let temp = TempHome::new();
        let paths = temp_paths(&temp);

        ensure_temp_sources(&paths);

        assert!(paths.root.exists());
        assert!(paths.mars.exists());
        assert!(paths.zellij.exists());
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
    }

    // Defends: source ids route writes to the selected backing file.
    #[test]
    fn source_routing_writes_mars_without_touching_config_toml() {
        let temp = TempHome::new();
        let paths = temp_paths(&temp);
        ensure_temp_sources(&paths);
        let before_root = fs::read_to_string(&paths.root).unwrap();

        write_source_field(&paths, SOURCE_MARS, "window.width", &json!(1200)).unwrap();

        assert_eq!(fs::read_to_string(&paths.root).unwrap(), before_root);
        let mars = read_toml_file_value(&paths.mars, "mars").unwrap();
        assert_eq!(get_toml_path(&mars, "window.width"), Some(&json!(1200)));
    }

    // Defends: the Zellij source writes the nested rounded-corners scalar in managed KDL.
    #[test]
    fn zellij_source_renders_nested_rounded_corners() {
        let temp = TempHome::new();
        let paths = temp_paths(&temp);
        ensure_temp_sources(&paths);

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

    // Defends: yzn config refuses to rewrite a sidecar that contains guarded runtime ownership.
    #[test]
    fn zellij_source_blocks_guarded_sidecar_nodes() {
        let temp = TempHome::new();
        let path = temp.path.join("zellij/config.kdl");
        atomic_write(&path, "keybinds {}\npane_frames true\n").unwrap();

        let (_config, diagnostics) = read_zellij_sidecar(&path).unwrap();
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.blocking));

        let error = write_zellij_config_field(&path, "pane_frames", &json!(false)).unwrap_err();
        assert!(error.to_string().contains("guarded Zellij node"));
    }

    // Defends: the UI-side KDL guard matches launch-time first-token behavior.
    #[test]
    fn zellij_sidecar_skips_hash_comments_and_blocks_compact_guarded_nodes() {
        let (config, diagnostics) = parse_zellij_sidecar("# note\npane_frames false;\n");
        assert!(diagnostics.is_empty());
        assert!(!config.pane_frames);

        let (_config, diagnostics) = parse_zellij_sidecar("# note\nkeybinds{}\n");
        assert!(has_diagnostic(&diagnostics, "guarded Zellij node"));
    }

    // Defends: yzn config does not generate or silently accept invalid Zellij scalars/blocks.
    #[test]
    fn zellij_sidecar_rejects_non_positive_scrollback_and_unclosed_blocks() {
        let temp = TempHome::new();
        let path = temp.path.join("zellij/config.kdl");
        atomic_write(&path, &default_zellij_config_kdl()).unwrap();

        let error = write_zellij_config_field(&path, "scroll_buffer_size", &json!(-1)).unwrap_err();
        assert!(error.to_string().contains("positive integer"));

        let (_config, diagnostics) = parse_zellij_sidecar("scroll_buffer_size -1\n");
        assert!(has_diagnostic(&diagnostics, "scroll_buffer_size"));

        let (_config, diagnostics) = parse_zellij_sidecar("ui {\n");
        assert!(has_diagnostic(&diagnostics, "unterminated"));
    }

    // Regression: unsupported terminal keys must be ignored, not converted to text input.
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
