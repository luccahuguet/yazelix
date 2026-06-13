use serde_json::{Map as JsonMap, Value as JsonValue};

pub const APPEARANCE_MODE_DARK: &str = "dark";
pub const APPEARANCE_MODE_LIGHT: &str = "light";
pub const APPEARANCE_MODE_AUTO: &str = "auto";

pub const GHOSTTY_THEME_DARK: &str = "Abernathy";
pub const GHOSTTY_THEME_LIGHT: &str = "Catppuccin Latte";
pub const WEZTERM_THEME_DARK: &str = "Abernathy";
pub const WEZTERM_THEME_LIGHT: &str = "Catppuccin Latte";
pub const ZELLIJ_THEME_LIGHT: &str = "catppuccin-latte";
pub const YAZI_THEME_LIGHT: &str = "catppuccin-latte";

pub fn appearance_mode_from_config(config: &JsonMap<String, JsonValue>) -> &str {
    config
        .get("appearance_mode")
        .and_then(JsonValue::as_str)
        .filter(|value| !value.trim().is_empty())
        .unwrap_or(APPEARANCE_MODE_DARK)
}

pub fn static_light_mode(mode: &str) -> bool {
    mode == APPEARANCE_MODE_LIGHT
}

pub fn auto_mode(mode: &str) -> bool {
    mode == APPEARANCE_MODE_AUTO
}

pub fn ghostty_theme(mode: &str) -> String {
    match mode {
        APPEARANCE_MODE_LIGHT => GHOSTTY_THEME_LIGHT.to_string(),
        APPEARANCE_MODE_AUTO => {
            format!("dark:{GHOSTTY_THEME_DARK},light:{GHOSTTY_THEME_LIGHT}")
        }
        _ => GHOSTTY_THEME_DARK.to_string(),
    }
}

pub fn wezterm_theme(mode: &str) -> String {
    match mode {
        APPEARANCE_MODE_LIGHT => WEZTERM_THEME_LIGHT.to_string(),
        _ => WEZTERM_THEME_DARK.to_string(),
    }
}

pub fn appearance_default_theme(configured_theme: &str, light_theme: &str, mode: &str) -> String {
    if static_light_mode(mode) && configured_theme == "default" {
        light_theme.to_string()
    } else {
        configured_theme.to_string()
    }
}
