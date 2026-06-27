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
    ConfigContract, ConfigUiApp, ConfigUiApplyStatus, ConfigUiEditBehavior, ConfigUiFieldRowSpec,
    ConfigUiIntent, ConfigUiKey, ConfigUiModel, ConfigUiPathOwner, build_config_ui_field,
    draw_config_ui, join_toml_contract_text_from_version, reconcile_joined_toml_contract_text,
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
    let path = ensure_config_file()?;
    let mut app = ConfigUiApp::new(build_model(&path)?);
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
            intent => {
                handle_ui_intent(&mut app, &path, intent)?;
            }
        }
    }

    Ok(())
}

fn handle_ui_intent(app: &mut ConfigUiApp, path: &Path, intent: ConfigUiIntent) -> Result<()> {
    match intent {
        ConfigUiIntent::None | ConfigUiIntent::Exit => {}
        ConfigUiIntent::BeginEdit { field_index, .. } => app.begin_edit_field(field_index),
        ConfigUiIntent::SetField {
            path: field_path,
            value,
            ..
        } => {
            write_config_field(path, &field_path, &value)?;
            app.model = build_model(path)?;
            app.notice_info(format!("Saved {field_path}."));
            app.finish_successful_write();
        }
        ConfigUiIntent::UnsetField {
            path: field_path, ..
        } => {
            restore_config_default(path, &field_path)?;
            app.model = build_model(path)?;
            app.notice_info(format!("Restored default for {field_path}."));
        }
    }
    Ok(())
}

struct TerminalSession;

impl TerminalSession {
    fn enter() -> Result<Self> {
        enable_raw_mode()?;
        execute!(io::stdout(), EnterAlternateScreen, cursor::Hide)?;
        Ok(Self)
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
        KeyCode::Char(ch) => char_key(ch, key.modifiers),
        _ => None,
    }
}

fn char_key(ch: char, modifiers: KeyModifiers) -> Option<ConfigUiKey> {
    let unsupported =
        KeyModifiers::ALT | KeyModifiers::SUPER | KeyModifiers::HYPER | KeyModifiers::META;
    if modifiers.intersects(unsupported) {
        return None;
    }
    if modifiers.contains(KeyModifiers::CONTROL) {
        Some(ConfigUiKey::Ctrl(ch))
    } else {
        Some(ConfigUiKey::Char(ch))
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
    ensure_config_file_at(config_path()?)
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
        atomic_write(&path, &completed)?;
    }
    Ok(path)
}

fn config_path() -> Result<PathBuf> {
    if let Some(path) = env::var_os("YAZELIX_NEXT_CONFIG_HOME") {
        return Ok(PathBuf::from(path).join("config.toml"));
    }
    if let Some(path) = env::var_os("XDG_CONFIG_HOME") {
        return Ok(PathBuf::from(path).join("yazelix-next/config.toml"));
    }
    let home = env::var_os("HOME").ok_or_else(|| error("HOME is required"))?;
    Ok(PathBuf::from(home).join(".config/yazelix-next/config.toml"))
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
    let raw = fs::read_to_string(path)?;
    parse_toml_value(&raw).map_err(|error| boxed_debug("invalid config.toml", error))
}

fn read_open_log_level(path: &Path) -> Result<String> {
    let value = read_config_value(path)?;
    let Some(value) = get_toml_path(&value, OPEN_LOG_LEVEL_PATH) else {
        return Err(error(format!("unknown config path: {OPEN_LOG_LEVEL_PATH}")));
    };
    Ok(open_log_level_from_json(value)?.to_string())
}

fn build_model(path: &Path) -> Result<ConfigUiModel> {
    let active = read_config_value(path)?;
    let default = open_log_level_default();
    let current = get_toml_path(&active, OPEN_LOG_LEVEL_PATH);
    let fields = vec![build_config_ui_field(ConfigUiFieldRowSpec {
        path: OPEN_LOG_LEVEL_PATH,
        tab: "general",
        kind: "string",
        current,
        default: Some(&default),
        description: "Diagnostics written by yzn-open for Yazi-to-Helix open requests.".to_string(),
        allowed_values: OPEN_LOG_LEVELS
            .iter()
            .map(|value| (*value).to_string())
            .collect(),
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
    })];

    Ok(ConfigUiModel {
        active_config_path: path.to_path_buf(),
        cursor_config_path: PathBuf::new(),
        default_cursor_config_path: PathBuf::new(),
        active_config_exists: path.exists(),
        config_owner: ConfigUiPathOwner::User,
        config_read_only: fs::metadata(path)
            .map(|metadata| metadata.permissions().readonly())
            .unwrap_or(false),
        tabs: vec!["general".to_string()],
        fields,
        sidecars: Vec::new(),
        native_config_statuses: Vec::new(),
        diagnostics: Vec::new(),
    })
}

fn open_log_level_default() -> JsonValue {
    json!(OPEN_LOG_LEVEL_DEFAULT)
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

fn restore_config_default(path: &Path, field_path: &str) -> Result<()> {
    write_config_field(path, field_path, &open_log_level_default())
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

    // Defends: hand-edited config is validated before managed runtime exports it.
    #[test]
    fn manual_invalid_log_level_is_rejected_on_read_and_marked_invalid() {
        let temp = TempHome::new();
        let path = temp.path.join("config.toml");
        fs::write(&path, "[open]\nlog_level = \"loud\"\n").unwrap();

        let error = read_open_log_level(&path).unwrap_err();
        assert!(error.to_string().contains("off, error, info, debug"));

        let model = build_model(&path).unwrap();
        assert_eq!(model.fields[0].state, ConfigUiValueState::Invalid);
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
