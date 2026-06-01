use crate::atomic_fs::write_text_atomic;
use crate::bridge::CoreError;
use crate::config_normalize::{NormalizeConfigRequest, normalize_config};
use crate::control_plane::config_dir_from_env;
use crate::ghostty_cursor_registry::{CursorRegistry, YazelixCursorRegistryExt};
use crate::ghostty_materialization::{
    GhosttyMaterializationRequest, generate_ghostty_materialization,
};
use crate::runtime_component_enabled;
use crate::user_config_paths;
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};

const YAZELIX_THEME: &str = "Abernathy";
const FONT_FIRACODE: &str = "FiraCode Nerd Font";

const TRANSPARENCY_VALUES: &[(&str, &str)] = &[
    ("none", "1.0"),
    ("very_low", "0.95"),
    ("low", "0.90"),
    ("medium", "0.85"),
    ("high", "0.80"),
    ("very_high", "0.70"),
    ("super_high", "0.60"),
];

#[derive(Debug, Clone)]
pub struct TerminalMaterializationRequest {
    pub config_path: PathBuf,
    pub cursor_config_path: PathBuf,
    pub default_config_path: PathBuf,
    pub contract_path: PathBuf,
    pub runtime_dir: PathBuf,
    pub state_dir: PathBuf,
    pub terminals: Vec<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct TerminalGeneratedConfig {
    pub terminal: String,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct TerminalMaterializationData {
    pub generated: Vec<TerminalGeneratedConfig>,
    pub ghostty: Option<crate::ghostty_materialization::GhosttyMaterializationData>,
}

fn get_opacity_value(transparency: &str) -> &str {
    TRANSPARENCY_VALUES
        .iter()
        .find(|(k, _)| *k == transparency)
        .map(|(_, v)| *v)
        .unwrap_or("1.0")
}

fn get_terminal_title(terminal: &str) -> String {
    let name = match terminal {
        "ghostty" => "Ghostty",
        "yazelix_terminal" => "Yazelix Terminal",
        "kitty" => "Kitty",
        "wezterm" => "WezTerm",
        "ratty" => "Ratty",
        _ => terminal,
    };
    format!("Yazelix - {}", name)
}

fn get_terminal_override_path(
    config_dir: &Path,
    terminal: &str,
) -> Result<Option<PathBuf>, CoreError> {
    let Some(current) = user_config_paths::terminal_config(config_dir, terminal) else {
        return Ok(None);
    };
    let Some(legacy) = user_config_paths::legacy_terminal_config(config_dir, terminal) else {
        return Ok(None);
    };
    let path = user_config_paths::resolve_current_config_file(
        &current,
        &legacy,
        &format!("{terminal} terminal override"),
    )?;
    Ok(Some(path))
}

fn build_transparency(transparency: &str, format: &str, key: &str) -> String {
    let opacity = get_opacity_value(transparency);
    if transparency == "none" {
        match format {
            "ini" => format!("# {} = 0.9", key),
            "ini-space" => format!("# {} 0.9", key),
            "lua" => "-- config.window_background_opacity = 0.9".to_string(),
            "toml" => "# opacity = 0.9".to_string(),
            _ => "".to_string(),
        }
    } else {
        match format {
            "ini" => format!("{} = {}", key, opacity),
            "ini-space" => format!("{} {}", key, opacity),
            "lua" => format!("config.window_background_opacity = {}", opacity),
            "toml" => format!("opacity = {}", opacity),
            _ => "".to_string(),
        }
    }
}

fn generate_wezterm_config(transparency: &str) -> String {
    format!(
        r##"-- WezTerm configuration for Yazelix
local wezterm = require 'wezterm'
local config = wezterm.config_builder()

config.window_decorations = "NONE"
config.window_padding = {{ left = 0, right = 0, top = 10, bottom = 0 }}
config.color_scheme = '{}'

-- Hide tab bar (Zellij handles tabs)
config.enable_tab_bar = false

-- Transparency (configurable via settings.jsonc)
{}

-- Cursor trails: Not supported in WezTerm

return config"##,
        YAZELIX_THEME,
        build_transparency(transparency, "lua", ""),
    )
}

fn generate_ratty_config(transparency: &str) -> String {
    format!(
        r##"# Ratty configuration for Yazelix

[window]
width = 960
height = 620
scale_factor = 1.0

# Transparency (configurable via settings.jsonc)
{}

[terminal]
default_cols = 104
default_rows = 32
scrollback = 2000

[env]
TERM = "xterm-256color"

[font]
family = "{}"
style = "Regular"
size = 18

[cursor.model]
path = "CairoSpinyMouse.obj"
scale_factor = 6.0
brightness = 0.5
x_offset = 0.5
plane_offset = 18.0
visible = true

[cursor.animation]
spin_speed = 1.4
bob_speed = 2.2
bob_amplitude = 0.08

[bindings]
keys = [
  {{ key = "C", with = "Control | alt", action = "Copy" }},
  {{ key = "V", with = "Control | alt", action = "Paste" }},
  {{ key = "PageUp", with = "alt", action = "ScrollPageUp" }},
  {{ key = "PageDown", with = "alt", action = "ScrollPageDown" }},
  {{ key = "Up", with = "alt", action = "ScrollUp" }},
  {{ key = "Down", with = "alt", action = "ScrollDown" }},
  {{ key = "Equal", with = "Control", action = "IncreaseFontSize" }},
  {{ key = "Minus", with = "Control", action = "DecreaseFontSize" }},
  {{ key = "Digit0", with = "Control | alt", action = "ResetFontSize" }},
  {{ key = "Enter", with = "Control | alt", action = "Toggle3DMode" }},
  {{ key = "M", with = "Control | alt", action = "ToggleMobiusMode" }},
  {{ key = "Up", with = "Control | alt", action = "IncreaseWarp" }},
  {{ key = "Down", with = "Control | alt", action = "DecreaseWarp" }},
]

[theme]
foreground = "#dcd7ba"
background = "#1f1f28"
cursor = "#7e9cd8"

[theme.normal]
black = "#000000"
red = "#cd3131"
green = "#0dbc79"
yellow = "#e5e510"
blue = "#2472c8"
magenta = "#bc3fbc"
cyan = "#11a8cd"
white = "#e5e5e5"

[theme.bright]
black = "#666666"
red = "#f14c4c"
green = "#23d18b"
yellow = "#f5f543"
blue = "#3b8eea"
magenta = "#d670d6"
cyan = "#29b8db"
white = "#ffffff"
"##,
        build_transparency(transparency, "toml", ""),
        FONT_FIRACODE,
    )
}

fn generate_yazelix_terminal_config(
    runtime_dir: &Path,
    transparency: &str,
) -> Result<String, CoreError> {
    let package_config = runtime_dir
        .join("share")
        .join("yazelix-terminal")
        .join("config.toml");
    let raw = fs::read_to_string(&package_config).map_err(|source| {
        CoreError::io(
            "read_yazelix_terminal_package_config",
            "Could not read the packaged Yazelix Terminal config",
            "Reinstall the Yazelix runtime so the yazelix-terminal child package is present.",
            package_config.to_string_lossy(),
            source,
        )
    })?;
    let mut table = toml::from_str::<toml::Table>(&raw).map_err(|source| {
        CoreError::classified(
            crate::bridge::ErrorClass::Runtime,
            "parse_yazelix_terminal_package_config",
            format!(
                "The packaged Yazelix Terminal config at {} is not valid TOML.",
                package_config.display()
            ),
            "Reinstall the Yazelix runtime or rebuild it from a valid yazelix-terminal package.",
            serde_json::json!({ "error": source.to_string() }),
        )
    })?;
    let opacity = get_opacity_value(transparency)
        .parse::<f64>()
        .map_err(|source| {
            CoreError::classified(
                crate::bridge::ErrorClass::Internal,
                "parse_yazelix_terminal_opacity",
                format!(
                    "Could not parse Yazelix Terminal opacity for transparency '{transparency}'."
                ),
                "Report this Yazelix bug with the active settings.jsonc.",
                serde_json::json!({ "error": source.to_string() }),
            )
        })?;
    let window = table
        .entry("window")
        .or_insert_with(|| toml::Value::Table(toml::map::Map::new()))
        .as_table_mut()
        .ok_or_else(|| {
            CoreError::classified(
                crate::bridge::ErrorClass::Runtime,
                "invalid_yazelix_terminal_window_config",
                format!(
                    "The packaged Yazelix Terminal config at {} has a non-table [window] value.",
                    package_config.display()
                ),
                "Reinstall the Yazelix runtime or rebuild it from a valid yazelix-terminal package.",
                serde_json::json!({}),
            )
        })?;
    window.insert("opacity".to_string(), toml::Value::Float(opacity));
    window.insert(
        "opacity-cells".to_string(),
        toml::Value::Boolean(transparency != "none"),
    );
    toml::to_string_pretty(&toml::Value::Table(table)).map_err(|source| {
        CoreError::classified(
            crate::bridge::ErrorClass::Internal,
            "render_yazelix_terminal_config",
            "Could not render the generated Yazelix Terminal config.",
            "Report this Yazelix bug with the current settings.jsonc.",
            serde_json::json!({ "error": source.to_string() }),
        )
    })
}

fn build_kitty_cursor(kitty_enable_cursor: bool) -> String {
    if kitty_enable_cursor {
        "cursor_shape block\ncursor_trail 3\ncursor_trail_decay 0.1 0.4".to_string()
    } else {
        "# cursor_trail 0  # disabled in settings.jsonc".to_string()
    }
}

fn generate_kitty_config(
    transparency: &str,
    kitty_enable_cursor: bool,
    override_path: Option<&Path>,
) -> String {
    let override_section = match override_path {
        Some(path) if path.exists() => {
            format!(
                "# Personal Yazelix Kitty overrides\ninclude {}",
                path.display()
            )
        }
        Some(path) => {
            format!(
                "# Personal Yazelix Kitty overrides (optional, user-owned)\n# Create {} if you want terminal-native Kitty tweaks.",
                path.display()
            )
        }
        None => "# Personal Yazelix Kitty overrides (optional, user-owned)".to_string(),
    };

    format!(
        r##"# Kitty configuration for Yazelix

hide_window_decorations yes
window_padding_width 2
include {}.conf
window_title {}

# Transparency (configurable via settings.jsonc)
{}

# Font settings
font_family      {}
bold_font        auto
italic_font      auto
bold_italic_font auto

# Performance
repaint_delay 10
input_delay 3
sync_to_monitor yes

# Cursor trail effect (configurable via settings.jsonc)
{}

# Personal Yazelix Kitty overrides
{}"##,
        YAZELIX_THEME,
        get_terminal_title("kitty"),
        build_transparency(transparency, "ini-space", "background_opacity"),
        FONT_FIRACODE,
        build_kitty_cursor(kitty_enable_cursor),
        override_section,
    )
}

pub fn generate_terminal_materialization(
    request: &TerminalMaterializationRequest,
) -> Result<TerminalMaterializationData, CoreError> {
    let normalized = normalize_config(&NormalizeConfigRequest {
        config_path: request.config_path.clone(),
        default_config_path: request.default_config_path.clone(),
        contract_path: request.contract_path.clone(),
        include_missing: true,
    })?;

    let config = &normalized.normalized_config;
    let transparency = config
        .get("transparency")
        .and_then(|v| v.as_str())
        .unwrap_or("none");

    let config_dir = config_dir_from_env()?;
    crate::managed_user_config_stubs::ensure_terminal_override_stubs(
        &config_dir,
        &request.terminals,
    )?;
    let cursors_enabled = runtime_component_enabled(&request.runtime_dir, "cursors")?;
    let kitty_enable_cursor = if cursors_enabled {
        CursorRegistry::load(&request.cursor_config_path)?
            .settings
            .kitty_enable_cursor
    } else {
        false
    };
    let generated_dir = request.state_dir.join("configs").join("terminal_emulators");

    let mut generated = Vec::new();
    let mut ghostty_data = None;

    for terminal in &request.terminals {
        match terminal.as_str() {
            "ghostty" => {
                let ghostty_request = GhosttyMaterializationRequest {
                    runtime_dir: request.runtime_dir.clone(),
                    config_dir: config_dir.clone(),
                    state_dir: request.state_dir.clone(),
                    transparency: transparency.to_string(),
                    cursor_config_path: request.cursor_config_path.clone(),
                };
                let data = generate_ghostty_materialization(&ghostty_request)?;
                let path = data.generated_path.clone();
                ghostty_data = Some(data);
                generated.push(TerminalGeneratedConfig {
                    terminal: "ghostty".to_string(),
                    path,
                });
            }
            "wezterm" => {
                let wezterm_dir = generated_dir.join("wezterm");
                fs::create_dir_all(&wezterm_dir).map_err(|source| {
                    CoreError::io(
                        "create_wezterm_dir",
                        "Could not create WezTerm output directory",
                        "Check permissions for the Yazelix state directory.",
                        wezterm_dir.to_string_lossy(),
                        source,
                    )
                })?;
                let path = wezterm_dir.join(".wezterm.lua");
                write_text_atomic(&path, &generate_wezterm_config(transparency))?;
                generated.push(TerminalGeneratedConfig {
                    terminal: "wezterm".to_string(),
                    path: path.to_string_lossy().into_owned(),
                });
            }
            "ratty" => {
                let ratty_dir = generated_dir.join("ratty");
                fs::create_dir_all(&ratty_dir).map_err(|source| {
                    CoreError::io(
                        "create_ratty_dir",
                        "Could not create Ratty output directory",
                        "Check permissions for the Yazelix state directory.",
                        ratty_dir.to_string_lossy(),
                        source,
                    )
                })?;
                let path = ratty_dir.join("ratty.toml");
                write_text_atomic(&path, &generate_ratty_config(transparency))?;
                generated.push(TerminalGeneratedConfig {
                    terminal: "ratty".to_string(),
                    path: path.to_string_lossy().into_owned(),
                });
            }
            "yazelix_terminal" => {
                let yazelix_terminal_dir = generated_dir.join("yazelix_terminal");
                fs::create_dir_all(&yazelix_terminal_dir).map_err(|source| {
                    CoreError::io(
                        "create_yazelix_terminal_dir",
                        "Could not create Yazelix Terminal output directory",
                        "Check permissions for the Yazelix state directory.",
                        yazelix_terminal_dir.to_string_lossy(),
                        source,
                    )
                })?;
                let path = yazelix_terminal_dir.join("config.toml");
                write_text_atomic(
                    &path,
                    &generate_yazelix_terminal_config(&request.runtime_dir, transparency)?,
                )?;
                generated.push(TerminalGeneratedConfig {
                    terminal: "yazelix_terminal".to_string(),
                    path: path.to_string_lossy().into_owned(),
                });
            }
            "kitty" => {
                let kitty_dir = generated_dir.join("kitty");
                fs::create_dir_all(&kitty_dir).map_err(|source| {
                    CoreError::io(
                        "create_kitty_dir",
                        "Could not create Kitty output directory",
                        "Check permissions for the Yazelix state directory.",
                        kitty_dir.to_string_lossy(),
                        source,
                    )
                })?;
                let override_path = get_terminal_override_path(&config_dir, "kitty")?;
                let path = kitty_dir.join("kitty.conf");
                write_text_atomic(
                    &path,
                    &generate_kitty_config(
                        transparency,
                        kitty_enable_cursor,
                        override_path.as_deref(),
                    ),
                )?;
                generated.push(TerminalGeneratedConfig {
                    terminal: "kitty".to_string(),
                    path: path.to_string_lossy().into_owned(),
                });
            }
            _ => {}
        }
    }

    Ok(TerminalMaterializationData {
        generated,
        ghostty: ghostty_data,
    })
}
