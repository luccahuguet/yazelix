use crate::appearance_mode::{
    APPEARANCE_MODE_AUTO, APPEARANCE_MODE_DARK, APPEARANCE_MODE_LIGHT, WEZTERM_THEME_DARK,
    WEZTERM_THEME_LIGHT, appearance_mode_from_config, auto_mode, static_light_mode, wezterm_theme,
};
use crate::atomic_fs::{copy_dir_all, write_text_atomic};
use crate::bridge::CoreError;
use crate::config_normalize::{NormalizeConfigRequest, normalize_config};
use crate::control_plane::config_dir_from_env;
use crate::ghostty_cursor_registry::{CursorRegistry, YazelixCursorRegistryExt};
use crate::ghostty_materialization::{
    GhosttyMaterializationData, GhosttyMaterializationRequest, generate_ghostty_materialization,
};
use crate::runtime_component_enabled;
use crate::terminal_cursor_materialization::{
    TerminalCursorMaterializationData, TerminalCursorMaterializationRequest, TerminalCursorState,
    cursor_shader_paths_for_state, generate_terminal_cursor_materialization,
};
use crate::terminal_variant::terminal_window_title;
use crate::user_config_paths;
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};

const YAZELIX_THEME: &str = "Abernathy";
const ABERNATHY_BACKGROUND: &str = "#111416";
const ABERNATHY_FOREGROUND: &str = "#eeeeec";
const CATPPUCCIN_LATTE_BACKGROUND: &str = "#eff1f5";
const CATPPUCCIN_LATTE_FOREGROUND: &str = "#4c4f69";
const FONT_FIRACODE: &str = "FiraCode Nerd Font";
const FONT_JETBRAINS_MONO: &str = "JetBrains Mono";
const FONT_SYMBOLS_NERD_MONO: &str = "Symbols Nerd Font Mono";
const FONT_SYMBOLS_NERD: &str = "Symbols Nerd Font";
const FONT_NOTO_COLOR_EMOJI: &str = "Noto Color Emoji";
const MARS_FONT_SIZE: f64 = 16.0;
const MARS_LINE_HEIGHT: f64 = 1.12;
const RIO_FONT_ROOT: &str = "share/yazelix/rio_fonts";
const RIO_FIRA_CODE_FONT_DIR: &str = "fira_code_nerd";
const RIO_SYMBOLS_FONT_DIR: &str = "symbols_nerd";
const RIO_EMOJI_FONT_DIR: &str = "noto_color_emoji";
pub(crate) const MARS_EMOJI_FONT_ENV: &str = "MARS_EMOJI_FONT";
pub(crate) const MARS_EMOJI_FONT_SOURCE_ENV: &str = "MARS_EMOJI_FONT_SOURCE";
pub(crate) const MARS_EMOJI_ENV_KEYS: [&str; 2] = [MARS_EMOJI_FONT_ENV, MARS_EMOJI_FONT_SOURCE_ENV];
const MARS_EMOJI_FONT_SOURCE_HOME_MANAGER: &str = "home-manager";
const TERMINAL_DARK_COLOR_PALETTE: &[(&str, &str)] = &[
    ("background", ABERNATHY_BACKGROUND),
    ("foreground", ABERNATHY_FOREGROUND),
    ("black", "#000000"),
    ("red", "#cd0000"),
    ("green", "#00cd00"),
    ("yellow", "#cdcd00"),
    ("blue", "#1093f5"),
    ("magenta", "#cd00cd"),
    ("cyan", "#00cdcd"),
    ("white", "#faebd7"),
    ("light-black", "#404040"),
    ("light-red", "#ff0000"),
    ("light-green", "#00ff00"),
    ("light-yellow", "#ffff00"),
    ("light-blue", "#11b5f6"),
    ("light-magenta", "#ff00ff"),
    ("light-cyan", "#00ffff"),
    ("light-white", "#ffffff"),
];
const CATPPUCCIN_LATTE_COLOR_PALETTE: &[(&str, &str)] = &[
    ("background", CATPPUCCIN_LATTE_BACKGROUND),
    ("foreground", CATPPUCCIN_LATTE_FOREGROUND),
    ("black", "#5c5f77"),
    ("red", "#d20f39"),
    ("green", "#40a02b"),
    ("yellow", "#df8e1d"),
    ("blue", "#1e66f5"),
    ("magenta", "#ea76cb"),
    ("cyan", "#179299"),
    ("white", "#acb0be"),
    ("light-black", "#6c6f85"),
    ("light-red", "#d20f39"),
    ("light-green", "#40a02b"),
    ("light-yellow", "#df8e1d"),
    ("light-blue", "#1e66f5"),
    ("light-magenta", "#ea76cb"),
    ("light-cyan", "#179299"),
    ("light-white", "#bcc0cc"),
];
const FOOT_DARK_COLOR_PALETTE: &[(&str, &str)] = &[
    ("background", "#1f1f28"),
    ("foreground", "#dcd7ba"),
    ("black", "#000000"),
    ("red", "#cd3131"),
    ("green", "#0dbc79"),
    ("yellow", "#e5e510"),
    ("blue", "#2472c8"),
    ("magenta", "#bc3fbc"),
    ("cyan", "#11a8cd"),
    ("white", "#e5e5e5"),
    ("light-black", "#666666"),
    ("light-red", "#f14c4c"),
    ("light-green", "#23d18b"),
    ("light-yellow", "#f5f543"),
    ("light-blue", "#3b8eea"),
    ("light-magenta", "#d670d6"),
    ("light-cyan", "#29b8db"),
    ("light-white", "#ffffff"),
];

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
    pub mars_profile: MarsProfile,
    pub mars_emoji_font: Option<MarsEmojiFont>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarsProfile {
    Full,
    Baseline,
    Shaders,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarsEmojiFont {
    Noto,
    Twitter,
    SerenityOs,
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
    pub cursor: Option<TerminalCursorMaterializationData>,
}

fn get_opacity_value(transparency: &str) -> &str {
    TRANSPARENCY_VALUES
        .iter()
        .find(|(k, _)| *k == transparency)
        .map(|(_, v)| *v)
        .unwrap_or("1.0")
}

fn get_terminal_title(terminal: &str) -> String {
    terminal_window_title(terminal, None)
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

fn terminal_palette_for_appearance(
    appearance_mode: &str,
) -> &'static [(&'static str, &'static str)] {
    if static_light_mode(appearance_mode) {
        CATPPUCCIN_LATTE_COLOR_PALETTE
    } else {
        TERMINAL_DARK_COLOR_PALETTE
    }
}

fn palette_color(palette: &'static [(&'static str, &'static str)], key: &str) -> &'static str {
    palette
        .iter()
        .find(|(candidate, _)| *candidate == key)
        .map(|(_, value)| *value)
        .expect("terminal palette must define every generated color key")
}

fn foot_color(value: &str) -> &str {
    value.strip_prefix('#').unwrap_or(value)
}

fn generate_wezterm_color_scheme(appearance_mode: &str) -> String {
    if auto_mode(appearance_mode) {
        return format!(
            r##"local function yazelix_color_scheme_for_appearance(appearance)
  if appearance:find("Dark") then
    return '{}'
  end
  return '{}'
end
config.color_scheme = yazelix_color_scheme_for_appearance(wezterm.gui.get_appearance())"##,
            WEZTERM_THEME_DARK, WEZTERM_THEME_LIGHT
        );
    }

    format!("config.color_scheme = '{}'", wezterm_theme(appearance_mode))
}

fn generate_wezterm_config(transparency: &str, appearance_mode: &str) -> String {
    format!(
        r##"-- WezTerm configuration for Yazelix
local wezterm = require 'wezterm'
local config = wezterm.config_builder()

config.window_decorations = "NONE"
config.window_padding = {{ left = 0, right = 0, top = 10, bottom = 0 }}
{}

-- Hide tab bar (Zellij handles tabs)
config.enable_tab_bar = false

-- Scrollback: Zellij handles pane history inside Yazelix
config.scrollback_lines = 0

-- Transparency (configurable via settings.jsonc)
{}

-- Cursor trails: Not supported in WezTerm

return config"##,
        generate_wezterm_color_scheme(appearance_mode),
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
scrollback = 0

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

fn generate_foot_config(transparency: &str, appearance_mode: &str) -> String {
    let alpha = get_opacity_value(transparency);
    let initial_color_theme = if static_light_mode(appearance_mode) {
        "light"
    } else {
        "dark"
    };
    let dark = FOOT_DARK_COLOR_PALETTE;
    let light = CATPPUCCIN_LATTE_COLOR_PALETTE;
    format!(
        r##"# Foot configuration for Yazelix

term=xterm-256color
font={}:size=14
initial-color-theme={}

[cursor]
style=block

[scrollback]
# Zellij handles pane history inside Yazelix.
lines=0

[csd]
# Compositor rules can still force server-side decorations.
preferred=none
size=0

[colors-dark]
background={}
foreground={}
alpha={}
regular0={}
regular1={}
regular2={}
regular3={}
regular4={}
regular5={}
regular6={}
regular7={}
bright0={}
bright1={}
bright2={}
bright3={}
bright4={}
bright5={}
bright6={}
bright7={}

[colors-light]
background={}
foreground={}
alpha={}
regular0={}
regular1={}
regular2={}
regular3={}
regular4={}
regular5={}
regular6={}
regular7={}
bright0={}
bright1={}
bright2={}
bright3={}
bright4={}
bright5={}
bright6={}
bright7={}
"##,
        FONT_FIRACODE,
        initial_color_theme,
        foot_color(palette_color(dark, "background")),
        foot_color(palette_color(dark, "foreground")),
        alpha,
        foot_color(palette_color(dark, "black")),
        foot_color(palette_color(dark, "red")),
        foot_color(palette_color(dark, "green")),
        foot_color(palette_color(dark, "yellow")),
        foot_color(palette_color(dark, "blue")),
        foot_color(palette_color(dark, "magenta")),
        foot_color(palette_color(dark, "cyan")),
        foot_color(palette_color(dark, "white")),
        foot_color(palette_color(dark, "light-black")),
        foot_color(palette_color(dark, "light-red")),
        foot_color(palette_color(dark, "light-green")),
        foot_color(palette_color(dark, "light-yellow")),
        foot_color(palette_color(dark, "light-blue")),
        foot_color(palette_color(dark, "light-magenta")),
        foot_color(palette_color(dark, "light-cyan")),
        foot_color(palette_color(dark, "light-white")),
        foot_color(palette_color(light, "background")),
        foot_color(palette_color(light, "foreground")),
        alpha,
        foot_color(palette_color(light, "black")),
        foot_color(palette_color(light, "red")),
        foot_color(palette_color(light, "green")),
        foot_color(palette_color(light, "yellow")),
        foot_color(palette_color(light, "blue")),
        foot_color(palette_color(light, "magenta")),
        foot_color(palette_color(light, "cyan")),
        foot_color(palette_color(light, "white")),
        foot_color(palette_color(light, "light-black")),
        foot_color(palette_color(light, "light-red")),
        foot_color(palette_color(light, "light-green")),
        foot_color(palette_color(light, "light-yellow")),
        foot_color(palette_color(light, "light-blue")),
        foot_color(palette_color(light, "light-magenta")),
        foot_color(palette_color(light, "light-cyan")),
        foot_color(palette_color(light, "light-white")),
    )
}

fn generate_rio_config(runtime_dir: &Path, transparency: &str, appearance_mode: &str) -> String {
    let opacity = get_opacity_value(transparency);
    let palette = terminal_palette_for_appearance(appearance_mode);
    let fira_code_dir = rio_font_dir(runtime_dir, RIO_FIRA_CODE_FONT_DIR);
    let symbols_dir = rio_font_dir(runtime_dir, RIO_SYMBOLS_FONT_DIR);
    let emoji_dir = rio_font_dir(runtime_dir, RIO_EMOJI_FONT_DIR);
    format!(
        r##"# Rio configuration for Yazelix

confirm-before-quit = true
scrollback-history-limit = 0

[effects]
trail-cursor = true

[title]
placeholder = "{}"
content = "{{{{ TITLE || RELATIVE_PATH }}}}"

[window]
width = 960
height = 620
decorations = "Disabled"
opacity = {}
opacity-cells = true

[fonts]
family = "{}"
size = 18.0
additional-dirs = ["{}", "{}", "{}"]
extras = [{{ family = "{}" }}, {{ family = "{}" }}]
emoji = {{ family = "{}" }}

[colors]
background = "{}"
foreground = "{}"
black = "{}"
red = "{}"
green = "{}"
yellow = "{}"
blue = "{}"
magenta = "{}"
cyan = "{}"
white = "{}"
light-black = "{}"
light-red = "{}"
light-green = "{}"
light-yellow = "{}"
light-blue = "{}"
light-magenta = "{}"
light-cyan = "{}"
light-white = "{}"

[navigation]
mode = "Plain"
"##,
        get_terminal_title("rio"),
        opacity,
        FONT_FIRACODE,
        fira_code_dir.to_string_lossy(),
        symbols_dir.to_string_lossy(),
        emoji_dir.to_string_lossy(),
        FONT_SYMBOLS_NERD_MONO,
        FONT_SYMBOLS_NERD,
        FONT_NOTO_COLOR_EMOJI,
        palette_color(palette, "background"),
        palette_color(palette, "foreground"),
        palette_color(palette, "black"),
        palette_color(palette, "red"),
        palette_color(palette, "green"),
        palette_color(palette, "yellow"),
        palette_color(palette, "blue"),
        palette_color(palette, "magenta"),
        palette_color(palette, "cyan"),
        palette_color(palette, "white"),
        palette_color(palette, "light-black"),
        palette_color(palette, "light-red"),
        palette_color(palette, "light-green"),
        palette_color(palette, "light-yellow"),
        palette_color(palette, "light-blue"),
        palette_color(palette, "light-magenta"),
        palette_color(palette, "light-cyan"),
        palette_color(palette, "light-white"),
    )
}

fn rio_font_dir(runtime_dir: &Path, font_dir: &str) -> PathBuf {
    runtime_dir.join(RIO_FONT_ROOT).join(font_dir)
}

pub fn mars_profile_from_env() -> Result<MarsProfile, CoreError> {
    let raw = std::env::var("MARS_PROFILE")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .or_else(|| {
            std::env::var("MARS_EFFECTS")
                .ok()
                .filter(|value| !value.trim().is_empty())
        })
        .unwrap_or_else(|| "full".to_string());
    parse_mars_profile(&raw)
}

pub fn mars_emoji_font_override_from_env() -> Result<Option<MarsEmojiFont>, CoreError> {
    let Some(source) = std::env::var(MARS_EMOJI_FONT_SOURCE_ENV)
        .ok()
        .filter(|value| !value.trim().is_empty())
    else {
        return Ok(None);
    };

    if source.trim() != MARS_EMOJI_FONT_SOURCE_HOME_MANAGER {
        return Err(CoreError::usage(format!(
            "Unsupported {MARS_EMOJI_FONT_SOURCE_ENV}: {source}. Use {MARS_EMOJI_FONT_SOURCE_HOME_MANAGER}."
        )));
    }

    let Some(raw) = std::env::var(MARS_EMOJI_FONT_ENV)
        .ok()
        .filter(|value| !value.trim().is_empty())
    else {
        return Err(CoreError::usage(format!(
            "{MARS_EMOJI_FONT_SOURCE_ENV}={MARS_EMOJI_FONT_SOURCE_HOME_MANAGER} requires {MARS_EMOJI_FONT_ENV}."
        )));
    };
    parse_mars_emoji_font(&raw).map(Some)
}

fn parse_mars_profile(raw: &str) -> Result<MarsProfile, CoreError> {
    match raw.trim() {
        "" | "full" | "Full" | "FULL" | "effects" | "Effects" | "EFFECTS" | "default"
        | "Default" | "DEFAULT" => Ok(MarsProfile::Full),
        "baseline" | "Baseline" | "BASELINE" | "no-effects" | "no_effects" | "none" | "None"
        | "NONE" | "0" => Ok(MarsProfile::Baseline),
        "shader" | "Shader" | "SHADER" | "shaders" | "Shaders" | "SHADERS" | "cursor-shaders"
        | "cursor_shaders" | "ghostty-shaders" | "ghostty_shaders" => Ok(MarsProfile::Shaders),
        other => Err(CoreError::usage(format!(
            "Unsupported MARS_PROFILE/MARS_EFFECTS: {other}. Use full, default, baseline, no-effects, shaders, none, or 0."
        ))),
    }
}

fn parse_mars_emoji_font(raw: &str) -> Result<MarsEmojiFont, CoreError> {
    match raw.trim() {
        "" | "noto" | "Noto" | "NOTO" | "default" | "Default" | "DEFAULT" => {
            Ok(MarsEmojiFont::Noto)
        }
        "twitter" | "Twitter" | "TWITTER" | "twemoji" | "Twemoji" | "TWEMOJI" => {
            Ok(MarsEmojiFont::Twitter)
        }
        "serenityos" | "SerenityOS" | "SERENITYOS" | "serenity" | "Serenity" | "SERENITY"
        | "serenity-os" | "Serenity-OS" | "SERENITY-OS" => Ok(MarsEmojiFont::SerenityOs),
        other => Err(CoreError::usage(format!(
            "Unsupported {MARS_EMOJI_FONT_ENV}: {other}. Use noto, twitter, or serenityos."
        ))),
    }
}

fn mars_emoji_font_from_config(
    config: &serde_json::Map<String, serde_json::Value>,
    env_override: Option<MarsEmojiFont>,
) -> Result<MarsEmojiFont, CoreError> {
    if let Some(emoji_font) = env_override {
        return Ok(emoji_font);
    }
    let raw = config
        .get("terminal_emoji_style")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("noto");
    parse_mars_emoji_font(raw)
}

fn mars_package_config_path(
    runtime_dir: &Path,
    profile: MarsProfile,
    emoji_font: MarsEmojiFont,
) -> Result<PathBuf, CoreError> {
    let metadata_path = runtime_dir
        .join("share")
        .join("mars")
        .join("package-metadata.json");
    let raw = fs::read_to_string(&metadata_path).map_err(|source| {
        CoreError::io(
            "read_mars_package_metadata",
            "Could not read the packaged Mars metadata",
            "Reinstall the Yazelix runtime so the Mars child package metadata is present.",
            metadata_path.to_string_lossy(),
            source,
        )
    })?;
    let metadata = serde_json::from_str::<serde_json::Value>(&raw).map_err(|source| {
        CoreError::classified(
            crate::bridge::ErrorClass::Runtime,
            "parse_mars_package_metadata",
            format!(
                "The packaged Mars metadata at {} is not valid JSON.",
                metadata_path.display()
            ),
            "Reinstall the Yazelix runtime or rebuild it from a valid Mars package.",
            serde_json::json!({ "error": source.to_string() }),
        )
    })?;
    let emoji_key = match emoji_font {
        MarsEmojiFont::Noto => "noto",
        MarsEmojiFont::Twitter => "twitter",
        MarsEmojiFont::SerenityOs => "serenityos",
    };
    let profile_key = match profile {
        MarsProfile::Full => "full",
        MarsProfile::Baseline => "baseline",
        MarsProfile::Shaders => "shaders",
    };
    let config_root = metadata
        .get("emoji_fonts")
        .and_then(serde_json::Value::as_object)
        .and_then(|fonts| fonts.get(emoji_key))
        .and_then(|font| font.get("config_roots"))
        .and_then(serde_json::Value::as_object)
        .and_then(|roots| roots.get(profile_key))
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| {
            CoreError::classified(
                crate::bridge::ErrorClass::Runtime,
                "missing_mars_package_config_root",
                format!(
                    "The packaged Mars metadata at {} does not declare the {emoji_key}/{profile_key} config root.",
                    metadata_path.display()
                ),
                "Reinstall Yazelix with a Mars package that advertises its profile config roots.",
                serde_json::json!({
                    "metadata_path": metadata_path.to_string_lossy(),
                    "emoji_font": emoji_key,
                    "profile": profile_key,
                }),
            )
        })?;
    let config_root = Path::new(config_root);
    if config_root.is_absolute() {
        return Err(CoreError::classified(
            crate::bridge::ErrorClass::Runtime,
            "absolute_mars_package_config_root",
            format!(
                "The packaged Mars metadata at {} declares an absolute {emoji_key}/{profile_key} config root.",
                metadata_path.display()
            ),
            "Rebuild Mars so package metadata uses runtime-relative config roots.",
            serde_json::json!({
                "metadata_path": metadata_path.to_string_lossy(),
                "config_root": config_root.to_string_lossy(),
            }),
        ));
    }
    Ok(runtime_dir.join(config_root).join("config.toml"))
}

fn shader_paths_to_toml(shader_paths: &[String]) -> toml::Value {
    toml::Value::Array(
        shader_paths
            .iter()
            .map(|path| toml::Value::String(path.clone()))
            .collect(),
    )
}

fn mars_config_table_mut<'a>(
    table: &'a mut toml::Table,
    section: &'static str,
    error_code: &'static str,
    package_config: &Path,
) -> Result<&'a mut toml::Table, CoreError> {
    table
        .entry(section)
        .or_insert_with(|| toml::Value::Table(toml::map::Map::new()))
        .as_table_mut()
        .ok_or_else(|| {
            CoreError::classified(
                crate::bridge::ErrorClass::Runtime,
                error_code,
                format!(
                    "The packaged Mars config at {} has a non-table [{section}] value.",
                    package_config.display()
                ),
                "Reinstall the Yazelix runtime or rebuild it from a valid Mars package.",
                serde_json::json!({}),
            )
        })
}

fn remove_path_if_exists(path: &Path, operation: &'static str) -> Result<(), CoreError> {
    let Ok(metadata) = fs::symlink_metadata(path) else {
        return Ok(());
    };

    if metadata.is_dir() && !metadata.file_type().is_symlink() {
        fs::remove_dir_all(path).map_err(|source| {
            CoreError::io(
                operation,
                "Could not remove the stale generated Mars themes directory",
                "Remove the stale generated Mars themes directory, then rerun `yzx refresh`.",
                path.to_string_lossy(),
                source,
            )
        })
    } else {
        fs::remove_file(path).map_err(|source| {
            CoreError::io(
                operation,
                "Could not remove the stale generated Mars themes path",
                "Remove the stale generated Mars themes path, then rerun `yzx refresh`.",
                path.to_string_lossy(),
                source,
            )
        })
    }
}

fn patch_mars_theme_cursor(theme_path: &Path, color_hex: &str) -> Result<(), CoreError> {
    let raw = fs::read_to_string(theme_path).map_err(|source| {
        CoreError::io(
            "read_mars_theme",
            "Could not read a copied Mars theme",
            "Reinstall the Yazelix runtime so the Mars child themes are present.",
            theme_path.to_string_lossy(),
            source,
        )
    })?;
    let mut table = toml::from_str::<toml::Table>(&raw).map_err(|source| {
        CoreError::classified(
            crate::bridge::ErrorClass::Runtime,
            "parse_mars_theme",
            format!(
                "The packaged Mars theme at {} is not valid TOML.",
                theme_path.display()
            ),
            "Reinstall the Yazelix runtime or rebuild it from a valid Mars package.",
            serde_json::json!({ "error": source.to_string() }),
        )
    })?;
    table
        .get_mut("colors")
        .and_then(toml::Value::as_table_mut)
        .ok_or_else(|| {
            CoreError::classified(
                crate::bridge::ErrorClass::Runtime,
                "invalid_mars_theme_colors",
                format!(
                    "The packaged Mars theme at {} is missing a [colors] table or has a non-table [colors] value.",
                    theme_path.display()
                ),
                "Reinstall the Yazelix runtime or rebuild it from a valid Mars package.",
                serde_json::json!({}),
            )
        })?
        .insert(
            "cursor".to_string(),
            toml::Value::String(color_hex.to_string()),
        );
    let rendered = toml::to_string_pretty(&toml::Value::Table(table)).map_err(|source| {
        CoreError::classified(
            crate::bridge::ErrorClass::Internal,
            "render_mars_theme",
            "Could not render a generated Mars theme.",
            "Report this Yazelix bug with the current settings.jsonc.",
            serde_json::json!({ "error": source.to_string() }),
        )
    })?;
    write_text_atomic(theme_path, &rendered)
}

fn copy_mars_themes(
    package_config: &Path,
    generated_config_dir: &Path,
    cursor_color_hex: Option<&str>,
) -> Result<(), CoreError> {
    let package_root = package_config.parent().ok_or_else(|| {
        CoreError::classified(
            crate::bridge::ErrorClass::Runtime,
            "invalid_mars_package_config_path",
            format!(
                "Packaged Mars config path has no parent directory: {}.",
                package_config.display()
            ),
            "Reinstall the Yazelix runtime or rebuild it from a valid Mars package.",
            serde_json::json!({}),
        )
    })?;
    let source = package_root.join("themes");
    if !source.is_dir() {
        return Err(CoreError::classified(
            crate::bridge::ErrorClass::Runtime,
            "missing_mars_themes",
            format!(
                "The packaged Mars config at {} does not have a sibling themes directory.",
                package_config.display()
            ),
            "Reinstall Yazelix with a Mars package that advertises and ships its appearance themes.",
            serde_json::json!({ "themes_dir": source.to_string_lossy() }),
        ));
    }

    let destination = generated_config_dir.join("themes");
    remove_path_if_exists(&destination, "remove_mars_themes")?;
    copy_dir_all(&source, &destination).map_err(|source_error| {
        CoreError::io(
            "copy_mars_themes",
            "Could not copy packaged Mars themes into the generated config root",
            "Check permissions for the generated Yazelix state directory, then rerun `yzx refresh`.",
            destination.to_string_lossy(),
            source_error,
        )
    })?;

    if let Some(color_hex) = cursor_color_hex {
        for theme in ["yazelix-dark.toml", "yazelix-light.toml"] {
            patch_mars_theme_cursor(&destination.join(theme), color_hex)?;
        }
    }

    Ok(())
}

fn apply_mars_appearance(
    table: &mut toml::Table,
    package_config: &Path,
    appearance_mode: &str,
) -> Result<(), CoreError> {
    if !matches!(
        table.get("adaptive-theme"),
        Some(toml::Value::Table(adaptive))
            if adaptive.contains_key("dark") && adaptive.contains_key("light")
    ) {
        return Err(CoreError::classified(
            crate::bridge::ErrorClass::Runtime,
            "missing_mars_adaptive_theme",
            format!(
                "The packaged Mars config at {} does not declare adaptive-theme dark/light themes.",
                package_config.display()
            ),
            "Reinstall Yazelix with a Mars package that advertises and ships dark/light/auto appearance support.",
            serde_json::json!({}),
        ));
    }

    if let Some(theme) = match appearance_mode {
        APPEARANCE_MODE_AUTO => None,
        APPEARANCE_MODE_LIGHT => Some(APPEARANCE_MODE_LIGHT),
        _ => Some(APPEARANCE_MODE_DARK),
    } {
        table.insert(
            "force-theme".to_string(),
            toml::Value::String(theme.to_string()),
        );
    } else {
        table.remove("force-theme");
    }
    Ok(())
}

fn generate_mars_config(
    runtime_dir: &Path,
    transparency: &str,
    cursor_state: Option<&TerminalCursorState>,
    shader_paths: &[String],
    profile: MarsProfile,
    emoji_font: MarsEmojiFont,
    appearance_mode: &str,
    generated_config_dir: &Path,
) -> Result<String, CoreError> {
    let package_config = mars_package_config_path(runtime_dir, profile, emoji_font)?;
    let raw = fs::read_to_string(&package_config).map_err(|source| {
        CoreError::io(
            "read_mars_package_config",
            "Could not read the packaged Mars config",
            "Reinstall the Yazelix runtime so the Mars child package is present.",
            package_config.to_string_lossy(),
            source,
        )
    })?;
    let mut table = toml::from_str::<toml::Table>(&raw).map_err(|source| {
        CoreError::classified(
            crate::bridge::ErrorClass::Runtime,
            "parse_mars_package_config",
            format!(
                "The packaged Mars config at {} is not valid TOML.",
                package_config.display()
            ),
            "Reinstall the Yazelix runtime or rebuild it from a valid Mars package.",
            serde_json::json!({ "error": source.to_string() }),
        )
    })?;
    let cursor_color_hex = cursor_state.and_then(|state| state.selected_color_hex.as_deref());
    copy_mars_themes(&package_config, generated_config_dir, cursor_color_hex)?;
    table.insert(
        "scrollback-history-limit".to_string(),
        toml::Value::Integer(0),
    );
    table.insert(
        "confirm-before-quit".to_string(),
        toml::Value::Boolean(true),
    );
    table.insert(
        "line-height".to_string(),
        toml::Value::Float(MARS_LINE_HEIGHT),
    );
    let fonts = mars_config_table_mut(
        &mut table,
        "fonts",
        "invalid_mars_fonts_config",
        &package_config,
    )?;
    fonts.insert(
        "family".to_string(),
        toml::Value::String(FONT_JETBRAINS_MONO.to_string()),
    );
    fonts.insert("size".to_string(), toml::Value::Float(MARS_FONT_SIZE));

    let opacity = get_opacity_value(transparency)
        .parse::<f64>()
        .map_err(|source| {
            CoreError::classified(
                crate::bridge::ErrorClass::Internal,
                "parse_mars_opacity",
                format!("Could not parse Mars opacity for transparency '{transparency}'."),
                "Report this Yazelix bug with the active settings.jsonc.",
                serde_json::json!({ "error": source.to_string() }),
            )
        })?;
    let window = mars_config_table_mut(
        &mut table,
        "window",
        "invalid_mars_window_config",
        &package_config,
    )?;
    window.insert("opacity".to_string(), toml::Value::Float(opacity));
    // Keep full-screen TUI cell backgrounds from compounding over the
    // already-translucent window background. mars's default background
    // path carries the configured opacity; explicit cells should stay crisp.
    window.insert("opacity-cells".to_string(), toml::Value::Boolean(false));

    let renderer = mars_config_table_mut(
        &mut table,
        "renderer",
        "invalid_mars_renderer_config",
        &package_config,
    )?;
    match profile {
        MarsProfile::Full | MarsProfile::Baseline => {
            renderer.remove("custom-shader");
        }
        MarsProfile::Shaders => {
            if shader_paths.is_empty() {
                renderer.remove("custom-shader");
            } else {
                renderer.insert(
                    "custom-shader".to_string(),
                    shader_paths_to_toml(shader_paths),
                );
            }
        }
    }

    apply_mars_appearance(&mut table, &package_config, appearance_mode)?;

    toml::to_string_pretty(&toml::Value::Table(table)).map_err(|source| {
        CoreError::classified(
            crate::bridge::ErrorClass::Internal,
            "render_mars_config",
            "Could not render the generated Mars config.",
            "Report this Yazelix bug with the current settings.jsonc.",
            serde_json::json!({ "error": source.to_string() }),
        )
    })
}

fn ensure_ghostty_materialization<'a>(
    data: &'a mut Option<GhosttyMaterializationData>,
    request: &GhosttyMaterializationRequest,
) -> Result<&'a GhosttyMaterializationData, CoreError> {
    if data.is_none() {
        *data = Some(generate_ghostty_materialization(request)?);
    }
    Ok(data
        .as_ref()
        .expect("ghostty materialization data was just initialized"))
}

