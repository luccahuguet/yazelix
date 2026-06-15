use crate::action_registry::{
    PANE_ORCHESTRATOR_PLUGIN_ALIAS, YZPP_PLUGIN_ALIAS, YazelixActionMetadata, ZELLIJ_ACTIONS,
    ZELLIJ_NATIVE_KEYBINDINGS, ZellijNativeKeybindingBlock, zellij_action_by_local_id,
    zellij_native_keybinding_by_local_id,
};
use crate::backup_timestamp::epoch_millis_timestamp;
use crate::bridge::{CoreError, ErrorClass};
use crate::config_normalize::{NormalizeConfigRequest, normalize_config};
use crate::control_plane::{
    config_dir_from_env, home_dir_from_env, state_dir_from_env, zellij_default_shell_from_runtime,
};
use crate::runtime_component_enabled;
use crate::terminal_variant::active_terminal_from_runtime_dir;
use crate::user_config_paths;
use crate::zellij_materialization_io::{
    hash_text, read_text, read_text_if_exists, write_text_atomic,
};
pub(crate) use crate::zellij_plugin_materialization::zellij_permissions_cache_path;
use crate::zellij_plugin_materialization::{
    PluginArtifact, resolve_plugin_artifacts, sync_plugin_artifacts,
};
use serde::{Deserialize, Serialize};
use serde_json::{Map as JsonMap, Value as JsonValue, json};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use yazelix_zellij_config_pack::{
    self as zellij_config_pack, ZellijRenderPlanData, ZellijRenderPlanError,
    ZellijRenderPlanRequest, compute_zellij_render_plan,
};

const GENERATION_METADATA_NAME: &str = ".yazelix_generation.json";
const GENERATION_FINGERPRINT_SCHEMA_VERSION: u64 = 8;
const GENERATED_CONFIG_MARKERS: &[&str] = &[
    "GENERATED ZELLIJ CONFIG (YAZELIX)",
    "yazelix_pane_orchestrator",
    "yzpp",
];
const GENERATED_LAYOUT_MARKER: &str = "GENERATED ZELLIJ LAYOUT (YAZELIX)";
const GENERATED_LAYOUT_FINGERPRINT_PREFIX: &str = "generation_fingerprint:";
const ZELLIJ_KEYBINDINGS_CONFIG_KEY: &str = "zellij_keybindings";
const ZELLIJ_NATIVE_KEYBINDINGS_CONFIG_KEY: &str = "zellij_native_keybindings";
const ZELLIJ_KEYBINDING_PARSE_POLICY: KeybindingParsePolicy = KeybindingParsePolicy {
    namespace: "zellij.keybindings",
    invalid_keys_code: "invalid_zellij_keybinding_keys",
    invalid_key_code: "invalid_zellij_keybinding_key",
    list_remediation: "Use a list such as `[\"Alt Shift J\"]`, or an empty list to disable that Yazelix action binding.",
    item_remediation: "Use Zellij key strings such as \"Alt Shift J\" or \"Ctrl y\".",
    invalid_item_remediation: "Use a non-empty single-line Zellij key string such as \"Alt Shift J\".",
};
const ZELLIJ_NATIVE_KEYBINDING_PARSE_POLICY: KeybindingParsePolicy = KeybindingParsePolicy {
    namespace: "zellij.native_keybindings",
    invalid_keys_code: "invalid_zellij_native_keybinding_keys",
    invalid_key_code: "invalid_zellij_native_keybinding_key",
    list_remediation: "Use a list such as `[\"Ctrl Alt s\"]`, or an empty list to disable that native policy binding.",
    item_remediation: "Use Zellij key strings such as \"Ctrl Alt s\" or \"Ctrl Alt h\".",
    invalid_item_remediation: "Use a non-empty single-line Zellij key string such as \"Ctrl Alt s\".",
};
const POPUP_COMMANDS_CONFIG_KEY: &str = "popup_commands";
const CUSTOM_POPUPS_CONFIG_KEY: &str = "custom_popups";
const BOTTOM_POPUP_COMMAND_KEY: &str = "bottom_popup";
const TOP_POPUP_COMMAND_KEY: &str = "top_popup";
const MENU_POPUP_COMMAND_KEY: &str = "menu";
const RESERVED_POPUP_IDS: &[&str] = &["popup", "bottom_popup", "top_popup", "menu", "config"];
const ZELLIJ_RENDER_PLAN_CONFIG_KEYS: &[&str] = &[
    "left_sidebar_width_percent",
    "left_sidebar_command",
    "left_sidebar_args",
    "right_sidebar_width_percent",
    "right_sidebar_command",
    "right_sidebar_args",
    "popup_width_percent",
    "popup_height_percent",
    "screen_saver_enabled",
    "screen_saver_idle_seconds",
    "screen_saver_style",
    "zellij_widget_tray",
    "zellij_custom_text",
    "zellij_theme",
    "appearance_mode",
    "zellij_pane_frames",
    "zellij_rounded_corners",
    "disable_zellij_tips",
    "support_kitty_keyboard_protocol",
    "zellij_default_mode",
    "zellij_tab_label_mode",
    "zellij_claude_usage_display",
    "zellij_codex_usage_display",
    "zellij_opencode_go_usage_display",
    "zellij_claude_usage_periods",
    "zellij_codex_usage_periods",
    "zellij_opencode_go_usage_periods",
];

