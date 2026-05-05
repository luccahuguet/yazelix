use crate::bridge::CoreError;
use crate::config_normalize::{NormalizeConfigRequest, normalize_config};
use crate::control_plane::config_dir_from_env;
use crate::ghostty_cursor_registry::CursorRegistry;
use crate::ghostty_materialization::{
    GhosttyMaterializationRequest, generate_ghostty_materialization,
};
use crate::user_config_paths;
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};

const YAZELIX_WINDOW_CLASS: &str = "com.yazelix.Yazelix";
const YAZELIX_X11_INSTANCE: &str = "yazelix";
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
        "kitty" => "Kitty",
        "wezterm" => "WezTerm",
        "alacritty" => "Alacritty",
        "foot" => "Foot",
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
    let path = user_config_paths::resolve_flat_config_file(
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

fn generate_alacritty_base_config(transparency: &str) -> String {
    format!(
        r##"# Alacritty base configuration for Yazelix

[env]
TERM = "xterm-256color"

[window]
decorations = "None"
padding = {{ x = 0, y = 10 }}
class = {{ instance = "{}", general = "{}" }}
title = "{}"

# Transparency (configurable via settings.jsonc)
{}

# Cursor trails: Not supported in Alacritty

[font]
normal = {{ family = "{}", style = "Regular" }}
bold = {{ family = "{}", style = "Bold" }}
italic = {{ family = "{}", style = "Italic" }}
bold_italic = {{ family = "{}", style = "Bold Italic" }}
builtin_box_drawing = true
size = 12

[colors]
primary = {{ background = "#000000", foreground = "#ffffff" }}"##,
        YAZELIX_X11_INSTANCE,
        YAZELIX_WINDOW_CLASS,
        get_terminal_title("alacritty"),
        build_transparency(transparency, "toml", ""),
        FONT_FIRACODE,
        FONT_FIRACODE,
        FONT_FIRACODE,
        FONT_FIRACODE,
    )
}

fn generate_alacritty_config(base_path: &Path, override_path: Option<&Path>) -> String {
    let imports: Vec<String> = match override_path {
        Some(path) if path.exists() => {
            vec![
                format!("\"{}\"", base_path.display()),
                format!("\"{}\"", path.display()),
            ]
        }
        _ => {
            vec![format!("\"{}\"", base_path.display())]
        }
    };

    let override_comment = match override_path {
        Some(path) => format!(
            "# Create {} if you want terminal-native Alacritty tweaks.",
            path.display()
        ),
        None => {
            "# Create a user override if you want terminal-native Alacritty tweaks.".to_string()
        }
    };

    format!(
        r##"# Alacritty configuration entrypoint for Yazelix

[general]
import = [{}]

# Personal Yazelix Alacritty overrides (optional, user-owned)
{}
"##,
        imports.join(", "),
        override_comment,
    )
}

fn generate_foot_config(transparency: &str, override_path: Option<PathBuf>) -> String {
    let override_include = match override_path {
        Some(path) => format!(
            r#"
# Personal Yazelix Foot overrides (optional, user-owned)
[main]
include={}
"#,
            path.display()
        ),
        None => r#"
# Create a user override if you want terminal-native Foot tweaks.
"#
        .to_string(),
    };

    format!(
        r##"# Foot configuration for Yazelix

[colors-dark]
# Transparency (configurable via settings.jsonc)
{}

[main]
app-id={}
title={}
locked-title=yes
font={}:size=12
pad=6x6 center

[csd]
preferred=client
size=0
border-width=0

[cursor]
style=block
blink=false
{}"##,
        build_transparency(transparency, "ini", "alpha"),
        YAZELIX_WINDOW_CLASS,
        get_terminal_title("foot"),
        FONT_FIRACODE,
        override_include,
    )
}

pub fn generate_terminal_materialization(
    request: &TerminalMaterializationRequest,
) -> Result<TerminalMaterializationData, CoreError> {
    let normalized = normalize_config(&NormalizeConfigRequest {
        config_path: request.config_path.clone(),
        default_config_path: request.default_config_path.clone(),
        contract_path: request.contract_path.clone(),
        include_missing: false,
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
    let cursor_config_path = request.config_path.clone();
    let cursor_registry = CursorRegistry::load(&cursor_config_path)?;
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
                    cursor_config_path: cursor_config_path.clone(),
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
                fs::write(&path, generate_wezterm_config(transparency)).map_err(|source| {
                    CoreError::io(
                        "write_wezterm_config",
                        "Could not write WezTerm config",
                        "Check permissions for the Yazelix state directory.",
                        path.to_string_lossy(),
                        source,
                    )
                })?;
                generated.push(TerminalGeneratedConfig {
                    terminal: "wezterm".to_string(),
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
                fs::write(
                    &path,
                    generate_kitty_config(
                        transparency,
                        cursor_registry.settings.kitty_enable_cursor,
                        override_path.as_deref(),
                    ),
                )
                .map_err(|source| {
                    CoreError::io(
                        "write_kitty_config",
                        "Could not write Kitty config",
                        "Check permissions for the Yazelix state directory.",
                        path.to_string_lossy(),
                        source,
                    )
                })?;
                generated.push(TerminalGeneratedConfig {
                    terminal: "kitty".to_string(),
                    path: path.to_string_lossy().into_owned(),
                });
            }
            "alacritty" => {
                let alacritty_dir = generated_dir.join("alacritty");
                fs::create_dir_all(&alacritty_dir).map_err(|source| {
                    CoreError::io(
                        "create_alacritty_dir",
                        "Could not create Alacritty output directory",
                        "Check permissions for the Yazelix state directory.",
                        alacritty_dir.to_string_lossy(),
                        source,
                    )
                })?;
                let base_path = alacritty_dir.join("alacritty_base.toml");
                fs::write(&base_path, generate_alacritty_base_config(transparency)).map_err(
                    |source| {
                        CoreError::io(
                            "write_alacritty_base",
                            "Could not write Alacritty base config",
                            "Check permissions for the Yazelix state directory.",
                            base_path.to_string_lossy(),
                            source,
                        )
                    },
                )?;
                let override_path = get_terminal_override_path(&config_dir, "alacritty")?;
                let entry_path = alacritty_dir.join("alacritty.toml");
                fs::write(
                    &entry_path,
                    generate_alacritty_config(&base_path, override_path.as_deref()),
                )
                .map_err(|source| {
                    CoreError::io(
                        "write_alacritty_config",
                        "Could not write Alacritty config",
                        "Check permissions for the Yazelix state directory.",
                        entry_path.to_string_lossy(),
                        source,
                    )
                })?;
                generated.push(TerminalGeneratedConfig {
                    terminal: "alacritty".to_string(),
                    path: entry_path.to_string_lossy().into_owned(),
                });
            }
            "foot" => {
                let foot_dir = generated_dir.join("foot");
                fs::create_dir_all(&foot_dir).map_err(|source| {
                    CoreError::io(
                        "create_foot_dir",
                        "Could not create Foot output directory",
                        "Check permissions for the Yazelix state directory.",
                        foot_dir.to_string_lossy(),
                        source,
                    )
                })?;
                let override_path = get_terminal_override_path(&config_dir, "foot")?;
                let path = foot_dir.join("foot.ini");
                fs::write(&path, generate_foot_config(transparency, override_path)).map_err(
                    |source| {
                        CoreError::io(
                            "write_foot_config",
                            "Could not write Foot config",
                            "Check permissions for the Yazelix state directory.",
                            path.to_string_lossy(),
                            source,
                        )
                    },
                )?;
                generated.push(TerminalGeneratedConfig {
                    terminal: "foot".to_string(),
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