fn ensure_terminal_cursor_materialization<'a>(
    data: &'a mut Option<TerminalCursorMaterializationData>,
    request: &TerminalCursorMaterializationRequest,
) -> Result<&'a TerminalCursorMaterializationData, CoreError> {
    if data.is_none() {
        *data = Some(generate_terminal_cursor_materialization(request)?);
    }
    Ok(data
        .as_ref()
        .expect("terminal cursor materialization data was just initialized"))
}

fn cursor_data_from_ghostty(
    state_dir: &Path,
    data: &GhosttyMaterializationData,
) -> TerminalCursorMaterializationData {
    TerminalCursorMaterializationData {
        cursor_state: data.cursor_state.clone(),
        shader_paths: cursor_shader_paths_for_state(state_dir, &data.cursor_state)
            .into_iter()
            .map(|path| path.to_string_lossy().into_owned())
            .collect(),
        shaders_synced: data.shaders_synced,
    }
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
scrollback_lines 0

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

fn write_single_terminal_config(
    generated_dir: &Path,
    terminal: &str,
    display_name: &str,
    file_name: &str,
    content: String,
) -> Result<TerminalGeneratedConfig, CoreError> {
    let dir = generated_dir.join(terminal);
    fs::create_dir_all(&dir).map_err(|source| {
        CoreError::io(
            format!("create_{terminal}_dir"),
            format!("Could not create {display_name} output directory"),
            "Check permissions for the Yazelix state directory.",
            dir.to_string_lossy(),
            source,
        )
    })?;
    let path = dir.join(file_name);
    write_text_atomic(&path, &content)?;
    Ok(TerminalGeneratedConfig {
        terminal: terminal.to_string(),
        path: path.to_string_lossy().into_owned(),
    })
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
    let appearance_mode = appearance_mode_from_config(config);
    let mars_emoji_font = mars_emoji_font_from_config(config, request.mars_emoji_font)?;

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
    let ghostty_request = GhosttyMaterializationRequest {
        runtime_dir: request.runtime_dir.clone(),
        config_dir: config_dir.clone(),
        state_dir: request.state_dir.clone(),
        transparency: transparency.to_string(),
        appearance_mode: appearance_mode.to_string(),
        cursor_config_path: request.cursor_config_path.clone(),
    };
    let cursor_request = TerminalCursorMaterializationRequest {
        runtime_dir: request.runtime_dir.clone(),
        state_dir: request.state_dir.clone(),
        cursor_config_path: request.cursor_config_path.clone(),
        appearance_mode: appearance_mode.to_string(),
    };

    let mut generated = Vec::new();
    let mut ghostty_data = None;
    let mut cursor_data = None;

    for terminal in &request.terminals {
        match terminal.as_str() {
            "ghostty" => {
                let data = ensure_ghostty_materialization(&mut ghostty_data, &ghostty_request)?;
                if cursor_data.is_none() {
                    cursor_data = Some(cursor_data_from_ghostty(&request.state_dir, data));
                }
                let path = data.generated_path.clone();
                generated.push(TerminalGeneratedConfig {
                    terminal: "ghostty".to_string(),
                    path,
                });
            }
            "wezterm" => {
                generated.push(write_single_terminal_config(
                    &generated_dir,
                    "wezterm",
                    "WezTerm",
                    ".wezterm.lua",
                    generate_wezterm_config(transparency, appearance_mode),
                )?);
            }
            "rio" => {
                generated.push(write_single_terminal_config(
                    &generated_dir,
                    "rio",
                    "Rio",
                    "config.toml",
                    generate_rio_config(&request.runtime_dir, transparency, appearance_mode),
                )?);
            }
            "ratty" => {
                generated.push(write_single_terminal_config(
                    &generated_dir,
                    "ratty",
                    "Ratty",
                    "ratty.toml",
                    generate_ratty_config(transparency),
                )?);
            }
            "foot" => {
                generated.push(write_single_terminal_config(
                    &generated_dir,
                    "foot",
                    "Foot",
                    "foot.ini",
                    generate_foot_config(transparency, appearance_mode),
                )?);
            }
            "mars" => {
                let mars_dir = generated_dir.join("mars");
                fs::create_dir_all(&mars_dir).map_err(|source| {
                    CoreError::io(
                        "create_mars_dir",
                        "Could not create Mars output directory",
                        "Check permissions for the Yazelix state directory.",
                        mars_dir.to_string_lossy(),
                        source,
                    )
                })?;
                let path = mars_dir.join("config.toml");
                let mars_cursor_data = if cursors_enabled {
                    Some(ensure_terminal_cursor_materialization(
                        &mut cursor_data,
                        &cursor_request,
                    )?)
                } else {
                    None
                };
                let cursor_state = mars_cursor_data.map(|data| &data.cursor_state);
                let shader_paths = mars_cursor_data
                    .map(|data| data.shader_paths.as_slice())
                    .unwrap_or_default();
                write_text_atomic(
                    &path,
                    &generate_mars_config(
                        &request.runtime_dir,
                        transparency,
                        cursor_state,
                        shader_paths,
                        request.mars_profile,
                        mars_emoji_font,
                        appearance_mode,
                        &mars_dir,
                    )?,
                )?;
                generated.push(TerminalGeneratedConfig {
                    terminal: "mars".to_string(),
                    path: path.to_string_lossy().into_owned(),
                });
            }
            "kitty" => {
                let override_path = get_terminal_override_path(&config_dir, "kitty")?;
                generated.push(write_single_terminal_config(
                    &generated_dir,
                    "kitty",
                    "Kitty",
                    "kitty.conf",
                    generate_kitty_config(
                        transparency,
                        kitty_enable_cursor,
                        override_path.as_deref(),
                    ),
                )?);
            }
            _ => {}
        }
    }

    Ok(TerminalMaterializationData {
        generated,
        ghostty: ghostty_data,
        cursor: cursor_data,
    })
}

#[cfg(test)]
// Test lane: default
mod tests {
    use super::*;
    use serde_json::{Map as JsonMap, Value as JsonValue};
    use std::ffi::OsString;
    use std::sync::{Mutex, OnceLock};

    fn env_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    struct EnvRestore(Vec<(&'static str, Option<OsString>)>);

    impl Drop for EnvRestore {
        fn drop(&mut self) {
            for (key, value) in self.0.drain(..) {
                restore_env(key, value);
            }
        }
    }

    fn with_env<T>(values: &[(&str, Option<&str>)], test: impl FnOnce() -> T) -> T {
        let _guard = env_lock().lock().unwrap();
        let keys = MARS_EMOJI_ENV_KEYS;
        let previous = keys
            .iter()
            .map(|key| (*key, std::env::var_os(key)))
            .collect::<Vec<_>>();
        let _restore = EnvRestore(previous);

        for key in keys {
            // Tests serialize process-env mutation through `env_lock`.
            unsafe {
                std::env::remove_var(key);
            }
        }
        for (key, value) in values {
            if let Some(value) = value {
                // Tests serialize process-env mutation through `env_lock`.
                unsafe {
                    std::env::set_var(key, value);
                }
            }
        }

        test()
    }

    fn restore_env(key: &str, value: Option<OsString>) {
        match value {
            Some(value) => unsafe {
                std::env::set_var(key, value);
            },
            None => unsafe {
                std::env::remove_var(key);
            },
        }
    }

    fn config_with_emoji_style(style: &str) -> JsonMap<String, JsonValue> {
        let mut config = JsonMap::new();
        config.insert(
            "terminal_emoji_style".to_string(),
            JsonValue::String(style.to_string()),
        );
        config
    }

    // Regression: stale terminal wrapper env from an existing shell/session must not override mutable settings.jsonc.
    #[test]
    fn mars_emoji_env_without_source_is_not_a_materialization_override() {
        with_env(&[(MARS_EMOJI_FONT_ENV, Some("twitter"))], || {
            assert_eq!(mars_emoji_font_override_from_env().unwrap(), None);

            let config = config_with_emoji_style("serenityos");
            assert_eq!(
                mars_emoji_font_from_config(&config, mars_emoji_font_override_from_env().unwrap(),)
                    .unwrap(),
                MarsEmojiFont::SerenityOs,
            );
        });
    }

    // Defends: Home Manager activation and desktop launchers can still pass an explicit active mars emoji preset.
    #[test]
    fn mars_emoji_home_manager_source_is_a_materialization_override() {
        with_env(
            &[
                (MARS_EMOJI_FONT_ENV, Some("serenityos")),
                (
                    MARS_EMOJI_FONT_SOURCE_ENV,
                    Some(MARS_EMOJI_FONT_SOURCE_HOME_MANAGER),
                ),
            ],
            || {
                assert_eq!(
                    mars_emoji_font_override_from_env().unwrap(),
                    Some(MarsEmojiFont::SerenityOs),
                );

                let config = config_with_emoji_style("twitter");
                assert_eq!(
                    mars_emoji_font_from_config(
                        &config,
                        mars_emoji_font_override_from_env().unwrap(),
                    )
                    .unwrap(),
                    MarsEmojiFont::SerenityOs,
                );
            },
        );
    }

    // Defends: selecting serenityos reads the child-owned emoji/serenityos profile root, not the default Noto root.
    #[test]
    fn mars_serenityos_config_uses_child_emoji_profile_root() {
        let temp = tempfile::tempdir().unwrap();
        let runtime = temp.path();
        let package_root = runtime.join("share").join("mars");
        write_mars_package_metadata(&package_root);
        write_mars_profile_config(&package_root.join("config.toml"), "Noto Color Emoji");
        write_mars_profile_config(
            &package_root
                .join("emoji")
                .join("serenityos")
                .join("config.toml"),
            "SerenityOS Emoji",
        );
        write_theme_files(&package_root.join("themes"));
        write_theme_files(&package_root.join("emoji").join("serenityos").join("themes"));

        let generated_dir = temp.path().join("generated");
        let rendered = generate_mars_config(
            runtime,
            "none",
            None,
            &[],
            MarsProfile::Full,
            MarsEmojiFont::SerenityOs,
            APPEARANCE_MODE_DARK,
            &generated_dir,
        )
        .unwrap();

        assert!(rendered.contains("SerenityOS Emoji"));
        assert!(!rendered.contains("Noto Color Emoji"));
        assert!(rendered.contains("scrollback-history-limit = 0"));
        assert!(
            generated_dir
                .join("themes")
                .join("yazelix-dark.toml")
                .is_file()
        );
    }

    // Defends: every generated terminal config leaves pane history to Zellij instead of retaining duplicate emulator scrollback.
    #[test]
    fn generated_terminal_configs_disable_emulator_scrollback() {
        let runtime = Path::new("/runtime");

        assert!(
            generate_wezterm_config("none", APPEARANCE_MODE_DARK)
                .contains("config.scrollback_lines = 0")
        );
        assert!(generate_ratty_config("none").contains("scrollback = 0"));
        assert!(generate_foot_config("none", APPEARANCE_MODE_DARK).contains("[scrollback]\n"));
        assert!(generate_foot_config("none", APPEARANCE_MODE_DARK).contains("lines=0"));
        assert!(
            generate_rio_config(runtime, "none", APPEARANCE_MODE_DARK)
                .contains("scrollback-history-limit = 0")
        );
        assert!(generate_kitty_config("none", false, None).contains("scrollback_lines 0"));
    }

    // Defends: Rio-family terminal config stays aligned with close confirmation and Mars status glyph font defaults.
    #[test]
    fn generated_rio_family_configs_keep_close_and_status_defaults() {
        let runtime = Path::new("/runtime");
        assert!(
            generate_rio_config(runtime, "none", APPEARANCE_MODE_DARK)
                .contains("confirm-before-quit = true")
        );

        let temp = tempfile::tempdir().unwrap();
        let package_root = temp.path().join("runtime/share/mars");
        let generated_dir = temp.path().join("state/configs/terminal_emulators/mars");
        write_mars_package_metadata(&package_root);
        write_mars_profile_config(&package_root.join("config.toml"), "Noto Color Emoji");
        write_theme_files(&package_root.join("themes"));
        let rendered = generate_mars_config(
            &temp.path().join("runtime"),
            "none",
            None,
            &[],
            MarsProfile::Full,
            MarsEmojiFont::Noto,
            APPEARANCE_MODE_DARK,
            &generated_dir,
        )
        .unwrap();

        assert!(rendered.contains("confirm-before-quit = true"));
        let config = toml::from_str::<toml::Table>(&rendered).unwrap();
        assert_eq!(
            config.get("line-height").and_then(toml::Value::as_float),
            Some(MARS_LINE_HEIGHT)
        );
        let fonts = config.get("fonts").and_then(toml::Value::as_table).unwrap();
        assert_eq!(
            fonts.get("family").and_then(toml::Value::as_str),
            Some(FONT_JETBRAINS_MONO)
        );
        assert_eq!(
            fonts.get("size").and_then(toml::Value::as_float),
            Some(MARS_FONT_SIZE)
        );
        assert!(rendered.contains("Noto Color Emoji"));
    }

    // Defends: checked-in reference snapshots stay aligned with the generated no-emulator-scrollback policy.
    #[test]
    fn reference_terminal_config_snapshots_disable_emulator_scrollback() {
        assert!(
            include_str!("../../../configs/terminal_emulators/ghostty/config")
                .contains("scrollback-limit = 0")
        );
        assert!(
            include_str!("../../../configs/terminal_emulators/kitty/kitty.conf")
                .contains("scrollback_lines 0")
        );
        assert!(
            include_str!("../../../configs/terminal_emulators/wezterm/.wezterm.lua")
                .contains("config.scrollback_lines = 0")
        );
    }

    fn write_mars_package_metadata(package_root: &Path) {
        fs::create_dir_all(package_root).unwrap();
        fs::write(
            package_root.join("package-metadata.json"),
            r#"{
  "emoji_fonts": {
    "noto": {
      "config_roots": {
        "full": "share/mars",
        "baseline": "share/mars/baseline",
        "shaders": "share/mars/profiles/shaders"
      }
    },
    "serenityos": {
      "config_roots": {
        "full": "share/mars/emoji/serenityos",
        "baseline": "share/mars/emoji/serenityos/baseline",
        "shaders": "share/mars/emoji/serenityos/profiles/shaders"
      }
    }
  }
}
"#,
        )
        .unwrap();
    }

    fn write_mars_profile_config(path: &Path, emoji_family: &str) {
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(
            path,
            format!(
                r#"
confirm-before-quit = true
scrollback-history-limit = 10000
force-theme = "dark"

[adaptive-theme]
dark = "yazelix-dark"
light = "yazelix-light"

[window]
decorations = "Disabled"

[renderer]
backend = "Webgpu"

[fonts]
family = "FiraCode Nerd Font"

[[fonts.additional-dirs]]
path = "/fonts/{emoji_family}"
"#
            ),
        )
        .unwrap();
    }

    fn write_theme_files(path: &Path) {
        fs::create_dir_all(path).unwrap();
        for file_name in ["yazelix-dark.toml", "yazelix-light.toml"] {
            fs::write(path.join(file_name), "[colors]\ncursor = '#ffffff'\n").unwrap();
        }
    }
}
