use crate::appearance_mode::{
    APPEARANCE_MODE_AUTO, APPEARANCE_MODE_DARK, APPEARANCE_MODE_LIGHT, appearance_mode_from_config,
};
use crate::atomic_fs::{copy_dir_all, write_text_atomic};
use crate::bridge::CoreError;
use crate::config_normalize::{NormalizeConfigRequest, normalize_config};
use crate::runtime_component_enabled;
use crate::terminal_cursor_materialization::{
    TerminalCursorMaterializationData, TerminalCursorMaterializationRequest, TerminalCursorState,
    generate_terminal_cursor_materialization,
};
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};

const FONT_JETBRAINS_MONO: &str = "JetBrains Mono";
const MARS_FONT_SIZE: f64 = 16.0;
const MARS_LINE_HEIGHT: f64 = 1.12;
const MARS_CLIPBOARD_BINDING_MODIFIERS: &str = "control | shift";
pub(crate) const MARS_EMOJI_FONT_ENV: &str = "MARS_EMOJI_FONT";
pub(crate) const MARS_EMOJI_FONT_SOURCE_ENV: &str = "MARS_EMOJI_FONT_SOURCE";
pub(crate) const MARS_EMOJI_ENV_KEYS: [&str; 2] = [MARS_EMOJI_FONT_ENV, MARS_EMOJI_FONT_SOURCE_ENV];
const MARS_EMOJI_FONT_SOURCE_HOME_MANAGER: &str = "home-manager";

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
    pub cursor: Option<TerminalCursorMaterializationData>,
}

fn get_opacity_value(transparency: &str) -> &str {
    TRANSPARENCY_VALUES
        .iter()
        .find(|(k, _)| *k == transparency)
        .map(|(_, v)| *v)
        .unwrap_or("1.0")
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

fn mars_native_trail_cursor_enabled(
    profile: MarsProfile,
    cursor_state: Option<&TerminalCursorState>,
) -> bool {
    if profile == MarsProfile::Baseline {
        return false;
    }
    cursor_state
        .and_then(|state| state.selected_color.as_deref())
        .map(str::trim)
        .is_some_and(|name| !name.is_empty() && name != "none")
}

fn normalized_mars_modifiers(raw: &str) -> Vec<String> {
    let mut modifiers = raw
        .split('|')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_ascii_lowercase)
        .collect::<Vec<_>>();
    modifiers.sort();
    modifiers
}

fn mars_binding_has_trigger(binding: &toml::Value, key: &str, modifiers: &str) -> bool {
    let Some(table) = binding.as_table() else {
        return false;
    };
    if table.contains_key("mode") {
        return false;
    }
    let Some(binding_key) = table.get("key").and_then(toml::Value::as_str) else {
        return false;
    };
    let Some(binding_modifiers) = table.get("with").and_then(toml::Value::as_str) else {
        return false;
    };
    binding_key.eq_ignore_ascii_case(key)
        && normalized_mars_modifiers(binding_modifiers) == normalized_mars_modifiers(modifiers)
}

fn mars_binding_value(key: &str, modifiers: &str, action: &str) -> toml::Value {
    let mut binding = toml::map::Map::new();
    binding.insert("key".to_string(), toml::Value::String(key.to_string()));
    binding.insert(
        "with".to_string(),
        toml::Value::String(modifiers.to_string()),
    );
    binding.insert(
        "action".to_string(),
        toml::Value::String(action.to_string()),
    );
    toml::Value::Table(binding)
}

fn ensure_mars_clipboard_binding(keys: &mut Vec<toml::Value>, key: &str, action: &str) {
    if keys
        .iter()
        .any(|binding| mars_binding_has_trigger(binding, key, MARS_CLIPBOARD_BINDING_MODIFIERS))
    {
        return;
    }
    keys.push(mars_binding_value(
        key,
        MARS_CLIPBOARD_BINDING_MODIFIERS,
        action,
    ));
}

