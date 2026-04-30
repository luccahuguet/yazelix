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
    "ai_activity",
    "token_budget",
    "claude_usage",
    "codex_usage",
    "opencode_usage",
    "cpu",
    "ram",
];
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
pub const DEFAULT_SIDEBAR_COMMAND: &str = "nu";
pub const DEFAULT_SIDEBAR_YAZI_ARG: &str =
    "__YAZELIX_RUNTIME_DIR__/configs/zellij/scripts/launch_sidebar_yazi.nu";

fn default_enable_sidebar() -> bool {
    true
}

fn default_initial_sidebar_state() -> String {
    "open".into()
}

fn default_sidebar_width_percent() -> i64 {
    20
}

fn default_sidebar_command() -> String {
    DEFAULT_SIDEBAR_COMMAND.into()
}

fn default_sidebar_args() -> Vec<String> {
    vec![DEFAULT_SIDEBAR_YAZI_ARG.into()]
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
    "false".into()
}

fn default_zellij_default_mode() -> String {
    "normal".into()
}

fn default_widget_tray() -> Vec<String> {
    vec![
        "editor".into(),
        "shell".into(),
        "term".into(),
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
pub struct ZellijRenderPlanRequest {
    #[serde(default = "default_enable_sidebar")]
    pub enable_sidebar: bool,
    #[serde(default = "default_initial_sidebar_state")]
    pub initial_sidebar_state: String,
    #[serde(default = "default_sidebar_width_percent")]
    pub sidebar_width_percent: i64,
    #[serde(default = "default_sidebar_command")]
    pub sidebar_command: String,
    #[serde(default = "default_sidebar_args")]
    pub sidebar_args: Vec<String>,
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
    /// Matches `config_metadata/main_config_contract.toml` (`zellij.support_kitty_keyboard_protocol` default false).
    #[serde(default = "default_support_kitty_keyboard_protocol")]
    pub support_kitty_keyboard_protocol: String,
    #[serde(default = "default_zellij_default_mode")]
    pub zellij_default_mode: String,
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
    pub sidebar_width_percent: String,
    pub open_content_width_percent: String,
    pub open_primary_width_percent: String,
    pub open_secondary_width_percent: String,
    pub closed_content_width_percent: String,
    pub closed_primary_width_percent: String,
    pub closed_secondary_width_percent: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct TopLevelSetting {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ZellijRenderPlanData {
    pub default_layout_name: String,
    pub sidebar_width_percent: i64,
    pub sidebar_command: String,
    pub sidebar_args: Vec<String>,
    pub popup_width_percent: i64,
    pub popup_height_percent: i64,
    pub screen_saver_enabled: bool,
    pub screen_saver_idle_seconds: i64,
    pub screen_saver_style: String,
    pub widget_tray: Vec<String>,
    pub editor_label: String,
    pub shell_label: String,
    pub terminal_label: String,
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

fn compute_layout_percentages(sidebar_width_percent: i64) -> LayoutPlaceholderPercents {
    let open_content_width_percent = 100 - sidebar_width_percent;
    let open_primary_width_percent = (open_content_width_percent * 3) / 5;
    let open_secondary_width_percent = open_content_width_percent - open_primary_width_percent;
    let closed_content_width_percent = 99;
    let closed_primary_width_percent = (closed_content_width_percent * 3) / 5;
    let closed_secondary_width_percent =
        closed_content_width_percent - closed_primary_width_percent;

    LayoutPlaceholderPercents {
        sidebar_width_percent: format!("{sidebar_width_percent}%"),
        open_content_width_percent: format!("{open_content_width_percent}%"),
        open_primary_width_percent: format!("{open_primary_width_percent}%"),
        open_secondary_width_percent: format!("{open_secondary_width_percent}%"),
        closed_content_width_percent: format!("{closed_content_width_percent}%"),
        closed_primary_width_percent: format!("{closed_primary_width_percent}%"),
        closed_secondary_width_percent: format!("{closed_secondary_width_percent}%"),
    }
}

fn validate_sidebar_width(value: i64) -> Result<(), CoreError> {
    if (10..=40).contains(&value) {
        Ok(())
    } else {
        Err(CoreError::classified(
            ErrorClass::Config,
            "invalid_sidebar_width_percent",
            format!("sidebar_width_percent must be between 10 and 40 (got {value})"),
            "Set editor.sidebar_width_percent within the documented range.",
            serde_json::json!({ "field": "editor.sidebar_width_percent" }),
        ))
    }
}

fn validate_sidebar_launcher(command: &str) -> Result<(), CoreError> {
    if !command.trim().is_empty() {
        Ok(())
    } else {
        Err(CoreError::classified(
            ErrorClass::Config,
            "invalid_sidebar_command",
            "editor.sidebar_command must not be empty",
            "Set editor.sidebar_command to a terminal command such as `nu`, `yazi`, or your side-surface launcher.",
            serde_json::json!({ "field": "editor.sidebar_command" }),
        ))
    }
}

pub fn effective_sidebar_args(command: &str, args: &[String]) -> Vec<String> {
    if !is_default_sidebar_command(command) && args == default_sidebar_args().as_slice() {
        Vec::new()
    } else {
        args.to_vec()
    }
}

fn is_default_sidebar_command(command: &str) -> bool {
    let trimmed = command.trim();
    if trimmed == DEFAULT_SIDEBAR_COMMAND {
        return true;
    }
    Path::new(trimmed)
        .file_name()
        .and_then(|name| name.to_str())
        .map(|name| name == DEFAULT_SIDEBAR_COMMAND)
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
    validate_sidebar_width(request.sidebar_width_percent)?;
    validate_sidebar_launcher(&request.sidebar_command)?;
    validate_popup_percent("zellij.popup_width_percent", request.popup_width_percent)?;
    validate_popup_percent("zellij.popup_height_percent", request.popup_height_percent)?;
    validate_screen_saver_idle_seconds(request.screen_saver_idle_seconds)?;
    validate_default_mode(&request.zellij_default_mode)?;
    let screen_saver_style = normalize_screen_saver_style(&request.screen_saver_style)?;

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

    let default_layout_name =
        managed_sidebar_layout_name(request.enable_sidebar, &request.initial_sidebar_state)?
            .to_string();

    let layout_percentages = compute_layout_percentages(request.sidebar_width_percent);

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
        sidebar_width_percent: request.sidebar_width_percent,
        sidebar_command: request.sidebar_command.trim().to_string(),
        sidebar_args: effective_sidebar_args(&request.sidebar_command, &request.sidebar_args),
        popup_width_percent: request.popup_width_percent,
        popup_height_percent: request.popup_height_percent,
        screen_saver_enabled: request.screen_saver_enabled,
        screen_saver_idle_seconds: request.screen_saver_idle_seconds,
        screen_saver_style,
        widget_tray,
        editor_label,
        shell_label,
        terminal_label,
        custom_text,
        layout_percentages,
        rounded_value: rounded_value.to_string(),
        dynamic_top_level_settings,
        enforced_top_level_settings,
        owned_top_level_setting_names,
    })
}

pub fn managed_sidebar_layout_name(
    enable_sidebar: bool,
    initial_sidebar_state: &str,
) -> Result<&'static str, CoreError> {
    let normalized = initial_sidebar_state.trim().to_lowercase();
    match normalized.as_str() {
        "open" if enable_sidebar => Ok("yzx_side"),
        "open" | "closed" => Ok("yzx_side_closed"),
        _ => Err(CoreError::classified(
            ErrorClass::Config,
            "invalid_initial_sidebar_state",
            format!(
                "editor.initial_sidebar_state must be `open` or `closed`, got `{}`.",
                initial_sidebar_state
            ),
            "Set editor.initial_sidebar_state to `open` or `closed`, then retry.",
            serde_json::json!({ "initial_sidebar_state": initial_sidebar_state }),
        )),
    }
}

// Test lane: maintainer

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_request() -> ZellijRenderPlanRequest {
        ZellijRenderPlanRequest {
            enable_sidebar: true,
            initial_sidebar_state: "open".into(),
            sidebar_width_percent: 20,
            sidebar_command: "nu".into(),
            sidebar_args: default_sidebar_args(),
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
            yazelix_layout_dir: "/tmp/yazelix/layouts".into(),
            resolved_default_shell: "/usr/bin/nu".into(),
            editor_label: "hx".into(),
            shell_label: "nu".into(),
            terminal_label: "wezterm".into(),
        }
    }

    // Defends: layout placeholder percents stay aligned with the historical Nushell geometry helper.
    // Strength: defect=1 behavior=2 resilience=1 cost=2 uniqueness=2 total=8/10
    #[test]
    fn layout_percentages_match_legacy_nushell() {
        let p = compute_layout_percentages(20);
        assert_eq!(p.sidebar_width_percent, "20%");
        assert_eq!(p.open_content_width_percent, "80%");
        assert_eq!(p.open_primary_width_percent, "48%");
        assert_eq!(p.open_secondary_width_percent, "32%");
        assert_eq!(p.closed_content_width_percent, "99%");
        assert_eq!(p.closed_primary_width_percent, "59%");
        assert_eq!(p.closed_secondary_width_percent, "40%");
    }

    // Defends: sidebar width contract bounds surface as structured config errors, not silent clamping.
    // Strength: defect=2 behavior=2 resilience=1 cost=2 uniqueness=1 total=8/10
    #[test]
    fn rejects_sidebar_out_of_range() {
        let mut req = sample_request();
        req.sidebar_width_percent = 9;
        assert!(compute_zellij_render_plan(&req).is_err());
    }

    // Defends: custom side-surface launchers fail fast when the command is empty instead of generating unusable Zellij KDL.
    // Strength: defect=2 behavior=2 resilience=1 cost=2 uniqueness=1 total=8/10
    #[test]
    fn rejects_empty_sidebar_command() {
        let mut req = sample_request();
        req.sidebar_command = "   ".into();
        let error = compute_zellij_render_plan(&req).unwrap_err();

        assert_eq!(error.code(), "invalid_sidebar_command");
    }

    // Regression: custom sidebar apps must not inherit the default Yazi launcher script as their first argument.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn custom_sidebar_command_drops_implicit_yazi_launcher_arg() {
        for command in ["lazygit", "opencode"] {
            let mut req = sample_request();
            req.sidebar_command = command.into();
            req.sidebar_args = default_sidebar_args();
            let plan = compute_zellij_render_plan(&req).unwrap();

            assert_eq!(plan.sidebar_command, command);
            assert!(plan.sidebar_args.is_empty());
        }
    }

    // Defends: explicit custom sidebar arguments still pass through after the implicit default launcher arg is removed.
    // Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
    #[test]
    fn custom_sidebar_command_preserves_explicit_args() {
        let mut req = sample_request();
        req.sidebar_command = "lazygit".into();
        req.sidebar_args = vec!["status".into()];
        let plan = compute_zellij_render_plan(&req).unwrap();

        assert_eq!(plan.sidebar_args, vec!["status"]);
    }

    // Defends: widget tray entries are validated against the same allowed set as config.normalize.
    // Strength: defect=2 behavior=2 resilience=1 cost=2 uniqueness=1 total=8/10
    #[test]
    fn rejects_invalid_tray_widget() {
        let mut req = sample_request();
        req.zellij_widget_tray = Some(vec!["editor".into(), "nope".into()]);
        assert!(compute_zellij_render_plan(&req).is_err());
    }

    // Defends: AI status and usage widgets are accepted as optional extension points without changing the default tray.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn accepts_ai_extension_tray_widgets_without_defaulting_them() {
        let mut req = sample_request();
        req.zellij_widget_tray = Some(vec![
            "ai_activity".into(),
            "token_budget".into(),
            "claude_usage".into(),
        ]);
        let plan = compute_zellij_render_plan(&req).unwrap();

        assert_eq!(
            plan.widget_tray,
            vec!["ai_activity", "token_budget", "claude_usage"]
        );
        assert_eq!(
            default_widget_tray(),
            vec!["editor", "shell", "term", "cpu", "ram"]
        );
    }

    // Defends: idle screen saver config is explicit and bounded before it reaches the pane-orchestrator plugin.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
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

    // Defends: managed default layout name tracks the initial sidebar state without disabling sidebar capability.
    // Strength: defect=2 behavior=2 resilience=1 cost=2 uniqueness=1 total=8/10
    #[test]
    fn default_layout_follows_initial_sidebar_state() {
        let mut req = sample_request();
        req.initial_sidebar_state = "closed".into();
        let plan = compute_zellij_render_plan(&req).unwrap();
        assert_eq!(plan.default_layout_name, "yzx_side_closed");

        req.initial_sidebar_state = "open".into();
        req.enable_sidebar = false;
        let plan = compute_zellij_render_plan(&req).unwrap();
        assert_eq!(plan.default_layout_name, "yzx_side_closed");
    }

    // Defends: enforced default_layout points at the computed managed layout file for the active initial sidebar state.
    // Strength: defect=2 behavior=2 resilience=1 cost=2 uniqueness=1 total=8/10
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
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=1 total=8/10
    #[test]
    fn omitted_support_kitty_keyboard_protocol_defaults_to_contract_false() {
        let json = serde_json::json!({
            "yazelix_layout_dir": "/tmp/yazelix/layouts",
            "resolved_default_shell": "/bin/sh",
        });
        let req: ZellijRenderPlanRequest = serde_json::from_value(json).unwrap();
        assert_eq!(req.support_kitty_keyboard_protocol, "false");
        let plan = compute_zellij_render_plan(&req).unwrap();
        let kitty = plan
            .enforced_top_level_settings
            .iter()
            .find(|s| s.name == "support_kitty_keyboard_protocol")
            .unwrap();
        assert_eq!(kitty.value, "false");
    }

    // Regression: status widget labels must never be empty, even when config values are paths or omitted.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
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
}
