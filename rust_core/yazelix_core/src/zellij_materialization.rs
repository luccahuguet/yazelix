use crate::bridge::{CoreError, ErrorClass};
use crate::config_normalize::{NormalizeConfigRequest, normalize_config};
use crate::control_plane::{
    config_dir_from_env, home_dir_from_env as home_dir_from_control_plane,
    state_dir_from_env as state_dir_from_control_plane,
};
use crate::zellij_render_plan::{
    DEFAULT_SIDEBAR_YAZI_ARG, TopLevelSetting, ZellijRenderPlanData, ZellijRenderPlanRequest,
    compute_zellij_render_plan, effective_sidebar_args,
};
use serde::Serialize;
use serde_json::{Map as JsonMap, Value as JsonValue, json};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};
use yazelix_bar::{
    BarRenderError, BarRenderRequest, CUSTOM_TEXT_PLACEHOLDER, WIDGET_TRAY_PLACEHOLDER,
    ZJSTATUS_NU_BIN_PLACEHOLDER, ZJSTATUS_PLUGIN_URL_PLACEHOLDER, ZJSTATUS_RUNTIME_DIR_PLACEHOLDER,
    ZJSTATUS_YZX_CONTROL_BIN_PLACEHOLDER, render_zjstatus_bar_segments,
};

const PANE_ORCHESTRATOR_PLUGIN_ALIAS: &str = "yazelix_pane_orchestrator";
const PANE_ORCHESTRATOR_PLUGIN_PREFIX: &str = "yazelix_pane_orchestrator";
const PANE_ORCHESTRATOR_WASM_NAME: &str = "yazelix_pane_orchestrator.wasm";
const ZJSTATUS_PLUGIN_PREFIX: &str = "zjstatus";
const ZJSTATUS_WASM_NAME: &str = "zjstatus.wasm";
const GENERATION_METADATA_NAME: &str = ".yazelix_generation.json";

const PANE_ORCHESTRATOR_PLUGIN_URL_PLACEHOLDER: &str = "__YAZELIX_PANE_ORCHESTRATOR_PLUGIN_URL__";
const HOME_DIR_PLACEHOLDER: &str = "__YAZELIX_HOME_DIR__";
const RUNTIME_DIR_PLACEHOLDER: &str = ZJSTATUS_RUNTIME_DIR_PLACEHOLDER;

const PANE_ORCHESTRATOR_REQUIRED_PERMISSIONS: &[&str] = &[
    "ReadApplicationState",
    "OpenTerminalsOrPlugins",
    "ChangeApplicationState",
    "RunCommands",
    "WriteToStdin",
    "ReadCliPipes",
    "ReadSessionEnvironmentVariables",
];
const ZJSTATUS_REQUIRED_PERMISSIONS: &[&str] = &[
    "ReadApplicationState",
    "ChangeApplicationState",
    "RunCommands",
];

#[derive(Debug, Clone)]
pub struct ZellijMaterializationRequest {
    pub config_path: PathBuf,
    pub default_config_path: PathBuf,
    pub contract_path: PathBuf,
    pub runtime_dir: PathBuf,
    pub zellij_config_dir: PathBuf,
    pub seed_plugin_permissions: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ZellijMaterializationData {
    pub merged_config_path: String,
    pub merged_config_dir: String,
    pub reused: bool,
    pub base_config_source: String,
    pub base_config_path: String,
    pub generation_fingerprint: String,
    pub pane_orchestrator_runtime_path: String,
    pub zjstatus_runtime_path: String,
    pub permissions_cache_path: String,
    pub seeded_plugin_permissions: bool,
    pub generated_layouts: Vec<String>,
}

#[derive(Debug, Clone)]
struct ZellijBaseConfigSource {
    source: String,
    path: Option<PathBuf>,
    content: String,
}

#[derive(Debug, Clone)]
struct PluginArtifact {
    name: &'static str,
    prefix: &'static str,
    wasm_name: &'static str,
    tracked_path: PathBuf,
    tracked_hash: String,
    runtime_path: PathBuf,
    required_permissions: &'static [&'static str],
}

#[derive(Debug, Clone)]
struct ExtractedSemanticBlocks {
    config_without_semantic_blocks: String,
    load_plugin_lines: Vec<String>,
    plugin_lines: Vec<String>,
    keybind_lines: Vec<String>,
    ui_lines: Vec<String>,
}

#[derive(Debug, Clone)]
struct PermissionBlock {
    path: String,
    permissions: Vec<String>,
}

pub fn generate_zellij_materialization(
    request: &ZellijMaterializationRequest,
) -> Result<ZellijMaterializationData, CoreError> {
    let normalized = normalize_config(&NormalizeConfigRequest {
        config_path: request.config_path.clone(),
        default_config_path: request.default_config_path.clone(),
        contract_path: request.contract_path.clone(),
        include_missing: false,
    })?;
    let config = normalized.normalized_config;
    let state_dir = state_dir_from_env()?;
    let merged_config_path = request.zellij_config_dir.join("config.kdl");
    let layout_dir = request.zellij_config_dir.join("layouts");
    let source_layouts_dir = request
        .runtime_dir
        .join("configs")
        .join("zellij")
        .join("layouts");
    let resolved_default_shell = resolve_zellij_default_shell(
        &request.runtime_dir,
        string_config(&config, "default_shell", "nu"),
    );
    let base_config_source = resolve_base_config_source()?;
    cleanup_legacy_popup_runner_artifacts(&state_dir)?;
    let plugin_artifacts = resolve_plugin_artifacts(&request.runtime_dir, &state_dir)?;
    let reuse_allowed = string_config(&config, "zellij_theme", "default") != "random";
    let generation_fingerprint = build_generation_fingerprint(
        &config,
        &request.runtime_dir,
        &base_config_source,
        &resolved_default_shell,
        &source_layouts_dir,
        &plugin_artifacts,
    )?;
    if reuse_allowed
        && can_reuse_generated_zellij_state(
            &request.zellij_config_dir,
            &merged_config_path,
            &source_layouts_dir,
            &generation_fingerprint,
            &plugin_artifacts,
        )?
    {
        if request.seed_plugin_permissions {
            upsert_plugin_permission_blocks(&plugin_artifacts)?;
        }
        return Ok(ZellijMaterializationData {
            merged_config_path: merged_config_path.to_string_lossy().to_string(),
            merged_config_dir: request.zellij_config_dir.to_string_lossy().to_string(),
            reused: true,
            base_config_source: base_config_source.source,
            base_config_path: base_config_source
                .path
                .map(|path| path.to_string_lossy().to_string())
                .unwrap_or_default(),
            generation_fingerprint,
            pane_orchestrator_runtime_path: plugin_artifacts[0]
                .runtime_path
                .to_string_lossy()
                .to_string(),
            zjstatus_runtime_path: plugin_artifacts[1]
                .runtime_path
                .to_string_lossy()
                .to_string(),
            permissions_cache_path: permissions_cache_path()?.to_string_lossy().to_string(),
            seeded_plugin_permissions: request.seed_plugin_permissions,
            generated_layouts: expected_layout_targets(
                &source_layouts_dir,
                &request.zellij_config_dir,
            )?
            .into_iter()
            .map(|path| path.to_string_lossy().to_string())
            .collect(),
        });
    }

    let render_plan = compute_zellij_render_plan(&build_render_plan_request(
        &config,
        &layout_dir,
        &resolved_default_shell,
    ))?;
    fs::create_dir_all(&request.zellij_config_dir).map_err(|source| {
        CoreError::io(
            "create_zellij_output_dir",
            "Could not create the managed Zellij output directory",
            "Check permissions for the Yazelix state directory and retry.",
            request.zellij_config_dir.to_string_lossy(),
            source,
        )
    })?;

    sync_plugin_artifacts(&plugin_artifacts, request.seed_plugin_permissions)?;
    let pane_orchestrator_runtime_path = plugin_artifacts[0].runtime_path.clone();
    let zjstatus_runtime_path = plugin_artifacts[1].runtime_path.clone();
    let generated_layouts = generate_all_layouts(
        &source_layouts_dir,
        &layout_dir,
        &request.runtime_dir,
        &render_plan,
        &pane_orchestrator_runtime_path,
        &zjstatus_runtime_path,
    )?;
    let merged_config = render_merged_config(
        &request.runtime_dir,
        &base_config_source.content,
        &render_plan,
        &pane_orchestrator_runtime_path,
    )?;
    write_text_atomic(&merged_config_path, &merged_config)?;
    record_generation_fingerprint(&request.zellij_config_dir, &generation_fingerprint)?;

    Ok(ZellijMaterializationData {
        merged_config_path: merged_config_path.to_string_lossy().to_string(),
        merged_config_dir: request.zellij_config_dir.to_string_lossy().to_string(),
        reused: false,
        base_config_source: base_config_source.source,
        base_config_path: base_config_source
            .path
            .map(|path| path.to_string_lossy().to_string())
            .unwrap_or_default(),
        generation_fingerprint,
        pane_orchestrator_runtime_path: pane_orchestrator_runtime_path
            .to_string_lossy()
            .to_string(),
        zjstatus_runtime_path: zjstatus_runtime_path.to_string_lossy().to_string(),
        permissions_cache_path: permissions_cache_path()?.to_string_lossy().to_string(),
        seeded_plugin_permissions: request.seed_plugin_permissions,
        generated_layouts: generated_layouts
            .into_iter()
            .map(|path| path.to_string_lossy().to_string())
            .collect(),
    })
}

