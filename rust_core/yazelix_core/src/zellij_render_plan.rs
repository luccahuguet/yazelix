//! Typed Zellij render-plan data for generated Zellij config and layout KDL.

use crate::bridge::{CoreError, ErrorClass};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

const ZELLIJ_THEMES: &[&str] = &[
    "ansi",
    "ao",
    "atelier-sulphurpool",
    "ayu_mirage",
    "ayu_dark",
    "catppuccin-frappe",
    "catppuccin-macchiato",
    "cyber-noir",
    "blade-runner",
    "retro-wave",
    "dracula",
    "everforest-dark",
    "gruvbox-dark",
    "iceberg-dark",
    "kanagawa",
    "lucario",
    "menace",
    "molokai-dark",
    "night-owl",
    "nightfox",
    "nord",
    "one-half-dark",
    "onedark",
    "solarized-dark",
    "tokyo-night-dark",
    "tokyo-night-storm",
    "tokyo-night",
    "vesper",
    "ayu_light",
    "catppuccin-latte",
    "everforest-light",
    "gruvbox-light",
    "iceberg-light",
    "dayfox",
    "pencil-light",
    "solarized-light",
    "tokyo-night-light",
];

const WIDGET_TRAY_ALLOWED: &[&str] = &[
    "editor",
    "shell",
    "term",
    "workspace",
    "cursor",
    "claude_usage",
    "codex_usage",
    "opencode_go_usage",
    "cpu",
    "ram",
];
const DEFAULT_LEFT_SIDEBAR_WIDTH_PERCENT: i64 = 20;
const DEFAULT_RIGHT_SIDEBAR_WIDTH_PERCENT: i64 = 40;
const SCREEN_SAVER_STYLE_ALLOWED: &[&str] = &[
    "logo",
    "boids",
    "boids_predator",
    "boids_schools",
    "mandelbrot",
    "game_of_life_gliders",
    "game_of_life_oscillators",
    "game_of_life_bloom",
    "random",
];
const TAB_LABEL_MODE_FULL: &str = "full";
const TAB_LABEL_MODE_COMPACT: &str = "compact";
const TAB_LABEL_MODE_ALLOWED: &[&str] = &[TAB_LABEL_MODE_FULL, TAB_LABEL_MODE_COMPACT];
const CLAUDE_CODEX_USAGE_PERIODS_ALLOWED: &[&str] = &["5h", "week"];
const OPENCODE_GO_USAGE_PERIODS_ALLOWED: &[&str] = &["5h", "week", "month"];
pub const DEFAULT_LEFT_SIDEBAR_COMMAND: &str = "yzx";
pub const DEFAULT_LEFT_SIDEBAR_YAZI_ARGS: &[&str] = &["sidebar", "yazi"];
pub const DEFAULT_RIGHT_SIDEBAR_COMMAND: &str = "yzx";
pub const DEFAULT_RIGHT_SIDEBAR_AGENT_ARGS: &[&str] = &["agent"];

fn default_left_sidebar_width_percent() -> i64 {
    DEFAULT_LEFT_SIDEBAR_WIDTH_PERCENT
}

fn default_right_sidebar_width_percent() -> i64 {
    DEFAULT_RIGHT_SIDEBAR_WIDTH_PERCENT
}

fn default_left_sidebar_command() -> String {
    DEFAULT_LEFT_SIDEBAR_COMMAND.into()
}

fn default_right_sidebar_command() -> String {
    DEFAULT_RIGHT_SIDEBAR_COMMAND.into()
}

fn default_left_sidebar_args() -> Vec<String> {
    DEFAULT_LEFT_SIDEBAR_YAZI_ARGS
        .iter()
        .map(|arg| (*arg).to_string())
        .collect()
}

fn default_right_sidebar_args() -> Vec<String> {
    DEFAULT_RIGHT_SIDEBAR_AGENT_ARGS
        .iter()
        .map(|arg| (*arg).to_string())
        .collect()
}

fn default_popup_percent() -> i64 {
    90
}

fn default_screen_saver_idle_seconds() -> i64 {
    300
}

fn default_screen_saver_style() -> String {
    "random".into()
}

fn default_zellij_theme() -> String {
    "default".into()
}

fn default_string_true() -> String {
    "true".into()
}

fn default_support_kitty_keyboard_protocol() -> String {
    "true".into()
}

fn default_zellij_default_mode() -> String {
    "normal".into()
}

fn default_tab_label_mode() -> String {
    TAB_LABEL_MODE_FULL.into()
}

fn default_claude_usage_display() -> String {
    "both".into()
}

fn default_codex_usage_display() -> String {
    "quota".into()
}

fn default_opencode_go_usage_display() -> String {
    "both".into()
}

