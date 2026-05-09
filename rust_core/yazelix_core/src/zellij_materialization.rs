use crate::action_registry::{PANE_ORCHESTRATOR_PLUGIN_ALIAS, YZPP_PLUGIN_ALIAS, ZELLIJ_ACTIONS};
use crate::bridge::{CoreError, ErrorClass};
use crate::config_normalize::{NormalizeConfigRequest, normalize_config};
use crate::control_plane::{config_dir_from_env, home_dir_from_env, state_dir_from_env};
use crate::runtime_component_enabled;
use crate::user_config_paths;
use crate::zellij_render_plan::{
    TopLevelSetting, ZellijRenderPlanData, ZellijRenderPlanRequest, compute_zellij_render_plan,
};
use directories::ProjectDirs;
use serde::Serialize;
use serde_json::{Map as JsonMap, Value as JsonValue, json};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};
use yazelix_bar::{
    BarRenderError, BarRenderRequest, CUSTOM_TEXT_PLACEHOLDER, TAB_ACTIVE_FULLSCREEN_PLACEHOLDER,
    TAB_ACTIVE_PLACEHOLDER, TAB_ACTIVE_SYNC_PLACEHOLDER, TAB_NORMAL_FULLSCREEN_PLACEHOLDER,
    TAB_NORMAL_PLACEHOLDER, TAB_NORMAL_SYNC_PLACEHOLDER, TAB_RENAME_PLACEHOLDER,
    WIDGET_TRAY_PLACEHOLDER, YazelixRuntimeCommandPaths, ZJSTATUS_PLUGIN_URL_PLACEHOLDER,
    ZJSTATUS_RUNTIME_DIR_PLACEHOLDER, render_yazelix_runtime_command_definitions,
    render_zjstatus_bar_segments, render_zjstatus_tab_label_formats,
};

const PANE_ORCHESTRATOR_PLUGIN_PREFIX: &str = PANE_ORCHESTRATOR_PLUGIN_ALIAS;
const PANE_ORCHESTRATOR_WASM_NAME: &str = "yazelix_pane_orchestrator.wasm";
const ZJSTATUS_PLUGIN_PREFIX: &str = "zjstatus";
const ZJSTATUS_WASM_NAME: &str = "zjstatus.wasm";
const YZPP_PLUGIN_PREFIX: &str = YZPP_PLUGIN_ALIAS;
const YZPP_WASM_NAME: &str = "yzpp.wasm";
const GENERATION_METADATA_NAME: &str = ".yazelix_generation.json";
const GENERATION_FINGERPRINT_SCHEMA_VERSION: u64 = 4;
const ZELLIJ_KEYBINDINGS_CONFIG_KEY: &str = "zellij_keybindings";