fn build_render_plan_request(
    config: &JsonMap<String, JsonValue>,
    layout_dir: &Path,
    resolved_default_shell: &str,
) -> ZellijRenderPlanRequest {
    ZellijRenderPlanRequest {
        enable_sidebar: bool_config(config, "enable_sidebar", true),
        initial_sidebar_state: string_config(config, "initial_sidebar_state", "open").to_string(),
        sidebar_width_percent: int_config(config, "sidebar_width_percent", 20),
        sidebar_command: string_config(config, "sidebar_command", "nu").to_string(),
        sidebar_args: string_list_config(config, "sidebar_args")
            .unwrap_or_else(|| vec![DEFAULT_SIDEBAR_YAZI_ARG.to_string()]),
        popup_width_percent: int_config(config, "popup_width_percent", 90),
        popup_height_percent: int_config(config, "popup_height_percent", 90),
        screen_saver_enabled: bool_config(config, "screen_saver_enabled", false),
        screen_saver_idle_seconds: int_config(config, "screen_saver_idle_seconds", 300),
        screen_saver_style: string_config(config, "screen_saver_style", "random").to_string(),
        zellij_widget_tray: string_list_config(config, "zellij_widget_tray"),
        zellij_custom_text: optional_string_config(config, "zellij_custom_text"),
        zellij_theme: string_config(config, "zellij_theme", "default").to_string(),
        zellij_pane_frames: string_config(config, "zellij_pane_frames", "true").to_string(),
        zellij_rounded_corners: string_config(config, "zellij_rounded_corners", "true").to_string(),
        disable_zellij_tips: string_config(config, "disable_zellij_tips", "true").to_string(),
        support_kitty_keyboard_protocol: string_config(
            config,
            "support_kitty_keyboard_protocol",
            "false",
        )
        .to_string(),
        zellij_default_mode: string_config(config, "zellij_default_mode", "normal").to_string(),
        yazelix_layout_dir: layout_dir.to_string_lossy().to_string(),
        resolved_default_shell: resolved_default_shell.to_string(),
        editor_label: string_config(config, "editor_command", "hx").to_string(),
        shell_label: string_config(config, "default_shell", "nu").to_string(),
        terminal_label: first_string_list_config(config, "terminals", "wezterm"),
    }
}

fn bool_config(config: &JsonMap<String, JsonValue>, key: &str, default: bool) -> bool {
    match config.get(key) {
        Some(JsonValue::Bool(value)) => *value,
        Some(JsonValue::String(value)) => value == "true",
        _ => default,
    }
}

fn int_config(config: &JsonMap<String, JsonValue>, key: &str, default: i64) -> i64 {
    config
        .get(key)
        .and_then(|value| match value {
            JsonValue::Number(number) => number.as_i64(),
            JsonValue::String(raw) => raw.parse::<i64>().ok(),
            _ => None,
        })
        .unwrap_or(default)
}

fn string_config<'a>(
    config: &'a JsonMap<String, JsonValue>,
    key: &str,
    default: &'a str,
) -> &'a str {
    config
        .get(key)
        .and_then(JsonValue::as_str)
        .filter(|value| !value.trim().is_empty())
        .unwrap_or(default)
}