fn default_claude_usage_periods() -> Vec<String> {
    vec!["5h".into(), "week".into()]
}

fn default_codex_usage_periods() -> Vec<String> {
    vec!["5h".into(), "week".into()]
}

fn default_opencode_go_usage_periods() -> Vec<String> {
    vec!["5h".into(), "week".into(), "month".into()]
}

fn default_widget_tray() -> Vec<String> {
    vec![
        "editor".into(),
        "shell".into(),
        "term".into(),
        "cursor".into(),
        "codex_usage".into(),
        "cpu".into(),
        "ram".into(),
    ]
}

fn default_editor_label() -> String {
    "hx".into()
}

fn default_terminal_label() -> String {
    "wezterm".into()
}

fn default_shell_label() -> String {
    "nu".into()
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ZellijRenderPlanRequest {
    #[serde(default = "default_left_sidebar_width_percent")]
    pub left_sidebar_width_percent: i64,
    #[serde(default = "default_left_sidebar_command")]
    pub left_sidebar_command: String,
    #[serde(default = "default_left_sidebar_args")]
    pub left_sidebar_args: Vec<String>,
    #[serde(default = "default_right_sidebar_width_percent")]
    pub right_sidebar_width_percent: i64,
    #[serde(default = "default_right_sidebar_command")]
    pub right_sidebar_command: String,
    #[serde(default = "default_right_sidebar_args")]
    pub right_sidebar_args: Vec<String>,
    #[serde(default = "default_popup_percent")]
    pub popup_width_percent: i64,
    #[serde(default = "default_popup_percent")]
    pub popup_height_percent: i64,
    #[serde(default)]
    pub screen_saver_enabled: bool,
    #[serde(default = "default_screen_saver_idle_seconds")]
    pub screen_saver_idle_seconds: i64,
    #[serde(default = "default_screen_saver_style")]
    pub screen_saver_style: String,
    #[serde(default)]
    pub zellij_widget_tray: Option<Vec<String>>,
    #[serde(default)]
    pub zellij_custom_text: Option<String>,
    #[serde(default = "default_zellij_theme")]
    pub zellij_theme: String,
    #[serde(default = "default_string_true")]
    pub zellij_pane_frames: String,
    #[serde(default = "default_string_true")]
    pub zellij_rounded_corners: String,
    #[serde(default = "default_string_true")]
    pub disable_zellij_tips: String,
    /// Matches `config_metadata/main_config_contract.toml` (`zellij.support_kitty_keyboard_protocol` default true).
    #[serde(default = "default_support_kitty_keyboard_protocol")]
    pub support_kitty_keyboard_protocol: String,
    #[serde(default = "default_zellij_default_mode")]
    pub zellij_default_mode: String,
    #[serde(default = "default_tab_label_mode")]
    pub zellij_tab_label_mode: String,
    #[serde(default = "default_claude_usage_display")]
    pub zellij_claude_usage_display: String,
    #[serde(default = "default_codex_usage_display")]
    pub zellij_codex_usage_display: String,
    #[serde(default = "default_opencode_go_usage_display")]
    pub zellij_opencode_go_usage_display: String,
    #[serde(default = "default_claude_usage_periods")]
    pub zellij_claude_usage_periods: Vec<String>,
    #[serde(default = "default_codex_usage_periods")]
    pub zellij_codex_usage_periods: Vec<String>,
    #[serde(default = "default_opencode_go_usage_periods")]
    pub zellij_opencode_go_usage_periods: Vec<String>,
    pub yazelix_layout_dir: String,
    pub resolved_default_shell: String,
    #[serde(default = "default_editor_label")]
    pub editor_label: String,
    #[serde(default = "default_shell_label")]
    pub shell_label: String,
    #[serde(default = "default_terminal_label")]
    pub terminal_label: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct LayoutPlaceholderPercents {
    pub left_sidebar_width_percent: String,
    pub right_sidebar_width_percent: String,
    pub open_content_width_percent: String,
    pub closed_content_width_percent: String,
    pub left_open_right_open_content_width_percent: String,
    pub left_open_right_closed_content_width_percent: String,
    pub left_closed_right_open_content_width_percent: String,
    pub left_closed_right_closed_content_width_percent: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct TopLevelSetting {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ZellijRenderPlanData {
    pub default_layout_name: String,
    pub left_sidebar_width_percent: i64,
    pub left_sidebar_command: String,
    pub left_sidebar_args: Vec<String>,
    pub right_sidebar_width_percent: i64,
    pub right_sidebar_command: String,
    pub right_sidebar_args: Vec<String>,
    pub popup_width_percent: i64,
    pub popup_height_percent: i64,
    pub screen_saver_enabled: bool,
    pub screen_saver_idle_seconds: i64,
    pub screen_saver_style: String,
    pub widget_tray: Vec<String>,
    pub editor_label: String,
    pub shell_label: String,
    pub terminal_label: String,
    pub tab_label_mode: String,
    pub claude_usage_display: String,
    pub codex_usage_display: String,
    pub opencode_go_usage_display: String,
    pub claude_usage_periods: Vec<String>,
    pub codex_usage_periods: Vec<String>,
    pub opencode_go_usage_periods: Vec<String>,
    pub custom_text: String,
    pub layout_percentages: LayoutPlaceholderPercents,
    pub rounded_value: String,
    pub dynamic_top_level_settings: Vec<TopLevelSetting>,
    pub enforced_top_level_settings: Vec<TopLevelSetting>,
    pub owned_top_level_setting_names: Vec<String>,
}

fn bool_setting_from_string(raw: &str) -> bool {
    !raw.trim_start().starts_with("false")
}

fn compute_layout_percentages(
    left_sidebar_width_percent: i64,
    right_sidebar_width_percent: i64,
) -> LayoutPlaceholderPercents {
    let open_content_width_percent = 100 - left_sidebar_width_percent;
    let closed_content_width_percent = 99;
    let left_open_right_open_content_width_percent =
        100 - left_sidebar_width_percent - right_sidebar_width_percent;
    let left_open_right_closed_content_width_percent = 99 - left_sidebar_width_percent;
    let left_closed_right_open_content_width_percent = 99 - right_sidebar_width_percent;
    let left_closed_right_closed_content_width_percent = 98;

    LayoutPlaceholderPercents {
        left_sidebar_width_percent: format!("{left_sidebar_width_percent}%"),
        right_sidebar_width_percent: format!("{right_sidebar_width_percent}%"),
        open_content_width_percent: format!("{open_content_width_percent}%"),
        closed_content_width_percent: format!("{closed_content_width_percent}%"),
        left_open_right_open_content_width_percent: format!(
            "{left_open_right_open_content_width_percent}%"
        ),
        left_open_right_closed_content_width_percent: format!(
            "{left_open_right_closed_content_width_percent}%"
        ),
        left_closed_right_open_content_width_percent: format!(
            "{left_closed_right_open_content_width_percent}%"
        ),
        left_closed_right_closed_content_width_percent: format!(
            "{left_closed_right_closed_content_width_percent}%"
        ),
    }
}

fn validate_sidebar_width(field: &str, value: i64) -> Result<(), CoreError> {
    if (1..=48).contains(&value) {
        Ok(())
    } else {
        Err(CoreError::classified(
            ErrorClass::Config,
            "invalid_sidebar_width_percent",
            format!("{field} must be between 1 and 48 (got {value})"),
            "Set the sidebar width_percent within the documented range.",
            serde_json::json!({ "field": field }),
        ))
    }
}

fn validate_sidebar_launcher(field: &str, command: &str) -> Result<(), CoreError> {
    if !command.trim().is_empty() {
        Ok(())
    } else {
        Err(CoreError::classified(
            ErrorClass::Config,
            "invalid_sidebar_command",
            format!("{field} must not be empty"),
            "Set the sidebar command to a terminal command such as `yzx`, `nu`, `yazi`, `codex`, or another side-surface launcher.",
            serde_json::json!({ "field": field }),
        ))
    }
}

pub fn effective_left_sidebar_args(command: &str, args: &[String]) -> Vec<String> {
    if !is_default_left_sidebar_command(command) && args == default_left_sidebar_args().as_slice() {
        Vec::new()
    } else {
        args.to_vec()
    }
}

fn is_default_left_sidebar_command(command: &str) -> bool {
    let trimmed = command.trim();
    if trimmed == DEFAULT_LEFT_SIDEBAR_COMMAND {
        return true;
    }
    Path::new(trimmed)
        .file_name()
        .and_then(|name| name.to_str())
        .map(|name| name == DEFAULT_LEFT_SIDEBAR_COMMAND)
        .unwrap_or(false)
}

fn validate_popup_percent(field: &str, value: i64) -> Result<(), CoreError> {
    if (1..=100).contains(&value) {
        Ok(())
    } else {
        Err(CoreError::classified(
            ErrorClass::Config,
            "invalid_popup_percent",
            format!("{field} must be between 1 and 100 (got {value})"),
            "Set the Zellij popup size percents within the documented range.",
            serde_json::json!({ "field": field }),
        ))
    }
}

fn validate_screen_saver_idle_seconds(value: i64) -> Result<(), CoreError> {
    if (10..=86400).contains(&value) {
        Ok(())
    } else {
        Err(CoreError::classified(
            ErrorClass::Config,
            "invalid_screen_saver_idle_seconds",
            format!("zellij.screen_saver_idle_seconds must be between 10 and 86400 (got {value})"),
            "Set zellij.screen_saver_idle_seconds to a supported idle threshold.",
            serde_json::json!({ "field": "zellij.screen_saver_idle_seconds" }),
        ))
    }
}

fn normalize_screen_saver_style(style: &str) -> Result<String, CoreError> {
    let normalized = style.trim().to_ascii_lowercase();
    if SCREEN_SAVER_STYLE_ALLOWED.contains(&normalized.as_str()) {
        Ok(normalized)
    } else {
        Err(CoreError::classified(
            ErrorClass::Config,
            "invalid_screen_saver_style",
            format!(
                "Invalid zellij.screen_saver_style `{normalized}`. Expected one of: {}",
                SCREEN_SAVER_STYLE_ALLOWED.join(", ")
            ),
            "Use one of the animated styles accepted by `yzx screen`.",
            serde_json::json!({ "field": "zellij.screen_saver_style", "style": normalized }),
        ))
    }
}

fn validate_widget_tray(entries: &[String]) -> Result<(), CoreError> {
    for entry in entries {
        if !WIDGET_TRAY_ALLOWED.contains(&entry.as_str()) {
            let allowed = WIDGET_TRAY_ALLOWED.join(", ");
            return Err(CoreError::classified(
                ErrorClass::Config,
                "invalid_widget_tray_entry",
                format!("Invalid zellij.widget_tray entry: {entry} (allowed: {allowed})"),
                "Use only documented widget tray identifiers.",
                serde_json::json!({ "field": "zellij.widget_tray", "entry": entry }),
            ));
        }
    }
    Ok(())
}

fn validate_default_mode(mode: &str) -> Result<(), CoreError> {
    if mode == "normal" || mode == "locked" {
        Ok(())
    } else {
        Err(CoreError::classified(
            ErrorClass::Config,
            "invalid_zellij_default_mode",
            format!("zellij_default_mode must be \"normal\" or \"locked\" (got {mode:?})"),
            "Set zellij.default_mode to a supported value.",
            serde_json::json!({ "field": "zellij.default_mode" }),
        ))
    }
}

fn normalize_usage_display(field: &str, raw: &str, default: &str) -> Result<String, CoreError> {
    let trimmed = raw.trim();
    let value = if trimmed.is_empty() { default } else { trimmed };
    match value {
        "both" => Ok("both".to_string()),
        "token" | "tokens" => Ok("token".to_string()),
        "quota" => Ok("quota".to_string()),
        _ => Err(CoreError::classified(
            ErrorClass::Config,
            "invalid_zellij_usage_display",
            format!("{field} must be one of: both, token, quota (got {raw})"),
            "Set the usage widget display mode to both, token, or quota.",
            serde_json::json!({ "field": field, "value": raw }),
        )),
    }
}

fn normalize_usage_periods(
    field: &str,
    raw: &[String],
    allowed: &[&str],
) -> Result<Vec<String>, CoreError> {
    if raw.is_empty() {
        return Err(CoreError::classified(
            ErrorClass::Config,
            "invalid_zellij_usage_periods",
            format!("{field} must include at least one period"),
            "Select at least one supported usage period, or remove the usage widget from zellij.widget_tray.",
            serde_json::json!({ "field": field }),
        ));
    }

    let mut periods = Vec::new();
    for value in raw {
        let normalized = value.trim().to_ascii_lowercase();
        if !allowed.contains(&normalized.as_str()) {
            return Err(CoreError::classified(
                ErrorClass::Config,
                "invalid_zellij_usage_periods",
                format!(
                    "{field} contains unsupported period `{}`. Expected one of: {}",
                    value,
                    allowed.join(", ")
                ),
                "Use only supported usage period identifiers.",
                serde_json::json!({ "field": field, "value": value }),
            ));
        }
        if !periods.contains(&normalized) {
            periods.push(normalized);
        }
    }
    Ok(periods)
}

fn normalize_tab_label_mode(mode: &str) -> Result<String, CoreError> {
    let normalized = mode.trim().to_ascii_lowercase();
    if TAB_LABEL_MODE_ALLOWED.contains(&normalized.as_str()) {
        Ok(normalized)
    } else {
        Err(CoreError::classified(
            ErrorClass::Config,
            "invalid_tab_label_mode",
            format!(
                "Invalid zellij.tab_label_mode `{normalized}`. Expected one of: {}",
                TAB_LABEL_MODE_ALLOWED.join(", ")
            ),
            "Set zellij.tab_label_mode to `full` or `compact`.",
            serde_json::json!({ "field": "zellij.tab_label_mode", "mode": normalized }),
        ))
    }
}

fn pick_theme(resolved_theme_config: &str) -> String {
    if resolved_theme_config == "random" {
        let idx = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| (d.as_nanos() as usize) % ZELLIJ_THEMES.len())
            .unwrap_or(0);
        ZELLIJ_THEMES
            .get(idx)
            .map(|s| (*s).to_string())
            .unwrap_or_else(|| "default".into())
    } else {
        resolved_theme_config.to_string()
    }
}

fn make_setting(name: impl Into<String>, value: impl Into<String>) -> TopLevelSetting {
    TopLevelSetting {
        name: name.into(),
        value: value.into(),
    }
}

fn kdl_quoted_path(path: &Path) -> String {
    format!("\"{}\"", path.to_string_lossy())
}

fn status_label(raw: &str, default: &str) -> String {
    let first_token = raw.split_whitespace().next().unwrap_or("");
    let candidate = Path::new(first_token)
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.trim().is_empty())
        .unwrap_or(first_token)
        .trim();
    let mut label = candidate
        .chars()
        .filter(|character| {
            !character.is_control() && !matches!(character, '[' | ']' | '{' | '}' | '"' | '\\')
        })
        .take(24)
        .collect::<String>();
    if label.trim().is_empty() {
        label = default.to_string();
    }
    label
}

pub fn compute_zellij_render_plan(
    request: &ZellijRenderPlanRequest,
) -> Result<ZellijRenderPlanData, CoreError> {
    validate_sidebar_width(
        "workspace.left_sidebar.width_percent",
        request.left_sidebar_width_percent,
    )?;
    validate_sidebar_width(
        "workspace.right_sidebar.width_percent",
        request.right_sidebar_width_percent,
    )?;
    validate_sidebar_launcher(
        "workspace.left_sidebar.command",
        &request.left_sidebar_command,
    )?;
    validate_sidebar_launcher(
        "workspace.right_sidebar.command",
        &request.right_sidebar_command,
    )?;
    validate_popup_percent("zellij.popup_width_percent", request.popup_width_percent)?;
    validate_popup_percent("zellij.popup_height_percent", request.popup_height_percent)?;
    validate_screen_saver_idle_seconds(request.screen_saver_idle_seconds)?;
    validate_default_mode(&request.zellij_default_mode)?;
    let screen_saver_style = normalize_screen_saver_style(&request.screen_saver_style)?;
    let tab_label_mode = normalize_tab_label_mode(&request.zellij_tab_label_mode)?;
    let claude_usage_display = normalize_usage_display(
        "zellij.claude_usage_display",
        &request.zellij_claude_usage_display,
        "both",
    )?;
    let codex_usage_display = normalize_usage_display(
        "zellij.codex_usage_display",
        &request.zellij_codex_usage_display,
        "quota",
    )?;
    let opencode_go_usage_display = normalize_usage_display(
        "zellij.opencode_go_usage_display",
        &request.zellij_opencode_go_usage_display,
        "both",
    )?;
    let claude_usage_periods = normalize_usage_periods(
        "zellij.claude_usage_periods",
        &request.zellij_claude_usage_periods,
        CLAUDE_CODEX_USAGE_PERIODS_ALLOWED,
    )?;
    let codex_usage_periods = normalize_usage_periods(
        "zellij.codex_usage_periods",
        &request.zellij_codex_usage_periods,
        CLAUDE_CODEX_USAGE_PERIODS_ALLOWED,
    )?;
    let opencode_go_usage_periods = normalize_usage_periods(
        "zellij.opencode_go_usage_periods",
        &request.zellij_opencode_go_usage_periods,
        OPENCODE_GO_USAGE_PERIODS_ALLOWED,
    )?;

    let widget_tray = request
        .zellij_widget_tray
        .clone()
        .unwrap_or_else(default_widget_tray);
    validate_widget_tray(&widget_tray)?;

    let custom_text = request
        .zellij_custom_text
        .as_deref()
        .unwrap_or("")
        .trim()
        .to_string();
    let editor_label = status_label(&request.editor_label, "hx");
    let shell_label = status_label(&request.shell_label, "nu");
    let terminal_label = status_label(&request.terminal_label, "ghostty");

    let default_layout_name = MANAGED_SIDEBAR_LAYOUT_NAME.to_string();

    let layout_percentages = compute_layout_percentages(
        request.left_sidebar_width_percent,
        request.right_sidebar_width_percent,
    );

    let theme = pick_theme(&request.zellij_theme);
    let pane_frames_value = if bool_setting_from_string(&request.zellij_pane_frames) {
        "true"
    } else {
        "false"
    };
    let rounded_value = if bool_setting_from_string(&request.zellij_rounded_corners) {
        "true"
    } else {
        "false"
    };
    let show_tips_value = if bool_setting_from_string(&request.disable_zellij_tips) {
        "false"
    } else {
        "true"
    };
    let kitty_protocol_value = if bool_setting_from_string(&request.support_kitty_keyboard_protocol)
    {
        "true"
    } else {
        "false"
    };

    let layout_dir_path = Path::new(&request.yazelix_layout_dir);
    let default_layout_path = layout_dir_path.join(format!("{default_layout_name}.kdl"));

    let dynamic_top_level_settings = vec![
        make_setting("theme", kdl_quoted_path(Path::new(&theme))),
        make_setting("show_startup_tips", show_tips_value),
        make_setting("show_release_notes", "false"),
        make_setting("on_force_close", kdl_quoted_path(Path::new("quit"))),
        make_setting("pane_frames", pane_frames_value),
    ];

    let enforced_top_level_settings = vec![
        make_setting("session_serialization", "true"),
        make_setting("serialize_pane_viewport", "true"),
        make_setting("support_kitty_keyboard_protocol", kitty_protocol_value),
        make_setting(
            "default_mode",
            kdl_quoted_path(Path::new(&request.zellij_default_mode)),
        ),
        make_setting(
            "default_shell",
            kdl_quoted_path(Path::new(&request.resolved_default_shell)),
        ),
        make_setting("default_layout", kdl_quoted_path(&default_layout_path)),
        make_setting("layout_dir", kdl_quoted_path(layout_dir_path)),
    ];

    let owned_top_level_setting_names: Vec<String> = dynamic_top_level_settings
        .iter()
        .chain(enforced_top_level_settings.iter())
        .map(|s| s.name.clone())
        .collect();

    Ok(ZellijRenderPlanData {
        default_layout_name,
        left_sidebar_width_percent: request.left_sidebar_width_percent,
        left_sidebar_command: request.left_sidebar_command.trim().to_string(),
        left_sidebar_args: effective_left_sidebar_args(
            &request.left_sidebar_command,
            &request.left_sidebar_args,
        ),
        right_sidebar_width_percent: request.right_sidebar_width_percent,
        right_sidebar_command: request.right_sidebar_command.trim().to_string(),
        right_sidebar_args: request.right_sidebar_args.clone(),
        popup_width_percent: request.popup_width_percent,
        popup_height_percent: request.popup_height_percent,
        screen_saver_enabled: request.screen_saver_enabled,
        screen_saver_idle_seconds: request.screen_saver_idle_seconds,
        screen_saver_style,
        widget_tray,
        editor_label,
        shell_label,
        terminal_label,
        tab_label_mode,
        claude_usage_display,
        codex_usage_display,
        opencode_go_usage_display,
        claude_usage_periods,
        codex_usage_periods,
        opencode_go_usage_periods,
        custom_text,
        layout_percentages,
        rounded_value: rounded_value.to_string(),
        dynamic_top_level_settings,
        enforced_top_level_settings,
        owned_top_level_setting_names,
    })
}

pub const MANAGED_SIDEBAR_LAYOUT_NAME: &str = "yzx_side";

// Test lane: maintainer
#[cfg(test)]
mod tests {
    use super::*;

    fn sample_request() -> ZellijRenderPlanRequest {
        ZellijRenderPlanRequest {
            left_sidebar_width_percent: 20,
            left_sidebar_command: "yzx".into(),
            left_sidebar_args: default_left_sidebar_args(),
            right_sidebar_width_percent: 40,
            right_sidebar_command: DEFAULT_RIGHT_SIDEBAR_COMMAND.into(),
            right_sidebar_args: default_right_sidebar_args(),
            popup_width_percent: 90,
            popup_height_percent: 90,
            screen_saver_enabled: false,
            screen_saver_idle_seconds: 300,
            screen_saver_style: "random".into(),
            zellij_widget_tray: None,
            zellij_custom_text: None,
            zellij_theme: "default".into(),
            zellij_pane_frames: "true".into(),
            zellij_rounded_corners: "true".into(),
            disable_zellij_tips: "true".into(),
            support_kitty_keyboard_protocol: "false".into(),
            zellij_default_mode: "normal".into(),
            zellij_tab_label_mode: "full".into(),
            zellij_claude_usage_display: "both".into(),
            zellij_codex_usage_display: "quota".into(),
            zellij_opencode_go_usage_display: "both".into(),
            zellij_claude_usage_periods: default_claude_usage_periods(),
            zellij_codex_usage_periods: default_codex_usage_periods(),
            zellij_opencode_go_usage_periods: default_opencode_go_usage_periods(),
            yazelix_layout_dir: "/tmp/yazelix/layouts".into(),
            resolved_default_shell: "/usr/bin/nu".into(),
            editor_label: "hx".into(),
            shell_label: "nu".into(),
            terminal_label: "wezterm".into(),
        }
    }

    // Defends: layout placeholder percents stay aligned with the historical Nushell geometry helper.
    #[test]
    fn layout_percentages_match_legacy_nushell() {
        let p = compute_layout_percentages(20, 40);
        assert_eq!(p.left_sidebar_width_percent, "20%");
        assert_eq!(p.right_sidebar_width_percent, "40%");
        assert_eq!(p.open_content_width_percent, "80%");
        assert_eq!(p.closed_content_width_percent, "99%");
        assert_eq!(p.left_open_right_open_content_width_percent, "40%");
        assert_eq!(p.left_open_right_closed_content_width_percent, "79%");
        assert_eq!(p.left_closed_right_open_content_width_percent, "59%");
        assert_eq!(p.left_closed_right_closed_content_width_percent, "98%");
    }

    // Defends: sidebar width contract bounds surface as structured config errors, not silent clamping.
    #[test]
    fn rejects_sidebar_out_of_range() {
        let mut req = sample_request();
        req.left_sidebar_width_percent = 0;
        assert!(compute_zellij_render_plan(&req).is_err());
    }

    // Defends: sidebar widths allow narrow launchers and wider side surfaces without clamping.
    #[test]
    fn accepts_sidebar_width_boundaries() {
        let mut req = sample_request();
        req.left_sidebar_width_percent = 1;
        req.right_sidebar_width_percent = 48;
        let plan = compute_zellij_render_plan(&req).expect("boundary widths");

        assert_eq!(plan.layout_percentages.left_sidebar_width_percent, "1%");
        assert_eq!(plan.layout_percentages.right_sidebar_width_percent, "48%");
        assert_eq!(
            plan.layout_percentages
                .left_open_right_open_content_width_percent,
            "51%"
        );
    }

    // Defends: custom side-surface launchers fail fast when the command is empty instead of generating unusable Zellij KDL.
    #[test]
    fn rejects_empty_sidebar_command() {
        let mut req = sample_request();
        req.left_sidebar_command = "   ".into();
        let error = compute_zellij_render_plan(&req).unwrap_err();

        assert_eq!(error.code(), "invalid_sidebar_command");
    }

    // Regression: custom sidebar apps must not inherit the default Yazi launcher args.
    #[test]
    fn custom_sidebar_command_drops_implicit_yazi_launcher_arg() {
        for command in ["lazygit", "opencode"] {
            let mut req = sample_request();
            req.left_sidebar_command = command.into();
            req.left_sidebar_args = default_left_sidebar_args();
            let plan = compute_zellij_render_plan(&req).unwrap();

            assert_eq!(plan.left_sidebar_command, command);
            assert!(plan.left_sidebar_args.is_empty());
        }
    }

    // Defends: explicit custom sidebar arguments still pass through after the implicit default launcher arg is removed.
    #[test]
    fn custom_sidebar_command_preserves_explicit_args() {
        let mut req = sample_request();
        req.left_sidebar_command = "lazygit".into();
        req.left_sidebar_args = vec!["status".into()];
        let plan = compute_zellij_render_plan(&req).unwrap();

        assert_eq!(plan.left_sidebar_args, vec!["status"]);
    }

    // Defends: widget tray entries are validated against the same allowed set as config.normalize.
    #[test]
    fn rejects_invalid_tray_widget() {
        let mut req = sample_request();
        req.zellij_widget_tray = Some(vec!["editor".into(), "nope".into()]);
        assert!(compute_zellij_render_plan(&req).is_err());
    }

    // Defends: supported dynamic widgets are accepted as optional extension points while Codex usage stays in the default tray.
    #[test]
    fn accepts_dynamic_tray_widgets_with_codex_defaulted() {
        let mut req = sample_request();
        req.zellij_widget_tray = Some(vec![
            "workspace".into(),
            "cursor".into(),
            "claude_usage".into(),
            "codex_usage".into(),
            "opencode_go_usage".into(),
        ]);
        let plan = compute_zellij_render_plan(&req).unwrap();

        assert_eq!(
            plan.widget_tray,
            vec![
                "workspace",
                "cursor",
                "claude_usage",
                "codex_usage",
                "opencode_go_usage"
            ]
        );
        assert_eq!(
            default_widget_tray(),
            vec![
                "editor",
                "shell",
                "term",
                "cursor",
                "codex_usage",
                "cpu",
                "ram"
            ]
        );
    }

    // Regression: Codex period selection is a generated status-bar contract, not a child command default hidden from settings.
    #[test]
    fn normalizes_agent_usage_periods() {
        let mut req = sample_request();
        req.zellij_codex_usage_periods = vec!["week".into(), "5h".into(), "week".into()];
        req.zellij_opencode_go_usage_periods = vec!["month".into(), "5h".into()];

        let plan = compute_zellij_render_plan(&req).unwrap();

        assert_eq!(plan.codex_usage_periods, vec!["week", "5h"]);
        assert_eq!(plan.claude_usage_periods, vec!["5h", "week"]);
        assert_eq!(plan.opencode_go_usage_periods, vec!["month", "5h"]);

        req.zellij_codex_usage_periods = vec!["month".into()];
        assert_eq!(
            compute_zellij_render_plan(&req).unwrap_err().code(),
            "invalid_zellij_usage_periods"
        );

        req.zellij_codex_usage_periods.clear();
        assert_eq!(
            compute_zellij_render_plan(&req).unwrap_err().code(),
            "invalid_zellij_usage_periods"
        );
    }

    // Defends: idle screen saver config is explicit and bounded before it reaches the pane-orchestrator plugin.
    #[test]
    fn screen_saver_config_is_normalized_and_bounded() {
        let mut req = sample_request();
        req.screen_saver_enabled = true;
        req.screen_saver_idle_seconds = 120;
        req.screen_saver_style = "Mandelbrot".into();

        let plan = compute_zellij_render_plan(&req).unwrap();

        assert!(plan.screen_saver_enabled);
        assert_eq!(plan.screen_saver_idle_seconds, 120);
        assert_eq!(plan.screen_saver_style, "mandelbrot");

        req.screen_saver_idle_seconds = 5;
        assert_eq!(
            compute_zellij_render_plan(&req).unwrap_err().code(),
            "invalid_screen_saver_idle_seconds"
        );

        req.screen_saver_idle_seconds = 120;
        req.screen_saver_style = "static".into();
        assert_eq!(
            compute_zellij_render_plan(&req).unwrap_err().code(),
            "invalid_screen_saver_style"
        );
    }

    // Defends: enforced default_layout points at the computed managed layout file for the active sidebar capability.
    #[test]
    fn enforced_default_layout_points_at_plan_layout() {
        let plan = compute_zellij_render_plan(&sample_request()).unwrap();
        let def = plan
            .enforced_top_level_settings
            .iter()
            .find(|s| s.name == "default_layout")
            .unwrap();
        assert!(def.value.contains("yzx_side.kdl"));
    }

    // Regression: omitted JSON fields use config-contract defaults so machine callers cannot drift from main_config_contract.toml.
    #[test]
    fn omitted_support_kitty_keyboard_protocol_defaults_to_contract_true() {
        let json = serde_json::json!({
            "yazelix_layout_dir": "/tmp/yazelix/layouts",
            "resolved_default_shell": "/bin/sh",
        });
        let req: ZellijRenderPlanRequest = serde_json::from_value(json).unwrap();
        assert_eq!(req.support_kitty_keyboard_protocol, "true");
        let plan = compute_zellij_render_plan(&req).unwrap();
        let kitty = plan
            .enforced_top_level_settings
            .iter()
            .find(|s| s.name == "support_kitty_keyboard_protocol")
            .unwrap();
        assert_eq!(kitty.value, "true");
    }

    // Regression: status widget labels must never be empty, even when config values are paths or omitted.
    #[test]
    fn status_widget_labels_use_basenames_and_defaults() {
        let mut req = sample_request();
        req.editor_label = "".into();
        req.shell_label = "/nix/store/example/bin/nu".into();
        req.resolved_default_shell = "/nix/store/example/bin/yazelix_nu.sh".into();
        req.terminal_label = "/opt/ghostty/bin/ghostty".into();
        let plan = compute_zellij_render_plan(&req).unwrap();

        assert_eq!(plan.editor_label, "hx");
        assert_eq!(plan.shell_label, "nu");
        assert_eq!(plan.terminal_label, "ghostty");
    }

    // Defends: compact tab labels are explicit and normalized before layout KDL rendering.
    #[test]
    fn normalizes_tab_label_mode() {
        let mut req = sample_request();
        req.zellij_tab_label_mode = "Compact".into();
        let plan = compute_zellij_render_plan(&req).unwrap();
        assert_eq!(plan.tab_label_mode, "compact");

        req.zellij_tab_label_mode = "tiny".into();
        assert_eq!(
            compute_zellij_render_plan(&req).unwrap_err().code(),
            "invalid_tab_label_mode"
        );
    }
}
