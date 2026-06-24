use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use serde_json::json;

pub const RENDERER_SCHEMA_VERSION: u64 = 2;
pub const MANAGED_SIDEBAR_LAYOUT_NAME: &str = "yzx_side";

const GENERATED_LAYOUT_MARKER: &str = "GENERATED ZELLIJ LAYOUT (YAZELIX)";
const GENERATED_LAYOUT_FINGERPRINT_PREFIX: &str = "generation_fingerprint:";
const ZJSTATUS_TAB_TEMPLATE_PLACEHOLDER: &str = "__YAZELIX_ZJSTATUS_TAB_TEMPLATE__";
const PANE_ORCHESTRATOR_PLUGIN_URL_PLACEHOLDER: &str = "__YAZELIX_PANE_ORCHESTRATOR_PLUGIN_URL__";
const HOME_DIR_PLACEHOLDER: &str = "__YAZELIX_HOME_DIR__";
const HOME_TAB_MARKER_PLACEHOLDER: &str = "__YAZELIX_HOME_TAB_MARKER__";
const RUNTIME_DIR_PLACEHOLDER: &str = "__YAZELIX_RUNTIME_DIR__";
const PANE_ORCHESTRATOR_PLUGIN_ALIAS: &str = "yazelix_pane_orchestrator";
const HOME_TAB_MARKER: &str = "\u{f015}";
const YZPP_PLUGIN_ALIAS: &str = "yzpp";
const BOTTOM_POPUP_COMMAND_KEY: &str = "bottom_popup";
const TOP_POPUP_COMMAND_KEY: &str = "top_popup";
const MENU_POPUP_COMMAND_KEY: &str = "menu";
const APPEARANCE_MODE_DARK: &str = "dark";
const APPEARANCE_MODE_LIGHT: &str = "light";
const ZELLIJ_THEME_LIGHT: &str = "catppuccin-latte";

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
    "session",
    "editor",
    "shell",
    "term",
    "workspace",
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
const WIDGET_FRAME_NONE: &str = "none";
const WIDGET_FRAME_SQUARE: &str = "square";
const WIDGET_FRAME_ROUND: &str = "round";
const WIDGET_FRAME_ALLOWED: &[&str] = &[WIDGET_FRAME_NONE, WIDGET_FRAME_SQUARE, WIDGET_FRAME_ROUND];
const WIDGET_SEPARATOR_DOT: &str = "dot";
const WIDGET_SEPARATOR_PIPE: &str = "pipe";
const WIDGET_SEPARATOR_EMPTY: &str = "empty";
const WIDGET_SEPARATOR_SPACE: &str = "space";
const WIDGET_SEPARATOR_ALLOWED: &[&str] = &[
    WIDGET_SEPARATOR_DOT,
    WIDGET_SEPARATOR_PIPE,
    WIDGET_SEPARATOR_EMPTY,
    WIDGET_SEPARATOR_SPACE,
];
const CLAUDE_CODEX_USAGE_PERIODS_ALLOWED: &[&str] = &["5h", "week"];
const OPENCODE_GO_USAGE_PERIODS_ALLOWED: &[&str] = &["5h", "week", "month"];
const STATUS_USAGE_PROVIDER_CLAUDE_ENABLED_KEY: &str = "status_usage_provider_claude_enabled";
const STATUS_USAGE_PROVIDER_CODEX_ENABLED_KEY: &str = "status_usage_provider_codex_enabled";
const STATUS_USAGE_PROVIDER_OPENCODE_GO_ENABLED_KEY: &str =
    "status_usage_provider_opencode_go_enabled";
pub const DEFAULT_LEFT_SIDEBAR_COMMAND: &str = "yzx";
pub const DEFAULT_LEFT_SIDEBAR_YAZI_ARGS: &[&str] = &["sidebar", "yazi"];
pub const DEFAULT_RIGHT_SIDEBAR_COMMAND: &str = "yzx";
pub const DEFAULT_RIGHT_SIDEBAR_AGENT_ARGS: &[&str] = &["agent"];

const REQUIRED_LAYOUT_PLACEHOLDERS: &[&str] = &[
    ZJSTATUS_TAB_TEMPLATE_PLACEHOLDER,
    PANE_ORCHESTRATOR_PLUGIN_URL_PLACEHOLDER,
    HOME_DIR_PLACEHOLDER,
    HOME_TAB_MARKER_PLACEHOLDER,
    RUNTIME_DIR_PLACEHOLDER,
    "__YAZELIX_SIDEBAR_COMMAND__",
    "__YAZELIX_SIDEBAR_ARGS__",
];

#[derive(Debug, Clone, PartialEq)]
pub struct ZellijRenderPlanError {
    code: &'static str,
    message: String,
    remediation: &'static str,
    details: serde_json::Value,
}

impl ZellijRenderPlanError {
    fn new(
        code: &'static str,
        message: impl Into<String>,
        remediation: &'static str,
        details: serde_json::Value,
    ) -> Self {
        Self {
            code,
            message: message.into(),
            remediation,
            details,
        }
    }

    pub fn code(&self) -> &'static str {
        self.code
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn remediation(&self) -> &'static str {
        self.remediation
    }

    pub fn details(&self) -> &serde_json::Value {
        &self.details
    }
}

impl fmt::Display for ZellijRenderPlanError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for ZellijRenderPlanError {}

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

fn string_vec(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| (*value).to_string()).collect()
}

fn default_left_sidebar_args() -> Vec<String> {
    string_vec(DEFAULT_LEFT_SIDEBAR_YAZI_ARGS)
}