const PANE_ORCHESTRATOR_PLUGIN_URL_PLACEHOLDER: &str = "__YAZELIX_PANE_ORCHESTRATOR_PLUGIN_URL__";
const HOME_DIR_PLACEHOLDER: &str = "__YAZELIX_HOME_DIR__";
const RUNTIME_DIR_PLACEHOLDER: &str = ZJSTATUS_RUNTIME_DIR_PLACEHOLDER;
const ZJSTATUS_COMMAND_DEFINITIONS_PLACEHOLDER: &str = "__YAZELIX_ZJSTATUS_COMMAND_DEFINITIONS__";

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
const YZPP_REQUIRED_PERMISSIONS: &[&str] = &[
    "ReadApplicationState",
    "ChangeApplicationState",
    "OpenTerminalsOrPlugins",
    "RunCommands",
    "ReadCliPipes",
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
    keybinds_clear_defaults: bool,
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
    if !runtime_component_enabled(&request.runtime_dir, "screen")?
        && bool_config(&config, "screen_saver_enabled", false)
    {
        return Err(CoreError::classified(
            ErrorClass::Config,
            "disabled_screen_component_screen_saver",
            "zellij.screen_saver_enabled cannot be true when the Yazelix screen component is disabled.",
            "Enable programs.yazelix.components.screen or set zellij.screen_saver_enabled to false.",
            json!({ "field": "zellij.screen_saver_enabled" }),
        ));
    }
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
    let plugin_artifacts = resolve_plugin_artifacts(&request.runtime_dir, &state_dir)?;
    let [pane_orchestrator_artifact, zjstatus_artifact, yzpp_artifact] = &plugin_artifacts;
    let zellij_keybindings = resolve_zellij_keybindings(&config)?;
    let popup_program = resolve_popup_program_config(&config);
    let render_plan_request =
        build_render_plan_request(&config, &layout_dir, &resolved_default_shell)?;
    let render_plan = compute_zellij_render_plan(&render_plan_request)?;
    let reuse_allowed = string_config(&config, "zellij_theme", "default") != "random";
    let generation_fingerprint = build_generation_fingerprint(
        &request.runtime_dir,
        &base_config_source,
        &source_layouts_dir,
        &plugin_artifacts,
        &zellij_keybindings,
        &render_plan,
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
            pane_orchestrator_runtime_path: pane_orchestrator_artifact
                .runtime_path
                .to_string_lossy()
                .to_string(),
            zjstatus_runtime_path: zjstatus_artifact.runtime_path.to_string_lossy().to_string(),
            permissions_cache_path: zellij_permissions_cache_path()?
                .to_string_lossy()
                .to_string(),
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
    let pane_orchestrator_runtime_path = pane_orchestrator_artifact.runtime_path.clone();
    let zjstatus_runtime_path = zjstatus_artifact.runtime_path.clone();
    let yzpp_runtime_path = yzpp_artifact.runtime_path.clone();
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
        base_config_source.source == "managed",
        &zellij_keybindings,
        &render_plan,
        &popup_program,
        &pane_orchestrator_runtime_path,
        &yzpp_runtime_path,
        &generation_fingerprint,
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
        permissions_cache_path: zellij_permissions_cache_path()?
            .to_string_lossy()
            .to_string(),
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
) -> Result<ZellijRenderPlanRequest, CoreError> {
    let mut request = config.clone();
    request.insert(
        "yazelix_layout_dir".to_string(),
        json!(layout_dir.to_string_lossy()),
    );
    request.insert(
        "resolved_default_shell".to_string(),
        json!(resolved_default_shell),
    );
    request.insert(
        "editor_label".to_string(),
        json!(string_config(config, "editor_command", "hx")),
    );
    request.insert(
        "shell_label".to_string(),
        json!(string_config(config, "default_shell", "nu")),
    );
    request.insert(
        "terminal_label".to_string(),
        json!(
            string_list_config(config, "terminals")
                .and_then(|values| {
                    values
                        .into_iter()
                        .map(|value| value.trim().to_string())
                        .find(|value| !value.is_empty())
                })
                .unwrap_or_else(|| "wezterm".to_string())
        ),
    );
    serde_json::from_value(JsonValue::Object(request)).map_err(|source| {
        CoreError::classified(
            ErrorClass::Config,
            "invalid_zellij_render_plan_request",
            format!("Could not build Zellij render plan from normalized config: {source}"),
            "Check settings.jsonc values under editor and zellij.",
            json!({}),
        )
    })
}

fn bool_config(config: &JsonMap<String, JsonValue>, key: &str, default: bool) -> bool {
    match config.get(key) {
        Some(JsonValue::Bool(value)) => *value,
        Some(JsonValue::String(value)) => value == "true",
        _ => default,
    }
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

fn resolve_popup_program_config(config: &JsonMap<String, JsonValue>) -> Vec<String> {
    let mut program = string_list_config(config, "popup_program")
        .unwrap_or_else(|| vec!["lazygit".to_string()])
        .into_iter()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>();
    if program.is_empty() {
        program.push("lazygit".to_string());
    }
    if program.first().map(String::as_str) == Some("editor") {
        program[0] = string_config(config, "editor_command", "hx").to_string();
    }
    program
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
    let managed_path = user_config_paths::resolve_current_config_file(
        &user_config_paths::zellij_config(&config_dir),
        &user_config_paths::legacy_zellij_config(&config_dir),
        "Zellij override",
    )?;
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
    preserve_keybinds_clear_defaults: bool,
    zellij_keybindings: &BTreeMap<String, Vec<String>>,
    render_plan: &ZellijRenderPlanData,
    popup_program: &[String],
    pane_orchestrator_wasm_path: &Path,
    yzpp_wasm_path: &Path,
    runtime_config_generation: &str,
) -> Result<String, CoreError> {
    let extracted_blocks = extract_semantic_config_blocks(base_config_content);
    let overrides_path = runtime_dir
        .join("configs")
        .join("zellij")
        .join("yazelix_overrides.kdl");
    let override_keybinds =
        read_yazelix_override_keybinds(&overrides_path, runtime_dir, zellij_keybindings)?;
    let base_config = strip_yazelix_owned_top_level_settings(
        &extracted_blocks.config_without_semantic_blocks,
        &render_plan.owned_top_level_setting_names,
    );
    let merged_keybinds = build_merged_keybinds_block(
        &extracted_blocks.keybind_lines,
        &override_keybinds,
        preserve_keybinds_clear_defaults && extracted_blocks.keybinds_clear_defaults,
    );
    let merged_ui = build_yazelix_ui_block(&extracted_blocks.ui_lines, &render_plan.rounded_value);
    let plugins_block = build_yazelix_plugins_block(
        &extracted_blocks.plugin_lines,
        pane_orchestrator_wasm_path,
        yzpp_wasm_path,
        runtime_dir,
        popup_program,
        render_plan.popup_width_percent,
        render_plan.popup_height_percent,
        render_plan.screen_saver_enabled,
        render_plan.screen_saver_idle_seconds,
        &render_plan.screen_saver_style,
        runtime_config_generation,
    );
    let load_plugins_block = build_yazelix_load_plugins_block(&extracted_blocks.load_plugin_lines);

    Ok([
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
    let mut keybinds_clear_defaults = false;
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
                if block == "keybinds" && keybinds_declares_clear_defaults(trimmed) {
                    keybinds_clear_defaults = true;
                }
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
        keybinds_clear_defaults,
        ui_lines,
    }
}

fn keybinds_declares_clear_defaults(line: &str) -> bool {
    let header = line.split('{').next().unwrap_or(line);
    header.contains("clear-defaults=true") || header.contains("clear-defaults = true")
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
    let mut merged_lines = existing_lines.to_vec();
    for alias in [PANE_ORCHESTRATOR_PLUGIN_ALIAS, YZPP_PLUGIN_ALIAS] {
        if !merged_lines.iter().any(|line| line.trim() == alias) {
            merged_lines.push(format!("  {alias}"));
        }
    }
    block_with_lines("load_plugins", &merged_lines)
}

fn build_yazelix_plugins_block(
    existing_lines: &[String],
    pane_orchestrator_wasm_path: &Path,
    yzpp_wasm_path: &Path,
    runtime_dir: &Path,
    popup_program: &[String],
    popup_width_percent: i64,
    popup_height_percent: i64,
    screen_saver_enabled: bool,
    screen_saver_idle_seconds: i64,
    screen_saver_style: &str,
    runtime_config_generation: &str,
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
            format!("        screen_saver_enabled \"{screen_saver_enabled}\""),
            format!("        screen_saver_idle_seconds \"{screen_saver_idle_seconds}\""),
            format!(
                "        screen_saver_style {}",
                json_quote(screen_saver_style)
            ),
            format!(
                "        runtime_config_generation {}",
                json_quote(runtime_config_generation)
            ),
            "    }".to_string(),
        ]);
    }

    let yzpp_present = merged_lines
        .iter()
        .any(|line| line.contains(&format!("{YZPP_PLUGIN_ALIAS} location=")));
    if !yzpp_present {
        merged_lines.extend(render_yzpp_plugin_block(
            yzpp_wasm_path,
            runtime_dir,
            popup_program,
            popup_width_percent,
            popup_height_percent,
        ));
    }

    if merged_lines.is_empty() {
        String::new()
    } else {
        block_with_lines("plugins", &merged_lines)
    }
}