fn apply_mars_clipboard_keybindings(
    table: &mut toml::Table,
    package_config: &Path,
) -> Result<(), CoreError> {
    let bindings = mars_config_table_mut(
        table,
        "bindings",
        "invalid_mars_bindings_config",
        package_config,
    )?;
    let keys = bindings
        .entry("keys")
        .or_insert_with(|| toml::Value::Array(Vec::new()))
        .as_array_mut()
        .ok_or_else(|| {
            CoreError::classified(
                crate::bridge::ErrorClass::Runtime,
                "invalid_mars_bindings_keys",
                format!(
                    "The packaged Mars config at {} has a non-list [bindings].keys value.",
                    package_config.display()
                ),
                "Reinstall the Yazelix runtime or rebuild it from a valid Mars package.",
                serde_json::json!({}),
            )
        })?;

    ensure_mars_clipboard_binding(keys, "c", "Copy");
    ensure_mars_clipboard_binding(keys, "v", "Paste");
    Ok(())
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
    apply_mars_clipboard_keybindings(&mut table, &package_config)?;
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

    let effects = mars_config_table_mut(
        &mut table,
        "effects",
        "invalid_mars_effects_config",
        &package_config,
    )?;
    effects.insert(
        "trail-cursor".to_string(),
        toml::Value::Boolean(mars_native_trail_cursor_enabled(profile, cursor_state)),
    );

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

/// Returns whether Yazelix owns a generated terminal config for this terminal.
///
/// Kitty and Ghostty keep their native config user-owned. The retained Mars
/// renderer remains available for legacy/package-specific tooling, but Mars is
/// no longer part of the supported launch chain.
pub fn terminal_has_generated_config(terminal: &str) -> bool {
    terminal == "mars"
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

    let cursors_enabled = runtime_component_enabled(&request.runtime_dir, "cursors")?;
    let generated_dir = request.state_dir.join("configs").join("terminal_emulators");
    let cursor_request = TerminalCursorMaterializationRequest {
        runtime_dir: request.runtime_dir.clone(),
        state_dir: request.state_dir.clone(),
        cursor_config_path: request.cursor_config_path.clone(),
        appearance_mode: appearance_mode.to_string(),
    };

    let mut generated = Vec::new();
    let mut cursor_data = None;

    for terminal in &request.terminals {
        match terminal.as_str() {
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
            other => {
                return Err(CoreError::usage(format!(
                    "Yazelix only materializes the packaged Mars terminal; configure host terminal '{other}' to run `yzx enter`."
                )));
            }
        }
    }

    Ok(TerminalMaterializationData {
        generated,
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

    // Defends: Mars terminal config stays aligned with close confirmation and status glyph font defaults.
    #[test]
    fn generated_mars_config_keeps_close_and_status_defaults() {
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
        let additional_dirs = fonts
            .get("additional-dirs")
            .and_then(toml::Value::as_array)
            .unwrap();
        assert!(
            additional_dirs
                .iter()
                .any(|entry| entry.as_str() == Some("/fonts/Symbols"))
        );
        let symbol_map = fonts
            .get("symbol-map")
            .and_then(toml::Value::as_array)
            .unwrap();
        assert!(symbol_map.iter().any(|entry| {
            entry
                .as_table()
                .and_then(|table| table.get("font-family"))
                .and_then(toml::Value::as_str)
                == Some("Symbols Nerd Font Mono")
        }));
        assert!(rendered.contains("Noto Color Emoji"));
    }

    // Regression: terminal copy/paste must be source-owned for Mars/Rio, not left to an ambient user config.
    #[test]
    fn generated_mars_config_pins_clipboard_keybindings() {
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

        assert_eq!(
            mars_binding_actions(&rendered, "c", MARS_CLIPBOARD_BINDING_MODIFIERS),
            vec!["Copy"]
        );
        assert_eq!(
            mars_binding_actions(&rendered, "v", MARS_CLIPBOARD_BINDING_MODIFIERS),
            vec!["Paste"]
        );
    }

    // Defends: packaged/user-owned Mars bindings for the same trigger stay authoritative.
    #[test]
    fn generated_mars_config_preserves_existing_clipboard_binding_triggers() {
        let temp = tempfile::tempdir().unwrap();
        let package_root = temp.path().join("runtime/share/mars");
        let generated_dir = temp.path().join("state/configs/terminal_emulators/mars");
        let package_config = package_root.join("config.toml");
        write_mars_package_metadata(&package_root);
        write_mars_profile_config(&package_config, "Noto Color Emoji");
        let mut raw = fs::read_to_string(&package_config).unwrap();
        raw.push_str(
            r#"
[bindings]
keys = [
  { key = "c", with = "shift | control", action = "None" },
]
"#,
        );
        fs::write(&package_config, raw).unwrap();
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

        assert_eq!(
            mars_binding_actions(&rendered, "c", MARS_CLIPBOARD_BINDING_MODIFIERS),
            vec!["None"]
        );
        assert_eq!(
            mars_binding_actions(&rendered, "v", MARS_CLIPBOARD_BINDING_MODIFIERS),
            vec!["Paste"]
        );
    }

    // Regression: GitHub #655, `trail = "none"` and disabled cursor components must disable Mars's native trail too.
    #[test]
    fn generated_mars_config_disables_native_trail_when_yazelix_cursor_is_disabled() {
        let temp = tempfile::tempdir().unwrap();
        let package_root = temp.path().join("runtime/share/mars");
        let generated_dir = temp.path().join("state/configs/terminal_emulators/mars");
        write_mars_package_metadata(&package_root);
        write_mars_profile_config(&package_root.join("config.toml"), "Noto Color Emoji");
        write_theme_files(&package_root.join("themes"));

        for cursor_state in [Some(cursor_state_with_color("none")), None] {
            let rendered = generate_mars_config(
                &temp.path().join("runtime"),
                "none",
                cursor_state.as_ref(),
                &[],
                MarsProfile::Full,
                MarsEmojiFont::Noto,
                APPEARANCE_MODE_DARK,
                &generated_dir,
            )
            .unwrap();

            assert_eq!(mars_trail_cursor_enabled(&rendered), Some(false));
            assert!(
                !rendered.contains("custom-shader"),
                "disabled cursor trail must not leave custom shaders behind"
            );
        }
    }

    // Defends: the full Mars profile keeps native trails for an active cursor, while baseline remains no-effects.
    #[test]
    fn generated_mars_config_keeps_native_trail_only_for_active_nonbaseline_cursor() {
        let temp = tempfile::tempdir().unwrap();
        let package_root = temp.path().join("runtime/share/mars");
        let generated_dir = temp.path().join("state/configs/terminal_emulators/mars");
        let cursor_state = cursor_state_with_color("cosmic");
        write_mars_package_metadata(&package_root);
        write_mars_profile_config(&package_root.join("config.toml"), "Noto Color Emoji");
        write_mars_profile_config(
            &package_root.join("baseline").join("config.toml"),
            "Noto Color Emoji",
        );
        write_theme_files(&package_root.join("themes"));
        write_theme_files(&package_root.join("baseline").join("themes"));

        let full = generate_mars_config(
            &temp.path().join("runtime"),
            "none",
            Some(&cursor_state),
            &[],
            MarsProfile::Full,
            MarsEmojiFont::Noto,
            APPEARANCE_MODE_DARK,
            &generated_dir,
        )
        .unwrap();
        assert_eq!(mars_trail_cursor_enabled(&full), Some(true));

        let baseline = generate_mars_config(
            &temp.path().join("runtime"),
            "none",
            Some(&cursor_state),
            &[],
            MarsProfile::Baseline,
            MarsEmojiFont::Noto,
            APPEARANCE_MODE_DARK,
            &generated_dir,
        )
        .unwrap();
        assert_eq!(mars_trail_cursor_enabled(&baseline), Some(false));
    }

    fn cursor_state_with_color(name: &str) -> TerminalCursorState {
        TerminalCursorState {
            selected_color: Some(name.to_string()),
            selected_color_hex: Some("#c761f5".to_string()),
            selected_family: Some("mono".to_string()),
            selected_divider: None,
            selected_primary_color_hex: None,
            selected_secondary_color_hex: None,
            selected_trail_effect: None,
            selected_mode_effect: None,
            trail_duration: 1.0,
            effect_color_literal: "#c761f5".to_string(),
        }
    }

    fn mars_trail_cursor_enabled(rendered: &str) -> Option<bool> {
        toml::from_str::<toml::Table>(rendered)
            .unwrap()
            .get("effects")
            .and_then(toml::Value::as_table)
            .and_then(|effects| effects.get("trail-cursor"))
            .and_then(toml::Value::as_bool)
    }

    fn mars_binding_actions(rendered: &str, key: &str, modifiers: &str) -> Vec<String> {
        let expected_modifiers = normalized_mars_modifiers(modifiers);
        toml::from_str::<toml::Table>(rendered)
            .unwrap()
            .get("bindings")
            .and_then(toml::Value::as_table)
            .and_then(|bindings| bindings.get("keys"))
            .and_then(toml::Value::as_array)
            .unwrap()
            .iter()
            .filter_map(toml::Value::as_table)
            .filter(|binding| {
                !binding.contains_key("mode")
                    && binding
                        .get("key")
                        .and_then(toml::Value::as_str)
                        .is_some_and(|binding_key| binding_key.eq_ignore_ascii_case(key))
                    && binding
                        .get("with")
                        .and_then(toml::Value::as_str)
                        .is_some_and(|binding_modifiers| {
                            normalized_mars_modifiers(binding_modifiers) == expected_modifiers
                        })
            })
            .filter_map(|binding| binding.get("action").and_then(toml::Value::as_str))
            .map(ToOwned::to_owned)
            .collect()
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

[effects]
trail-cursor = true

[fonts]
family = "FiraCode Nerd Font"
additional-dirs = [
  "/fonts/JetBrainsMono",
  "/fonts/Symbols",
  "/fonts/{emoji_family}",
]
symbol-map = [
  {{ start = "E000", end = "F900", font-family = "Symbols Nerd Font Mono" }},
  {{ start = "2600", end = "276F", font-family = "{emoji_family}" }},
]
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