fn optional_string_config(config: &JsonMap<String, JsonValue>, key: &str) -> Option<String> {
    config
        .get(key)
        .and_then(JsonValue::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

fn string_list_config(config: &JsonMap<String, JsonValue>, key: &str) -> Option<Vec<String>> {
    match config.get(key) {
        Some(JsonValue::Array(values)) => Some(
            values
                .iter()
                .filter_map(JsonValue::as_str)
                .map(ToOwned::to_owned)
                .collect(),
        ),
        _ => None,
    }
}

fn first_string_list_config(
    config: &JsonMap<String, JsonValue>,
    key: &str,
    default: &str,
) -> String {
    string_list_config(config, key)
        .and_then(|values| {
            values
                .into_iter()
                .map(|value| value.trim().to_string())
                .find(|value| !value.is_empty())
        })
        .unwrap_or_else(|| default.to_string())
}

fn resolve_zellij_default_shell(runtime_dir: &Path, default_shell: &str) -> String {
    if default_shell.eq_ignore_ascii_case("nu") {
        runtime_dir
            .join("shells")
            .join("posix")
            .join("yazelix_nu.sh")
            .to_string_lossy()
            .to_string()
    } else {
        default_shell.to_string()
    }
}

fn resolve_base_config_source() -> Result<ZellijBaseConfigSource, CoreError> {
    let config_dir = config_dir_from_env()?;
    crate::managed_user_config_stubs::ensure_zellij_surface_stub(&config_dir)?;
    let managed_path = config_dir
        .join("user_configs")
        .join("zellij")
        .join("config.kdl");
    if managed_path.exists() {
        return Ok(ZellijBaseConfigSource {
            source: "managed".to_string(),
            path: Some(managed_path.clone()),
            content: read_text(&managed_path, "read_managed_zellij_config")?,
        });
    }

    let native_path = home_dir_from_env()?
        .join(".config")
        .join("zellij")
        .join("config.kdl");
    if native_path.exists() {
        return Ok(ZellijBaseConfigSource {
            source: "native".to_string(),
            path: Some(native_path.clone()),
            content: read_text(&native_path, "read_native_zellij_config")?,
        });
    }

    let output = Command::new("zellij")
        .arg("setup")
        .arg("--dump-config")
        .output()
        .map_err(|source| {
            CoreError::io(
                "dump_zellij_defaults",
                "Cannot fetch Zellij defaults",
                "Run Yazelix inside its Nix environment so zellij is available in PATH.",
                "zellij",
                source,
            )
        })?;
    if !output.status.success() {
        return Err(CoreError::classified(
            ErrorClass::Runtime,
            "dump_zellij_defaults_failed",
            format!(
                "Cannot fetch Zellij defaults: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            ),
            "Run Yazelix inside its Nix environment so zellij is available in PATH.",
            json!({}),
        ));
    }

    Ok(ZellijBaseConfigSource {
        source: "defaults".to_string(),
        path: None,
        content: String::from_utf8_lossy(&output.stdout).to_string(),
    })
}

fn render_merged_config(
    runtime_dir: &Path,
    base_config_content: &str,
    render_plan: &ZellijRenderPlanData,
    pane_orchestrator_wasm_path: &Path,
) -> Result<String, CoreError> {
    let extracted_blocks = extract_semantic_config_blocks(base_config_content);
    let overrides_path = runtime_dir
        .join("configs")
        .join("zellij")
        .join("yazelix_overrides.kdl");
    let override_keybinds = read_yazelix_override_keybinds(&overrides_path, runtime_dir)?;
    let base_config = strip_yazelix_owned_top_level_settings(
        &extracted_blocks.config_without_semantic_blocks,
        &render_plan.owned_top_level_setting_names,
    );
    let merged_keybinds =
        build_merged_keybinds_block(&extracted_blocks.keybind_lines, &override_keybinds);
    let merged_ui = build_yazelix_ui_block(&extracted_blocks.ui_lines, &render_plan.rounded_value);
    let plugins_block = build_yazelix_plugins_block(
        &extracted_blocks.plugin_lines,
        pane_orchestrator_wasm_path,
        runtime_dir,
        render_plan.popup_width_percent,
        render_plan.popup_height_percent,
        render_plan.screen_saver_enabled,
        render_plan.screen_saver_idle_seconds,
        &render_plan.screen_saver_style,
    );
    let load_plugins_block = build_yazelix_load_plugins_block(&extracted_blocks.load_plugin_lines);

    Ok([
        "// ========================================".to_string(),
        "// GENERATED ZELLIJ CONFIG (YAZELIX)".to_string(),
        "// ========================================".to_string(),
        "// Source preference:".to_string(),
        "//   1) ~/.config/yazelix/user_configs/zellij/config.kdl (user-managed)".to_string(),
        "//   2) ~/.config/zellij/config.kdl (native fallback, read-only)".to_string(),
        "//   3) zellij setup --dump-config (defaults)".to_string(),
        "//".to_string(),
        format!("// Generated: {}", timestamp_for_header()),
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
            "// === YAZELIX DYNAMIC SETTINGS (from yazelix.toml) ===",
            &render_plan.dynamic_top_level_settings,
        ),
        String::new(),
        render_top_level_settings_block(
            "// === YAZELIX ENFORCED SETTINGS ===",
            &render_plan.enforced_top_level_settings,
        ),
        String::new(),
        "// === YAZELIX BACKGROUND PLUGINS ===".to_string(),
        load_plugins_block,
    ]
    .join("\n"))
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
        let open_braces = line.chars().filter(|c| *c == '{').count() as i64;
        let close_braces = line.chars().filter(|c| *c == '}').count() as i64;

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

fn build_yazelix_load_plugins_block(existing_lines: &[String]) -> String {
    let mut merged_lines = existing_lines
        .iter()
        .filter(|line| {
            !line.contains("yazelix_popup_runner.wasm") && !line.contains("yazelix_popup_runner")
        })
        .cloned()
        .collect::<Vec<_>>();
    let present = merged_lines
        .iter()
        .any(|line| line.trim() == PANE_ORCHESTRATOR_PLUGIN_ALIAS);
    if !present {
        merged_lines.push(format!("  {PANE_ORCHESTRATOR_PLUGIN_ALIAS}"));
    }
    block_with_lines("load_plugins", &merged_lines)
}

fn build_yazelix_plugins_block(
    existing_lines: &[String],
    pane_orchestrator_wasm_path: &Path,
    runtime_dir: &Path,
    popup_width_percent: i64,
    popup_height_percent: i64,
    screen_saver_enabled: bool,
    screen_saver_idle_seconds: i64,
    screen_saver_style: &str,
) -> String {
    let mut merged_lines = existing_lines.to_vec();
    let alias_present = merged_lines
        .iter()
        .any(|line| line.contains(&format!("{PANE_ORCHESTRATOR_PLUGIN_ALIAS} location=")));
    if !alias_present {
        merged_lines.extend([
            format!(
                "    {PANE_ORCHESTRATOR_PLUGIN_ALIAS} location=\"file:{}\" {{",
                pane_orchestrator_wasm_path.to_string_lossy()
            ),
            format!(
                "        runtime_dir {}",
                json_quote(&runtime_dir.to_string_lossy())
            ),
            format!("        popup_width_percent \"{popup_width_percent}\""),
            format!("        popup_height_percent \"{popup_height_percent}\""),
            format!("        screen_saver_enabled \"{screen_saver_enabled}\""),
            format!("        screen_saver_idle_seconds \"{screen_saver_idle_seconds}\""),
            format!(
                "        screen_saver_style {}",
                json_quote(screen_saver_style)
            ),
            "    }".to_string(),
        ]);
    }

    if merged_lines.is_empty() {
        String::new()
    } else {
        block_with_lines("plugins", &merged_lines)
    }
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

fn strip_yazelix_owned_top_level_settings(content: &str, owned_setting_names: &[String]) -> String {
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

fn read_yazelix_override_keybinds(
    overrides_path: &Path,
    runtime_dir: &Path,
) -> Result<Vec<String>, CoreError> {
    if !overrides_path.exists() {
        return Err(CoreError::classified(
            ErrorClass::Io,
            "missing_zellij_overrides",
            format!(
                "Missing Yazelix Zellij overrides file: {}",
                overrides_path.to_string_lossy()
            ),
            "Reinstall Yazelix so the runtime includes configs/zellij/yazelix_overrides.kdl.",
            json!({ "path": overrides_path.to_string_lossy() }),
        ));
    }
    let content = read_text(overrides_path, "read_zellij_overrides")?
        .replace(
            "__YAZELIX_PANE_ORCHESTRATOR_PLUGIN_URL__",
            PANE_ORCHESTRATOR_PLUGIN_ALIAS,
        )
        .replace(
            RUNTIME_DIR_PLACEHOLDER,
            runtime_dir.to_string_lossy().as_ref(),
        );
    Ok(extract_semantic_config_blocks(&content).keybind_lines)
}

fn block_with_lines(name: &str, lines: &[String]) -> String {
    std::iter::once(format!("{name} {{"))
        .chain(lines.iter().cloned())
        .chain(std::iter::once("}".to_string()))
        .collect::<Vec<_>>()
        .join("\n")
}

fn generate_all_layouts(
    source_dir: &Path,
    target_dir: &Path,
    runtime_dir: &Path,
    render_plan: &ZellijRenderPlanData,
    pane_orchestrator_wasm_path: &Path,
    zjstatus_wasm_path: &Path,
) -> Result<Vec<PathBuf>, CoreError> {
    if !source_dir.exists() {
        return Ok(Vec::new());
    }
    fs::create_dir_all(target_dir).map_err(|source| {
        CoreError::io(
            "create_zellij_layout_dir",
            "Could not create generated Zellij layout directory",
            "Check permissions for the Yazelix state directory and retry.",
            target_dir.to_string_lossy(),
            source,
        )
    })?;
    let layout_files = list_top_level_kdl_files(source_dir)?;
    let expected_targets = layout_files
        .iter()
        .map(|source| Ok(target_dir.join(Path::new(required_file_name(source)?))))
        .collect::<Result<Vec<_>, CoreError>>()?;
    remove_stale_layouts(target_dir, &expected_targets)?;
    let static_fragments = load_static_fragments(source_dir)?;
    let bar_segments = render_bar_segments(render_plan)?;
    let pane_orchestrator_plugin_url =
        format!("file:{}", pane_orchestrator_wasm_path.to_string_lossy());
    let zjstatus_plugin_url = format!("file:{}", zjstatus_wasm_path.to_string_lossy());
    let home_dir = home_dir_from_env()?;

    for (source, target) in layout_files.iter().zip(expected_targets.iter()) {
        let content = read_text(source, "read_zellij_layout")?;
        let rendered = render_layout_template(
            &content,
            &static_fragments,
            &bar_segments.widget_tray_segment,
            &bar_segments.custom_text_segment,
            &pane_orchestrator_plugin_url,
            &zjstatus_plugin_url,
            &home_dir,
            runtime_dir,
            render_plan,
        )?;
        write_text_atomic(target, &rendered)?;
    }

    Ok(expected_targets)
}

fn render_bar_segments(
    render_plan: &ZellijRenderPlanData,
) -> Result<yazelix_bar::BarRenderData, CoreError> {
    let request = BarRenderRequest {
        widget_tray: render_plan.widget_tray.clone(),
        editor_label: render_plan.editor_label.clone(),
        shell_label: render_plan.shell_label.clone(),
        terminal_label: render_plan.terminal_label.clone(),
        custom_text: render_plan.custom_text.clone(),
    };
    render_zjstatus_bar_segments(&request).map_err(|error| match error {
        BarRenderError::InvalidWidgetTrayEntry { entry } => CoreError::classified(
            ErrorClass::Config,
            "invalid_widget_tray_entry",
            format!("Invalid zellij.widget_tray token in layout renderer: {entry}"),
            "Use only documented widget tray identifiers.",
            json!({ "field": "zellij.widget_tray", "entry": entry }),
        ),
    })
}

fn render_layout_template(
    content: &str,
    static_fragments: &BTreeMap<String, String>,
    widget_tray_segment: &str,
    custom_text_segment: &str,
    pane_orchestrator_plugin_url: &str,
    zjstatus_plugin_url: &str,
    home_dir: &Path,
    runtime_dir: &Path,
    render_plan: &ZellijRenderPlanData,
) -> Result<String, CoreError> {
    let mut updated = apply_static_fragments(content, static_fragments);
    let replacements = [
        (WIDGET_TRAY_PLACEHOLDER, widget_tray_segment.to_string()),
        (CUSTOM_TEXT_PLACEHOLDER, custom_text_segment.to_string()),
        (
            PANE_ORCHESTRATOR_PLUGIN_URL_PLACEHOLDER,
            pane_orchestrator_plugin_url.to_string(),
        ),
        (
            ZJSTATUS_PLUGIN_URL_PLACEHOLDER,
            zjstatus_plugin_url.to_string(),
        ),
        (HOME_DIR_PLACEHOLDER, home_dir.to_string_lossy().to_string()),
        (
            RUNTIME_DIR_PLACEHOLDER,
            runtime_dir.to_string_lossy().to_string(),
        ),
        (
            ZJSTATUS_NU_BIN_PLACEHOLDER,
            resolve_zjstatus_nu_bin(runtime_dir),
        ),
        (
            ZJSTATUS_YZX_CONTROL_BIN_PLACEHOLDER,
            resolve_zjstatus_yzx_control_bin(runtime_dir),
        ),
        (
            "__YAZELIX_SIDEBAR_COMMAND__",
            json_quote(expand_runtime_placeholder(
                &render_plan.sidebar_command,
                runtime_dir,
            )),
        ),
        (
            "__YAZELIX_SIDEBAR_ARGS__",
            render_sidebar_args(&render_plan.sidebar_args, runtime_dir),
        ),
        (
            "__YAZELIX_SIDEBAR_WIDTH_PERCENT__",
            render_plan.layout_percentages.sidebar_width_percent.clone(),
        ),
        (
            "__YAZELIX_OPEN_CONTENT_WIDTH_PERCENT__",
            render_plan
                .layout_percentages
                .open_content_width_percent
                .clone(),
        ),
        (
            "__YAZELIX_OPEN_PRIMARY_WIDTH_PERCENT__",
            render_plan
                .layout_percentages
                .open_primary_width_percent
                .clone(),
        ),
        (
            "__YAZELIX_OPEN_SECONDARY_WIDTH_PERCENT__",
            render_plan
                .layout_percentages
                .open_secondary_width_percent
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
            "__YAZELIX_CLOSED_PRIMARY_WIDTH_PERCENT__",
            render_plan
                .layout_percentages
                .closed_primary_width_percent
                .clone(),
        ),
        (
            "__YAZELIX_CLOSED_SECONDARY_WIDTH_PERCENT__",
            render_plan
                .layout_percentages
                .closed_secondary_width_percent
                .clone(),
        ),
    ];
    for (placeholder, value) in replacements {
        updated = updated.replace(placeholder, &value);
    }
    for placeholder in [
        WIDGET_TRAY_PLACEHOLDER,
        ZJSTATUS_NU_BIN_PLACEHOLDER,
        ZJSTATUS_YZX_CONTROL_BIN_PLACEHOLDER,
        "__YAZELIX_SIDEBAR_COMMAND__",
        "__YAZELIX_SIDEBAR_ARGS__",
    ] {
        if updated.contains(placeholder) {
            return Err(CoreError::classified(
                ErrorClass::Internal,
                "unexpanded_zellij_layout_placeholder",
                "Failed to expand a Zellij layout placeholder",
                "Report this as a Yazelix internal error.",
                json!({ "placeholder": placeholder }),
            ));
        }
    }
    Ok(updated)
}

fn expand_runtime_placeholder(value: &str, runtime_dir: &Path) -> String {
    value.replace(
        RUNTIME_DIR_PLACEHOLDER,
        runtime_dir.to_string_lossy().as_ref(),
    )
}

fn resolve_zjstatus_nu_bin(runtime_dir: &Path) -> String {
    let runtime_nu = runtime_dir.join("libexec").join("nu");
    if runtime_nu.is_file() {
        runtime_nu.to_string_lossy().to_string()
    } else if let Some(path) = env_path_if_file("YAZELIX_NU_BIN") {
        path.to_string_lossy().to_string()
    } else {
        "nu".to_string()
    }
}

fn resolve_zjstatus_yzx_control_bin(runtime_dir: &Path) -> String {
    for candidate in [
        runtime_dir.join("libexec").join("yzx_control"),
        runtime_dir
            .join("rust_core")
            .join("target")
            .join("release")
            .join("yzx_control"),
        runtime_dir
            .join("rust_core")
            .join("target")
            .join("debug")
            .join("yzx_control"),
    ] {
        if candidate.is_file() {
            return candidate.to_string_lossy().to_string();
        }
    }
    if let Some(path) = env_path_if_file("YAZELIX_YZX_CONTROL_BIN") {
        return path.to_string_lossy().to_string();
    }
    runtime_dir
        .join("libexec")
        .join("yzx_control")
        .to_string_lossy()
        .to_string()
}

fn env_path_if_file(variable: &str) -> Option<PathBuf> {
    let raw = std::env::var(variable).ok()?;
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }
    let path = PathBuf::from(trimmed);
    path.is_file().then_some(path)
}

fn render_sidebar_args(args: &[String], runtime_dir: &Path) -> String {
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

fn load_static_fragments(source_dir: &Path) -> Result<BTreeMap<String, String>, CoreError> {
    let specs = [
        (
            "__YAZELIX_ZJSTATUS_TAB_TEMPLATE__",
            "fragments/zjstatus_tab_template.kdl",
        ),
        (
            "__YAZELIX_KEYBINDS_COMMON__",
            "fragments/keybinds_common.kdl",
        ),
        (
            "__YAZELIX_SWAP_SIDEBAR_OPEN__",
            "fragments/swap_sidebar_open.kdl",
        ),
        (
            "__YAZELIX_SWAP_SIDEBAR_CLOSED__",
            "fragments/swap_sidebar_closed.kdl",
        ),
    ];
    let mut fragments = BTreeMap::new();
    for (placeholder, relative) in specs {
        let path = source_dir.join(relative);
        if !path.exists() {
            return Err(CoreError::classified(
                ErrorClass::Io,
                "missing_zellij_layout_fragment",
                format!(
                    "Missing required layout fragment: {}",
                    path.to_string_lossy()
                ),
                "Reinstall Yazelix so the runtime includes all Zellij layout fragments.",
                json!({ "path": path.to_string_lossy() }),
            ));
        }
        fragments.insert(
            placeholder.to_string(),
            read_text(&path, "read_zellij_fragment")?,
        );
    }
    Ok(fragments)
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

fn list_top_level_kdl_files(dir: &Path) -> Result<Vec<PathBuf>, CoreError> {
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut files = fs::read_dir(dir)
        .map_err(|source| {
            CoreError::io(
                "read_zellij_layout_source_dir",
                "Could not read Zellij layout source directory",
                "Reinstall Yazelix so the runtime includes readable Zellij layouts.",
                dir.to_string_lossy(),
                source,
            )
        })?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("kdl"))
        .collect::<Vec<_>>();
    files.sort();
    Ok(files)
}

fn expected_layout_targets(
    source_layouts_dir: &Path,
    merged_config_dir: &Path,
) -> Result<Vec<PathBuf>, CoreError> {
    Ok(list_top_level_kdl_files(source_layouts_dir)?
        .into_iter()
        .map(|source| {
            Ok(merged_config_dir
                .join("layouts")
                .join(Path::new(required_file_name(&source)?)))
        })
        .collect::<Result<Vec<_>, CoreError>>()?)
}

fn list_source_layout_files(source_layouts_dir: &Path) -> Result<Vec<PathBuf>, CoreError> {
    let mut paths = list_top_level_kdl_files(source_layouts_dir)?;
    let fragment_dir = source_layouts_dir.join("fragments");
    if fragment_dir.exists() {
        paths.extend(list_top_level_kdl_files(&fragment_dir)?);
    }
    paths.sort();
    Ok(paths)
}

fn remove_stale_layouts(target_dir: &Path, expected_targets: &[PathBuf]) -> Result<(), CoreError> {
    if !target_dir.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(target_dir).map_err(|source| {
        CoreError::io(
            "read_zellij_layout_target_dir",
            "Could not inspect generated Zellij layout directory",
            "Check permissions for the Yazelix state directory and retry.",
            target_dir.to_string_lossy(),
            source,
        )
    })? {
        let path = entry
            .map_err(|source| {
                CoreError::io(
                    "read_zellij_layout_target_entry",
                    "Could not inspect generated Zellij layout entry",
                    "Check permissions for the Yazelix state directory and retry.",
                    target_dir.to_string_lossy(),
                    source,
                )
            })?
            .path();
        if path.extension().and_then(|ext| ext.to_str()) == Some("kdl")
            && !expected_targets.iter().any(|expected| expected == &path)
        {
            fs::remove_file(&path).map_err(|source| {
                CoreError::io(
                    "remove_stale_zellij_layout",
                    "Could not remove stale generated Zellij layout",
                    "Check permissions for the Yazelix state directory and retry.",
                    path.to_string_lossy(),
                    source,
                )
            })?;
        }
    }
    Ok(())
}

fn resolve_plugin_artifacts(
    runtime_dir: &Path,
    state_dir: &Path,
) -> Result<Vec<PluginArtifact>, CoreError> {
    let plugin_dir = state_dir.join("configs").join("zellij").join("plugins");
    let specs = [
        (
            "pane_orchestrator",
            PANE_ORCHESTRATOR_PLUGIN_PREFIX,
            PANE_ORCHESTRATOR_WASM_NAME,
            PANE_ORCHESTRATOR_REQUIRED_PERMISSIONS,
        ),
        (
            "zjstatus",
            ZJSTATUS_PLUGIN_PREFIX,
            ZJSTATUS_WASM_NAME,
            ZJSTATUS_REQUIRED_PERMISSIONS,
        ),
    ];
    specs
        .into_iter()
        .map(|(name, prefix, wasm_name, required_permissions)| {
            let tracked_path = runtime_dir
                .join("configs")
                .join("zellij")
                .join("plugins")
                .join(wasm_name);
            if !tracked_path.exists() {
                return Err(CoreError::classified(
                    ErrorClass::Io,
                    "missing_tracked_zellij_plugin",
                    format!("Tracked {name} wasm not found at: {}", tracked_path.to_string_lossy()),
                    "Reinstall Yazelix so the runtime includes all tracked Zellij plugin wasm artifacts.",
                    json!({ "path": tracked_path.to_string_lossy(), "plugin": name }),
                ));
            }
            Ok(PluginArtifact {
                name,
                prefix,
                wasm_name,
                tracked_hash: hash_file(&tracked_path)?,
                runtime_path: plugin_dir.join(wasm_name),
                tracked_path,
                required_permissions,
            })
        })
        .collect()
}

fn sync_plugin_artifacts(
    plugin_artifacts: &[PluginArtifact],
    seed_plugin_permissions: bool,
) -> Result<(), CoreError> {
    for artifact in plugin_artifacts {
        copy_file_atomic(&artifact.tracked_path, &artifact.runtime_path)?;
        remove_runtime_plugins_by_prefix(artifact.prefix, Some(&artifact.runtime_path))?;
        preserve_plugin_permissions(
            artifact.prefix,
            &artifact.tracked_path,
            &artifact.runtime_path,
            artifact.required_permissions,
        )?;
    }
    if seed_plugin_permissions {
        upsert_plugin_permission_blocks(plugin_artifacts)?;
    }
    Ok(())
}

fn cleanup_legacy_popup_runner_artifacts(state_dir: &Path) -> Result<(), CoreError> {
    let plugin_dir = state_dir.join("configs").join("zellij").join("plugins");
    remove_runtime_plugins_by_prefix_in_dir(&plugin_dir, "yazelix_popup_runner", None)?;
    remove_permission_blocks_by_prefix("yazelix_popup_runner")?;
    Ok(())
}

fn remove_runtime_plugins_by_prefix(
    prefix: &str,
    excluded_path: Option<&Path>,
) -> Result<(), CoreError> {
    let runtime_dir = state_dir_from_env()?
        .join("configs")
        .join("zellij")
        .join("plugins");
    remove_runtime_plugins_by_prefix_in_dir(&runtime_dir, prefix, excluded_path)
}

fn remove_runtime_plugins_by_prefix_in_dir(
    runtime_dir: &Path,
    prefix: &str,
    excluded_path: Option<&Path>,
) -> Result<(), CoreError> {
    if !runtime_dir.exists() {
        return Ok(());
    }
    let excluded = excluded_path.map(|path| path.to_path_buf());
    for entry in fs::read_dir(runtime_dir).map_err(|source| {
        CoreError::io(
            "read_zellij_plugin_runtime_dir",
            "Could not inspect the managed Zellij plugin directory",
            "Check permissions for the Yazelix state directory and retry.",
            runtime_dir.to_string_lossy(),
            source,
        )
    })? {
        let path = entry
            .map_err(|source| {
                CoreError::io(
                    "read_zellij_plugin_runtime_entry",
                    "Could not inspect a managed Zellij plugin entry",
                    "Check permissions for the Yazelix state directory and retry.",
                    runtime_dir.to_string_lossy(),
                    source,
                )
            })?
            .path();
        if excluded.as_ref().is_some_and(|excluded| excluded == &path) {
            continue;
        }
        let file_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("");
        if plugin_name_matches_prefix(file_name, prefix) {
            fs::remove_file(&path).map_err(|source| {
                CoreError::io(
                    "remove_stale_zellij_plugin",
                    "Could not remove a stale managed Zellij plugin",
                    "Check permissions for the Yazelix state directory and retry.",
                    path.to_string_lossy(),
                    source,
                )
            })?;
        }
    }
    Ok(())
}

fn plugin_name_matches_prefix(file_name: &str, prefix: &str) -> bool {
    file_name == format!("{prefix}.wasm")
        || (file_name.starts_with(&format!("{prefix}_")) && file_name.ends_with(".wasm"))
}
fn preserve_plugin_permissions(
    prefix: &str,
    tracked_path: &Path,
    runtime_path: &Path,
    required_permissions: &[&str],
) -> Result<(), CoreError> {
    let permissions_cache_path = permissions_cache_path()?;
    if !permissions_cache_path.exists() {
        return Ok(());
    }
    let blocks = parse_permission_blocks(&read_text(
        &permissions_cache_path,
        "read_zellij_permissions_cache",
    )?);
    let matching = blocks
        .iter()
        .any(|block| plugin_name_matches_prefix(path_basename(&block.path), prefix));
    if !matching {
        return Ok(());
    }
    let mut retained = blocks
        .into_iter()
        .filter(|block| !plugin_name_matches_prefix(path_basename(&block.path), prefix))
        .map(|block| build_permission_block(&block.path, &block.permissions))
        .collect::<Vec<_>>();
    retained.push(build_permission_block(
        &tracked_path.to_string_lossy(),
        &required_permissions
            .iter()
            .map(|permission| (*permission).to_string())
            .collect::<Vec<_>>(),
    ));
    retained.push(build_permission_block(
        &runtime_path.to_string_lossy(),
        &required_permissions
            .iter()
            .map(|permission| (*permission).to_string())
            .collect::<Vec<_>>(),
    ));
    write_text_atomic(&permissions_cache_path, &retained.join("\n\n"))?;
    Ok(())
}

fn remove_permission_blocks_by_prefix(prefix: &str) -> Result<(), CoreError> {
    let permissions_cache_path = permissions_cache_path()?;
    if !permissions_cache_path.exists() {
        return Ok(());
    }
    let blocks = parse_permission_blocks(&read_text(
        &permissions_cache_path,
        "read_zellij_permissions_cache",
    )?);
    let retained = blocks
        .into_iter()
        .filter(|block| !plugin_name_matches_prefix(path_basename(&block.path), prefix))
        .map(|block| build_permission_block(&block.path, &block.permissions))
        .collect::<Vec<_>>();
    write_text_atomic(&permissions_cache_path, &retained.join("\n\n"))?;
    Ok(())
}

fn parse_permission_blocks(content: &str) -> Vec<PermissionBlock> {
    let mut blocks = Vec::new();
    let mut current_path: Option<String> = None;
    let mut current_permissions = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if current_path.is_none() {
            if let Some(path) = trimmed
                .strip_prefix('"')
                .and_then(|rest| rest.split('"').next())
            {
                if trimmed.ends_with('{') {
                    current_path = Some(path.to_string());
                    current_permissions.clear();
                }
            }
            continue;
        }
        if trimmed == "}" {
            if let Some(path) = current_path.take() {
                blocks.push(PermissionBlock {
                    path,
                    permissions: current_permissions.clone(),
                });
                current_permissions.clear();
            }
            continue;
        }
        if !trimmed.is_empty() {
            current_permissions.push(trimmed.to_string());
        }
    }
    blocks
}

fn build_permission_block(plugin_path: &str, permissions: &[String]) -> String {
    std::iter::once(format!("\"{plugin_path}\" {{"))
        .chain(
            permissions
                .iter()
                .map(|permission| format!("    {permission}")),
        )
        .chain(std::iter::once("}".to_string()))
        .collect::<Vec<_>>()
        .join("\n")
}

fn upsert_plugin_permission_blocks(plugin_artifacts: &[PluginArtifact]) -> Result<(), CoreError> {
    let permissions_cache_path = permissions_cache_path()?;
    let existing_blocks = if permissions_cache_path.exists() {
        parse_permission_blocks(&read_text(
            &permissions_cache_path,
            "read_zellij_permissions_cache",
        )?)
    } else {
        Vec::new()
    };
    let managed_prefixes = plugin_artifacts
        .iter()
        .map(|artifact| artifact.prefix)
        .collect::<BTreeSet<_>>();
    let mut updated = existing_blocks
        .into_iter()
        .filter(|block| {
            !managed_prefixes
                .iter()
                .any(|prefix| plugin_name_matches_prefix(path_basename(&block.path), prefix))
        })
        .map(|block| build_permission_block(&block.path, &block.permissions))
        .collect::<Vec<_>>();

    for artifact in plugin_artifacts {
        let permissions = artifact
            .required_permissions
            .iter()
            .map(|permission| (*permission).to_string())
            .collect::<Vec<_>>();
        updated.push(build_permission_block(
            &artifact.tracked_path.to_string_lossy(),
            &permissions,
        ));
        updated.push(build_permission_block(
            &artifact.runtime_path.to_string_lossy(),
            &permissions,
        ));
    }

    write_text_atomic(&permissions_cache_path, &updated.join("\n\n"))
}

fn build_generation_fingerprint(
    config: &JsonMap<String, JsonValue>,
    runtime_dir: &Path,
    base_config_source: &ZellijBaseConfigSource,
    resolved_default_shell: &str,
    source_layouts_dir: &Path,
    plugin_artifacts: &[PluginArtifact],
) -> Result<String, CoreError> {
    let overrides_path = runtime_dir
        .join("configs")
        .join("zellij")
        .join("yazelix_overrides.kdl");
    let layout_sources = list_source_layout_files(source_layouts_dir)?
        .into_iter()
        .map(|path| {
            Ok(json!({
                "path": path.to_string_lossy(),
                "hash": hash_file(&path)?,
            }))
        })
        .collect::<Result<Vec<_>, CoreError>>()?;
    let plugins = plugin_artifacts
        .iter()
        .map(|artifact| {
            json!({
                "name": artifact.name,
                "tracked_path": artifact.tracked_path.to_string_lossy(),
                "tracked_hash": artifact.tracked_hash,
                "runtime_path": artifact.runtime_path.to_string_lossy(),
                "wasm_name": artifact.wasm_name,
            })
        })
        .collect::<Vec<_>>();
    let sidebar_command = string_config(config, "sidebar_command", "nu");
    let sidebar_args = string_list_config(config, "sidebar_args")
        .unwrap_or_else(|| vec![DEFAULT_SIDEBAR_YAZI_ARG.to_string()]);
    let relevant_config = json!({
        "zellij_widget_tray": config.get("zellij_widget_tray").cloned().unwrap_or_else(|| json!(["editor", "shell", "term", "cpu", "ram"])),
        "zellij_custom_text": string_config(config, "zellij_custom_text", ""),
        "support_kitty_keyboard_protocol": string_config(config, "support_kitty_keyboard_protocol", "false"),
        "default_shell": string_config(config, "default_shell", "nu"),
        "resolved_default_shell": resolved_default_shell,
        "editor_command": string_config(config, "editor_command", ""),
        "terminals": config.get("terminals").cloned().unwrap_or_else(|| json!(["ghostty", "wezterm"])),
        "zellij_default_mode": string_config(config, "zellij_default_mode", "normal"),
        "enable_sidebar": bool_config(config, "enable_sidebar", true),
        "initial_sidebar_state": string_config(config, "initial_sidebar_state", "open"),
        "sidebar_width_percent": int_config(config, "sidebar_width_percent", 20),
        "sidebar_command": sidebar_command,
        "sidebar_args": effective_sidebar_args(sidebar_command, &sidebar_args),
        "popup_width_percent": int_config(config, "popup_width_percent", 90),
        "popup_height_percent": int_config(config, "popup_height_percent", 90),
        "screen_saver_enabled": bool_config(config, "screen_saver_enabled", false),
        "screen_saver_idle_seconds": int_config(config, "screen_saver_idle_seconds", 300),
        "screen_saver_style": string_config(config, "screen_saver_style", "random"),
        "disable_zellij_tips": string_config(config, "disable_zellij_tips", "true"),
        "zellij_pane_frames": string_config(config, "zellij_pane_frames", "true"),
        "zellij_rounded_corners": string_config(config, "zellij_rounded_corners", "true"),
        "zellij_theme": string_config(config, "zellij_theme", "default"),
    });
    let fingerprint_payload = json!({
        "schema_version": 1,
        "runtime_dir": runtime_dir.to_string_lossy(),
        "relevant_config": relevant_config,
        "base_config": {
            "source": base_config_source.source,
            "path": base_config_source.path.as_ref().map(|path| path.to_string_lossy().to_string()).unwrap_or_default(),
            "hash": hash_text(&base_config_source.content),
        },
        "overrides_hash": hash_text(&read_text_if_exists(&overrides_path)?),
        "layout_sources": layout_sources,
        "plugins": plugins,
    });
    Ok(hash_text(
        &serde_json::to_string(&fingerprint_payload).map_err(|source| {
            CoreError::classified(
                ErrorClass::Internal,
                "serialize_zellij_fingerprint",
                format!("Could not serialize Zellij generation fingerprint: {source}"),
                "Report this as a Yazelix internal error.",
                json!({}),
            )
        })?,
    ))
}

fn can_reuse_generated_zellij_state(
    merged_config_dir: &Path,
    merged_config_path: &Path,
    source_layouts_dir: &Path,
    fingerprint: &str,
    plugin_artifacts: &[PluginArtifact],
) -> Result<bool, CoreError> {
    let cached_fingerprint = load_cached_generation_fingerprint(merged_config_dir)?;
    if cached_fingerprint != fingerprint || !merged_config_path.exists() {
        return Ok(false);
    }
    for target in expected_layout_targets(source_layouts_dir, merged_config_dir)? {
        if !target.exists() {
            return Ok(false);
        }
    }
    for artifact in plugin_artifacts {
        if !artifact.runtime_path.exists()
            || hash_file(&artifact.runtime_path)? != artifact.tracked_hash
        {
            return Ok(false);
        }
    }
    Ok(true)
}

fn load_cached_generation_fingerprint(merged_config_dir: &Path) -> Result<String, CoreError> {
    let metadata_path = merged_config_dir.join(GENERATION_METADATA_NAME);
    if !metadata_path.exists() {
        return Ok(String::new());
    }
    let raw = read_text(&metadata_path, "read_zellij_generation_metadata")?;
    let parsed = serde_json::from_str::<JsonValue>(&raw).unwrap_or(JsonValue::Null);
    Ok(parsed
        .get("fingerprint")
        .and_then(JsonValue::as_str)
        .unwrap_or("")
        .to_string())
}

fn record_generation_fingerprint(
    merged_config_dir: &Path,
    fingerprint: &str,
) -> Result<(), CoreError> {
    let metadata_path = merged_config_dir.join(GENERATION_METADATA_NAME);
    let content = serde_json::to_string(&json!({
        "fingerprint": fingerprint,
        "generated_at": timestamp_for_metadata(),
    }))
    .map_err(|source| {
        CoreError::classified(
            ErrorClass::Internal,
            "serialize_zellij_generation_metadata",
            format!("Could not serialize Zellij generation metadata: {source}"),
            "Report this as a Yazelix internal error.",
            json!({}),
        )
    })?;
    write_text_atomic(&metadata_path, &content)
}

fn copy_file_atomic(source: &Path, target: &Path) -> Result<(), CoreError> {
    let bytes = fs::read(source).map_err(|source_err| {
        CoreError::io(
            "read_zellij_plugin_source",
            "Could not read tracked Zellij plugin artifact",
            "Reinstall Yazelix so the runtime includes readable Zellij plugin artifacts.",
            source.to_string_lossy(),
            source_err,
        )
    })?;
    write_bytes_atomic(target, &bytes)
}

fn required_file_name(path: &Path) -> Result<&std::ffi::OsStr, CoreError> {
    path.file_name().ok_or_else(|| {
        CoreError::classified(
            ErrorClass::Internal,
            "missing_zellij_input_file_name",
            "A Zellij materialization input path has no file name",
            "Report this as a Yazelix internal error.",
            json!({ "path": path.to_string_lossy() }),
        )
    })
}

fn write_text_atomic(path: &Path, content: &str) -> Result<(), CoreError> {
    write_bytes_atomic(path, content.as_bytes())
}

fn write_bytes_atomic(path: &Path, content: &[u8]) -> Result<(), CoreError> {
    let parent = path.parent().ok_or_else(|| {
        CoreError::classified(
            ErrorClass::Internal,
            "invalid_zellij_output_path",
            "Generated Zellij output path has no parent directory",
            "Report this as a Yazelix internal error.",
            json!({ "path": path.to_string_lossy() }),
        )
    })?;
    fs::create_dir_all(parent).map_err(|source| {
        CoreError::io(
            "create_zellij_output_parent",
            "Could not create parent directory for generated Zellij output",
            "Check permissions for the Yazelix state directory and retry.",
            parent.to_string_lossy(),
            source,
        )
    })?;
    let temporary_path = path.with_file_name(format!(
        ".{}.yazelix-tmp-{}-{}",
        path.file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("zellij"),
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or(0)
    ));
    fs::write(&temporary_path, content).map_err(|source| {
        CoreError::io(
            "write_zellij_output_temp",
            "Could not write temporary generated Zellij output",
            "Check permissions for the Yazelix state directory and retry.",
            temporary_path.to_string_lossy(),
            source,
        )
    })?;
    fs::rename(&temporary_path, path).map_err(|source| {
        CoreError::io(
            "rename_zellij_output_temp",
            "Could not replace generated Zellij output",
            "Check permissions for the Yazelix state directory and retry.",
            path.to_string_lossy(),
            source,
        )
    })
}

fn read_text(path: &Path, code: &str) -> Result<String, CoreError> {
    fs::read_to_string(path).map_err(|source| {
        CoreError::io(
            code,
            "Could not read a Zellij materialization input",
            "Check permissions or reinstall Yazelix if a runtime input is missing.",
            path.to_string_lossy(),
            source,
        )
    })
}

fn read_text_if_exists(path: &Path) -> Result<String, CoreError> {
    if path.exists() {
        read_text(path, "read_zellij_optional_input")
    } else {
        Ok(String::new())
    }
}

fn hash_file(path: &Path) -> Result<String, CoreError> {
    let bytes = fs::read(path).map_err(|source| {
        CoreError::io(
            "hash_zellij_input",
            "Could not hash a Zellij materialization input",
            "Check permissions or reinstall Yazelix if a runtime input is missing.",
            path.to_string_lossy(),
            source,
        )
    })?;
    Ok(hash_bytes(&bytes))
}

fn hash_text(value: &str) -> String {
    hash_bytes(value.as_bytes())
}

fn hash_bytes(value: &[u8]) -> String {
    let digest = Sha256::digest(value);
    let mut output = String::with_capacity(digest.len() * 2);
    for byte in digest {
        output.push_str(&format!("{byte:02x}"));
    }
    output
}

fn json_quote(value: impl AsRef<str>) -> String {
    serde_json::to_string(value.as_ref()).unwrap_or_else(|_| "\"\"".to_string())
}

fn path_basename(path: &str) -> &str {
    path.rsplit('/').next().unwrap_or(path)
}

fn state_dir_from_env() -> Result<PathBuf, CoreError> {
    state_dir_from_control_plane()
}

fn home_dir_from_env() -> Result<PathBuf, CoreError> {
    home_dir_from_control_plane()
}

fn permissions_cache_path() -> Result<PathBuf, CoreError> {
    Ok(home_dir_from_env()?
        .join(".cache")
        .join("zellij")
        .join("permissions.kdl"))
}

fn timestamp_for_header() -> String {
    "1970-01-01 00:00:00".to_string()
}

fn timestamp_for_metadata() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis().to_string())
        .unwrap_or_else(|_| "0".to_string())
}

// Test lane: maintainer
#[cfg(test)]
mod tests {
    use super::*;

    fn sample_render_plan_for_widgets(
        widget_tray: Vec<&str>,
        editor_label: &str,
        shell: &str,
        terminal_label: &str,
    ) -> ZellijRenderPlanData {
        compute_zellij_render_plan(&ZellijRenderPlanRequest {
            enable_sidebar: true,
            initial_sidebar_state: "open".into(),
            sidebar_width_percent: 20,
            sidebar_command: "nu".into(),
            sidebar_args: vec![
                "__YAZELIX_RUNTIME_DIR__/configs/zellij/scripts/launch_sidebar_yazi.nu".into(),
            ],
            popup_width_percent: 90,
            popup_height_percent: 90,
            screen_saver_enabled: false,
            screen_saver_idle_seconds: 300,
            screen_saver_style: "random".into(),
            zellij_widget_tray: Some(widget_tray.into_iter().map(str::to_string).collect()),
            zellij_custom_text: None,
            zellij_theme: "default".into(),
            zellij_pane_frames: "true".into(),
            zellij_rounded_corners: "true".into(),
            disable_zellij_tips: "true".into(),
            support_kitty_keyboard_protocol: "false".into(),
            zellij_default_mode: "normal".into(),
            yazelix_layout_dir: "/tmp/yazelix/layouts".into(),
            resolved_default_shell: shell.into(),
            editor_label: editor_label.into(),
            shell_label: "nu".into(),
            terminal_label: terminal_label.into(),
        })
        .unwrap()
    }

    // Defends: semantic block extraction removes first-class KDL blocks while preserving unrelated top-level lines.
    // Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
    #[test]
    fn extracts_semantic_blocks() {
        let extracted = extract_semantic_config_blocks(
            r#"scroll_buffer_size 123
keybinds {
    normal { bind "Ctrl y" { SwitchToMode "Normal"; } }
}
ui { pane_frames { hide_session_name true } }
"#,
        );
        assert!(
            extracted
                .config_without_semantic_blocks
                .contains("scroll_buffer_size 123")
        );
        assert!(
            extracted
                .keybind_lines
                .iter()
                .any(|line| line.contains("Ctrl y"))
        );
        assert!(
            extracted
                .ui_lines
                .iter()
                .any(|line| line.contains("hide_session_name"))
        );
    }

    // Regression: widget tray rendering must not leave empty command placeholders when dynamic helper scripts are unavailable.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn renders_widget_tray_segment_with_static_identity_labels() {
        let plan = sample_render_plan_for_widgets(
            vec!["editor", "shell", "term", "workspace", "cpu"],
            "",
            "/nix/store/example/bin/nu",
            "ghostty",
        );
        let rendered = render_bar_segments(&plan).unwrap().widget_tray_segment;

        assert!(rendered.contains("[editor: hx]"));
        assert!(rendered.contains("[shell: nu]"));
        assert!(rendered.contains("[term: ghostty]"));
        assert!(rendered.contains("{command_workspace}"));
        assert!(rendered.contains("{command_cpu}"));
        assert!(!rendered.contains("{command_editor}"));
        assert!(!rendered.contains("[editor: ]"));
    }

    // Defends: the managed sidebar launcher is a generated config concern rather than a hardcoded Yazi script in layout templates.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn renders_configured_sidebar_launcher_placeholders() {
        let mut plan =
            sample_render_plan_for_widgets(vec!["editor"], "hx", "/nix/store/bin/nu", "ghostty");
        plan.sidebar_command = "__YAZELIX_RUNTIME_DIR__/bin/custom-sidebar".into();
        plan.sidebar_args = vec!["--root".into(), "__YAZELIX_RUNTIME_DIR__/side".into()];
        let rendered = render_layout_template(
            r#"pane name="sidebar" {
    command __YAZELIX_SIDEBAR_COMMAND__
    __YAZELIX_SIDEBAR_ARGS__
}"#,
            &BTreeMap::new(),
            "",
            "",
            "",
            "",
            std::path::Path::new("/home/user"),
            std::path::Path::new("/opt/yazelix"),
            &plan,
        )
        .unwrap();

        assert!(rendered.contains(r#"command "/opt/yazelix/bin/custom-sidebar""#));
        assert!(rendered.contains(r#"args "--root" "/opt/yazelix/side""#));
        assert!(!rendered.contains("__YAZELIX_SIDEBAR_COMMAND__"));
        assert!(!rendered.contains("__YAZELIX_SIDEBAR_ARGS__"));
    }

    // Regression: custom sidebar apps must not receive the default Yazi launcher script from normalized config.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn renders_custom_sidebar_command_without_implicit_yazi_launcher_arg() {
        let mut config = JsonMap::new();
        config.insert("sidebar_command".into(), json!("lazygit"));
        config.insert("sidebar_args".into(), json!([DEFAULT_SIDEBAR_YAZI_ARG]));
        let request = build_render_plan_request(
            &config,
            std::path::Path::new("/tmp/yazelix/layouts"),
            "/nix/store/bin/nu",
        );
        let plan = compute_zellij_render_plan(&request).unwrap();
        let rendered = render_layout_template(
            r#"pane name="sidebar" {
    command __YAZELIX_SIDEBAR_COMMAND__
    __YAZELIX_SIDEBAR_ARGS__
}"#,
            &BTreeMap::new(),
            "",
            "",
            "",
            "",
            std::path::Path::new("/home/user"),
            std::path::Path::new("/opt/yazelix"),
            &plan,
        )
        .unwrap();

        assert!(rendered.contains(r#"command "lazygit""#));
        assert!(!rendered.contains("args "));
        assert!(!rendered.contains("launch_sidebar_yazi.nu"));
    }

    // Regression: shipped zjstatus templates use cache readers for dynamic widgets and do not revive direct Zellij pipe or usage polling.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn zjstatus_template_uses_cached_dynamic_widget_helpers() {
        let template =
            include_str!("../../../configs/zellij/layouts/fragments/zjstatus_tab_template.kdl");
        assert!(!template.contains("zjstatus_widget.nu"));
        assert!(!template.contains("command_editor_command"));
        assert!(!template.contains("command_shell_command"));
        assert!(!template.contains("command_term_command"));
        assert!(template.contains(ZJSTATUS_NU_BIN_PLACEHOLDER));
        assert!(template.contains(ZJSTATUS_YZX_CONTROL_BIN_PLACEHOLDER));
        assert!(template.contains("command_workspace_command"));
        assert!(template.contains("status-cache-widget workspace"));
        assert!(template.contains(r##"command_workspace_format "#[fg=#00ff88,bold]{stdout}""##));
        assert!(template.contains(r#"command_workspace_interval "1""#));
        assert!(!template.contains("status-bus-workspace"));
        assert!(template.contains("command_claude_usage_command"));
        assert!(template.contains(r##"command_claude_usage_format "#[fg=#bb88ff,bold]{stdout}""##));
        assert!(template.contains("command_codex_usage_command"));
        assert!(template.contains(r##"command_codex_usage_format "#[fg=#bb88ff,bold]{stdout}""##));
        assert!(template.contains("command_opencode_go_usage_command"));
        assert!(
            template.contains(r##"command_opencode_go_usage_format "#[fg=#bb88ff,bold]{stdout}""##)
        );
        assert!(!template.contains("command_amp_usage_command"));
        assert!(!template.contains("agent-usage"));
    }

    // Regression: generated zjstatus command widgets use resolved runtime helpers while dynamic widgets read only the local cache.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn renders_cached_zjstatus_widget_commands_with_runtime_helper_paths() {
        let temp = tempfile::tempdir().unwrap();
        let runtime_dir = temp.path();
        let libexec = runtime_dir.join("libexec");
        std::fs::create_dir_all(&libexec).unwrap();
        std::fs::write(libexec.join("nu"), "").unwrap();
        std::fs::write(libexec.join("yzx_control"), "").unwrap();
        let plan =
            sample_render_plan_for_widgets(vec!["workspace"], "hx", "/nix/store/bin/nu", "ghostty");
        let rendered = render_layout_template(
            include_str!("../../../configs/zellij/layouts/fragments/zjstatus_tab_template.kdl"),
            &BTreeMap::new(),
            "",
            "",
            "file:/tmp/pane.wasm",
            "file:/tmp/zjstatus.wasm",
            std::path::Path::new("/home/user"),
            runtime_dir,
            &plan,
        )
        .unwrap();
        let expected_nu = libexec.join("nu").to_string_lossy().to_string();
        let expected_yzx_control = libexec.join("yzx_control").to_string_lossy().to_string();

        assert!(rendered.contains(&format!(
            r#"command_cpu_command "{} {}/configs/zellij/scripts/cpu_usage.nu""#,
            expected_nu,
            runtime_dir.to_string_lossy()
        )));
        assert!(rendered.contains(&format!(
            r#"command_workspace_command "{} zellij status-cache-widget workspace""#,
            expected_yzx_control
        )));
        assert!(!rendered.contains(ZJSTATUS_YZX_CONTROL_BIN_PLACEHOLDER));
        assert!(!rendered.contains(ZJSTATUS_NU_BIN_PLACEHOLDER));
        assert!(!rendered.contains("status-bus-workspace"));
        assert!(!rendered.contains("agent-usage"));
    }
    // Regression: legacy plugin permission blocks are recognized by both stable and hashed wasm names.
    // Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
    #[test]
    fn plugin_prefix_matches_stable_and_hashed_names() {
        assert!(plugin_name_matches_prefix("zjstatus.wasm", "zjstatus"));
        assert!(plugin_name_matches_prefix(
            "zjstatus_abc123.wasm",
            "zjstatus"
        ));
        assert!(!plugin_name_matches_prefix("not_zjstatus.wasm", "zjstatus"));
    }

    // Regression: the shipped override keybinds keep popup/menu and sidebar-focus actions on the pane orchestrator instead of reviving helper panes.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn override_keybinds_keep_pane_orchestrator_contract() {
        let temp = tempfile::tempdir().unwrap();
        let overrides_path = temp.path().join("yazelix_overrides.kdl");
        std::fs::write(
            &overrides_path,
            r#"
keybinds {
    normal {
        bind "Alt t" {
            MessagePlugin "__YAZELIX_PANE_ORCHESTRATOR_PLUGIN_URL__" {
                name "toggle_transient_pane"
                payload "popup"
            }
        }
        bind "Alt Shift M" {
            MessagePlugin "__YAZELIX_PANE_ORCHESTRATOR_PLUGIN_URL__" {
                name "toggle_transient_pane"
                payload "menu"
            }
        }
        bind "Ctrl y" {
            MessagePlugin "__YAZELIX_PANE_ORCHESTRATOR_PLUGIN_URL__" {
                name "toggle_editor_sidebar_focus"
                payload "__YAZELIX_RUNTIME_DIR__"
            }
        }
    }
}
"#,
        )
        .unwrap();
        let override_lines =
            read_yazelix_override_keybinds(&overrides_path, std::path::Path::new("/opt/yazelix"))
                .unwrap();
        let merged = override_lines.join("\n");

        assert!(merged.contains("toggle_transient_pane"));
        assert!(merged.contains("payload \"popup\""));
        assert!(merged.contains("payload \"menu\""));
        assert!(merged.contains("toggle_editor_sidebar_focus"));
        assert!(!merged.contains("yazelix_popup_runner.wasm"));
    }

    // Regression: pane-orchestrator plugin config must carry one shared sidebar/popup/runtime contract without duplicate alias injection.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn plugin_block_carries_runtime_and_popup_contract_once() {
        let block = build_yazelix_plugins_block(
            &[],
            std::path::Path::new("/opt/yazelix/plugins/yazelix_pane_orchestrator.wasm"),
            std::path::Path::new("/opt/yazelix"),
            82,
            76,
            true,
            180,
            "mandelbrot",
        );

        assert!(block.contains("yazelix_pane_orchestrator location=\"file:/opt/yazelix/plugins/yazelix_pane_orchestrator.wasm\""));
        assert!(block.contains("runtime_dir \"/opt/yazelix\""));
        assert!(block.contains("popup_width_percent \"82\""));
        assert!(block.contains("popup_height_percent \"76\""));
        assert!(block.contains("screen_saver_enabled \"true\""));
        assert!(block.contains("screen_saver_idle_seconds \"180\""));
        assert!(block.contains("screen_saver_style \"mandelbrot\""));
        assert!(!block.contains("widget_tray_segment"));
        assert!(!block.contains("custom_text_segment"));
        assert!(!block.contains("sidebar_width_percent"));
        assert_eq!(
            block.matches("yazelix_pane_orchestrator location=").count(),
            1
        );
    }
}