const ZJSTATUS_BAR_RENDER_COMMAND: &str = "render-yazelix-runtime";
const ZJSTATUS_BAR_RENDER_SCHEMA_VERSION: u64 = 3;

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
struct ExtractedSemanticBlocks {
    keybind_lines: Vec<String>,
    keybinds_block_present: bool,
    keybinds_clear_defaults: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
struct CustomPopup {
    id: String,
    command: Vec<String>,
    keybindings: Vec<String>,
    keep_alive: bool,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawCustomPopup {
    id: String,
    command: Vec<String>,
    #[serde(default)]
    keybindings: Vec<String>,
    #[serde(default)]
    keep_alive: Option<bool>,
}

struct KeybindingParsePolicy {
    namespace: &'static str,
    invalid_keys_code: &'static str,
    invalid_key_code: &'static str,
    list_remediation: &'static str,
    item_remediation: &'static str,
    invalid_item_remediation: &'static str,
}

#[derive(Debug, Clone, Serialize)]
struct ZellijBarRenderRequest {
    zjstatus_plugin_url: String,
    widget_tray: Vec<String>,
    editor_label: String,
    shell_label: String,
    terminal_label: String,
    custom_text: String,
    appearance_mode: String,
    tab_label_mode: String,
    nu_bin: String,
    yzx_control_bin: String,
    yazelix_zellij_bar_widget_bin: String,
    runtime_dir: String,
    claude_usage_display: String,
    codex_usage_display: String,
    opencode_go_usage_display: String,
    claude_usage_periods: Vec<String>,
    codex_usage_periods: Vec<String>,
    opencode_go_usage_periods: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct ZellijBarRenderEnvelope {
    schema_version: u64,
    plugin_block: String,
}

pub fn generate_zellij_materialization(
    request: &ZellijMaterializationRequest,
) -> Result<ZellijMaterializationData, CoreError> {
    let normalized = normalize_config(&NormalizeConfigRequest {
        config_path: request.config_path.clone(),
        default_config_path: request.default_config_path.clone(),
        contract_path: request.contract_path.clone(),
        include_missing: true,
    })?;
    let config = normalized.normalized_config;
    let screen_saver_enabled = match config.get("screen_saver_enabled") {
        Some(JsonValue::Bool(value)) => *value,
        Some(JsonValue::String(value)) => value == "true",
        _ => false,
    };
    if !runtime_component_enabled(&request.runtime_dir, "screen")? && screen_saver_enabled {
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
    let resolved_default_shell = zellij_default_shell_from_runtime(
        &request.runtime_dir,
        string_config(&config, "default_shell", "nu"),
    );
    let base_config_source = resolve_base_config_source()?;
    validate_base_config_keybinding_policy(&base_config_source)?;
    let plugin_artifacts = resolve_plugin_artifacts(&request.runtime_dir, &state_dir)?;
    let [pane_orchestrator_artifact, zjstatus_artifact, yzpp_artifact] = &plugin_artifacts;
    let zellij_keybindings = resolve_zellij_keybindings(&config)?;
    let zellij_native_keybindings = resolve_zellij_native_keybindings(&config)?;
    let popup_commands = resolve_popup_commands_config(&config)?;
    let custom_popups = resolve_custom_popups_config(&config)?;
    validate_custom_popup_keybindings(&zellij_keybindings, &custom_popups)?;
    let terminal_label = active_terminal_from_runtime_dir(&request.runtime_dir)?;
    let render_plan_request = build_render_plan_request(
        &config,
        &layout_dir,
        &resolved_default_shell,
        &terminal_label,
    )?;
    let render_plan = compute_zellij_render_plan(&render_plan_request)
        .map_err(zellij_render_plan_error_to_core)?;
    let generation_fingerprint = build_generation_fingerprint(
        &request.runtime_dir,
        &base_config_source,
        &plugin_artifacts,
        &zellij_keybindings,
        &zellij_native_keybindings,
        &popup_commands,
        &custom_popups,
        &render_plan,
    )?;

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
    let overrides_path = request
        .runtime_dir
        .join("configs")
        .join("zellij")
        .join("yazelix_overrides.kdl");
    let override_keybinds = read_yazelix_override_keybinds(
        &overrides_path,
        &request.runtime_dir,
        &zellij_keybindings,
        &zellij_native_keybindings,
        &custom_popups,
    )?;
    let zjstatus_plugin_url = format!("file:{}", zjstatus_runtime_path.to_string_lossy());
    let zjstatus_plugin_block =
        render_integrated_zjstatus_bar(&request.runtime_dir, &render_plan, &zjstatus_plugin_url)?;
    let config_pack_request = zellij_config_pack::ZellijConfigPackRenderRequest {
        base_config_content: base_config_source.content.clone(),
        override_keybinds,
        render_plan: render_plan.clone(),
        popup_commands,
        custom_popups: child_custom_popups(&custom_popups),
        layout_templates: None,
        static_fragments: None,
        zjstatus_plugin_block,
        pane_orchestrator_plugin_url: format!(
            "file:{}",
            pane_orchestrator_runtime_path.to_string_lossy()
        ),
        yzpp_plugin_url: format!("file:{}", yzpp_runtime_path.to_string_lossy()),
        home_dir: home_dir_from_env()?.to_string_lossy().to_string(),
        runtime_dir: request.runtime_dir.to_string_lossy().to_string(),
        generation_fingerprint,
    };
    let config_pack_output = zellij_config_pack::render_zellij_config_pack(&config_pack_request)
        .map_err(|message| {
            CoreError::classified(
                ErrorClass::Internal,
                "render_zellij_config_pack",
                format!("Zellij config-pack renderer failed: {message}"),
                "Report this as a Yazelix internal error.",
                json!({ "error": message }),
            )
        })?;
    let generated_layouts =
        write_zellij_config_pack_output(&merged_config_path, &layout_dir, &config_pack_output)?;
    record_generation_fingerprint(
        &request.zellij_config_dir,
        &config_pack_output.generation_fingerprint,
    )?;

    Ok(ZellijMaterializationData {
        merged_config_path: merged_config_path.to_string_lossy().to_string(),
        merged_config_dir: request.zellij_config_dir.to_string_lossy().to_string(),
        base_config_source: base_config_source.source,
        base_config_path: base_config_source
            .path
            .map(|path| path.to_string_lossy().to_string())
            .unwrap_or_default(),
        generation_fingerprint: config_pack_output.generation_fingerprint.clone(),
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
    terminal_label: &str,
) -> Result<ZellijRenderPlanRequest, CoreError> {
    let mut request = ZELLIJ_RENDER_PLAN_CONFIG_KEYS
        .iter()
        .filter_map(|key| {
            config
                .get(*key)
                .map(|value| ((*key).to_string(), value.clone()))
        })
        .collect::<JsonMap<String, JsonValue>>();
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
    request.insert("terminal_label".to_string(), json!(terminal_label));
    serde_json::from_value(JsonValue::Object(request)).map_err(|source| {
        CoreError::classified(
            ErrorClass::Config,
            "invalid_zellij_render_plan_request",
            format!("Could not build Zellij render plan from normalized config: {source}"),
            "Check settings.jsonc values under workspace, editor, and zellij.",
            json!({}),
        )
    })
}

fn child_custom_popups(popups: &[CustomPopup]) -> Vec<zellij_config_pack::CustomPopup> {
    popups
        .iter()
        .map(|popup| zellij_config_pack::CustomPopup {
            id: popup.id.clone(),
            command: popup.command.clone(),
            keybindings: popup.keybindings.clone(),
            keep_alive: popup.keep_alive,
        })
        .collect()
}

fn zellij_render_plan_error_to_core(error: ZellijRenderPlanError) -> CoreError {
    CoreError::classified(
        ErrorClass::Config,
        error.code(),
        error.message(),
        error.remediation(),
        error.details().clone(),
    )
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

fn default_popup_commands() -> BTreeMap<String, Vec<String>> {
    BTreeMap::from([
        (BOTTOM_POPUP_COMMAND_KEY.to_string(), vec!["lazygit".into()]),
        (
            TOP_POPUP_COMMAND_KEY.to_string(),
            vec!["yzx".into(), "config".into(), "ui".into()],
        ),
        (
            MENU_POPUP_COMMAND_KEY.to_string(),
            vec!["yzx".into(), "menu".into()],
        ),
    ])
}

fn resolve_popup_commands_config(
    config: &JsonMap<String, JsonValue>,
) -> Result<BTreeMap<String, Vec<String>>, CoreError> {
    let mut resolved = default_popup_commands();
    let Some(raw_commands) = config.get(POPUP_COMMANDS_CONFIG_KEY) else {
        return Ok(resolved);
    };
    let Some(command_map) = raw_commands.as_object() else {
        return Err(CoreError::classified(
            ErrorClass::Config,
            "invalid_popup_commands",
            "zellij.popup_commands must be an object whose values are argv lists.",
            "Use settings such as `\"bottom_popup\": [\"lazygit\"]`.",
            json!({ "field": "zellij.popup_commands" }),
        ));
    };

    for (name, raw_command) in command_map {
        if !resolved.contains_key(name) {
            return Err(CoreError::classified(
                ErrorClass::Config,
                "unknown_popup_command",
                format!("Unsupported zellij.popup_commands entry: {name}."),
                if name == "zenith" {
                    "Move zenith to zellij.custom_popups: { \"id\": \"zenith\", \"command\": [\"zenith\"], \"keybindings\": [\"Alt Shift I\"], \"keep_alive\": true }."
                } else {
                    "Use one of: bottom_popup, top_popup, menu."
                },
                json!({ "field": format!("zellij.popup_commands.{name}") }),
            ));
        }
        let mut command = parse_config_string_list(
            Some(raw_command),
            &format!("zellij.popup_commands.{name}"),
            "Use a non-empty command argv list such as [\"lazygit\"], or remove the entry to use the Yazelix default.",
            false,
        )?;
        if command.first().map(String::as_str) == Some("editor") {
            command[0] = string_config(config, "editor_command", "hx").to_string();
        }
        resolved.insert(name.clone(), command);
    }

    Ok(resolved)
}

fn default_custom_popups() -> Vec<CustomPopup> {
    vec![CustomPopup {
        id: "zenith".to_string(),
        command: vec!["zenith".into()],
        keybindings: vec!["Alt Shift I".into()],
        keep_alive: true,
    }]
}

fn resolve_custom_popups_config(
    config: &JsonMap<String, JsonValue>,
) -> Result<Vec<CustomPopup>, CoreError> {
    let Some(raw_popups) = config.get(CUSTOM_POPUPS_CONFIG_KEY) else {
        return Ok(default_custom_popups());
    };
    let items = serde_json::from_value::<Vec<RawCustomPopup>>(raw_popups.clone()).map_err(
        |source| CoreError::classified(
            ErrorClass::Config,
            "invalid_custom_popups",
            "zellij.custom_popups must be a list of popup definitions.",
            "Use objects such as { \"id\": \"gitui\", \"command\": [\"gitui\"], \"keybindings\": [\"Alt Shift G\"] }.",
            json!({ "field": "zellij.custom_popups", "error": source.to_string() }),
        ),
    )?;

    let mut seen_ids = BTreeSet::new();
    let mut popups = Vec::with_capacity(items.len());
    for (index, item) in items.into_iter().enumerate() {
        let path = format!("zellij.custom_popups[{index}]");
        let id = item.id.trim().to_string();
        if id.is_empty() {
            return Err(CoreError::classified(
                ErrorClass::Config,
                "invalid_custom_popup_id",
                format!("{path}.id must be a non-empty string."),
                "Use a stable id such as \"gitui\".",
                json!({ "field": format!("{path}.id") }),
            ));
        }
        validate_custom_popup_id(&id, &path)?;
        if RESERVED_POPUP_IDS.contains(&id.as_str()) {
            return Err(CoreError::classified(
                ErrorClass::Config,
                "reserved_custom_popup_id",
                format!("zellij.custom_popups id {id:?} is reserved by Yazelix."),
                "Use zellij.popup_commands for bottom_popup, top_popup, and menu; use another id for custom popups.",
                json!({ "field": format!("{path}.id"), "id": id }),
            ));
        }
        if !seen_ids.insert(id.clone()) {
            return Err(CoreError::classified(
                ErrorClass::Config,
                "duplicate_custom_popup_id",
                format!("Duplicate zellij.custom_popups id: {id}."),
                "Give each custom popup a unique id.",
                json!({ "field": format!("{path}.id"), "id": id }),
            ));
        }

        let command = normalize_config_strings(
            item.command,
            &format!("{path}.command"),
            "Use a non-empty command argv list such as [\"gitui\"].",
            false,
        )?;
        let mut command = command;
        if command.first().map(String::as_str) == Some("editor") {
            command[0] = string_config(config, "editor_command", "hx").to_string();
        }
        let keybindings = normalize_config_strings(
            item.keybindings,
            &format!("{path}.keybindings"),
            "Use Zellij key strings such as [\"Alt Shift G\"], or [] for an unbound custom popup.",
            true,
        )?;
        let default_keep_alive = id == "zenith" && command.len() == 1 && command[0] == "zenith";
        let keep_alive = item.keep_alive.unwrap_or(default_keep_alive);

        popups.push(CustomPopup {
            id,
            command,
            keybindings,
            keep_alive,
        });
    }

    Ok(popups)
}

pub(crate) fn validate_zellij_custom_popup_config(
    config: &JsonMap<String, JsonValue>,
) -> Result<(), CoreError> {
    let zellij_keybindings = resolve_zellij_keybindings(config)?;
    let custom_popups = resolve_custom_popups_config(config)?;
    validate_custom_popup_keybindings(&zellij_keybindings, &custom_popups)
}

fn validate_custom_popup_id(id: &str, path: &str) -> Result<(), CoreError> {
    let mut chars = id.chars();
    let first = chars.next().expect("custom popup ids are non-empty");
    if !(first.is_ascii_alphabetic() || first == '_')
        || !chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
    {
        return Err(CoreError::classified(
            ErrorClass::Config,
            "invalid_custom_popup_id",
            format!("Invalid zellij.custom_popups id: {id}."),
            "Use only ASCII letters, numbers, and underscores, and start with a letter or underscore.",
            json!({ "field": format!("{path}.id"), "id": id }),
        ));
    }
    Ok(())
}

fn parse_config_string_list(
    value: Option<&JsonValue>,
    field_path: &str,
    remediation: &str,
    empty_allowed: bool,
) -> Result<Vec<String>, CoreError> {
    let Some(value) = value else {
        return if empty_allowed {
            Ok(Vec::new())
        } else {
            Err(CoreError::classified(
                ErrorClass::Config,
                "missing_config_string_list",
                format!("{field_path} is required."),
                remediation,
                json!({ "field": field_path }),
            ))
        };
    };
    let Some(items) = value.as_array() else {
        return Err(CoreError::classified(
            ErrorClass::Config,
            "invalid_config_string_list",
            format!("{field_path} must be a list of strings."),
            remediation,
            json!({ "field": field_path, "actual": value }),
        ));
    };
    let mut values = Vec::with_capacity(items.len());
    for item in items {
        let Some(raw_item) = item.as_str() else {
            return Err(CoreError::classified(
                ErrorClass::Config,
                "invalid_config_string_list_item",
                format!("{field_path} contains a non-string item."),
                remediation,
                json!({ "field": field_path, "actual": item }),
            ));
        };
        values.push(raw_item.to_string());
    }
    normalize_config_strings(values, field_path, remediation, empty_allowed)
}

fn normalize_config_strings(
    values: Vec<String>,
    field_path: &str,
    remediation: &str,
    empty_allowed: bool,
) -> Result<Vec<String>, CoreError> {
    let mut normalized = Vec::with_capacity(values.len());
    for raw_item in values {
        let item = raw_item.trim();
        if item.is_empty() || item.contains('\n') || item.contains('\r') {
            return Err(CoreError::classified(
                ErrorClass::Config,
                "invalid_config_string_list_item",
                format!("{field_path} contains an invalid string item."),
                remediation,
                json!({ "field": field_path, "actual": raw_item }),
            ));
        }
        normalized.push(item.to_string());
    }
    if !empty_allowed && normalized.is_empty() {
        return Err(CoreError::classified(
            ErrorClass::Config,
            "empty_config_string_list",
            format!("{field_path} cannot be empty."),
            remediation,
            json!({ "field": field_path }),
        ));
    }
    Ok(normalized)
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

fn extract_semantic_config_blocks(config_content: &str) -> ExtractedSemanticBlocks {
    let mut keybind_lines = Vec::new();
    let mut keybinds_block_present = false;
    let mut keybinds_clear_defaults = false;
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
                if block == "keybinds" {
                    keybinds_block_present = true;
                    if keybinds_declares_clear_defaults(trimmed) {
                        keybinds_clear_defaults = true;
                    }
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
                    if block == "keybinds" && !inline_body.is_empty() {
                        keybind_lines.push(inline_body.to_string());
                    }
                    active_block.clear();
                    brace_depth = 0;
                }
            }
        } else {
            brace_depth += open_braces - close_braces;
            if brace_depth > 0 && active_block == "keybinds" {
                keybind_lines.push(line.to_string());
            } else {
                active_block.clear();
            }
        }
    }

    ExtractedSemanticBlocks {
        keybind_lines,
        keybinds_block_present,
        keybinds_clear_defaults,
    }
}

pub(crate) fn zellij_config_contains_keybinds_block(config_content: &str) -> bool {
    extract_semantic_config_blocks(config_content).keybinds_block_present
}

fn validate_base_config_keybinding_policy(
    base_config_source: &ZellijBaseConfigSource,
) -> Result<(), CoreError> {
    if base_config_source.source != "managed" {
        return Ok(());
    }

    let extracted = extract_semantic_config_blocks(&base_config_source.content);
    if !extracted.keybinds_block_present {
        return Ok(());
    }

    let block = if extracted.keybinds_clear_defaults {
        "keybinds clear-defaults=true"
    } else {
        "keybinds"
    };
    let path = base_config_source
        .path
        .as_ref()
        .map(|path| path.to_string_lossy().to_string())
        .unwrap_or_else(|| "~/.config/yazelix/zellij.kdl".to_string());

    Err(CoreError::classified(
        ErrorClass::Config,
        "managed_zellij_keybinds_unsupported",
        format!("Managed Zellij config cannot contain a `{block}` block: {path}"),
        "Remove that keybinds block from ~/.config/yazelix/zellij.kdl. Use zellij.keybindings and zellij.native_keybindings in settings.jsonc for Yazelix key remaps; use plain zellij outside Yazelix for full native keybinding ownership.",
        json!({
            "path": path,
            "block": block,
        }),
    ))
}

fn keybinds_declares_clear_defaults(line: &str) -> bool {
    let header = line.split('{').next().unwrap_or(line);
    header.contains("clear-defaults=true") || header.contains("clear-defaults = true")
}

fn default_zellij_keybindings() -> BTreeMap<String, Vec<String>> {
    default_keybindings(ZELLIJ_ACTIONS.iter().map(|spec| &spec.action))
}

fn default_zellij_native_keybindings() -> BTreeMap<String, Vec<String>> {
    default_keybindings(ZELLIJ_NATIVE_KEYBINDINGS.iter().map(|spec| &spec.action))
}

fn default_keybindings<'a>(
    actions: impl IntoIterator<Item = &'a YazelixActionMetadata>,
) -> BTreeMap<String, Vec<String>> {
    actions
        .into_iter()
        .map(|action| {
            let keys = action
                .default_keys
                .iter()
                .map(|key| (*key).to_string())
                .collect();
            (action.local_id.to_string(), keys)
        })
        .collect()
}

fn parse_keybinding_keys(
    action: &str,
    raw_keys: &JsonValue,
    policy: &KeybindingParsePolicy,
) -> Result<Vec<String>, CoreError> {
    let Some(values) = raw_keys.as_array() else {
        return Err(CoreError::classified(
            ErrorClass::Config,
            policy.invalid_keys_code,
            format!(
                "{}.{action} must be a list of Zellij key strings.",
                policy.namespace
            ),
            policy.list_remediation,
            json!({ "action": action, "actual": raw_keys }),
        ));
    };
    let mut keys = Vec::with_capacity(values.len());
    for value in values {
        let Some(raw_key) = value.as_str() else {
            return Err(CoreError::classified(
                ErrorClass::Config,
                policy.invalid_key_code,
                format!("{}.{action} contains a non-string key.", policy.namespace),
                policy.item_remediation,
                json!({ "action": action, "actual": value }),
            ));
        };
        let key = raw_key.trim();
        if key.is_empty() || key.contains('\n') || key.contains('\r') {
            return Err(CoreError::classified(
                ErrorClass::Config,
                policy.invalid_key_code,
                format!(
                    "{}.{action} contains an invalid key string.",
                    policy.namespace
                ),
                policy.invalid_item_remediation,
                json!({ "action": action, "actual": raw_key }),
            ));
        }
        keys.push(key.to_string());
    }
    Ok(keys)
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
            "Use settings such as `\"bottom_popup\": [\"Alt Shift J\"]`, or remove zellij.keybindings to use Yazelix defaults.",
            json!({ "actual": value }),
        ));
    };