fn render_yzpp_plugin_block(
    yzpp_wasm_path: &Path,
    runtime_dir: &Path,
    popup_program: &[String],
    popup_width_percent: i64,
    popup_height_percent: i64,
) -> Vec<String> {
    let yzx_cli = runtime_dir
        .join("shells")
        .join("posix")
        .join("yzx_cli.sh")
        .to_string_lossy()
        .to_string();
    let mut lines = vec![
        format!(
            "    {YZPP_PLUGIN_ALIAS} location=\"file:{}\" {{",
            yzpp_wasm_path.to_string_lossy()
        ),
        "        popups {".to_string(),
        "            popup {".to_string(),
    ];
    if let Some(command_path) = popup_program.first() {
        lines.push(format!(
            "                command {}",
            json_quote(command_path)
        ));
        for (index, arg) in popup_program.iter().skip(1).enumerate() {
            lines.push(format!(
                "                arg_{} {}",
                index + 1,
                json_quote(arg)
            ));
        }
        lines.push(format!(
            "                command_marker {}",
            json_quote(command_path)
        ));
    }
    lines.extend([
        "                pane_title \"yzx_popup\"".to_string(),
        format!("                width_percent \"{popup_width_percent}\""),
        format!("                height_percent \"{popup_height_percent}\""),
        "                on_close {".to_string(),
        format!("                    command {}", json_quote(&yzx_cli)),
        "                    arg_1 \"sidebar\"".to_string(),
        "                    arg_2 \"refresh\"".to_string(),
        "                }".to_string(),
        "            }".to_string(),
        "            menu {".to_string(),
        format!("                command {}", json_quote(&yzx_cli)),
        "                arg_1 \"menu\"".to_string(),
        "                arg_2 \"--pane\"".to_string(),
        "                pane_title \"yzx_menu\"".to_string(),
        "                command_marker \"yzx menu --pane\"".to_string(),
        format!("                width_percent \"{popup_width_percent}\""),
        format!("                height_percent \"{popup_height_percent}\""),
        "            }".to_string(),
        "            config {".to_string(),
        format!("                command {}", json_quote(&yzx_cli)),
        "                arg_1 \"config\"".to_string(),
        "                arg_2 \"ui\"".to_string(),
        "                pane_title \"yzx_config\"".to_string(),
        "                command_marker \"yzx config ui\"".to_string(),
        format!("                width_percent \"{popup_width_percent}\""),
        format!("                height_percent \"{popup_height_percent}\""),
        "            }".to_string(),
        "        }".to_string(),
        "    }".to_string(),
    ]);
    lines
}