fn default_right_sidebar_args() -> Vec<String> {
    string_vec(DEFAULT_RIGHT_SIDEBAR_AGENT_ARGS)
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

fn default_appearance_mode() -> String {
    APPEARANCE_MODE_DARK.into()
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

fn default_widget_frame() -> String {
    WIDGET_FRAME_NONE.into()
}

fn default_widget_separator() -> String {
    WIDGET_SEPARATOR_DOT.into()
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
    string_vec(CLAUDE_CODEX_USAGE_PERIODS_ALLOWED)
}

fn default_codex_usage_periods() -> Vec<String> {
    string_vec(CLAUDE_CODEX_USAGE_PERIODS_ALLOWED)
}

fn default_opencode_go_usage_periods() -> Vec<String> {
    string_vec(OPENCODE_GO_USAGE_PERIODS_ALLOWED)
}

fn default_widget_tray() -> Vec<String> {
    string_vec(&[
        "session",
        "editor",
        "shell",
        "term",
        "codex_usage",
        "cpu",
        "ram",
    ])
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
    #[serde(default = "default_widget_frame")]
    pub zellij_widget_frame: String,
    #[serde(default = "default_widget_separator")]
    pub zellij_widget_separator: String,
    #[serde(default)]
    pub zellij_custom_text: Option<String>,
    #[serde(default = "default_zellij_theme")]
    pub zellij_theme: String,
    #[serde(default = "default_appearance_mode")]
    pub appearance_mode: String,
    #[serde(default = "default_string_true")]
    pub zellij_pane_frames: String,
    #[serde(default = "default_string_true")]
    pub zellij_rounded_corners: String,
    #[serde(default = "default_string_true")]
    pub disable_zellij_tips: String,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZellijConfigPackRenderRequest {
    pub base_config_content: String,
    pub override_keybinds: Vec<String>,
    pub render_plan: ZellijRenderPlanData,
    pub popup_commands: BTreeMap<String, Vec<String>>,
    pub custom_popups: Vec<CustomPopup>,
    #[serde(default)]
    pub layout_templates: Option<Vec<ZellijConfigPackLayoutTemplate>>,
    #[serde(default)]
    pub static_fragments: Option<BTreeMap<String, String>>,
    pub zjstatus_plugin_block: String,
    pub pane_orchestrator_plugin_url: String,
    pub yzpp_plugin_url: String,
    pub home_dir: String,
    pub runtime_dir: String,
    pub generation_fingerprint: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZellijRenderPlanData {
    #[serde(default)]
    pub default_layout_name: String,
    #[serde(default)]
    pub appearance_mode: String,
    #[serde(default)]
    pub left_sidebar_width_percent: i64,
    #[serde(default)]
    pub right_sidebar_width_percent: i64,
    #[serde(default)]
    pub widget_tray: Vec<String>,
    #[serde(default = "default_widget_frame")]
    pub widget_frame: String,
    #[serde(default = "default_widget_separator")]
    pub widget_separator: String,
    #[serde(default)]
    pub editor_label: String,
    #[serde(default)]
    pub shell_label: String,
    #[serde(default)]
    pub terminal_label: String,
    #[serde(default)]
    pub tab_label_mode: String,
    #[serde(default)]
    pub claude_usage_display: String,
    #[serde(default)]
    pub codex_usage_display: String,
    #[serde(default)]
    pub opencode_go_usage_display: String,
    #[serde(default)]
    pub claude_usage_periods: Vec<String>,
    #[serde(default)]
    pub codex_usage_periods: Vec<String>,
    #[serde(default)]
    pub opencode_go_usage_periods: Vec<String>,
    #[serde(default)]
    pub custom_text: String,
    pub owned_top_level_setting_names: Vec<String>,
    pub dynamic_top_level_settings: Vec<TopLevelSetting>,
    pub enforced_top_level_settings: Vec<TopLevelSetting>,
    pub rounded_value: String,
    pub popup_width_percent: i64,
    pub popup_height_percent: i64,
    pub screen_saver_enabled: bool,
    pub screen_saver_idle_seconds: i64,
    pub screen_saver_style: String,
    pub right_sidebar_command: String,
    pub right_sidebar_args: Vec<String>,
    pub left_sidebar_command: String,
    pub left_sidebar_args: Vec<String>,
    pub layout_percentages: ZellijLayoutPercentages,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopLevelSetting {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZellijLayoutPercentages {
    pub left_sidebar_width_percent: String,
    pub right_sidebar_width_percent: String,
    pub open_content_width_percent: String,
    pub closed_content_width_percent: String,
    pub left_open_right_open_content_width_percent: String,
    pub left_open_right_closed_content_width_percent: String,
    pub left_closed_right_open_content_width_percent: String,
    pub left_closed_right_closed_content_width_percent: String,
}

fn bool_setting_from_string(raw: &str) -> bool {
    !raw.trim_start().starts_with("false")
}

fn compute_layout_percentages(
    left_sidebar_width_percent: i64,
    right_sidebar_width_percent: i64,
) -> ZellijLayoutPercentages {
    let open_content_width_percent = 100 - left_sidebar_width_percent;
    let closed_content_width_percent = 99;
    let left_open_right_open_content_width_percent =
        100 - left_sidebar_width_percent - right_sidebar_width_percent;
    let left_open_right_closed_content_width_percent = 99 - left_sidebar_width_percent;
    let left_closed_right_open_content_width_percent = 99 - right_sidebar_width_percent;
    let left_closed_right_closed_content_width_percent = 98;

    ZellijLayoutPercentages {
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

fn validate_sidebar_width(field: &str, value: i64) -> Result<(), ZellijRenderPlanError> {
    if (1..=48).contains(&value) {
        Ok(())
    } else {
        Err(ZellijRenderPlanError::new(
            "invalid_sidebar_width_percent",
            format!("{field} must be between 1 and 48 (got {value})"),
            "Set the sidebar width_percent within the documented range.",
            json!({ "field": field }),
        ))
    }
}

fn validate_sidebar_launcher(field: &str, command: &str) -> Result<(), ZellijRenderPlanError> {
    if !command.trim().is_empty() {
        Ok(())
    } else {
        Err(ZellijRenderPlanError::new(
            "invalid_sidebar_command",
            format!("{field} must not be empty"),
            "Set the sidebar command to a terminal command such as `yzx`, `nu`, `yazi`, `codex`, or another side-surface launcher.",
            json!({ "field": field }),
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

fn validate_popup_percent(field: &str, value: i64) -> Result<(), ZellijRenderPlanError> {
    if (1..=100).contains(&value) {
        Ok(())
    } else {
        Err(ZellijRenderPlanError::new(
            "invalid_popup_percent",
            format!("{field} must be between 1 and 100 (got {value})"),
            "Set the Zellij popup size percents within the documented range.",
            json!({ "field": field }),
        ))
    }
}

fn validate_screen_saver_idle_seconds(value: i64) -> Result<(), ZellijRenderPlanError> {
    if (10..=86400).contains(&value) {
        Ok(())
    } else {
        Err(ZellijRenderPlanError::new(
            "invalid_screen_saver_idle_seconds",
            format!("zellij.screen_saver_idle_seconds must be between 10 and 86400 (got {value})"),
            "Set zellij.screen_saver_idle_seconds to a supported idle threshold.",
            json!({ "field": "zellij.screen_saver_idle_seconds" }),
        ))
    }
}

fn normalize_screen_saver_style(style: &str) -> Result<String, ZellijRenderPlanError> {
    let normalized = style.trim().to_ascii_lowercase();
    if SCREEN_SAVER_STYLE_ALLOWED.contains(&normalized.as_str()) {
        Ok(normalized)
    } else {
        Err(ZellijRenderPlanError::new(
            "invalid_screen_saver_style",
            format!(
                "Invalid zellij.screen_saver_style `{normalized}`. Expected one of: {}",
                SCREEN_SAVER_STYLE_ALLOWED.join(", ")
            ),
            "Use one of the animated styles accepted by `yzx screen`.",
            json!({ "field": "zellij.screen_saver_style", "style": normalized }),
        ))
    }
}

fn validate_widget_tray(entries: &[String]) -> Result<(), ZellijRenderPlanError> {
    for entry in entries {
        if !WIDGET_TRAY_ALLOWED.contains(&entry.as_str()) {
            let allowed = WIDGET_TRAY_ALLOWED.join(", ");
            return Err(ZellijRenderPlanError::new(
                "invalid_widget_tray_entry",
                format!("Invalid zellij.widget_tray entry: {entry} (allowed: {allowed})"),
                "Use only documented widget tray identifiers.",
                json!({ "field": "zellij.widget_tray", "entry": entry }),
            ));
        }
    }
    Ok(())
}

fn validate_default_mode(mode: &str) -> Result<(), ZellijRenderPlanError> {
    if mode == "normal" || mode == "locked" {
        Ok(())
    } else {
        Err(ZellijRenderPlanError::new(
            "invalid_zellij_default_mode",
            format!("zellij_default_mode must be \"normal\" or \"locked\" (got {mode:?})"),
            "Set zellij.default_mode to a supported value.",
            json!({ "field": "zellij.default_mode" }),
        ))
    }
}

fn normalize_usage_display(
    field: &str,
    raw: &str,
    default: &str,
) -> Result<String, ZellijRenderPlanError> {
    let trimmed = raw.trim();
    let value = if trimmed.is_empty() { default } else { trimmed };
    match value {
        "both" => Ok("both".to_string()),
        "token" | "tokens" => Ok("token".to_string()),
        "quota" => Ok("quota".to_string()),
        _ => Err(ZellijRenderPlanError::new(
            "invalid_zellij_usage_display",
            format!("{field} must be one of: both, token, quota (got {raw})"),
            "Set the usage widget display mode to both, token, or quota.",
            json!({ "field": field, "value": raw }),
        )),
    }
}

fn normalize_usage_periods(
    field: &str,
    raw: &[String],
    allowed: &[&str],
) -> Result<Vec<String>, ZellijRenderPlanError> {
    if raw.is_empty() {
        return Err(ZellijRenderPlanError::new(
            "invalid_zellij_usage_periods",
            format!("{field} must include at least one period"),
            "Select at least one supported usage period, or remove the usage widget from zellij.widget_tray.",
            json!({ "field": field }),
        ));
    }

    let mut periods = Vec::new();
    for value in raw {
        let normalized = value.trim().to_ascii_lowercase();
        if !allowed.contains(&normalized.as_str()) {
            return Err(ZellijRenderPlanError::new(
                "invalid_zellij_usage_periods",
                format!(
                    "{field} contains unsupported period `{}`. Expected one of: {}",
                    value,
                    allowed.join(", ")
                ),
                "Use only supported usage period identifiers.",
                json!({ "field": field, "value": value }),
            ));
        }
        if !periods.contains(&normalized) {
            periods.push(normalized);
        }
    }
    Ok(periods)
}

macro_rules! allowed_value_normalizer {
    ($fn_name:ident, $allowed:ident, $code:literal, $field:literal, $detail_key:literal, $remediation:literal) => {
        fn $fn_name(value: &str) -> Result<String, ZellijRenderPlanError> {
            let normalized = value.trim().to_ascii_lowercase();
            if $allowed.contains(&normalized.as_str()) {
                Ok(normalized)
            } else {
                Err(ZellijRenderPlanError::new(
                    $code,
                    format!(
                        "Invalid {} `{normalized}`. Expected one of: {}",
                        $field,
                        $allowed.join(", ")
                    ),
                    $remediation,
                    json!({ "field": $field, $detail_key: normalized }),
                ))
            }
        }
    };
}

allowed_value_normalizer!(
    normalize_tab_label_mode,
    TAB_LABEL_MODE_ALLOWED,
    "invalid_tab_label_mode",
    "zellij.tab_label_mode",
    "mode",
    "Set zellij.tab_label_mode to `full` or `compact`."
);
allowed_value_normalizer!(
    normalize_widget_frame,
    WIDGET_FRAME_ALLOWED,
    "invalid_widget_frame",
    "zellij.widget_frame",
    "frame",
    "Set zellij.widget_frame to `none`, `square`, or `round`."
);
allowed_value_normalizer!(
    normalize_widget_separator,
    WIDGET_SEPARATOR_ALLOWED,
    "invalid_widget_separator",
    "zellij.widget_separator",
    "separator",
    "Set zellij.widget_separator to `dot`, `pipe`, `empty`, or `space`."
);

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

fn appearance_default_theme(configured_theme: &str, light_theme: &str, mode: &str) -> String {
    if mode == APPEARANCE_MODE_LIGHT && configured_theme == "default" {
        light_theme.to_string()
    } else {
        configured_theme.to_string()
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
) -> Result<ZellijRenderPlanData, ZellijRenderPlanError> {
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
    let widget_frame = normalize_widget_frame(&request.zellij_widget_frame)?;
    let widget_separator = normalize_widget_separator(&request.zellij_widget_separator)?;
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
    let theme_config = appearance_default_theme(
        &request.zellij_theme,
        ZELLIJ_THEME_LIGHT,
        &request.appearance_mode,
    );
    let theme = pick_theme(&theme_config);
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
    let owned_top_level_setting_names = dynamic_top_level_settings
        .iter()
        .chain(enforced_top_level_settings.iter())
        .map(|setting| setting.name.clone())
        .collect();

    Ok(ZellijRenderPlanData {
        default_layout_name,
        appearance_mode: request.appearance_mode.trim().to_ascii_lowercase(),
        left_sidebar_width_percent: request.left_sidebar_width_percent,
        right_sidebar_width_percent: request.right_sidebar_width_percent,
        widget_tray,
        widget_frame,
        widget_separator,
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
        owned_top_level_setting_names,
        dynamic_top_level_settings,
        enforced_top_level_settings,
        rounded_value: rounded_value.to_string(),
        popup_width_percent: request.popup_width_percent,
        popup_height_percent: request.popup_height_percent,
        screen_saver_enabled: request.screen_saver_enabled,
        screen_saver_idle_seconds: request.screen_saver_idle_seconds,
        screen_saver_style,
        right_sidebar_command: request.right_sidebar_command.trim().to_string(),
        right_sidebar_args: request.right_sidebar_args.clone(),
        left_sidebar_command: request.left_sidebar_command.trim().to_string(),
        left_sidebar_args: effective_left_sidebar_args(
            &request.left_sidebar_command,
            &request.left_sidebar_args,
        ),
        layout_percentages,
    })
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CustomPopup {
    pub id: String,
    pub command: Vec<String>,
    #[serde(default)]
    pub keybindings: Vec<String>,
    pub keep_alive: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ZellijKeybindRenderRequest {
    pub override_template_content: String,
    pub runtime_dir: String,
    pub home_dir: String,
    pub zellij_keybindings: BTreeMap<String, Vec<String>>,
    pub zellij_native_keybindings: BTreeMap<String, Vec<String>>,
    pub custom_popups: Vec<CustomPopup>,
    pub integration_actions: Vec<ZellijKeybindActionSpec>,
    pub native_actions: Vec<ZellijNativeKeybindSpec>,
    pub pane_orchestrator_plugin_alias: String,
    pub popup_plugin_alias: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ZellijKeybindActionSpec {
    pub local_id: String,
    pub mode: String,
    pub plugin_alias: String,
    pub message_name: String,
    #[serde(default)]
    pub payload: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ZellijNativeKeybindSpec {
    pub local_id: String,
    pub blocks: Vec<ZellijNativeKeybindBlockSpec>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ZellijNativeKeybindBlockSpec {
    pub mode: String,
    pub action_lines: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZellijKeybindRenderOutput {
    pub override_keybinds: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZellijConfigPackLayoutTemplate {
    pub relative_path: String,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZellijConfigPackRenderedFile {
    pub relative_path: String,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZellijConfigPackRenderOutput {
    pub renderer_schema_version: u64,
    pub merged_config: String,
    pub layout_files: Vec<ZellijConfigPackRenderedFile>,
    pub generation_fingerprint: String,
}

pub fn render_zellij_config_pack(
    request: &ZellijConfigPackRenderRequest,
) -> Result<ZellijConfigPackRenderOutput, String> {
    Ok(ZellijConfigPackRenderOutput {
        renderer_schema_version: RENDERER_SCHEMA_VERSION,
        merged_config: render_merged_config(request),
        layout_files: render_config_pack_layouts(request)?,
        generation_fingerprint: request.generation_fingerprint.clone(),
    })
}

pub fn render_zellij_keybinds(request: &ZellijKeybindRenderRequest) -> ZellijKeybindRenderOutput {
    let content = request
        .override_template_content
        .replace(
            PANE_ORCHESTRATOR_PLUGIN_URL_PLACEHOLDER,
            &request.pane_orchestrator_plugin_alias,
        )
        .replace(RUNTIME_DIR_PLACEHOLDER, &request.runtime_dir);
    let assigned_keys = assigned_generated_zellij_binding_keys(request);
    let mut override_keybinds = extract_semantic_config_blocks(&content)
        .keybind_lines
        .into_iter()
        .filter(|line| !unbind_line_conflicts_with_generated_key(line, &assigned_keys))
        .collect::<Vec<_>>();
    override_keybinds.extend(
        build_native_zellij_keybind_lines(request)
            .into_iter()
            .filter(|line| !unbind_line_conflicts_with_generated_key(line, &assigned_keys)),
    );
    override_keybinds.extend(build_yazelix_default_new_tab_keybind_lines(request));
    override_keybinds.extend(build_zellij_integration_keybind_lines(request));
    ZellijKeybindRenderOutput { override_keybinds }
}

pub fn bundled_layout_templates() -> Vec<ZellijConfigPackLayoutTemplate> {
    vec![
        ZellijConfigPackLayoutTemplate {
            relative_path: "yzx_side.kdl".to_string(),
            content: include_str!("../layouts/yzx_side.kdl").to_string(),
        },
        ZellijConfigPackLayoutTemplate {
            relative_path: "yzx_side.swap.kdl".to_string(),
            content: include_str!("../layouts/yzx_side.swap.kdl").to_string(),
        },
    ]
}

pub fn bundled_static_fragments() -> BTreeMap<String, String> {
    [
        (
            "__YAZELIX_SWAP_SIDEBAR_OPEN__",
            include_str!("../layouts/fragments/swap_sidebar_open.kdl"),
        ),
        (
            "__YAZELIX_SWAP_SIDEBAR_CLOSED__",
            include_str!("../layouts/fragments/swap_sidebar_closed.kdl"),
        ),
        (
            "__YAZELIX_SWAP_AGENT_OPEN__",
            include_str!("../layouts/fragments/swap_agent_open.kdl"),
        ),
        (
            "__YAZELIX_SWAP_AGENT_CLOSED__",
            include_str!("../layouts/fragments/swap_agent_closed.kdl"),
        ),
    ]
    .into_iter()
    .map(|(placeholder, content)| (placeholder.to_string(), content.to_string()))
    .collect()
}

fn render_config_pack_layouts(
    request: &ZellijConfigPackRenderRequest,
) -> Result<Vec<ZellijConfigPackRenderedFile>, String> {
    let bundled_templates;
    let templates = if let Some(templates) = &request.layout_templates {
        templates
    } else {
        bundled_templates = bundled_layout_templates();
        &bundled_templates
    };
    let bundled_fragments;
    let fragments = if let Some(fragments) = &request.static_fragments {
        fragments
    } else {
        bundled_fragments = bundled_static_fragments();
        &bundled_fragments
    };

    templates
        .iter()
        .map(|template| {
            let rendered = render_layout_template(
                &template.content,
                fragments,
                &request.zjstatus_plugin_block,
                &request.pane_orchestrator_plugin_url,
                &request.home_dir,
                &request.runtime_dir,
                &request.render_plan,
            )?;
            Ok(ZellijConfigPackRenderedFile {
                relative_path: template.relative_path.clone(),
                content: format!(
                    "{}{}",
                    generated_zellij_layout_header(&request.generation_fingerprint),
                    rendered
                ),
            })
        })
        .collect()
}

fn render_merged_config(request: &ZellijConfigPackRenderRequest) -> String {
    let extracted_blocks = extract_semantic_config_blocks(&request.base_config_content);
    let base_config = strip_owned_top_level_settings(
        &extracted_blocks.config_without_semantic_blocks,
        &request.render_plan.owned_top_level_setting_names,
    );
    let merged_keybinds =
        build_merged_keybinds_block(&extracted_blocks.keybind_lines, &request.override_keybinds);
    let merged_ui = build_yazelix_ui_block(
        &extracted_blocks.ui_lines,
        &request.render_plan.rounded_value,
    );
    let plugins_block = build_yazelix_plugins_block(
        &extracted_blocks.plugin_lines,
        request,
        &request.render_plan,
    );
    let load_plugins_block = build_yazelix_load_plugins_block(
        &extracted_blocks.load_plugin_lines,
        request,
        &request.render_plan,
    );

    [
        "// ========================================".to_string(),
        "// GENERATED ZELLIJ CONFIG (YAZELIX)".to_string(),
        "// ========================================".to_string(),
        "// Source preference:".to_string(),
        "//   1) ~/.config/yazelix/zellij.kdl (Yazelix-managed override)".to_string(),
        "//   2) ~/.config/zellij/config.kdl (native fallback, read-only)".to_string(),
        "//   3) zellij setup --dump-config (defaults)".to_string(),
        "//".to_string(),
        "// Generated: 1970-01-01 00:00:00".to_string(),
        "// ========================================".to_string(),
        String::new(),
        base_config,
        String::new(),
        merged_keybinds,
        String::new(),
        plugins_block,
        String::new(),
        merged_ui,
        String::new(),
        render_top_level_settings_block(
            "// === YAZELIX DYNAMIC SETTINGS (from settings.jsonc) ===",
            &request.render_plan.dynamic_top_level_settings,
        ),
        String::new(),
        render_top_level_settings_block(
            "// === YAZELIX ENFORCED SETTINGS ===",
            &request.render_plan.enforced_top_level_settings,
        ),
        String::new(),
        "// === YAZELIX BACKGROUND PLUGINS ===".to_string(),
        load_plugins_block,
    ]
    .join("\n")
}

fn build_yazelix_plugins_block(
    existing_lines: &[String],
    request: &ZellijConfigPackRenderRequest,
    render_plan: &ZellijRenderPlanData,
) -> String {
    let mut merged_lines = existing_lines.to_vec();
    let orchestrator_present = merged_lines
        .iter()
        .any(|line| line.contains(&format!("{PANE_ORCHESTRATOR_PLUGIN_ALIAS} location=")));
    if !orchestrator_present {
        merged_lines.extend([format!(
            "    {PANE_ORCHESTRATOR_PLUGIN_ALIAS} location=\"{}\" {{",
            request.pane_orchestrator_plugin_url
        )]);
        merged_lines.extend(render_pane_orchestrator_config_lines(
            request,
            render_plan,
            "        ",
        ));
        merged_lines.push("    }".to_string());
    }

    let yzpp_present = merged_lines
        .iter()
        .any(|line| line.contains(&format!("{YZPP_PLUGIN_ALIAS} location=")));
    if !yzpp_present {
        merged_lines.extend(render_yzpp_plugin_block(request, render_plan));
    }

    if merged_lines.is_empty() {
        String::new()
    } else {
        block_with_lines("plugins", &merged_lines)
    }
}

fn render_yzpp_plugin_block(
    request: &ZellijConfigPackRenderRequest,
    render_plan: &ZellijRenderPlanData,
) -> Vec<String> {
    let mut lines = vec![format!(
        "    {YZPP_PLUGIN_ALIAS} location=\"{}\" {{",
        request.yzpp_plugin_url
    )];
    lines.extend(render_yzpp_config_lines(request, render_plan));
    lines.push("    }".to_string());
    lines
}

fn render_yzpp_config_lines(
    request: &ZellijConfigPackRenderRequest,
    render_plan: &ZellijRenderPlanData,
) -> Vec<String> {
    let yzx_cli = format!("{}/shells/posix/yzx_cli.sh", request.runtime_dir);
    let bottom_popup_program =
        generated_popup_command(&request.popup_commands, BOTTOM_POPUP_COMMAND_KEY, &yzx_cli);
    let top_popup_program =
        generated_popup_command(&request.popup_commands, TOP_POPUP_COMMAND_KEY, &yzx_cli);
    let menu_program =
        generated_popup_command(&request.popup_commands, MENU_POPUP_COMMAND_KEY, &yzx_cli);
    let mut lines = vec!["        popups {".to_string()];

    append_generated_popup_spec(
        &mut lines,
        "bottom_popup",
        "yzx_bottom_popup",
        Some("yzx_bottom_popup"),
        &bottom_popup_program,
        render_plan.popup_width_percent,
        render_plan.popup_height_percent,
        None,
        Some(&yzx_cli),
    );
    append_generated_popup_spec(
        &mut lines,
        "top_popup",
        "yzx_top_popup",
        Some("yzx_top_popup"),
        &top_popup_program,
        render_plan.popup_width_percent,
        render_plan.popup_height_percent,
        None,
        None,
    );
    append_generated_popup_spec(
        &mut lines,
        "menu",
        "yzx_menu",
        Some("yzx menu"),
        &menu_program,
        render_plan.popup_width_percent,
        render_plan.popup_height_percent,
        None,
        None,
    );
    for custom_popup in &request.custom_popups {
        let custom_popup_program =
            popup_command_argv_for_yazelix_runtime(&custom_popup.command, &yzx_cli);
        let pane_title = format!("yzx_{}", custom_popup.id);
        append_generated_popup_spec(
            &mut lines,
            &custom_popup.id,
            &pane_title,
            Some(&pane_title),
            &custom_popup_program,
            render_plan.popup_width_percent,
            render_plan.popup_height_percent,
            custom_popup.keep_alive.then_some("hide"),
            None,
        );
    }
    lines.extend([
        "            config {".to_string(),
        format!("                command {}", json_quote(&yzx_cli)),
        "                arg_1 \"config\"".to_string(),
        "                arg_2 \"ui\"".to_string(),
        "                pane_title \"yzx_config\"".to_string(),
        "                command_marker \"yzx config ui\"".to_string(),
        format!(
            "                width_percent \"{}\"",
            render_plan.popup_width_percent
        ),
        format!(
            "                height_percent \"{}\"",
            render_plan.popup_height_percent
        ),
        "            }".to_string(),
        "        }".to_string(),
    ]);
    lines
}

fn generated_popup_command(
    popup_commands: &BTreeMap<String, Vec<String>>,
    key: &str,
    yzx_cli: &str,
) -> Vec<String> {
    popup_commands
        .get(key)
        .map(|command| popup_command_argv_for_yazelix_runtime(command, yzx_cli))
        .unwrap_or_default()
}

fn popup_command_argv_for_yazelix_runtime(command: &[String], yzx_cli: &str) -> Vec<String> {
    let Some(command_path) = command.first() else {
        return Vec::new();
    };
    if command_path == yzx_cli {
        return command.to_vec();
    }
    if command_path == "yzx" {
        return std::iter::once(yzx_cli.to_string())
            .chain(command.iter().skip(1).cloned())
            .collect();
    }
    std::iter::once(yzx_cli.to_string())
        .chain(std::iter::once("popup_run".to_string()))
        .chain(command.iter().cloned())
        .collect()
}

fn append_generated_popup_spec(
    lines: &mut Vec<String>,
    id: &str,
    pane_title: &str,
    command_marker: Option<&str>,
    popup_argv: &[String],
    popup_width_percent: i64,
    popup_height_percent: i64,
    toggle_close_behavior: Option<&str>,
    on_close_yzx_cli: Option<&str>,
) {
    lines.push(format!("            {id} {{"));
    if let Some(command_path) = popup_argv.first() {
        lines.push(format!(
            "                command {}",
            json_quote(command_path)
        ));
        for (index, arg) in popup_argv.iter().skip(1).enumerate() {
            lines.push(format!(
                "                arg_{} {}",
                index + 1,
                json_quote(arg)
            ));
        }
        let marker = command_marker.unwrap_or(command_path);
        lines.push(format!(
            "                command_marker {}",
            json_quote(marker)
        ));
    }
    lines.extend([
        format!("                pane_title {}", json_quote(pane_title)),
        format!("                width_percent \"{popup_width_percent}\""),
        format!("                height_percent \"{popup_height_percent}\""),
    ]);
    if let Some(toggle_close_behavior) = toggle_close_behavior {
        lines.push(format!(
            "                toggle_close_behavior {}",
            json_quote(toggle_close_behavior)
        ));
    }
    if let Some(yzx_cli) = on_close_yzx_cli {
        lines.extend([
            "                on_close {".to_string(),
            format!("                    command {}", json_quote(yzx_cli)),
            "                    arg_1 \"sidebar\"".to_string(),
            "                    arg_2 \"refresh\"".to_string(),
            "                }".to_string(),
        ]);
    }
    lines.push("            }".to_string());
}

fn build_yazelix_load_plugins_block(
    existing_lines: &[String],
    request: &ZellijConfigPackRenderRequest,
    render_plan: &ZellijRenderPlanData,
) -> String {
    let mut seen = BTreeSet::new();
    let mut merged_lines = Vec::new();
    for line in existing_lines {
        let trimmed = line.trim();
        if !trimmed.is_empty() && seen.insert(trimmed.to_string()) {
            merged_lines.push(line.clone());
        }
    }
    let orchestrator_present = merged_lines.iter().any(|line| {
        let trimmed = line.trim();
        trimmed == PANE_ORCHESTRATOR_PLUGIN_ALIAS
            || trimmed.starts_with(&format!("{PANE_ORCHESTRATOR_PLUGIN_ALIAS} "))
    });
    if !orchestrator_present {
        merged_lines.push(format!("    {PANE_ORCHESTRATOR_PLUGIN_ALIAS} {{"));
        merged_lines.extend(render_pane_orchestrator_config_lines(
            request,
            render_plan,
            "        ",
        ));
        merged_lines.push("    }".to_string());
    }
    let yzpp_present = merged_lines.iter().any(|line| {
        let trimmed = line.trim();
        trimmed == YZPP_PLUGIN_ALIAS || trimmed.starts_with(&format!("{YZPP_PLUGIN_ALIAS} "))
    });
    if !yzpp_present {
        merged_lines.push(format!("    {YZPP_PLUGIN_ALIAS} {{"));
        merged_lines.extend(render_yzpp_config_lines(request, render_plan));
        merged_lines.push("    }".to_string());
    }
    if merged_lines.is_empty() {
        String::new()
    } else {
        block_with_lines("load_plugins", &merged_lines)
    }
}

fn render_pane_orchestrator_config_lines(
    request: &ZellijConfigPackRenderRequest,
    render_plan: &ZellijRenderPlanData,
    indent: &str,
) -> Vec<String> {
    let mut lines = vec![
        format!("{indent}runtime_dir {}", json_quote(&request.runtime_dir)),
        format!(
            "{indent}screen_saver_enabled \"{}\"",
            render_plan.screen_saver_enabled
        ),
        format!(
            "{indent}screen_saver_idle_seconds \"{}\"",
            render_plan.screen_saver_idle_seconds
        ),
        format!(
            "{indent}screen_saver_style {}",
            json_quote(&render_plan.screen_saver_style)
        ),
        format!(
            "{indent}runtime_config_generation {}",
            json_quote(&request.generation_fingerprint)
        ),
        status_usage_provider_config_line(
            indent,
            STATUS_USAGE_PROVIDER_CLAUDE_ENABLED_KEY,
            widget_tray_has(&render_plan.widget_tray, "claude_usage"),
        ),
        status_usage_provider_config_line(
            indent,
            STATUS_USAGE_PROVIDER_CODEX_ENABLED_KEY,
            widget_tray_has(&render_plan.widget_tray, "codex_usage"),
        ),
        status_usage_provider_config_line(
            indent,
            STATUS_USAGE_PROVIDER_OPENCODE_GO_ENABLED_KEY,
            widget_tray_has(&render_plan.widget_tray, "opencode_go_usage"),
        ),
        format!(
            "{indent}right_sidebar_command {}",
            json_quote(expand_runtime_placeholder(
                &render_plan.right_sidebar_command,
                &request.runtime_dir,
            ))
        ),
    ];
    for (index, arg) in render_plan.right_sidebar_args.iter().enumerate() {
        lines.push(format!(
            "{indent}right_sidebar_arg_{} {}",
            index + 1,
            json_quote(expand_runtime_placeholder(arg, &request.runtime_dir))
        ));
    }
    lines
}

fn status_usage_provider_config_line(indent: &str, key: &str, enabled: bool) -> String {
    format!("{indent}{key} \"{enabled}\"")
}

fn widget_tray_has(widget_tray: &[String], expected: &str) -> bool {
    widget_tray.iter().any(|widget| widget == expected)
}

fn build_merged_keybinds_block(existing_lines: &[String], override_lines: &[String]) -> String {
    let mut merged = existing_lines.to_vec();
    merged.extend_from_slice(override_lines);
    if merged.is_empty() {
        String::new()
    } else {
        block_with_lines("keybinds", &merged)
    }
}

fn assigned_generated_zellij_binding_keys(
    request: &ZellijKeybindRenderRequest,
) -> BTreeSet<String> {
    let mut assigned = request
        .zellij_keybindings
        .values()
        .flatten()
        .cloned()
        .collect::<BTreeSet<_>>();
    assigned.extend(
        request
            .custom_popups
            .iter()
            .flat_map(|popup| popup.keybindings.iter().cloned()),
    );
    for spec in &request.native_actions {
        if !spec
            .blocks
            .iter()
            .any(|block| !block.action_lines.is_empty())
        {
            continue;
        }
        if let Some(keys) = request.zellij_native_keybindings.get(&spec.local_id) {
            assigned.extend(keys.iter().cloned());
        }
    }
    assigned
}

fn unbind_line_conflicts_with_generated_key(line: &str, assigned_keys: &BTreeSet<String>) -> bool {
    let trimmed = line.trim();
    trimmed.starts_with("unbind ")
        && quoted_kdl_strings(trimmed)
            .iter()
            .any(|key| assigned_keys.contains(key))
}

fn quoted_kdl_strings(line: &str) -> Vec<String> {
    let mut strings = Vec::new();
    let mut chars = line.chars().peekable();
    while let Some(character) = chars.next() {
        if character != '"' {
            continue;
        }
        let mut value = String::new();
        let mut escaped = false;
        for next in chars.by_ref() {
            if escaped {
                value.push(next);
                escaped = false;
            } else if next == '\\' {
                escaped = true;
            } else if next == '"' {
                break;
            } else {
                value.push(next);
            }
        }
        strings.push(value);
    }
    strings
}

fn build_zellij_integration_keybind_lines(request: &ZellijKeybindRenderRequest) -> Vec<String> {
    let mut by_mode = BTreeMap::<&str, Vec<String>>::new();
    for spec in &request.integration_actions {
        let Some(keys) = request.zellij_keybindings.get(&spec.local_id) else {
            continue;
        };
        if keys.is_empty() {
            continue;
        }
        let mode_lines = by_mode.entry(&spec.mode).or_default();
        push_zellij_message_bind(
            mode_lines,
            keys,
            &spec.plugin_alias,
            &spec.message_name,
            spec.payload.as_deref(),
        );
    }
    for popup in &request.custom_popups {
        if popup.keybindings.is_empty() {
            continue;
        }
        let mode_lines = by_mode.entry("shared").or_default();
        push_zellij_message_bind(
            mode_lines,
            &popup.keybindings,
            &request.popup_plugin_alias,
            "toggle",
            Some(&popup.id),
        );
    }

    let mut lines = Vec::new();
    for (mode, binds) in by_mode {
        if !lines.is_empty() {
            lines.push(String::new());
        }
        lines.push(format!("    {mode} {{"));
        lines.extend(binds);
        lines.push("    }".to_string());
    }
    lines
}

fn build_native_zellij_keybind_lines(request: &ZellijKeybindRenderRequest) -> Vec<String> {
    let mut blocks = Vec::<(&str, Vec<String>)>::new();
    for spec in &request.native_actions {
        let Some(keys) = request.zellij_native_keybindings.get(&spec.local_id) else {
            continue;
        };
        if keys.is_empty() {
            continue;
        }
        for block in &spec.blocks {
            push_native_zellij_block_lines(&mut blocks, keys, block);
        }
    }

    let mut lines = Vec::new();
    for (mode, block_lines) in blocks {
        if !lines.is_empty() {
            lines.push(String::new());
        }
        lines.push(format!("    {mode} {{"));
        lines.extend(block_lines);
        lines.push("    }".to_string());
    }
    lines
}

fn build_yazelix_default_new_tab_keybind_lines(
    request: &ZellijKeybindRenderRequest,
) -> Vec<String> {
    vec![
        "    tab {".to_string(),
        format!(
            "        bind \"n\" {{ NewTab {{ cwd {}; name {}; }}; SwitchToMode \"Normal\"; }}",
            json_quote(&request.home_dir),
            json_quote(HOME_TAB_MARKER),
        ),
        "    }".to_string(),
    ]
}

fn push_native_zellij_block_lines<'a>(
    blocks: &mut Vec<(&'a str, Vec<String>)>,
    keys: &[String],
    block: &'a ZellijNativeKeybindBlockSpec,
) {
    let key_list = keys.iter().map(json_quote).collect::<Vec<_>>().join(" ");
    let block_lines = blocks
        .iter_mut()
        .find(|(mode, _)| *mode == block.mode)
        .map(|(_, lines)| lines);
    let lines = if let Some(lines) = block_lines {
        lines
    } else {
        blocks.push((&block.mode, Vec::new()));
        &mut blocks.last_mut().expect("just pushed").1
    };
    if block.action_lines.is_empty() {
        lines.push(format!("        unbind {key_list}"));
    } else if block.action_lines.iter().any(|line| line.contains('\n')) {
        lines.push(format!("        bind {key_list} {{"));
        for action_line in &block.action_lines {
            for line in action_line.lines() {
                lines.push(format!("            {line}"));
            }
        }
        lines.push("        }".to_string());
    } else {
        lines.push(format!(
            "        bind {key_list} {{ {}; }}",
            block.action_lines.join("; ")
        ));
    }
}

fn push_zellij_message_bind(
    lines: &mut Vec<String>,
    keys: &[String],
    plugin_alias: &str,
    message_name: &str,
    payload: Option<&str>,
) {
    let key_list = keys.iter().map(json_quote).collect::<Vec<_>>().join(" ");
    lines.push(format!("        bind {key_list} {{"));
    lines.push(format!(
        "            MessagePlugin {} {{",
        json_quote(plugin_alias)
    ));
    lines.push(format!("                name {}", json_quote(message_name)));
    if let Some(payload) = payload {
        lines.push(format!("                payload {}", json_quote(payload)));
    }
    lines.push("            }".to_string());
    lines.push("        }".to_string());
}

fn build_yazelix_ui_block(existing_ui_lines: &[String], rounded_value: &str) -> String {
    let existing_ui_text = existing_ui_lines.join("\n");
    let hide_session_name = existing_ui_text.contains("hide_session_name true");
    let mut lines = vec![
        "ui {".to_string(),
        "    pane_frames {".to_string(),
        format!("        rounded_corners {rounded_value}"),
    ];
    if hide_session_name {
        lines.push("        hide_session_name true".to_string());
    }
    lines.extend(["    }".to_string(), "}".to_string()]);
    lines.join("\n")
}

fn render_top_level_settings_block(header: &str, settings: &[TopLevelSetting]) -> String {
    std::iter::once(header.to_string())
        .chain(
            settings
                .iter()
                .map(|setting| format!("{} {}", setting.name, setting.value)),
        )
        .collect::<Vec<_>>()
        .join("\n")
}

fn strip_owned_top_level_settings(content: &str, owned_setting_names: &[String]) -> String {
    content
        .lines()
        .filter(|line| {
            let trimmed = line.trim();
            !owned_setting_names
                .iter()
                .any(|name| trimmed.starts_with(&format!("{name} ")))
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn render_layout_template(
    content: &str,
    static_fragments: &BTreeMap<String, String>,
    zjstatus_plugin_block: &str,
    pane_orchestrator_plugin_url: &str,
    home_dir: &str,
    runtime_dir: &str,
    render_plan: &ZellijRenderPlanData,
) -> Result<String, String> {
    let mut updated = apply_static_fragments(content, static_fragments);
    let replacements = [
        (
            ZJSTATUS_TAB_TEMPLATE_PLACEHOLDER,
            zjstatus_plugin_block.to_string(),
        ),
        (
            PANE_ORCHESTRATOR_PLUGIN_URL_PLACEHOLDER,
            pane_orchestrator_plugin_url.to_string(),
        ),
        (HOME_DIR_PLACEHOLDER, home_dir.to_string()),
        (HOME_TAB_MARKER_PLACEHOLDER, json_quote(HOME_TAB_MARKER)),
        (RUNTIME_DIR_PLACEHOLDER, runtime_dir.to_string()),
        (
            "__YAZELIX_SIDEBAR_COMMAND__",
            json_quote(expand_runtime_placeholder(
                &render_plan.left_sidebar_command,
                runtime_dir,
            )),
        ),
        (
            "__YAZELIX_SIDEBAR_ARGS__",
            render_sidebar_args(&render_plan.left_sidebar_args, runtime_dir),
        ),
        (
            "__YAZELIX_SIDEBAR_WIDTH_PERCENT__",
            render_plan
                .layout_percentages
                .left_sidebar_width_percent
                .clone(),
        ),
        (
            "__YAZELIX_AGENT_WIDTH_PERCENT__",
            render_plan
                .layout_percentages
                .right_sidebar_width_percent
                .clone(),
        ),
        (
            "__YAZELIX_OPEN_CONTENT_WIDTH_PERCENT__",
            render_plan
                .layout_percentages
                .open_content_width_percent
                .clone(),
        ),
        (
            "__YAZELIX_CLOSED_CONTENT_WIDTH_PERCENT__",
            render_plan
                .layout_percentages
                .closed_content_width_percent
                .clone(),
        ),
        (
            "__YAZELIX_OPEN_AGENT_OPEN_CONTENT_WIDTH_PERCENT__",
            render_plan
                .layout_percentages
                .left_open_right_open_content_width_percent
                .clone(),
        ),
        (
            "__YAZELIX_OPEN_AGENT_CLOSED_CONTENT_WIDTH_PERCENT__",
            render_plan
                .layout_percentages
                .left_open_right_closed_content_width_percent
                .clone(),
        ),
        (
            "__YAZELIX_CLOSED_AGENT_OPEN_CONTENT_WIDTH_PERCENT__",
            render_plan
                .layout_percentages
                .left_closed_right_open_content_width_percent
                .clone(),
        ),
        (
            "__YAZELIX_CLOSED_AGENT_CLOSED_CONTENT_WIDTH_PERCENT__",
            render_plan
                .layout_percentages
                .left_closed_right_closed_content_width_percent
                .clone(),
        ),
    ];
    for (placeholder, value) in replacements {
        updated = updated.replace(placeholder, &value);
    }
    for placeholder in REQUIRED_LAYOUT_PLACEHOLDERS {
        if updated.contains(placeholder) {
            return Err(format!(
                "failed to expand Zellij layout placeholder: {placeholder}"
            ));
        }
    }
    Ok(updated)
}

fn generated_zellij_layout_header(generation_fingerprint: &str) -> String {
    format!(
        "// ========================================\n// {GENERATED_LAYOUT_MARKER}\n// {GENERATED_LAYOUT_FINGERPRINT_PREFIX} {generation_fingerprint}\n// ========================================\n"
    )
}

fn apply_static_fragments(content: &str, fragments: &BTreeMap<String, String>) -> String {
    let mut updated = content.to_string();
    for (placeholder, value) in fragments {
        if !updated.contains(placeholder) {
            continue;
        }
        let fragment_lines = value.lines().collect::<Vec<_>>();
        updated = updated
            .lines()
            .map(|line| {
                if line.contains(placeholder) {
                    let indent = line
                        .chars()
                        .take_while(|ch| ch.is_whitespace())
                        .collect::<String>();
                    fragment_lines
                        .iter()
                        .map(|fragment_line| format!("{indent}{fragment_line}"))
                        .collect::<Vec<_>>()
                        .join("\n")
                } else {
                    line.to_string()
                }
            })
            .collect::<Vec<_>>()
            .join("\n");
    }
    updated
}

fn render_sidebar_args(args: &[String], runtime_dir: &str) -> String {
    if args.is_empty() {
        String::new()
    } else {
        format!(
            "args {}",
            args.iter()
                .map(|arg| json_quote(expand_runtime_placeholder(arg, runtime_dir)))
                .collect::<Vec<_>>()
                .join(" ")
        )
    }
}

fn expand_runtime_placeholder(value: &str, runtime_dir: &str) -> String {
    value.replace(RUNTIME_DIR_PLACEHOLDER, runtime_dir)
}

#[derive(Debug, Default)]
struct ExtractedSemanticBlocks {
    config_without_semantic_blocks: String,
    load_plugin_lines: Vec<String>,
    plugin_lines: Vec<String>,
    keybind_lines: Vec<String>,
    ui_lines: Vec<String>,
}

fn extract_semantic_config_blocks(config_content: &str) -> ExtractedSemanticBlocks {
    let mut stripped_lines = Vec::new();
    let mut load_plugin_lines = Vec::new();
    let mut plugin_lines = Vec::new();
    let mut keybind_lines = Vec::new();
    let mut ui_lines = Vec::new();
    let mut active_block = String::new();
    let mut brace_depth: i64 = 0;

    for line in config_content.lines() {
        let trimmed = line.trim();
        let open_braces = line.chars().filter(|ch| *ch == '{').count() as i64;
        let close_braces = line.chars().filter(|ch| *ch == '}').count() as i64;

        if active_block.is_empty() {
            let matched_block = ["load_plugins", "plugins", "keybinds", "ui"]
                .into_iter()
                .find(|block| trimmed.starts_with(block));
            if let Some(block) = matched_block {
                active_block = block.to_string();
                brace_depth = open_braces - close_braces;
                if brace_depth <= 0 {
                    let inline_body = trimmed
                        .trim_start_matches(block)
                        .trim()
                        .trim_start_matches('{')
                        .trim_end_matches('}')
                        .trim();
                    if !inline_body.is_empty() {
                        push_semantic_line(
                            block,
                            inline_body.to_string(),
                            &mut load_plugin_lines,
                            &mut plugin_lines,
                            &mut keybind_lines,
                            &mut ui_lines,
                        );
                    }
                    active_block.clear();
                    brace_depth = 0;
                }
            } else {
                stripped_lines.push(line.to_string());
            }
        } else {
            brace_depth += open_braces - close_braces;
            if brace_depth > 0 {
                push_semantic_line(
                    &active_block,
                    line.to_string(),
                    &mut load_plugin_lines,
                    &mut plugin_lines,
                    &mut keybind_lines,
                    &mut ui_lines,
                );
            } else {
                active_block.clear();
            }
        }
    }

    ExtractedSemanticBlocks {
        config_without_semantic_blocks: stripped_lines.join("\n"),
        load_plugin_lines,
        plugin_lines,
        keybind_lines,
        ui_lines,
    }
}

fn push_semantic_line(
    block: &str,
    line: String,
    load_plugin_lines: &mut Vec<String>,
    plugin_lines: &mut Vec<String>,
    keybind_lines: &mut Vec<String>,
    ui_lines: &mut Vec<String>,
) {
    match block {
        "load_plugins" => load_plugin_lines.push(line),
        "plugins" => plugin_lines.push(line),
        "keybinds" => keybind_lines.push(line),
        "ui" => ui_lines.push(line),
        _ => {}
    }
}

fn block_with_lines(name: &str, lines: &[String]) -> String {
    std::iter::once(format!("{name} {{"))
        .chain(lines.iter().cloned())
        .chain(std::iter::once("}".to_string()))
        .collect::<Vec<_>>()
        .join("\n")
}

fn json_quote(value: impl AsRef<str>) -> String {
    serde_json::to_string(value.as_ref()).unwrap_or_else(|_| "\"\"".to_string())
}

// Test lane: default
#[cfg(test)]
mod tests {
    use super::*;

    fn sample_request() -> ZellijConfigPackRenderRequest {
        ZellijConfigPackRenderRequest {
            base_config_content: "scroll_buffer_size 100\nkeybinds { normal { bind \"Alt h\" { MoveFocusOrTab \"left\"; } } }\n".to_string(),
            override_keybinds: vec![
                r#"    normal { bind "Alt X" { SwitchToMode "Normal"; } }"#.to_string(),
            ],
            render_plan: ZellijRenderPlanData {
                default_layout_name: MANAGED_SIDEBAR_LAYOUT_NAME.to_string(),
                appearance_mode: "dark".to_string(),
                left_sidebar_width_percent: 20,
                right_sidebar_width_percent: 40,
                widget_tray: default_widget_tray(),
                widget_frame: default_widget_frame(),
                widget_separator: default_widget_separator(),
                editor_label: "hx".to_string(),
                shell_label: "nu".to_string(),
                terminal_label: "ghostty".to_string(),
                tab_label_mode: "full".to_string(),
                claude_usage_display: "both".to_string(),
                codex_usage_display: "quota".to_string(),
                opencode_go_usage_display: "both".to_string(),
                claude_usage_periods: default_claude_usage_periods(),
                codex_usage_periods: default_codex_usage_periods(),
                opencode_go_usage_periods: default_opencode_go_usage_periods(),
                custom_text: String::new(),
                owned_top_level_setting_names: vec!["default_layout".to_string()],
                dynamic_top_level_settings: vec![TopLevelSetting {
                    name: "theme".to_string(),
                    value: "\"default\"".to_string(),
                }],
                enforced_top_level_settings: vec![TopLevelSetting {
                    name: "default_layout".to_string(),
                    value: "\"/tmp/yazelix/layouts/yzx_side.kdl\"".to_string(),
                }],
                rounded_value: "true".to_string(),
                popup_width_percent: 90,
                popup_height_percent: 80,
                screen_saver_enabled: false,
                screen_saver_idle_seconds: 300,
                screen_saver_style: "random".to_string(),
                right_sidebar_command: "__YAZELIX_RUNTIME_DIR__/bin/agent".to_string(),
                right_sidebar_args: vec!["--right".to_string()],
                left_sidebar_command: "__YAZELIX_RUNTIME_DIR__/bin/sidebar".to_string(),
                left_sidebar_args: vec![
                    "--root".to_string(),
                    "__YAZELIX_RUNTIME_DIR__/side".to_string(),
                ],
                layout_percentages: ZellijLayoutPercentages {
                    left_sidebar_width_percent: "20%".to_string(),
                    right_sidebar_width_percent: "40%".to_string(),
                    open_content_width_percent: "80%".to_string(),
                    closed_content_width_percent: "100%".to_string(),
                    left_open_right_open_content_width_percent: "40%".to_string(),
                    left_open_right_closed_content_width_percent: "80%".to_string(),
                    left_closed_right_open_content_width_percent: "60%".to_string(),
                    left_closed_right_closed_content_width_percent: "100%".to_string(),
                },
            },
            popup_commands: BTreeMap::from([
                ("bottom_popup".to_string(), vec!["lazygit".to_string()]),
                ("top_popup".to_string(), vec!["btop".to_string()]),
                ("menu".to_string(), vec!["yzx".to_string(), "menu".to_string()]),
            ]),
            custom_popups: vec![CustomPopup {
                id: "gitui".to_string(),
                command: vec!["gitui".to_string()],
                keybindings: vec!["Alt Shift G".to_string()],
                keep_alive: false,
            }],
            layout_templates: None,
            static_fragments: None,
            zjstatus_plugin_block: r#"plugin location="file:/tmp/zjstatus.wasm" {
    pipe_workspace_format "renderer-owned-workspace"
}"#
            .to_string(),
            pane_orchestrator_plugin_url: "file:/tmp/pane.wasm".to_string(),
            yzpp_plugin_url: "file:/tmp/yzpp.wasm".to_string(),
            home_dir: "/home/user".to_string(),
            runtime_dir: "/opt/yazelix".to_string(),
            generation_fingerprint: "gen-test".to_string(),
        }
    }

    // Defends: the config-pack render API is deterministic from explicit request data and bundled assets.
    #[test]
    fn renders_bundled_config_pack_without_main_checkout_state() {
        let output = render_zellij_config_pack(&sample_request()).unwrap();

        assert_eq!(output.renderer_schema_version, RENDERER_SCHEMA_VERSION);
        assert!(output.merged_config.contains("scroll_buffer_size 100"));
        assert!(output.merged_config.contains("Alt X"));
        assert!(output.merged_config.contains("gitui"));
        assert!(output.merged_config.contains("file:/tmp/pane.wasm"));
        assert!(output.merged_config.contains("file:/tmp/yzpp.wasm"));
        assert_eq!(output.layout_files.len(), 2);
        let side = output
            .layout_files
            .iter()
            .find(|file| file.relative_path == "yzx_side.kdl")
            .unwrap();
        assert!(
            side.content
                .starts_with(&generated_zellij_layout_header("gen-test"))
        );
        assert!(
            side.content
                .contains(r#"plugin location="file:/tmp/zjstatus.wasm" {"#)
        );
        assert!(side.content.contains(r#"cwd="/home/user""#));
        assert!(
            side.content
                .contains(&format!(r#"tab name="{}""#, HOME_TAB_MARKER))
        );
        assert!(
            side.content
                .contains(r#"command "/opt/yazelix/bin/sidebar""#)
        );
        assert!(
            side.content
                .contains(r#"args "--root" "/opt/yazelix/side""#)
        );
        for placeholder in REQUIRED_LAYOUT_PLACEHOLDERS {
            assert!(!side.content.contains(placeholder));
        }
    }

    // Regression: background controllers must be loaded, not only declared as plugin aliases for first-message launch.
    #[test]
    fn background_loads_workspace_plugins_with_session_runtime_config() {
        let output = render_zellij_config_pack(&sample_request()).unwrap();
        let config = output.merged_config;

        assert!(
            config.contains(r#"    yazelix_pane_orchestrator location="file:/tmp/pane.wasm" {"#)
        );
        assert!(config.contains(r#"    yzpp location="file:/tmp/yzpp.wasm" {"#));
        let load_plugins = &config[config.find("load_plugins {").unwrap()..];
        assert!(load_plugins.contains("    yazelix_pane_orchestrator {\n"));
        assert!(load_plugins.contains("        runtime_dir \"/opt/yazelix\""));
        assert!(load_plugins.contains("    yzpp {\n        popups {"));
        assert!(load_plugins.contains("            bottom_popup {"));
        assert!(load_plugins.contains("                arg_2 \"lazygit\""));
        assert!(load_plugins.contains("            top_popup {"));
        assert!(load_plugins.contains("                arg_2 \"btop\""));
        assert!(load_plugins.contains("            gitui {"));
        assert!(load_plugins.contains("                arg_2 \"gitui\""));
    }

    // Defends: usage refresh timers in the pane orchestrator follow the visible widget tray.
    #[test]
    fn pane_orchestrator_usage_provider_flags_follow_widget_tray() {
        let mut request = sample_request();
        request.render_plan.widget_tray = vec![
            "editor".to_string(),
            "workspace".to_string(),
            "opencode_go_usage".to_string(),
        ];

        let config = render_zellij_config_pack(&request).unwrap().merged_config;

        assert_eq!(
            config
                .matches(r#"status_usage_provider_claude_enabled "false""#)
                .count(),
            2
        );
        assert_eq!(
            config
                .matches(r#"status_usage_provider_codex_enabled "false""#)
                .count(),
            2
        );
        assert_eq!(
            config
                .matches(r#"status_usage_provider_opencode_go_enabled "true""#)
                .count(),
            2
        );
    }

    // Regression: external popup commands are wrapped through the runtime CLI wrapper before yzpp sees them.
    #[test]
    fn wraps_external_popup_commands_through_runtime_cli() {
        assert_eq!(
            popup_command_argv_for_yazelix_runtime(
                &["lazygit".to_string(), "status".to_string()],
                "/opt/yazelix/shells/posix/yzx_cli.sh",
            ),
            vec![
                "/opt/yazelix/shells/posix/yzx_cli.sh".to_string(),
                "popup_run".to_string(),
                "lazygit".to_string(),
                "status".to_string(),
            ]
        );
    }

    fn sample_plan_request() -> ZellijRenderPlanRequest {
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
            zellij_widget_frame: default_widget_frame(),
            zellij_widget_separator: default_widget_separator(),
            zellij_custom_text: None,
            zellij_theme: "default".into(),
            appearance_mode: APPEARANCE_MODE_DARK.into(),
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

    fn sample_keybind_render_request() -> ZellijKeybindRenderRequest {
        ZellijKeybindRenderRequest {
            override_template_content: r#"
keybinds {
    shared {
        unbind "Alt p"
        unbind "Ctrl s"
        unbind "Alt \"Q\""
    }
}
"#
            .to_string(),
            runtime_dir: "/opt/yazelix".to_string(),
            home_dir: "/home/user".to_string(),
            zellij_keybindings: BTreeMap::from([
                ("menu".to_string(), vec!["Alt p".to_string()]),
                (
                    "open_workspace_terminal".to_string(),
                    vec!["Alt Shift T".to_string()],
                ),
            ]),
            zellij_native_keybindings: BTreeMap::from([
                ("scroll_mode_unbind".to_string(), vec!["Ctrl s".to_string()]),
                ("scroll_mode".to_string(), vec!["Ctrl s".to_string()]),
                ("multiline_mode".to_string(), vec!["Ctrl Alt m".to_string()]),
            ]),
            custom_popups: vec![CustomPopup {
                id: "gitui".to_string(),
                command: vec!["gitui".to_string()],
                keybindings: vec!["Alt Shift G".to_string()],
                keep_alive: false,
            }],
            integration_actions: vec![
                ZellijKeybindActionSpec {
                    local_id: "menu".to_string(),
                    mode: "shared".to_string(),
                    plugin_alias: "yzpp".to_string(),
                    message_name: "toggle".to_string(),
                    payload: Some("menu".to_string()),
                },
                ZellijKeybindActionSpec {
                    local_id: "open_workspace_terminal".to_string(),
                    mode: "shared".to_string(),
                    plugin_alias: "yazelix_pane_orchestrator".to_string(),
                    message_name: "open_workspace_terminal".to_string(),
                    payload: None,
                },
            ],
            native_actions: vec![
                ZellijNativeKeybindSpec {
                    local_id: "scroll_mode_unbind".to_string(),
                    blocks: vec![ZellijNativeKeybindBlockSpec {
                        mode: "shared".to_string(),
                        action_lines: Vec::new(),
                    }],
                },
                ZellijNativeKeybindSpec {
                    local_id: "scroll_mode".to_string(),
                    blocks: vec![
                        ZellijNativeKeybindBlockSpec {
                            mode: "shared".to_string(),
                            action_lines: vec!["SwitchToMode \"Scroll\"".to_string()],
                        },
                        ZellijNativeKeybindBlockSpec {
                            mode: "scroll".to_string(),
                            action_lines: vec!["SwitchToMode \"Normal\"".to_string()],
                        },
                    ],
                },
                ZellijNativeKeybindSpec {
                    local_id: "multiline_mode".to_string(),
                    blocks: vec![ZellijNativeKeybindBlockSpec {
                        mode: "shared".to_string(),
                        action_lines: vec!["SwitchToMode \"Pane\"\nTogglePaneFrames".to_string()],
                    }],
                },
            ],
            pane_orchestrator_plugin_alias: "yazelix_pane_orchestrator".to_string(),
            popup_plugin_alias: "yzpp".to_string(),
        }
    }

    // Regression: generated semantic/native keybinds own conflicting unbind cleanup in the config-pack renderer.
    #[test]
    fn keybind_renderer_filters_generated_unbind_conflicts() {
        let output = render_zellij_keybinds(&sample_keybind_render_request());
        let rendered = output.override_keybinds.join("\n");

        assert!(!rendered.contains(r#"unbind "Alt p""#));
        assert!(!rendered.contains(r#"unbind "Ctrl s""#));
        assert!(rendered.contains(r#"unbind "Alt \"Q\"""#));
        assert!(rendered.contains(r#"bind "Alt p" {"#));
        assert!(rendered.contains(r#"payload "menu""#));
        assert!(rendered.contains(r#"bind "Ctrl s" { SwitchToMode "Scroll"; }"#));
        assert!(rendered.contains(r#"bind "Ctrl s" { SwitchToMode "Normal"; }"#));
    }

    // Defends: default tab-mode new tab creation gets the home cwd and compact home marker directly from the generated NewTab action.
    #[test]
    fn keybind_renderer_names_default_new_tabs_from_home() {
        let output = render_zellij_keybinds(&sample_keybind_render_request());
        let rendered = output.override_keybinds.join("\n");
        let expected = format!(
            r#"    tab {{
        bind "n" {{ NewTab {{ cwd "/home/user"; name "{}"; }}; SwitchToMode "Normal"; }}
    }}"#,
            HOME_TAB_MARKER
        );

        assert!(rendered.contains(&expected));
    }

    // Defends: main passes explicit action specs, while the config-pack renderer emits the MessagePlugin KDL.
    #[test]
    fn keybind_renderer_routes_actions_and_custom_popups_to_declared_plugins() {
        let output = render_zellij_keybinds(&sample_keybind_render_request());
        let rendered = output.override_keybinds.join("\n");

        assert!(rendered.contains(r#"MessagePlugin "yzpp" {"#));
        assert!(rendered.contains(r#"payload "menu""#));
        assert!(rendered.contains(r#"payload "gitui""#));
        assert!(rendered.contains(r#"MessagePlugin "yazelix_pane_orchestrator" {"#));
        assert!(rendered.contains(r#"name "open_workspace_terminal""#));
    }

    // Defends: config-pack native rendering preserves multi-line Zellij action blocks.
    #[test]
    fn keybind_renderer_preserves_multiline_native_action_blocks() {
        let output = render_zellij_keybinds(&sample_keybind_render_request());
        let rendered = output.override_keybinds.join("\n");

        assert!(rendered.contains(r#"bind "Ctrl Alt m" {"#));
        assert!(rendered.contains("            SwitchToMode \"Pane\""));
        assert!(rendered.contains("            TogglePaneFrames"));
    }

    // Defends: layout placeholder percents stay aligned with the generated layout templates.
    #[test]
    fn layout_percentages_match_template_placeholders() {
        let plan = compute_zellij_render_plan(&sample_plan_request()).unwrap();
        assert_eq!(plan.layout_percentages.left_sidebar_width_percent, "20%");
        assert_eq!(plan.layout_percentages.right_sidebar_width_percent, "40%");
        assert_eq!(plan.layout_percentages.open_content_width_percent, "80%");
        assert_eq!(plan.layout_percentages.closed_content_width_percent, "99%");
        assert_eq!(
            plan.layout_percentages
                .left_open_right_open_content_width_percent,
            "40%"
        );
        assert_eq!(
            plan.layout_percentages
                .left_open_right_closed_content_width_percent,
            "79%"
        );
        assert_eq!(
            plan.layout_percentages
                .left_closed_right_open_content_width_percent,
            "59%"
        );
        assert_eq!(
            plan.layout_percentages
                .left_closed_right_closed_content_width_percent,
            "98%"
        );
    }

    // Defends: the config-pack planner rejects invalid sidebar geometry before KDL rendering.
    #[test]
    fn rejects_sidebar_out_of_range() {
        let mut req = sample_plan_request();
        req.left_sidebar_width_percent = 0;
        assert_eq!(
            compute_zellij_render_plan(&req).unwrap_err().code(),
            "invalid_sidebar_width_percent"
        );
    }

    // Regression: custom sidebar apps must not inherit the default Yazi launcher args.
    #[test]
    fn custom_sidebar_command_drops_implicit_yazi_launcher_arg() {
        for command in ["lazygit", "opencode"] {
            let mut req = sample_plan_request();
            req.left_sidebar_command = command.into();
            req.left_sidebar_args = default_left_sidebar_args();
            let plan = compute_zellij_render_plan(&req).unwrap();

            assert_eq!(plan.left_sidebar_command, command);
            assert!(plan.left_sidebar_args.is_empty());
        }
    }

    // Defends: status-bar widget tray entries are validated in the config-pack planner before runtime rendering.
    #[test]
    fn rejects_invalid_tray_widget() {
        let mut req = sample_plan_request();
        req.zellij_widget_tray = Some(vec!["editor".into(), "cursor".into()]);
        assert_eq!(
            compute_zellij_render_plan(&req).unwrap_err().code(),
            "invalid_widget_tray_entry"
        );
    }

    // Regression: session is a first-class status-bar widget token, not a hardcoded renderer-only segment.
    #[test]
    fn accepts_session_tray_widget() {
        let mut req = sample_plan_request();
        req.zellij_widget_tray = Some(vec!["session".into(), "editor".into()]);
        let plan = compute_zellij_render_plan(&req).unwrap();

        assert_eq!(plan.widget_tray, vec!["session", "editor"]);
    }

    // Defends: status-bar widget chrome is normalized and rejected by the config-pack planner before child KDL rendering.
    #[test]
    fn normalizes_widget_chrome() {
        let mut req = sample_plan_request();
        req.zellij_widget_frame = " Square ".into();
        req.zellij_widget_separator = " Pipe ".into();
        let plan = compute_zellij_render_plan(&req).unwrap();

        assert_eq!(plan.widget_frame, "square");
        assert_eq!(plan.widget_separator, "pipe");

        req.zellij_widget_frame = "curly".into();
        assert_eq!(
            compute_zellij_render_plan(&req).unwrap_err().code(),
            "invalid_widget_frame"
        );

        let mut req = sample_plan_request();
        req.zellij_widget_separator = "comma".into();
        assert_eq!(
            compute_zellij_render_plan(&req).unwrap_err().code(),
            "invalid_widget_separator"
        );
    }

    // Defends: usage widget periods are normalized and deduplicated before the status-bar renderer sees them.
    #[test]
    fn normalizes_agent_usage_periods() {
        let mut req = sample_plan_request();
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
    }

    // Defends: the config-pack planner keeps explicit screen saver config normalized and bounded before pane-orchestrator KDL.
    #[test]
    fn screen_saver_config_is_normalized_and_bounded() {
        let mut req = sample_plan_request();
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

    // Regression: omitted JSON fields use config-contract defaults for machine callers.
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

    // Defends: light appearance changes only the implicit default Zellij theme, preserving explicit user choices.
    #[test]
    fn light_appearance_changes_default_theme_only() {
        let mut req = sample_plan_request();
        req.appearance_mode = "light".into();
        let plan = compute_zellij_render_plan(&req).unwrap();
        assert_eq!(plan.appearance_mode, "light");
        let theme = plan
            .dynamic_top_level_settings
            .iter()
            .find(|setting| setting.name == "theme")
            .unwrap();
        assert_eq!(theme.value, "\"catppuccin-latte\"");

        req.zellij_theme = "dracula".into();
        let plan = compute_zellij_render_plan(&req).unwrap();
        let theme = plan
            .dynamic_top_level_settings
            .iter()
            .find(|setting| setting.name == "theme")
            .unwrap();
        assert_eq!(theme.value, "\"dracula\"");
    }

    // Regression: status widget labels must never be empty, even when config values are paths or omitted.
    #[test]
    fn status_widget_labels_use_basenames_and_defaults() {
        let mut req = sample_plan_request();
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
        let mut req = sample_plan_request();
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
