use crate::bridge::{CoreError, ErrorClass};
use crate::ghostty_cursor_registry::{
    CursorDefinition, CursorRegistry, DEFAULT_GHOSTTY_TRAIL_DURATION, GHOSTTY_TRAIL_DURATION_MAX,
    GHOSTTY_TRAIL_DURATION_MIN, ResolvedCursorRegistryState, YazelixCursorRegistryExt,
    write_ghostty_cursor_effect_shaders, write_ghostty_cursor_palette_shaders,
};
use crate::runtime_component_enabled;
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone)]
pub struct TerminalCursorMaterializationRequest {
    pub runtime_dir: PathBuf,
    pub state_dir: PathBuf,
    pub cursor_config_path: PathBuf,
    pub appearance_mode: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct TerminalCursorState {
    pub selected_color: Option<String>,
    pub selected_color_hex: Option<String>,
    pub selected_family: Option<String>,
    pub selected_divider: Option<String>,
    pub selected_primary_color_hex: Option<String>,
    pub selected_secondary_color_hex: Option<String>,
    pub selected_trail_effect: Option<String>,
    pub selected_mode_effect: Option<String>,
    pub trail_duration: f64,
    pub effect_color_literal: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct TerminalCursorMaterializationData {
    pub cursor_state: TerminalCursorState,
    pub shader_paths: Vec<String>,
    pub shaders_synced: bool,
}

pub fn cursor_shader_paths_for_state(
    state_dir: &Path,
    cursor_state: &TerminalCursorState,
) -> Vec<PathBuf> {
    let shaders_dir = state_dir
        .join("configs")
        .join("terminal_emulators")
        .join("ghostty")
        .join("shaders");
    let mut paths = Vec::new();

    if let Some(name) = cursor_state
        .selected_color
        .as_deref()
        .map(str::trim)
        .filter(|name| !name.is_empty() && *name != "none")
    {
        paths.push(shaders_dir.join(format!("cursor_trail_{name}.glsl")));
    }

    paths
}

pub fn disabled_terminal_cursor_state() -> TerminalCursorState {
    TerminalCursorState {
        selected_color: None,
        selected_color_hex: None,
        selected_family: None,
        selected_divider: None,
        selected_primary_color_hex: None,
        selected_secondary_color_hex: None,
        selected_trail_effect: None,
        selected_mode_effect: None,
        trail_duration: DEFAULT_GHOSTTY_TRAIL_DURATION,
        effect_color_literal: "#ffb929".to_string(),
    }
}

pub fn generate_terminal_cursor_materialization(
    request: &TerminalCursorMaterializationRequest,
) -> Result<TerminalCursorMaterializationData, CoreError> {
    if !runtime_component_enabled(&request.runtime_dir, "cursors")? {
        return Ok(TerminalCursorMaterializationData {
            cursor_state: disabled_terminal_cursor_state(),
            shader_paths: Vec::new(),
            shaders_synced: false,
        });
    }

    let registry = CursorRegistry::load(&request.cursor_config_path)?;
    let registry_state = registry.resolve_for_appearance(&request.appearance_mode);
    validate_terminal_cursor_trail_duration(registry_state.duration)?;
    let cursor_state = build_terminal_cursor_render_state(&registry_state);
    let shaders_synced = sync_terminal_cursor_shader_assets(
        &request.runtime_dir,
        &request.state_dir,
        &registry_state.glow,
        &cursor_state.effect_color_literal,
        registry_state.duration,
        &registry,
    )?;
    let shader_paths = cursor_shader_paths_for_state(&request.state_dir, &cursor_state)
        .into_iter()
        .map(|path| path.to_string_lossy().into_owned())
        .collect();

    Ok(TerminalCursorMaterializationData {
        cursor_state,
        shader_paths,
        shaders_synced,
    })
}

fn validate_terminal_cursor_trail_duration(duration: f64) -> Result<(), CoreError> {
    if !duration.is_finite()
        || !(GHOSTTY_TRAIL_DURATION_MIN..=GHOSTTY_TRAIL_DURATION_MAX).contains(&duration)
    {
        return Err(CoreError::classified(
            ErrorClass::Config,
            "invalid_ghostty_trail_duration",
            format!(
                "Invalid cursor settings.duration value '{}'. Expected a number from {} to {}.",
                duration, GHOSTTY_TRAIL_DURATION_MIN, GHOSTTY_TRAIL_DURATION_MAX
            ),
            "Update ~/.config/yazelix_cursors/settings.jsonc with a supported cursor trail duration multiplier, then retry.",
            serde_json::json!({
                "field": "settings.duration",
                "actual": duration.to_string(),
                "min": GHOSTTY_TRAIL_DURATION_MIN,
                "max": GHOSTTY_TRAIL_DURATION_MAX,
            }),
        ));
    }
    Ok(())
}

fn build_terminal_cursor_render_state(
    registry_state: &ResolvedCursorRegistryState,
) -> TerminalCursorState {
    let selected_cursor = registry_state.selected_cursor.as_ref();
    let selected_color = if registry_state.trail_disabled {
        Some("none".to_string())
    } else {
        selected_cursor.map(|cursor| cursor.name.clone())
    };
    let selected_color_hex = selected_cursor.map(|cursor| cursor.cursor_color_hex().to_string());

    TerminalCursorState {
        selected_color,
        selected_color_hex,
        selected_family: selected_cursor.map(|cursor| cursor.family_name().to_string()),
        selected_divider: selected_cursor
            .and_then(|cursor| cursor.divider_name().map(|divider| divider.to_string())),
        selected_primary_color_hex: selected_cursor
            .and_then(CursorDefinition::split_primary_color_hex)
            .map(str::to_string),
        selected_secondary_color_hex: selected_cursor
            .and_then(CursorDefinition::split_secondary_color_hex)
            .map(str::to_string),
        selected_trail_effect: registry_state.selected_trail_effect.clone(),
        selected_mode_effect: registry_state.selected_mode_effect.clone(),
        trail_duration: registry_state.duration,
        effect_color_literal: registry_state
            .selected_cursor
            .as_ref()
            .map(CursorDefinition::cursor_color_literal)
            .unwrap_or_else(|| "iCurrentCursorColor".to_string()),
    }
}

fn sync_terminal_cursor_shader_assets(
    runtime_dir: &Path,
    state_dir: &Path,
    glow_level: &str,
    effect_color_literal: &str,
    trail_duration: f64,
    registry: &CursorRegistry,
) -> Result<bool, CoreError> {
    let shaders_src = runtime_dir
        .join("configs")
        .join("terminal_emulators")
        .join("ghostty")
        .join("shaders");
    let shaders_dest = state_dir
        .join("configs")
        .join("terminal_emulators")
        .join("ghostty")
        .join("shaders");

    if shaders_dest.exists() {
        let _ = Command::new("chmod")
            .args(["-R", "u+w", &shaders_dest.to_string_lossy()])
            .output();

        fs::remove_dir_all(&shaders_dest).map_err(|source| {
            CoreError::io(
                "remove_terminal_cursor_shaders",
                "Failed to remove previous terminal cursor shader assets",
                "Check permissions for the Yazelix state directory and retry.",
                shaders_dest.to_string_lossy(),
                source,
            )
        })?;
    }

    fs::create_dir_all(&shaders_dest).map_err(|source| {
        CoreError::io(
            "create_terminal_cursor_shaders",
            "Could not create the terminal cursor shader output directory",
            "Check permissions for the Yazelix state directory and retry.",
            shaders_dest.to_string_lossy(),
            source,
        )
    })?;

    if shaders_src.exists() {
        copy_dir_all(&shaders_src, &shaders_dest).map_err(|source| {
            CoreError::io(
                "copy_terminal_cursor_shaders",
                "Failed to copy terminal cursor shader assets",
                "Check permissions and disk space, then retry.",
                format!("{} -> {}", shaders_src.display(), shaders_dest.display()),
                source,
            )
        })?;

        let _ = Command::new("chmod")
            .args(["-R", "u+w", &shaders_dest.to_string_lossy()])
            .output();
    }

    write_ghostty_cursor_palette_shaders(&shaders_dest, registry, glow_level, trail_duration)?;
    write_ghostty_cursor_effect_shaders(
        &shaders_dest,
        glow_level,
        effect_color_literal,
        trail_duration,
    )?;

    Ok(true)
}

fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> std::io::Result<()> {
    fs::create_dir_all(&dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let src_path = entry.path();
        let dst_path = dst.as_ref().join(entry.file_name());
        if ty.is_dir() {
            copy_dir_all(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}
