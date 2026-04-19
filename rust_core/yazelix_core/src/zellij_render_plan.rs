//! Typed Zellij render-plan data for Nushell KDL renderers.

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

const WIDGET_TRAY_ALLOWED: &[&str] = &["editor", "shell", "term", "cpu", "ram"];

fn default_enable_sidebar() -> bool {
    true
}

fn default_sidebar_width_percent() -> i64 {
    20
}

fn default_popup_percent() -> i64 {
    90
}

fn default_zellij_theme() -> String {
    "default".into()
}

fn default_string_true() -> String {
    "true".into()
}

fn default_string_false() -> String {
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

#[derive(Debug, Deserialize)]
pub struct ZellijRenderPlanRequest {
    #[serde(default = "default_enable_sidebar")]
    pub enable_sidebar: bool,
    #[serde(default = "default_sidebar_width_percent")]
    pub sidebar_width_percent: i64,
    #[serde(default = "default_popup_percent")]
    pub popup_width_percent: i64,
    #[serde(default = "default_popup_percent")]
    pub popup_height_percent: i64,
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
    #[serde(default = "default_string_false")]
    pub persistent_sessions: String,
    /// Matches legacy Nushell `zellij_owned_settings.nu` default when the field is absent.
    #[serde(default = "default_string_true")]
    pub support_kitty_keyboard_protocol: String,
    #[serde(default = "default_zellij_default_mode")]
    pub zellij_default_mode: String,
    pub yazelix_layout_dir: String,
    pub resolved_default_shell: String,
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
    pub popup_width_percent: i64,
    pub popup_height_percent: i64,
    pub widget_tray: Vec<String>,
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

pub fn compute_zellij_render_plan(
    request: &ZellijRenderPlanRequest,
) -> Result<ZellijRenderPlanData, CoreError> {
    validate_sidebar_width(request.sidebar_width_percent)?;
    validate_popup_percent("zellij.popup_width_percent", request.popup_width_percent)?;
    validate_popup_percent("zellij.popup_height_percent", request.popup_height_percent)?;
    validate_default_mode(&request.zellij_default_mode)?;

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

    let default_layout_name = if request.enable_sidebar {
        "yzx_side".to_string()
    } else {
        "yzx_no_side".to_string()
    };

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
    let on_force_close_value = if bool_setting_from_string(&request.persistent_sessions) {
        "detach"
    } else {
        "quit"
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
        make_setting(
            "on_force_close",
            kdl_quoted_path(Path::new(on_force_close_value)),
        ),
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
        popup_width_percent: request.popup_width_percent,
        popup_height_percent: request.popup_height_percent,
        widget_tray,
        custom_text,
        layout_percentages,
        rounded_value: rounded_value.to_string(),
        dynamic_top_level_settings,
        enforced_top_level_settings,
        owned_top_level_setting_names,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_request() -> ZellijRenderPlanRequest {
        ZellijRenderPlanRequest {
            enable_sidebar: true,
            sidebar_width_percent: 20,
            popup_width_percent: 90,
            popup_height_percent: 90,
            zellij_widget_tray: None,
            zellij_custom_text: None,
            zellij_theme: "default".into(),
            zellij_pane_frames: "true".into(),
            zellij_rounded_corners: "true".into(),
            disable_zellij_tips: "true".into(),
            persistent_sessions: "false".into(),
            support_kitty_keyboard_protocol: "true".into(),
            zellij_default_mode: "normal".into(),
            yazelix_layout_dir: "/tmp/yazelix/layouts".into(),
            resolved_default_shell: "/usr/bin/nu".into(),
        }
    }

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

    #[test]
    fn rejects_sidebar_out_of_range() {
        let mut req = sample_request();
        req.sidebar_width_percent = 9;
        assert!(compute_zellij_render_plan(&req).is_err());
    }

    #[test]
    fn rejects_invalid_tray_widget() {
        let mut req = sample_request();
        req.zellij_widget_tray = Some(vec!["editor".into(), "nope".into()]);
        assert!(compute_zellij_render_plan(&req).is_err());
    }

    #[test]
    fn default_layout_follows_sidebar_flag() {
        let mut req = sample_request();
        req.enable_sidebar = false;
        let plan = compute_zellij_render_plan(&req).unwrap();
        assert_eq!(plan.default_layout_name, "yzx_no_side");
    }

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
}