    for (action, raw_keys) in object {
        if zellij_action_by_local_id(action).is_none() {
            return Err(CoreError::classified(
                ErrorClass::Config,
                "unsupported_zellij_keybinding_action",
                format!("Unsupported Zellij keybinding action: {action}."),
                "Use one of the supported Yazelix Zellij action ids, or remove the unsupported keybinding entry.",
                json!({
                    "action": action,
                    "supported_actions": ZELLIJ_ACTIONS
                        .iter()
                        .map(|spec| spec.action.local_id)
                        .collect::<Vec<_>>(),
                }),
            ));
        }
        resolved.insert(
            action.clone(),
            parse_keybinding_keys(action, raw_keys, &ZELLIJ_KEYBINDING_PARSE_POLICY)?,
        );
    }

    validate_zellij_keybindings(&resolved)?;
    Ok(resolved)
}

fn resolve_zellij_native_keybindings(
    config: &JsonMap<String, JsonValue>,
) -> Result<BTreeMap<String, Vec<String>>, CoreError> {
    let mut resolved = default_zellij_native_keybindings();
    let Some(value) = config.get(ZELLIJ_NATIVE_KEYBINDINGS_CONFIG_KEY) else {
        return Ok(resolved);
    };
    let Some(object) = value.as_object() else {
        return Err(CoreError::classified(
            ErrorClass::Config,
            "invalid_zellij_native_keybindings",
            "zellij.native_keybindings must be an object whose values are lists of Zellij key strings.",
            "Use settings such as `\"scroll_mode\": [\"Ctrl Alt s\"]`, or remove zellij.native_keybindings to use Yazelix defaults.",
            json!({ "actual": value }),
        ));
    };

    for (action, raw_keys) in object {
        if zellij_native_keybinding_by_local_id(action).is_none() {
            return Err(CoreError::classified(
                ErrorClass::Config,
                "unsupported_zellij_native_keybinding_action",
                format!("Unsupported Zellij native keybinding action: {action}."),
                "Use one of the supported Yazelix native Zellij policy ids, or remove the unsupported entry.",
                json!({
                    "action": action,
                    "supported_actions": ZELLIJ_NATIVE_KEYBINDINGS
                        .iter()
                        .map(|spec| spec.action.local_id)
                        .collect::<Vec<_>>(),
                }),
            ));
        }
        resolved.insert(
            action.clone(),
            parse_keybinding_keys(action, raw_keys, &ZELLIJ_NATIVE_KEYBINDING_PARSE_POLICY)?,
        );
    }

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

fn read_yazelix_override_keybinds(
    overrides_path: &Path,
    runtime_dir: &Path,
    zellij_keybindings: &BTreeMap<String, Vec<String>>,
    zellij_native_keybindings: &BTreeMap<String, Vec<String>>,
    custom_popups: &[CustomPopup],
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
            "__YAZELIX_RUNTIME_DIR__",
            runtime_dir.to_string_lossy().as_ref(),
        );
    let assigned_keys = assigned_generated_zellij_binding_keys(
        zellij_keybindings,
        zellij_native_keybindings,
        custom_popups,
    );
    let mut keybind_lines = extract_semantic_config_blocks(&content)
        .keybind_lines
        .into_iter()
        .filter(|line| !unbind_line_conflicts_with_generated_key(line, &assigned_keys))
        .collect::<Vec<_>>();
    keybind_lines.extend(
        build_native_zellij_keybind_lines(zellij_native_keybindings)
            .into_iter()
            .filter(|line| !unbind_line_conflicts_with_generated_key(line, &assigned_keys)),
    );
    keybind_lines.extend(build_zellij_integration_keybind_lines(
        zellij_keybindings,
        custom_popups,
    ));
    Ok(keybind_lines)
}

fn assigned_generated_zellij_binding_keys(
    zellij_keybindings: &BTreeMap<String, Vec<String>>,
    zellij_native_keybindings: &BTreeMap<String, Vec<String>>,
    custom_popups: &[CustomPopup],
) -> BTreeSet<String> {
    let mut assigned = zellij_keybindings
        .values()
        .flatten()
        .cloned()
        .collect::<BTreeSet<_>>();
    assigned.extend(
        custom_popups
            .iter()
            .flat_map(|popup| popup.keybindings.iter().cloned()),
    );
    for spec in ZELLIJ_NATIVE_KEYBINDINGS {
        if !spec
            .blocks
            .iter()
            .any(|block| !block.action_lines.is_empty())
        {
            continue;
        }
        if let Some(keys) = zellij_native_keybindings.get(spec.action.local_id) {
            assigned.extend(keys.iter().cloned());
        }
    }
    assigned
}

fn unbind_line_conflicts_with_generated_key(line: &str, assigned_keys: &BTreeSet<String>) -> bool {
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

fn build_zellij_integration_keybind_lines(
    keybindings: &BTreeMap<String, Vec<String>>,
    custom_popups: &[CustomPopup],
) -> Vec<String> {
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
    for popup in custom_popups {
        if popup.keybindings.is_empty() {
            continue;
        }
        let mode_lines = by_mode.entry("shared").or_default();
        push_zellij_message_bind(
            mode_lines,
            &popup.keybindings,
            YZPP_PLUGIN_ALIAS,
            "toggle",
            Some(&popup.id),
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

fn validate_custom_popup_keybindings(
    zellij_keybindings: &BTreeMap<String, Vec<String>>,
    custom_popups: &[CustomPopup],
) -> Result<(), CoreError> {
    let mut seen = BTreeMap::<String, String>::new();
    for spec in ZELLIJ_ACTIONS {
        if let Some(keys) = zellij_keybindings.get(spec.action.local_id) {
            for key in keys {
                seen.insert(key.clone(), spec.action.local_id.to_string());
            }
        }
    }

    for popup in custom_popups {
        for key in &popup.keybindings {
            if let Some(existing_action) = seen.insert(key.clone(), popup.id.clone()) {
                return Err(CoreError::classified(
                    ErrorClass::Config,
                    "duplicate_custom_popup_keybinding",
                    format!(
                        "Zellij keybinding {key:?} is assigned to both {existing_action} and custom popup {}.",
                        popup.id
                    ),
                    "Give each Yazelix Zellij action and custom popup a distinct key.",
                    json!({
                        "key": key,
                        "first_action": existing_action,
                        "second_action": format!("custom_popups.{}", popup.id),
                    }),
                ));
            }
        }
    }
    Ok(())
}

fn build_native_zellij_keybind_lines(keybindings: &BTreeMap<String, Vec<String>>) -> Vec<String> {
    let mut blocks = Vec::<(&str, Vec<String>)>::new();
    for spec in ZELLIJ_NATIVE_KEYBINDINGS {
        let Some(keys) = keybindings.get(spec.action.local_id) else {
            continue;
        };
        if keys.is_empty() {
            continue;
        }
        for block in spec.blocks {
            push_native_zellij_block_lines(&mut blocks, keys, block);
        }
    }

    let mut lines = Vec::new();
    for (mode, block_lines) in blocks {
        if !lines.is_empty() {
            lines.push(String::new());
        }
        lines.push(format!("    {mode} {{"));
        lines.extend(block_lines);
        lines.push("    }".to_string());
    }
    lines
}

fn push_native_zellij_block_lines(
    blocks: &mut Vec<(&'static str, Vec<String>)>,
    keys: &[String],
    block: &ZellijNativeKeybindingBlock,
) {
    let key_list = keys.iter().map(json_quote).collect::<Vec<_>>().join(" ");
    let block_lines = blocks
        .iter_mut()
        .find(|(mode, _)| *mode == block.mode)
        .map(|(_, lines)| lines);
    let lines = if let Some(lines) = block_lines {
        lines
    } else {
        blocks.push((block.mode, Vec::new()));
        &mut blocks.last_mut().expect("just pushed").1
    };
    if block.action_lines.is_empty() {
        lines.push(format!("        unbind {key_list}"));
    } else if block.action_lines.iter().any(|line| line.contains('\n')) {
        lines.push(format!("        bind {key_list} {{"));
        for action_line in block.action_lines {
            for line in action_line.lines() {
                lines.push(format!("            {line}"));
            }
        }
        lines.push("        }".to_string());
    } else {
        lines.push(format!(
            "        bind {key_list} {{ {}; }}",
            block.action_lines.join("; ")
        ));
    }
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

fn write_zellij_config_pack_output(
    merged_config_path: &Path,
    layout_dir: &Path,
    output: &zellij_config_pack::ZellijConfigPackRenderOutput,
) -> Result<Vec<PathBuf>, CoreError> {
    write_text_atomic(merged_config_path, &output.merged_config)?;
    fs::create_dir_all(layout_dir).map_err(|source| {
        CoreError::io(
            "create_zellij_layout_dir",
            "Could not create generated Zellij layout directory",
            "Check permissions for the Yazelix state directory and retry.",
            layout_dir.to_string_lossy(),
            source,
        )
    })?;
    let expected_targets = output
        .layout_files
        .iter()
        .map(|file| layout_dir.join(&file.relative_path))
        .collect::<Vec<_>>();
    remove_stale_layouts(layout_dir, &expected_targets)?;
    for (file, target) in output.layout_files.iter().zip(expected_targets.iter()) {
        write_text_atomic(target, &file.content)?;
    }

    Ok(expected_targets)
}

fn render_integrated_zjstatus_bar(
    runtime_dir: &Path,
    render_plan: &ZellijRenderPlanData,
    zjstatus_plugin_url: &str,
) -> Result<String, CoreError> {
    let renderer = resolve_zjstatus_yazelix_zellij_bar_widget_bin(runtime_dir);
    let request = ZellijBarRenderRequest {
        zjstatus_plugin_url: zjstatus_plugin_url.to_string(),
        widget_tray: render_plan.widget_tray.clone(),
        editor_label: render_plan.editor_label.clone(),
        shell_label: render_plan.shell_label.clone(),
        terminal_label: render_plan.terminal_label.clone(),
        custom_text: render_plan.custom_text.clone(),
        appearance_mode: render_plan.appearance_mode.clone(),
        tab_label_mode: render_plan.tab_label_mode.clone(),
        nu_bin: resolve_zjstatus_nu_bin(runtime_dir),
        yzx_control_bin: resolve_zjstatus_yzx_control_bin(runtime_dir),
        yazelix_zellij_bar_widget_bin: renderer.clone(),
        runtime_dir: runtime_dir.to_string_lossy().to_string(),
        claude_usage_display: render_plan.claude_usage_display.clone(),
        codex_usage_display: render_plan.codex_usage_display.clone(),
        opencode_go_usage_display: render_plan.opencode_go_usage_display.clone(),
        claude_usage_periods: render_plan.claude_usage_periods.clone(),
        codex_usage_periods: render_plan.codex_usage_periods.clone(),
        opencode_go_usage_periods: render_plan.opencode_go_usage_periods.clone(),
    };
    let request_json = serde_json::to_string(&request).map_err(|source| {
        CoreError::classified(
            ErrorClass::Internal,
            "serialize_zellij_bar_render_request",
            "Could not serialize Yazelix Zellij bar render request.",
            "Report this as a Yazelix internal error.",
            json!({ "error": source.to_string() }),
        )
    })?;
    let output = Command::new(&renderer)
        .args([ZJSTATUS_BAR_RENDER_COMMAND, "--json", request_json.as_str()])
        .output()
        .map_err(|source| {
            CoreError::io(
                "run_zellij_bar_renderer",
                "Could not run the Yazelix Zellij bar renderer",
                "Reinstall Yazelix so the runtime includes yazelix_zellij_bar_widget.",
                renderer.clone(),
                source,
            )
        })?;
    if !output.status.success() {
        return Err(CoreError::classified(
            ErrorClass::Runtime,
            "render_zellij_bar_plugin_block",
            format!(
                "Yazelix Zellij bar renderer failed with status {}.",
                output.status
            ),
            "Check zellij bar settings or reinstall Yazelix so the bar widget matches this Yazelix build.",
            json!({
                "renderer": renderer,
                "stdout": String::from_utf8_lossy(&output.stdout),
                "stderr": String::from_utf8_lossy(&output.stderr),
            }),
        ));
    }
    let envelope: ZellijBarRenderEnvelope =
        serde_json::from_slice(&output.stdout).map_err(|source| {
            CoreError::classified(
                ErrorClass::Runtime,
                "parse_zellij_bar_plugin_block",
                "Yazelix Zellij bar renderer returned invalid JSON.",
                "Reinstall Yazelix so the bar widget matches this Yazelix build.",
                json!({
                    "renderer": renderer,
                    "error": source.to_string(),
                    "stdout": String::from_utf8_lossy(&output.stdout),
                }),
            )
        })?;
    if envelope.schema_version != ZJSTATUS_BAR_RENDER_SCHEMA_VERSION {
        return Err(CoreError::classified(
            ErrorClass::Runtime,
            "unsupported_zellij_bar_renderer_schema",
            format!(
                "Yazelix Zellij bar renderer returned schema version {}, expected {}.",
                envelope.schema_version, ZJSTATUS_BAR_RENDER_SCHEMA_VERSION
            ),
            "Reinstall Yazelix so the bar widget matches this Yazelix build.",
            json!({
                "renderer": renderer,
                "schema_version": envelope.schema_version,
                "expected_schema_version": ZJSTATUS_BAR_RENDER_SCHEMA_VERSION,
            }),
        ));
    }
    if envelope.plugin_block.trim().is_empty() {
        return Err(CoreError::classified(
            ErrorClass::Runtime,
            "empty_zellij_bar_plugin_block",
            "Yazelix Zellij bar renderer returned an empty plugin block.",
            "Reinstall Yazelix so the bar widget matches this Yazelix build.",
            json!({ "renderer": renderer }),
        ));
    }
    Ok(envelope.plugin_block)
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

fn resolve_zjstatus_yazelix_zellij_bar_widget_bin(runtime_dir: &Path) -> String {
    let runtime_widget = runtime_dir
        .join("libexec")
        .join("yazelix_zellij_bar_widget");
    if runtime_widget.is_file() {
        runtime_widget.to_string_lossy().to_string()
    } else if let Some(path) = env_path_if_file("YAZELIX_BAR_WIDGET_BIN") {
        path.to_string_lossy().to_string()
    } else {
        "yazelix_zellij_bar_widget".to_string()
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

fn build_generation_fingerprint(
    runtime_dir: &Path,
    base_config_source: &ZellijBaseConfigSource,
    plugin_artifacts: &[PluginArtifact; 3],
    zellij_keybindings: &BTreeMap<String, Vec<String>>,
    zellij_native_keybindings: &BTreeMap<String, Vec<String>>,
    popup_commands: &BTreeMap<String, Vec<String>>,
    custom_popups: &[CustomPopup],
    render_plan: &ZellijRenderPlanData,
) -> Result<String, CoreError> {
    let overrides_path = runtime_dir
        .join("configs")
        .join("zellij")
        .join("yazelix_overrides.kdl");
    let fingerprint_payload = json!({
        "schema_version": GENERATION_FINGERPRINT_SCHEMA_VERSION,
        "zellij_config_pack_renderer_schema_version": zellij_config_pack::RENDERER_SCHEMA_VERSION,
        "runtime_dir": runtime_dir.to_string_lossy(),
        "render_plan": render_plan,
        "zellij_keybindings": zellij_keybindings,
        "zellij_native_keybindings": zellij_native_keybindings,
        "popup_commands": popup_commands,
        "custom_popups": custom_popups,
        "base_config": {
            "source": base_config_source.source,
            "path": base_config_source.path.as_ref().map(|path| path.to_string_lossy().to_string()).unwrap_or_default(),
            "hash": hash_text(&base_config_source.content),
        },
        "overrides_hash": hash_text(&read_text_if_exists(&overrides_path)?),
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

fn record_generation_fingerprint(
    merged_config_dir: &Path,
    fingerprint: &str,
) -> Result<(), CoreError> {
    let metadata_path = merged_config_dir.join(GENERATION_METADATA_NAME);
    let content = serde_json::to_string(&json!({
        "fingerprint": fingerprint,
        "generated_at": epoch_millis_timestamp(),
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

pub(crate) fn generated_zellij_config_has_yazelix_markers(path: &Path) -> Result<bool, CoreError> {
    let content = fs::read_to_string(path).map_err(|source| {
        CoreError::io(
            "read_generated_zellij_config",
            "Could not read generated Zellij config",
            "Check permissions for the Yazelix state directory and retry.",
            path.to_string_lossy(),
            source,
        )
    })?;
    Ok(GENERATED_CONFIG_MARKERS
        .iter()
        .all(|marker| content.contains(marker)))
}

pub(crate) fn generated_zellij_layout_has_yazelix_markers(
    path: &Path,
    expected_fingerprint: Option<&str>,
) -> Result<bool, CoreError> {
    let content = fs::read_to_string(path).map_err(|source| {
        CoreError::io(
            "read_generated_zellij_layout",
            "Could not read generated Zellij layout",
            "Check permissions for the Yazelix state directory and retry.",
            path.to_string_lossy(),
            source,
        )
    })?;
    if !content.contains(GENERATED_LAYOUT_MARKER) {
        return Ok(false);
    }
    if let Some(fingerprint) = expected_fingerprint {
        let expected_line = format!("{GENERATED_LAYOUT_FINGERPRINT_PREFIX} {fingerprint}");
        return Ok(content.contains(&expected_line));
    }
    Ok(true)
}

fn json_quote(value: impl AsRef<str>) -> String {
    serde_json::to_string(value.as_ref()).unwrap_or_else(|_| "\"\"".to_string())
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
            left_sidebar_width_percent: 20,
            left_sidebar_command: "yzx".into(),
            left_sidebar_args: vec!["sidebar".into(), "yazi".into()],
            right_sidebar_width_percent: 40,
            right_sidebar_command: "yzx".into(),
            right_sidebar_args: vec!["agent".into()],
            popup_width_percent: 90,
            popup_height_percent: 90,
            screen_saver_enabled: false,
            screen_saver_idle_seconds: 300,
            screen_saver_style: "random".into(),
            zellij_widget_tray: Some(widget_tray.into_iter().map(str::to_string).collect()),
            zellij_custom_text: None,
            zellij_theme: "default".into(),
            appearance_mode: "dark".into(),
            zellij_pane_frames: "true".into(),
            zellij_rounded_corners: "true".into(),
            disable_zellij_tips: "true".into(),
            support_kitty_keyboard_protocol: "false".into(),
            zellij_default_mode: "normal".into(),
            zellij_tab_label_mode: "full".into(),
            zellij_claude_usage_display: "both".into(),
            zellij_codex_usage_display: "quota".into(),
            zellij_opencode_go_usage_display: "both".into(),
            zellij_claude_usage_periods: vec!["5h".into(), "week".into()],
            zellij_codex_usage_periods: vec!["5h".into(), "week".into()],
            zellij_opencode_go_usage_periods: vec!["5h".into(), "week".into(), "month".into()],
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

    fn sample_zellij_native_keybindings() -> BTreeMap<String, Vec<String>> {
        default_zellij_native_keybindings()
    }

    // Defends: managed override parsing keeps keybind block facts while ignoring child-owned semantic blocks.
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
                .keybind_lines
                .iter()
                .any(|line| line.contains("Ctrl y"))
        );
        assert!(extracted.keybinds_block_present);
        assert!(!extracted.keybinds_clear_defaults);
    }

    // Defends: clear-defaults remains detectable so managed config can reject the strongest keybinding bypass explicitly.
    #[test]
    fn extracts_keybinds_clear_defaults_ownership() {
        let extracted = extract_semantic_config_blocks(
            r#"keybinds clear-defaults=true {
    locked { bind "Ctrl `" { SwitchToMode "Normal"; } }
}
"#,
        );

        assert!(extracted.keybinds_clear_defaults);
        assert!(extracted.keybinds_block_present);
        assert!(
            extracted
                .keybind_lines
                .iter()
                .any(|line| line.contains("Ctrl `"))
        );
    }

    // Defends: managed zellij.kdl remains a native settings sidecar, not a second keybinding owner.
    #[test]
    fn managed_zellij_keybind_blocks_are_rejected() {
        let cases: &[(&str, &str, &[&str])] = &[
            (
                r#"keybinds clear-defaults=true {
    locked { bind "Ctrl `" { SwitchToMode "Normal"; } }
}
"#,
                "keybinds clear-defaults=true",
                &["zellij.keybindings", "settings.jsonc"],
            ),
            (
                r#"keybinds {
    normal { bind "Alt t" { ToggleFloatingPanes; } }
}
"#,
                "`keybinds` block",
                &[],
            ),
        ];

        for &(content, message_fragment, remediation_fragments) in cases {
            let err = validate_base_config_keybinding_policy(&ZellijBaseConfigSource {
                source: "managed".to_string(),
                path: Some(PathBuf::from("/home/user/.config/yazelix/zellij.kdl")),
                content: content.to_string(),
            })
            .unwrap_err();

            match err {
                CoreError::Classified {
                    code,
                    message,
                    remediation,
                    ..
                } => {
                    assert_eq!(code, "managed_zellij_keybinds_unsupported");
                    assert!(message.contains(message_fragment));
                    for expected in remediation_fragments {
                        assert!(remediation.contains(expected));
                    }
                }
                other => panic!("unexpected error: {other:?}"),
            }
        }
    }

    // Regression: native Zellij mode leaders remain user/Zellij-owned instead of being reasserted by Yazelix overrides after a native fallback remap.
    #[test]
    fn native_fallback_zellij_remap_can_remove_ctrl_n_resize_leader() {
        let temp = tempfile::tempdir().unwrap();
        let runtime_dir = temp.path();
        let overrides_dir = runtime_dir.join("configs").join("zellij");
        std::fs::create_dir_all(&overrides_dir).unwrap();
        std::fs::write(
            overrides_dir.join("yazelix_overrides.kdl"),
            include_str!("../../../configs/zellij/yazelix_overrides.kdl"),
        )
        .unwrap();
        let override_keybinds = read_yazelix_override_keybinds(
            &overrides_dir.join("yazelix_overrides.kdl"),
            runtime_dir,
            &sample_zellij_keybindings(),
            &sample_zellij_native_keybindings(),
            &default_custom_popups(),
        )
        .unwrap();
        let rendered = override_keybinds.join("\n");

        assert!(!rendered.contains(r#"bind "Ctrl n" { SwitchToMode "Resize"; }"#));
        assert!(!rendered.contains(r#"bind "Ctrl n" { SwitchToMode "Normal"; }"#));
        assert!(rendered.contains(r#"bind "Ctrl Alt s" { SwitchToMode "Scroll"; }"#));
        assert!(rendered.contains(r#"bind "Ctrl Alt o" { SwitchToMode "Session"; }"#));
        assert!(rendered.contains(r#"bind "Ctrl Alt g" { SwitchToMode "Locked"; }"#));
    }

    // Defends: remaining Yazelix native Zellij key policy is generated from config data rather than hardcoded KDL overrides.
    #[test]
    fn native_zellij_keybindings_generate_default_policy() {
        let rendered =
            build_native_zellij_keybind_lines(&sample_zellij_native_keybindings()).join("\n");

        assert!(rendered.contains(r#"unbind "Alt i""#));
        assert!(rendered.contains(r#"bind "Ctrl Alt h" { MoveTab "Left"; }"#));
        assert!(!rendered.contains(r#"Run "yzx" "agent""#));
        assert!(rendered.contains(r#"bind "Ctrl Alt s" { SwitchToMode "Scroll"; }"#));
        assert!(rendered.contains(r#"bind "Ctrl Alt s" { SwitchToMode "Normal"; }"#));
    }

    // Defends: the Codex agent key is orchestrator-managed so it can avoid duplicate panes and preserve layout state.
    #[test]
    fn semantic_zellij_keybindings_generate_agent_toggle() {
        let rendered =
            build_zellij_integration_keybind_lines(&sample_zellij_keybindings(), &[]).join("\n");

        assert!(rendered.contains(r#"bind "Alt Shift L" {"#));
        assert!(rendered.contains(r#"MessagePlugin "yazelix_pane_orchestrator" {"#));
        assert!(rendered.contains(r#"name "toggle_agent_sidebar""#));
        assert!(rendered.contains(r#"bind "Ctrl Shift Y" {"#));
        assert!(rendered.contains(r#"name "toggle_editor_right_sidebar_focus""#));
    }

    // Defends: users can remap or disable one curated native Zellij policy entry without copying the full keybind block.
    #[test]
    fn native_zellij_keybindings_honor_remaps_and_disabled_entries() {
        let mut keybindings = sample_zellij_native_keybindings();
        keybindings.insert("scroll_mode".to_string(), vec!["Ctrl Alt x".to_string()]);
        keybindings.insert("scroll_mode_unbind".to_string(), Vec::new());
        keybindings.insert("go_to_tab_1".to_string(), Vec::new());
        let rendered = build_native_zellij_keybind_lines(&keybindings).join("\n");

        assert!(rendered.contains(r#"bind "Ctrl Alt x" { SwitchToMode "Scroll"; }"#));
        assert!(rendered.contains(r#"bind "Ctrl Alt x" { SwitchToMode "Normal"; }"#));
        assert!(!rendered.contains(r#"unbind "Ctrl s""#));
        assert!(!rendered.contains(r#"bind "Alt 1" { GoToTab 1; }"#));
        assert!(rendered.contains(r#"bind "Alt 2" { GoToTab 2; }"#));
    }

    // Defends: native unbind defaults do not suppress a user remap that intentionally reuses the original key.
    #[test]
    fn native_zellij_unbinds_drop_when_remap_reuses_unbound_key() {
        let temp = tempfile::tempdir().unwrap();
        let overrides_path = temp.path().join("yazelix_overrides.kdl");
        std::fs::write(&overrides_path, "").unwrap();
        let mut keybindings = sample_zellij_native_keybindings();
        keybindings.insert("scroll_mode".to_string(), vec!["Ctrl s".to_string()]);

        let rendered = read_yazelix_override_keybinds(
            &overrides_path,
            std::path::Path::new("/opt/yazelix"),
            &BTreeMap::new(),
            &keybindings,
            &[],
        )
        .unwrap()
        .join("\n");

        assert!(rendered.contains(r#"bind "Ctrl s" { SwitchToMode "Scroll"; }"#));
        assert!(rendered.contains(r#"bind "Ctrl s" { SwitchToMode "Normal"; }"#));
        assert!(!rendered.contains(r#"unbind "Ctrl s""#));
    }

    // Defends: integrated zjstatus layout data comes from the child widget command as an opaque plugin block.
    #[test]
    fn renders_zjstatus_plugin_block_with_child_renderer_command() {
        let temp = tempfile::tempdir().unwrap();
        let runtime_dir = temp.path();
        let libexec = runtime_dir.join("libexec");
        std::fs::create_dir_all(&libexec).unwrap();
        let renderer = libexec.join("yazelix_zellij_bar_widget");
        std::fs::write(
            &renderer,
            r#"#!/bin/sh
[ "$1" = "render-yazelix-runtime" ] || exit 11
[ "$2" = "--json" ] || exit 12
case "$3" in
  *'"widget_tray":["editor","workspace","cpu"]'*) ;;
  *) exit 13 ;;
esac
case "$3" in
  *'"zjstatus_plugin_url":"file:/tmp/zjstatus.wasm"'*) ;;
  *) exit 14 ;;
esac
case "$3" in
  *'"codex_usage_periods":["5h","week"]'*) ;;
  *) exit 15 ;;
esac
case "$3" in
  *'"appearance_mode":"dark"'*) ;;
  *) exit 16 ;;
esac
printf '%s\n' '{"schema_version":3,"plugin_block":"CHILD_PLUGIN_BLOCK"}'
"#,
        )
        .unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut permissions = std::fs::metadata(&renderer).unwrap().permissions();
            permissions.set_mode(0o755);
            std::fs::set_permissions(&renderer, permissions).unwrap();
        }
        let plan = sample_render_plan_for_widgets(
            vec!["editor", "workspace", "cpu"],
            "hx",
            "/nix/store/example/bin/nu",
            "ghostty",
        );
        let plugin_block =
            render_integrated_zjstatus_bar(runtime_dir, &plan, "file:/tmp/zjstatus.wasm").unwrap();

        assert_eq!(plugin_block, "CHILD_PLUGIN_BLOCK");
    }

    // Regression: startup layouts declare the initial tab explicitly so launch cwd stays owned by
    // Zellij --default-cwd while new tabs remain home-scoped.
    #[test]
    fn startup_layouts_keep_initial_tab_distinct_from_home_scoped_new_tabs() {
        let name = "yzx_side.kdl";
        let content = zellij_config_pack::bundled_layout_templates()
            .into_iter()
            .find(|template| template.relative_path == name)
            .unwrap_or_else(|| panic!("child config pack missing {name}"))
            .content;
        let mut default_template_line = None;
        let mut initial_tab_line = None;
        let mut new_tab_line = None;
        let mut layout_depth = 0usize;

        for (line_number, line) in content.lines().enumerate() {
            let trimmed = line.trim();

            if layout_depth == 1 {
                match trimmed {
                    "default_tab_template {" => {
                        assert!(
                            default_template_line.replace(line_number).is_none(),
                            "{name} declares default_tab_template more than once"
                        );
                    }
                    "tab" => {
                        assert!(
                            initial_tab_line.replace(line_number).is_none(),
                            "{name} declares more than one explicit initial tab"
                        );
                    }
                    r#"new_tab_template cwd="__YAZELIX_HOME_DIR__" {"# => {
                        assert!(
                            new_tab_line.replace(line_number).is_none(),
                            "{name} declares home-scoped new_tab_template more than once"
                        );
                    }
                    _ => {}
                }
            }

            let closing_braces = trimmed.matches('}').count();
            assert!(
                closing_braces <= layout_depth,
                "{name} closes more layout blocks than it opens before line {}",
                line_number + 1
            );
            layout_depth -= closing_braces;
            layout_depth += trimmed.matches('{').count();
        }
        assert_eq!(layout_depth, 0, "{name} leaves layout blocks unclosed");

        let default_template_line =
            default_template_line.unwrap_or_else(|| panic!("{name} missing default_tab_template"));
        let initial_tab_line =
            initial_tab_line.unwrap_or_else(|| panic!("{name} missing explicit initial tab"));
        let new_tab_line =
            new_tab_line.unwrap_or_else(|| panic!("{name} missing home-scoped new_tab_template"));

        assert!(
            default_template_line < initial_tab_line,
            "{name} initial tab must follow default_tab_template"
        );
        assert!(
            initial_tab_line < new_tab_line,
            "{name} initial tab must precede home-scoped new_tab_template"
        );
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
            &sample_zellij_native_keybindings(),
            &default_custom_popups(),
        )
        .unwrap();
        let merged = override_lines.join("\n");

        assert!(merged.contains("MessagePlugin \"yzpp\""));
        assert!(merged.contains("name \"toggle\""));
        assert!(merged.contains("open_workspace_terminal"));
        assert!(merged.contains("payload \"bottom_popup\""));
        assert!(merged.contains("payload \"menu\""));
        assert!(merged.contains("payload \"config\""));
        assert!(merged.contains("MessagePlugin \"yazelix_pane_orchestrator\""));
        assert!(merged.contains("toggle_editor_sidebar_focus"));
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
        keybindings.insert("menu".to_string(), vec!["Alt p".to_string()]);
        validate_zellij_keybindings(&keybindings).unwrap();

        let merged = read_yazelix_override_keybinds(
            &overrides_path,
            std::path::Path::new("/opt/yazelix"),
            &keybindings,
            &sample_zellij_native_keybindings(),
            &[],
        )
        .unwrap()
        .join("\n");

        assert!(!merged.contains(r#"unbind "Alt p""#));
        assert!(merged.contains(r#"bind "Alt p" {"#));
        assert!(merged.contains(r#"payload "menu""#));
        assert!(!merged.contains(r#"bind "Alt t" {"#));
        assert!(!merged.contains(r#"bind "Alt Shift M" {"#));
    }

    // Defends: generated semantic Zellij action bindings fail fast when two actions claim the same key.
    #[test]
    fn semantic_keybinds_reject_duplicate_action_keys() {
        let mut keybindings = sample_zellij_keybindings();
        keybindings.insert("bottom_popup".to_string(), vec!["Alt Space".to_string()]);
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
                "toggle_left_sidebar": [],
            }),
        );

        let keybindings = resolve_zellij_keybindings(&config).unwrap();

        assert_eq!(keybindings["menu"], vec!["Alt Space"]);
        assert_eq!(keybindings["toggle_left_sidebar"], Vec::<String>::new());
        assert_eq!(keybindings["bottom_popup"], vec!["Alt Shift J"]);
        assert_eq!(keybindings["top_popup"], vec!["Alt Shift K"]);
        assert_eq!(
            keybindings["toggle_editor_right_sidebar_focus"],
            vec!["Ctrl Shift Y"]
        );
    }

    // Defends: named popup surfaces can use distinct commands without changing the generic `yzx popup` program.
    #[test]
    fn popup_commands_config_inherits_defaults_and_resolves_editor_token() {
        let mut config = JsonMap::new();
        config.insert("editor_command".into(), json!("nvim"));
        config.insert(
            POPUP_COMMANDS_CONFIG_KEY.to_string(),
            json!({
                "top_popup": ["editor", "settings.md"],
            }),
        );

        let popup_commands = resolve_popup_commands_config(&config).unwrap();

        assert_eq!(popup_commands[BOTTOM_POPUP_COMMAND_KEY], vec!["lazygit"]);
        assert_eq!(
            popup_commands[TOP_POPUP_COMMAND_KEY],
            vec!["nvim", "settings.md"]
        );
        assert_eq!(popup_commands[MENU_POPUP_COMMAND_KEY], vec!["yzx", "menu"]);
    }

    // Defends: custom popups generate managed yzpp specs and keybindings without expanding popup_commands.
    #[test]
    fn custom_popups_config_defaults_to_zenith_and_resolves_editor_token() {
        let mut config = JsonMap::new();
        let default_popups = resolve_custom_popups_config(&config).unwrap();
        assert_eq!(default_popups, default_custom_popups());

        config.insert("editor_command".into(), json!("nvim"));
        config.insert(
            CUSTOM_POPUPS_CONFIG_KEY.to_string(),
            json!([
                {
                    "id": "gitui",
                    "command": ["editor", "repo.md"],
                    "keybindings": ["Alt Shift G"],
                    "keep_alive": true,
                },
                {
                    "id": "zenith",
                    "command": ["zenith"],
                    "keybindings": ["Alt Shift I"],
                },
                {
                    "id": "btop",
                    "command": ["btop"],
                    "keybindings": ["Alt Shift Y"],
                    "keep_alive": null,
                }
            ]),
        );

        let custom_popups = resolve_custom_popups_config(&config).unwrap();

        assert_eq!(
            custom_popups,
            vec![
                CustomPopup {
                    id: "gitui".to_string(),
                    command: vec!["nvim".to_string(), "repo.md".to_string()],
                    keybindings: vec!["Alt Shift G".to_string()],
                    keep_alive: true,
                },
                CustomPopup {
                    id: "zenith".to_string(),
                    command: vec!["zenith".to_string()],
                    keybindings: vec!["Alt Shift I".to_string()],
                    keep_alive: true,
                },
                CustomPopup {
                    id: "btop".to_string(),
                    command: vec!["btop".to_string()],
                    keybindings: vec!["Alt Shift Y".to_string()],
                    keep_alive: false,
                }
            ]
        );
    }
}