fn build_merged_keybinds_block(
    existing_lines: &[String],
    override_lines: &[String],
    clear_defaults: bool,
) -> String {
    let mut merged = existing_lines.to_vec();
    if !clear_defaults {
        merged.extend_from_slice(override_lines);
    }
    if clear_defaults {
        block_with_lines("keybinds clear-defaults=true", &merged)
    } else if merged.is_empty() {
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

fn default_zellij_keybindings() -> BTreeMap<String, Vec<String>> {
    ZELLIJ_ACTIONS
        .iter()
        .map(|spec| {
            (
                spec.action.local_id.to_string(),
                spec.action
                    .default_keys
                    .iter()
                    .map(|key| (*key).to_string())
                    .collect(),
            )
        })
        .collect()
}

fn resolve_zellij_keybindings(
    config: &JsonMap<String, JsonValue>,
) -> Result<BTreeMap<String, Vec<String>>, CoreError> {
    let mut resolved = default_zellij_keybindings();
    let Some(value) = config.get(ZELLIJ_KEYBINDINGS_CONFIG_KEY) else {
        validate_zellij_keybindings(&resolved)?;
        return Ok(resolved);
    };
    let Some(object) = value.as_object() else {
        return Err(CoreError::classified(
            ErrorClass::Config,
            "invalid_zellij_keybindings",
            "zellij.keybindings must be an object whose values are lists of Zellij key strings.",
            "Use settings such as `\"popup\": [\"Alt t\"]`, or remove zellij.keybindings to use Yazelix defaults.",
            json!({ "actual": value }),
        ));
    };

    for (action, raw_keys) in object {
        if !is_supported_zellij_keybinding_action(action) {
            return Err(CoreError::classified(
                ErrorClass::Config,
                "unsupported_zellij_keybinding_action",
                format!("Unsupported Zellij keybinding action: {action}."),
                "Use one of the supported Yazelix Zellij action ids, or remove the unsupported keybinding entry.",
                json!({
                    "action": action,
                    "supported_actions": supported_zellij_keybinding_actions(),
                }),
            ));
        }
        let Some(values) = raw_keys.as_array() else {
            return Err(CoreError::classified(
                ErrorClass::Config,
                "invalid_zellij_keybinding_keys",
                format!("zellij.keybindings.{action} must be a list of Zellij key strings."),
                "Use a list such as `[\"Alt t\"]`, or an empty list to disable that Yazelix action binding.",
                json!({ "action": action, "actual": raw_keys }),
            ));
        };
        let mut keys = Vec::with_capacity(values.len());
        for value in values {
            let Some(raw_key) = value.as_str() else {
                return Err(CoreError::classified(
                    ErrorClass::Config,
                    "invalid_zellij_keybinding_key",
                    format!("zellij.keybindings.{action} contains a non-string key."),
                    "Use Zellij key strings such as \"Alt t\" or \"Ctrl y\".",
                    json!({ "action": action, "actual": value }),
                ));
            };
            let key = raw_key.trim();
            if key.is_empty() || key.contains('\n') || key.contains('\r') {
                return Err(CoreError::classified(
                    ErrorClass::Config,
                    "invalid_zellij_keybinding_key",
                    format!("zellij.keybindings.{action} contains an invalid key string."),
                    "Use a non-empty single-line Zellij key string such as \"Alt t\".",
                    json!({ "action": action, "actual": raw_key }),
                ));
            }
            keys.push(key.to_string());
        }
        resolved.insert(action.clone(), keys);
    }

    validate_zellij_keybindings(&resolved)?;
    Ok(resolved)
}

fn validate_zellij_keybindings(
    keybindings: &BTreeMap<String, Vec<String>>,
) -> Result<(), CoreError> {
    let mut seen = BTreeMap::<String, String>::new();
    for spec in ZELLIJ_ACTIONS {
        let Some(keys) = keybindings.get(spec.action.local_id) else {
            continue;
        };
        for key in keys {
            if let Some(existing_action) =
                seen.insert(key.clone(), spec.action.local_id.to_string())
            {
                return Err(CoreError::classified(
                    ErrorClass::Config,
                    "duplicate_zellij_keybinding",
                    format!(
                        "Zellij keybinding {key:?} is assigned to both {existing_action} and {}.",
                        spec.action.local_id
                    ),
                    "Give each Yazelix Zellij action a distinct key, or set one action to an empty list to disable its binding.",
                    json!({
                        "key": key,
                        "first_action": existing_action,
                        "second_action": spec.action.local_id,
                    }),
                ));
            }
        }
    }
    Ok(())
}

fn is_supported_zellij_keybinding_action(action: &str) -> bool {
    ZELLIJ_ACTIONS
        .iter()
        .any(|spec| spec.action.local_id == action)
}

fn supported_zellij_keybinding_actions() -> Vec<&'static str> {
    ZELLIJ_ACTIONS
        .iter()
        .map(|spec| spec.action.local_id)
        .collect()
}

fn read_yazelix_override_keybinds(
    overrides_path: &Path,
    runtime_dir: &Path,
    zellij_keybindings: &BTreeMap<String, Vec<String>>,
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
    let assigned_keys = assigned_zellij_keybinding_keys(zellij_keybindings);
    let mut keybind_lines = extract_semantic_config_blocks(&content)
        .keybind_lines
        .into_iter()
        .filter(|line| !unbind_line_conflicts_with_semantic_key(line, &assigned_keys))
        .collect::<Vec<_>>();
    keybind_lines.extend(build_semantic_zellij_keybind_lines(zellij_keybindings));
    Ok(keybind_lines)
}

fn assigned_zellij_keybinding_keys(
    keybindings: &BTreeMap<String, Vec<String>>,
) -> BTreeSet<String> {
    keybindings
        .values()
        .flatten()
        .cloned()
        .collect::<BTreeSet<_>>()
}

fn unbind_line_conflicts_with_semantic_key(line: &str, assigned_keys: &BTreeSet<String>) -> bool {
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

fn build_semantic_zellij_keybind_lines(keybindings: &BTreeMap<String, Vec<String>>) -> Vec<String> {
    let mut by_mode = BTreeMap::<&str, Vec<String>>::new();
    for spec in ZELLIJ_ACTIONS {
        let Some(keys) = keybindings.get(spec.action.local_id) else {
            continue;
        };
        if keys.is_empty() {
            continue;
        }
        let mode_lines = by_mode.entry(spec.mode).or_default();
        push_zellij_message_bind(
            mode_lines,
            keys,
            spec.plugin_alias,
            spec.message_name,
            spec.payload,
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
    let expected_targets = expected_layout_targets_for_dir(source_dir, target_dir)?;
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
    render_zjstatus_bar_segments(&request).map_err(bar_render_error)
}

fn integrated_zjstatus_runtime_paths(
    runtime_dir: &Path,
    render_plan: &ZellijRenderPlanData,
) -> YazelixRuntimeCommandPaths {
    YazelixRuntimeCommandPaths {
        nu_bin: resolve_zjstatus_nu_bin(runtime_dir),
        yzx_control_bin: resolve_zjstatus_yzx_control_bin(runtime_dir),
        yazelix_bar_widget_bin: resolve_zjstatus_yazelix_bar_widget_bin(runtime_dir),
        runtime_dir: runtime_dir.to_string_lossy().to_string(),
        claude_usage_display: render_plan.claude_usage_display.clone(),
        codex_usage_display: render_plan.codex_usage_display.clone(),
        opencode_go_usage_display: render_plan.opencode_go_usage_display.clone(),
    }
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
    let tab_labels =
        render_zjstatus_tab_label_formats(&render_plan.tab_label_mode).map_err(bar_render_error)?;
    let replacements = [
        (WIDGET_TRAY_PLACEHOLDER, widget_tray_segment.to_string()),
        (CUSTOM_TEXT_PLACEHOLDER, custom_text_segment.to_string()),
        (TAB_NORMAL_PLACEHOLDER, tab_labels.tab_normal.to_string()),
        (
            TAB_NORMAL_FULLSCREEN_PLACEHOLDER,
            tab_labels.tab_normal_fullscreen.to_string(),
        ),
        (
            TAB_NORMAL_SYNC_PLACEHOLDER,
            tab_labels.tab_normal_sync.to_string(),
        ),
        (TAB_ACTIVE_PLACEHOLDER, tab_labels.tab_active.to_string()),
        (
            TAB_ACTIVE_FULLSCREEN_PLACEHOLDER,
            tab_labels.tab_active_fullscreen.to_string(),
        ),
        (
            TAB_ACTIVE_SYNC_PLACEHOLDER,
            tab_labels.tab_active_sync.to_string(),
        ),
        (TAB_RENAME_PLACEHOLDER, tab_labels.tab_rename.to_string()),
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
            ZJSTATUS_COMMAND_DEFINITIONS_PLACEHOLDER,
            render_yazelix_runtime_command_definitions(&integrated_zjstatus_runtime_paths(
                runtime_dir,
                render_plan,
            )),
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
        ZJSTATUS_COMMAND_DEFINITIONS_PLACEHOLDER,
        TAB_NORMAL_PLACEHOLDER,
        TAB_NORMAL_FULLSCREEN_PLACEHOLDER,
        TAB_NORMAL_SYNC_PLACEHOLDER,
        TAB_ACTIVE_PLACEHOLDER,
        TAB_ACTIVE_FULLSCREEN_PLACEHOLDER,
        TAB_ACTIVE_SYNC_PLACEHOLDER,
        TAB_RENAME_PLACEHOLDER,
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

fn bar_render_error(error: BarRenderError) -> CoreError {
    match error {
        BarRenderError::InvalidWidgetTrayEntry { entry } => CoreError::classified(
            ErrorClass::Config,
            "invalid_widget_tray_entry",
            format!("Invalid zellij.widget_tray token in layout renderer: {entry}"),
            "Use only documented widget tray identifiers.",
            json!({ "field": "zellij.widget_tray", "entry": entry }),
        ),
        BarRenderError::InvalidTabLabelMode { mode } => CoreError::classified(
            ErrorClass::Config,
            "invalid_tab_label_mode",
            format!("Invalid zellij.tab_label_mode in layout renderer: {mode}"),
            "Set zellij.tab_label_mode to `full` or `compact`.",
            json!({ "field": "zellij.tab_label_mode", "mode": mode }),
        ),
    }
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

fn resolve_zjstatus_yazelix_bar_widget_bin(runtime_dir: &Path) -> String {
    let runtime_widget = runtime_dir.join("libexec").join("yazelix_bar_widget");
    if runtime_widget.is_file() {
        runtime_widget.to_string_lossy().to_string()
    } else if let Some(path) = env_path_if_file("YAZELIX_BAR_WIDGET_BIN") {
        path.to_string_lossy().to_string()
    } else {
        "yazelix_bar_widget".to_string()
    }
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
    expected_layout_targets_for_dir(source_layouts_dir, &merged_config_dir.join("layouts"))
}

fn expected_layout_targets_for_dir(
    source_layouts_dir: &Path,
    target_dir: &Path,
) -> Result<Vec<PathBuf>, CoreError> {
    list_top_level_kdl_files(source_layouts_dir)?
        .into_iter()
        .map(|source| Ok(target_dir.join(Path::new(required_file_name(&source)?))))
        .collect()
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
) -> Result<[PluginArtifact; 3], CoreError> {
    let plugin_dir = state_dir.join("configs").join("zellij").join("plugins");
    let [pane_orchestrator, zjstatus, yzpp] = [
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
        (
            "yzpp",
            YZPP_PLUGIN_PREFIX,
            YZPP_WASM_NAME,
            YZPP_REQUIRED_PERMISSIONS,
        ),
    ]
    .map(|(name, prefix, wasm_name, permissions)| {
        resolve_plugin_artifact(
            runtime_dir,
            &plugin_dir,
            name,
            prefix,
            wasm_name,
            permissions,
        )
    });
    Ok([pane_orchestrator?, zjstatus?, yzpp?])
}

fn resolve_plugin_artifact(
    runtime_dir: &Path,
    plugin_dir: &Path,
    name: &'static str,
    prefix: &'static str,
    wasm_name: &'static str,
    required_permissions: &'static [&'static str],
) -> Result<PluginArtifact, CoreError> {
    let tracked_path = runtime_dir
        .join("configs")
        .join("zellij")
        .join("plugins")
        .join(wasm_name);
    if !tracked_path.exists() {
        return Err(CoreError::classified(
            ErrorClass::Io,
            "missing_tracked_zellij_plugin",
            format!(
                "Tracked {name} wasm not found at: {}",
                tracked_path.to_string_lossy()
            ),
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
}

fn sync_plugin_artifacts(
    plugin_artifacts: &[PluginArtifact; 3],
    seed_plugin_permissions: bool,
) -> Result<(), CoreError> {
    for artifact in plugin_artifacts.iter() {
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
    let permissions_cache_path = zellij_permissions_cache_path()?;
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
    retained.extend(required_permission_blocks(
        tracked_path,
        runtime_path,
        required_permissions,
    ));
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

fn upsert_plugin_permission_blocks(
    plugin_artifacts: &[PluginArtifact; 3],
) -> Result<(), CoreError> {
    let permissions_cache_path = zellij_permissions_cache_path()?;
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

    for artifact in plugin_artifacts.iter() {
        updated.extend(required_permission_blocks(
            &artifact.tracked_path,
            &artifact.runtime_path,
            artifact.required_permissions,
        ));
    }

    write_text_atomic(&permissions_cache_path, &updated.join("\n\n"))
}

fn required_permission_blocks(
    tracked_path: &Path,
    runtime_path: &Path,
    required_permissions: &[&str],
) -> [String; 2] {
    let permissions = required_permissions
        .iter()
        .map(|permission| (*permission).to_string())
        .collect::<Vec<_>>();
    [
        build_permission_block(&tracked_path.to_string_lossy(), &permissions),
        build_permission_block(&runtime_path.to_string_lossy(), &permissions),
    ]
}

fn build_generation_fingerprint(
    runtime_dir: &Path,
    base_config_source: &ZellijBaseConfigSource,
    source_layouts_dir: &Path,
    plugin_artifacts: &[PluginArtifact; 3],
    zellij_keybindings: &BTreeMap<String, Vec<String>>,
    render_plan: &ZellijRenderPlanData,
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
    let fingerprint_payload = json!({
        "schema_version": GENERATION_FINGERPRINT_SCHEMA_VERSION,
        "runtime_dir": runtime_dir.to_string_lossy(),
        "render_plan": render_plan,
        "zellij_keybindings": zellij_keybindings,
        "base_config": {
            "source": base_config_source.source,
            "path": base_config_source.path.as_ref().map(|path| path.to_string_lossy().to_string()).unwrap_or_default(),
            "hash": hash_text(&base_config_source.content),
        },
        "overrides_hash": hash_text(&read_text_if_exists(&overrides_path)?),
        "layout_sources": layout_sources,
        "plugins": plugin_artifacts.iter().map(|artifact| {
            json!({
                "name": artifact.name,
                "tracked_path": artifact.tracked_path.to_string_lossy(),
                "tracked_hash": artifact.tracked_hash,
                "runtime_path": artifact.runtime_path.to_string_lossy(),
                "wasm_name": artifact.wasm_name,
            })
        }).collect::<Vec<_>>(),
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
    plugin_artifacts: &[PluginArtifact; 3],
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
    for artifact in plugin_artifacts.iter() {
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

pub(crate) fn zellij_permissions_cache_path() -> Result<PathBuf, CoreError> {
    ProjectDirs::from("org", "Zellij Contributors", "Zellij")
        .map(|dirs| dirs.cache_dir().join("permissions.kdl"))
        .ok_or_else(|| {
            CoreError::classified(
                ErrorClass::Runtime,
                "resolve_zellij_permissions_cache",
                "Could not resolve Zellij's plugin permission cache directory.",
                "Ensure HOME is set, then retry.",
                json!({}),
            )
        })
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
    use crate::zellij_render_plan::DEFAULT_SIDEBAR_YAZI_ARG;

    fn sample_render_plan_for_widgets(
        widget_tray: Vec<&str>,
        editor_label: &str,
        shell: &str,
        terminal_label: &str,
    ) -> ZellijRenderPlanData {
        compute_zellij_render_plan(&ZellijRenderPlanRequest {
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
            zellij_tab_label_mode: "full".into(),
            zellij_claude_usage_display: "both".into(),
            zellij_codex_usage_display: "quota".into(),
            zellij_opencode_go_usage_display: "both".into(),
            yazelix_layout_dir: "/tmp/yazelix/layouts".into(),
            resolved_default_shell: shell.into(),
            editor_label: editor_label.into(),
            shell_label: "nu".into(),
            terminal_label: terminal_label.into(),
        })
        .unwrap()
    }

    fn sample_zellij_keybindings() -> BTreeMap<String, Vec<String>> {
        default_zellij_keybindings()
    }

    // Regression: macOS Zellij reads plugin permissions from ProjectDirs' Library/Caches path, not ~/.cache/zellij.
    #[cfg(target_os = "macos")]
    #[test]
    fn zellij_permissions_cache_path_uses_macos_cache_location() {
        let path = zellij_permissions_cache_path()
            .unwrap()
            .to_string_lossy()
            .to_string();

        assert!(path.ends_with("/Library/Caches/org.Zellij-Contributors.Zellij/permissions.kdl"));
    }

    // Defends: semantic block extraction removes first-class KDL blocks while preserving unrelated top-level lines.
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
        assert!(!extracted.keybinds_clear_defaults);
    }

    // Defends: user-owned Zellij keybinding mode survives semantic block extraction instead of being reduced to an ordinary keybind body.
    #[test]
    fn extracts_keybinds_clear_defaults_ownership() {
        let extracted = extract_semantic_config_blocks(
            r#"keybinds clear-defaults=true {
    locked { bind "Ctrl `" { SwitchToMode "Normal"; } }
}
"#,
        );

        assert!(extracted.keybinds_clear_defaults);
        assert!(
            extracted
                .keybind_lines
                .iter()
                .any(|line| line.contains("Ctrl `"))
        );
    }

    // Defends: full user-owned Zellij keybinding mode preserves the clear-defaults header and does not append Yazelix default integrations.
    #[test]
    fn clear_defaults_keybinds_skip_yazelix_overrides() {
        let existing_lines = vec![
            r#"    locked { bind "Ctrl `" { SwitchToMode "Normal"; } }"#.to_string(),
            r#"    normal { bind "Alt y" { Write 27; } }"#.to_string(),
        ];
        let override_lines = vec![
            r#"    shared_except "locked" {"#.to_string(),
            r#"        bind "Alt y" { MessagePlugin "yazelix_pane_orchestrator" { name "toggle_sidebar" } }"#.to_string(),
            r#"    }"#.to_string(),
        ];

        let merged = build_merged_keybinds_block(&existing_lines, &override_lines, true);

        assert!(merged.starts_with("keybinds clear-defaults=true {"));
        assert!(merged.contains("Ctrl `"));
        assert!(merged.contains("Write 27"));
        assert!(!merged.contains("MessagePlugin"));
        assert!(!merged.contains("toggle_sidebar"));
    }

    // Regression: clear-defaults from the read-only native fallback must not disable Yazelix integration keybindings.
    // Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
    #[test]
    fn native_fallback_clear_defaults_keeps_yazelix_keybind_overrides() {
        let temp = tempfile::tempdir().unwrap();
        let runtime_dir = temp.path();
        let overrides_dir = runtime_dir.join("configs").join("zellij");
        std::fs::create_dir_all(&overrides_dir).unwrap();
        std::fs::write(
            overrides_dir.join("yazelix_overrides.kdl"),
            r#"
keybinds {
    normal {
        bind "Alt Shift M" {
            MessagePlugin "yzpp" {
                name "toggle"
                payload "menu"
            }
        }
    }
}
"#,
        )
        .unwrap();
        let plan =
            sample_render_plan_for_widgets(vec!["workspace"], "hx", "/nix/store/bin/nu", "ghostty");
        let rendered = render_merged_config(
            runtime_dir,
            r#"keybinds clear-defaults=true {
    normal { bind "Alt h" { MoveFocusOrTab "left"; } }
}
"#,
            false,
            &sample_zellij_keybindings(),
            &plan,
            &["lazygit".to_string()],
            Path::new("/tmp/pane.wasm"),
            Path::new("/tmp/yzpp.wasm"),
            "gen-test",
        )
        .unwrap();

        assert!(rendered.contains("keybinds {"));
        assert!(!rendered.contains("keybinds clear-defaults=true"));
        assert!(rendered.contains("MoveFocusOrTab"));
        assert!(rendered.contains("Alt Shift M"));
        assert!(rendered.contains("MessagePlugin \"yzpp\""));
        assert!(rendered.contains("payload \"menu\""));
    }

    // Defends: explicit managed zellij.kdl clear-defaults remains the full user-owned Zellij keybinding mode.
    // Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
    #[test]
    fn managed_clear_defaults_skips_yazelix_keybind_overrides() {
        let temp = tempfile::tempdir().unwrap();
        let runtime_dir = temp.path();
        let overrides_dir = runtime_dir.join("configs").join("zellij");
        std::fs::create_dir_all(&overrides_dir).unwrap();
        std::fs::write(
            overrides_dir.join("yazelix_overrides.kdl"),
            r#"
keybinds {
    normal {
        bind "Alt Shift M" {
            MessagePlugin "yzpp" {
                name "toggle"
                payload "menu"
            }
        }
    }
}
"#,
        )
        .unwrap();
        let plan =
            sample_render_plan_for_widgets(vec!["workspace"], "hx", "/nix/store/bin/nu", "ghostty");
        let rendered = render_merged_config(
            runtime_dir,
            r#"keybinds clear-defaults=true {
    locked { bind "Ctrl `" { SwitchToMode "Normal"; } }
}
"#,
            true,
            &sample_zellij_keybindings(),
            &plan,
            &["lazygit".to_string()],
            Path::new("/tmp/pane.wasm"),
            Path::new("/tmp/yzpp.wasm"),
            "gen-test",
        )
        .unwrap();

        assert!(rendered.contains("keybinds clear-defaults=true"));
        assert!(rendered.contains("Ctrl `"));
        assert!(!rendered.contains("Alt Shift M"));
        assert!(!rendered.contains("payload \"menu\""));
    }

    // Defends: ordinary Zellij keybinding customization keeps Yazelix integration bindings appended for default managed behavior.
    #[test]
    fn ordinary_keybinds_keep_yazelix_overrides() {
        let existing_lines = vec![r#"    normal { bind "Alt y" { Write 27; } }"#.to_string()];
        let override_lines = vec![
            r#"    shared_except "locked" {"#.to_string(),
            r#"        bind "Alt y" { MessagePlugin "yazelix_pane_orchestrator" { name "toggle_sidebar" } }"#.to_string(),
            r#"    }"#.to_string(),
        ];

        let merged = build_merged_keybinds_block(&existing_lines, &override_lines, false);

        assert!(merged.starts_with("keybinds {"));
        assert!(!merged.starts_with("keybinds clear-defaults=true"));
        assert!(merged.contains("Write 27"));
        assert!(merged.contains("MessagePlugin"));
        assert!(merged.contains("toggle_sidebar"));
    }

    // Regression: widget tray rendering must not leave empty command placeholders when dynamic helper scripts are unavailable.
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
    #[test]
    fn renders_custom_sidebar_command_without_implicit_yazi_launcher_arg() {
        let mut config = JsonMap::new();
        config.insert("sidebar_command".into(), json!("lazygit"));
        config.insert("sidebar_args".into(), json!([DEFAULT_SIDEBAR_YAZI_ARG]));
        let request = build_render_plan_request(
            &config,
            std::path::Path::new("/tmp/yazelix/layouts"),
            "/nix/store/bin/nu",
        )
        .unwrap();
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

    // Regression: generated zjstatus command widgets use resolved runtime helpers while dynamic widgets read only the local cache.
    #[test]
    fn renders_cached_zjstatus_widget_commands_with_runtime_helper_paths() {
        let temp = tempfile::tempdir().unwrap();
        let runtime_dir = temp.path();
        let libexec = runtime_dir.join("libexec");
        std::fs::create_dir_all(&libexec).unwrap();
        std::fs::write(libexec.join("nu"), "").unwrap();
        std::fs::write(libexec.join("yzx_control"), "").unwrap();
        std::fs::write(libexec.join("yazelix_bar_widget"), "").unwrap();
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
        let expected_bar_widget = libexec
            .join("yazelix_bar_widget")
            .to_string_lossy()
            .to_string();

        assert!(rendered.contains(&format!(
            r#"command_cpu_command "{} cpu""#,
            expected_bar_widget
        )));
        assert!(rendered.contains(&format!(
            r#"command_ram_command "{} ram""#,
            expected_bar_widget
        )));
        assert!(rendered.contains(&format!(
            r#"command_workspace_command "{} zellij status-cache-widget workspace""#,
            expected_yzx_control
        )));
        assert!(rendered.contains(r##"command_workspace_format "#[fg=#00ff88,bold]{stdout}""##));
        assert!(rendered.contains(r#"command_workspace_interval "1""#));
        assert!(rendered.contains(&format!(
            r#"command_cursor_command "{} cursor""#,
            expected_bar_widget
        )));
        assert!(rendered.contains(r#"command_cursor_format "{stdout}""#));
        assert!(rendered.contains(r#"command_cursor_rendermode "dynamic""#));
        assert!(rendered.contains(r#"command_cursor_interval "10""#));
        assert!(rendered.contains(&format!(
            r#"command_claude_usage_command "{} claude_usage --display both""#,
            expected_bar_widget
        )));
        assert!(rendered.contains(r##"command_claude_usage_format "#[fg=#bb88ff,bold]{stdout}""##));
        assert!(rendered.contains(r#"command_claude_usage_interval "10""#));
        assert!(rendered.contains(&format!(
            r#"command_codex_usage_command "{} codex_usage --display quota""#,
            expected_bar_widget
        )));
        assert!(rendered.contains(r##"command_codex_usage_format "#[fg=#bb88ff,bold]{stdout}""##));
        assert!(rendered.contains(r#"command_codex_usage_interval "10""#));
        assert!(rendered.contains(&format!(
            r#"command_opencode_go_usage_command "{} opencode_go_usage --display both""#,
            expected_bar_widget
        )));
        assert!(
            rendered.contains(r##"command_opencode_go_usage_format "#[fg=#bb88ff,bold]{stdout}""##)
        );
        assert!(rendered.contains(r#"command_opencode_go_usage_interval "10""#));
        assert!(rendered.contains(&format!(
            r#"command_version_command "{} -c 'use {}/nushell/scripts/utils/constants.nu YAZELIX_VERSION; $YAZELIX_VERSION'""#,
            expected_nu,
            runtime_dir.to_string_lossy()
        )));
        assert!(!rendered.contains(yazelix_bar::ZJSTATUS_YZX_CONTROL_BIN_PLACEHOLDER));
        assert!(!rendered.contains(yazelix_bar::ZJSTATUS_NU_BIN_PLACEHOLDER));
        assert!(!rendered.contains(ZJSTATUS_COMMAND_DEFINITIONS_PLACEHOLDER));
        assert!(!rendered.contains("status-bus-workspace"));
        assert!(!rendered.contains("agent-usage"));
    }

    // Invariant: compact tab-label mode shortens zjstatus tab labels in generated layout KDL without affecting rename text.
    #[test]
    fn renders_compact_tab_label_mode_in_zjstatus_template() {
        let mut plan =
            sample_render_plan_for_widgets(vec!["workspace"], "hx", "/nix/store/bin/nu", "ghostty");
        plan.tab_label_mode = "compact".into();
        let rendered = render_layout_template(
            include_str!("../../../configs/zellij/layouts/fragments/zjstatus_tab_template.kdl"),
            &BTreeMap::new(),
            "",
            "",
            "file:/tmp/pane.wasm",
            "file:/tmp/zjstatus.wasm",
            std::path::Path::new("/home/user"),
            std::path::Path::new("/opt/yazelix"),
            &plan,
        )
        .unwrap();

        assert!(rendered.contains(r##"tab_normal   "#[fg=#ffff00] [{index}] ""##));
        assert!(rendered.contains(
            r##"tab_active   "#[bg=#ff6600,fg=#000000,bold] [{index}] {floating_indicator}""##
        ));
        assert!(rendered.contains(
            r##"tab_rename    "#[bg=#ff6600,fg=#000000,bold] {index} {name} {floating_indicator} ""##
        ));
        assert!(!rendered.contains(r##"tab_normal   "#[fg=#ffff00] [{index}] {name} ""##));
        assert!(!rendered.contains("__YAZELIX_ZJSTATUS_TAB_NORMAL__"));
    }

    // Regression: legacy plugin permission blocks are recognized by both stable and hashed wasm names.
    #[test]
    fn plugin_prefix_matches_stable_and_hashed_names() {
        assert!(plugin_name_matches_prefix("zjstatus.wasm", "zjstatus"));
        assert!(plugin_name_matches_prefix(
            "zjstatus_abc123.wasm",
            "zjstatus"
        ));
        assert!(!plugin_name_matches_prefix("not_zjstatus.wasm", "zjstatus"));
    }

    // Regression: semantic keybinding generation routes popup/menu/config to yzpp while keeping workspace actions on the pane orchestrator.
    #[test]
    fn semantic_keybinds_route_popup_actions_to_yzpp() {
        let temp = tempfile::tempdir().unwrap();
        let overrides_path = temp.path().join("yazelix_overrides.kdl");
        std::fs::write(
            &overrides_path,
            r#"
keybinds {
    shared {
        unbind "Alt p"
    }
}
"#,
        )
        .unwrap();
        let override_lines = read_yazelix_override_keybinds(
            &overrides_path,
            std::path::Path::new("/opt/yazelix"),
            &sample_zellij_keybindings(),
        )
        .unwrap();
        let merged = override_lines.join("\n");

        assert!(merged.contains("MessagePlugin \"yzpp\""));
        assert!(merged.contains("name \"toggle\""));
        assert!(merged.contains("open_workspace_terminal"));
        assert!(merged.contains("payload \"popup\""));
        assert!(merged.contains("payload \"menu\""));
        assert!(merged.contains("payload \"config\""));
        assert!(merged.contains("MessagePlugin \"yazelix_pane_orchestrator\""));
        assert!(merged.contains("toggle_editor_sidebar_focus"));
        assert!(merged.contains("move_focus_left_or_tab"));
    }

    // Defends: semantic remaps replace Yazelix-owned Zellij action keys without copying the full keybind block.
    #[test]
    fn semantic_keybinds_honor_user_remaps_and_drop_matching_unbinds() {
        let temp = tempfile::tempdir().unwrap();
        let overrides_path = temp.path().join("yazelix_overrides.kdl");
        std::fs::write(
            &overrides_path,
            r#"
keybinds {
    shared {
        unbind "Alt p"
    }
}
"#,
        )
        .unwrap();
        let mut keybindings = sample_zellij_keybindings();
        keybindings.insert("popup".to_string(), vec!["Alt p".to_string()]);
        keybindings.insert("menu".to_string(), vec!["Alt Space".to_string()]);
        validate_zellij_keybindings(&keybindings).unwrap();

        let merged = read_yazelix_override_keybinds(
            &overrides_path,
            std::path::Path::new("/opt/yazelix"),
            &keybindings,
        )
        .unwrap()
        .join("\n");

        assert!(!merged.contains(r#"unbind "Alt p""#));
        assert!(merged.contains(r#"bind "Alt p" {"#));
        assert!(merged.contains(r#"payload "popup""#));
        assert!(merged.contains(r#"bind "Alt Space" {"#));
        assert!(merged.contains(r#"payload "menu""#));
        assert!(!merged.contains(r#"bind "Alt t" {"#));
        assert!(!merged.contains(r#"bind "Alt Shift M" {"#));
    }

    // Defends: generated semantic Zellij action bindings fail fast when two actions claim the same key.
    #[test]
    fn semantic_keybinds_reject_duplicate_action_keys() {
        let mut keybindings = sample_zellij_keybindings();
        keybindings.insert("popup".to_string(), vec!["Alt Space".to_string()]);
        keybindings.insert("menu".to_string(), vec!["Alt Space".to_string()]);

        let error = validate_zellij_keybindings(&keybindings).unwrap_err();

        assert_eq!(error.code(), "duplicate_zellij_keybinding");
        assert_eq!(error.class().as_str(), "config");
    }

    // Defends: partial settings.jsonc remaps inherit defaults for omitted Yazelix-owned Zellij actions.
    #[test]
    fn partial_zellij_keybinding_config_inherits_omitted_defaults() {
        let mut config = JsonMap::new();
        config.insert(
            ZELLIJ_KEYBINDINGS_CONFIG_KEY.to_string(),
            json!({
                "menu": ["Alt Space"],
                "toggle_sidebar": [],
            }),
        );

        let keybindings = resolve_zellij_keybindings(&config).unwrap();

        assert_eq!(keybindings["menu"], vec!["Alt Space"]);
        assert_eq!(keybindings["toggle_sidebar"], Vec::<String>::new());
        assert_eq!(keybindings["popup"], vec!["Alt t"]);
    }

    // Regression: generated plugin config must carry the pane-orchestrator runtime contract and yzpp popup contract without duplicate alias injection.
    // Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
    #[test]
    fn plugin_block_carries_runtime_and_popup_contract_once() {
        let block = build_yazelix_plugins_block(
            &[],
            std::path::Path::new("/opt/yazelix/plugins/yazelix_pane_orchestrator.wasm"),
            std::path::Path::new("/opt/yazelix/plugins/yzpp.wasm"),
            std::path::Path::new("/opt/yazelix"),
            &["lazygit".to_string()],
            82,
            76,
            true,
            180,
            "mandelbrot",
            "gen-test",
        );

        assert!(block.contains("yazelix_pane_orchestrator location=\"file:/opt/yazelix/plugins/yazelix_pane_orchestrator.wasm\""));
        assert!(block.contains("runtime_dir \"/opt/yazelix\""));
        assert!(!block.contains("popup_width_percent"));
        assert!(!block.contains("popup_height_percent"));
        assert!(block.contains("screen_saver_enabled \"true\""));
        assert!(block.contains("screen_saver_idle_seconds \"180\""));
        assert!(block.contains("screen_saver_style \"mandelbrot\""));
        assert!(block.contains("runtime_config_generation \"gen-test\""));
        assert!(block.contains("yzpp location=\"file:/opt/yazelix/plugins/yzpp.wasm\""));
        assert!(block.contains("command \"lazygit\""));
        assert!(block.contains("width_percent \"82\""));
        assert!(block.contains("height_percent \"76\""));
        assert!(block.contains("on_close {"));
        assert!(block.contains("arg_1 \"sidebar\""));
        assert!(block.contains("arg_2 \"refresh\""));
        assert!(!block.contains("widget_tray_segment"));
        assert!(!block.contains("custom_text_segment"));
        assert!(!block.contains("sidebar_width_percent"));
        assert_eq!(
            block.matches("yazelix_pane_orchestrator location=").count(),
            1
        );
        assert_eq!(block.matches("yzpp location=").count(), 1);
    }
}
