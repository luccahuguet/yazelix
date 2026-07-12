use crate::action_registry::{
    DEFAULT_INFORMATION_POPUP_KEYS, PANE_ORCHESTRATOR_PLUGIN_ALIAS, YZPP_PLUGIN_ALIAS,
    YazelixActionMetadata, ZELLIJ_ACTIONS, ZELLIJ_NATIVE_KEYBINDINGS, zellij_action_by_local_id,
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
    self as zellij_config_pack, ZellijKeybindActionSpec, ZellijKeybindRenderRequest,
    ZellijNativeKeybindBlockSpec, ZellijNativeKeybindSpec, ZellijRenderPlanData,
    ZellijRenderPlanError, ZellijRenderPlanRequest, ZellijSidecarError, combine_zellij_sidecars,
    compute_zellij_render_plan, validate_zellij_config_sidecar, validate_zellij_plugins_sidecar,
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
    "zellij_widget_frame",
    "zellij_widget_separator",
    "zellij_custom_text",
    "zellij_theme",
    "appearance_mode",
    "support_kitty_keyboard_protocol",
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
    pub session_terminal_label: Option<String>,
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
    widget_frame: String,
    widget_separator: String,
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
    let plugin_artifacts = resolve_plugin_artifacts(&request.runtime_dir, &state_dir)?;
    let [pane_orchestrator_artifact, zjstatus_artifact, yzpp_artifact] = &plugin_artifacts;
    let zellij_keybindings = resolve_zellij_keybindings(&config)?;
    let zellij_native_keybindings = resolve_zellij_native_keybindings(&config)?;
    let popup_commands = resolve_popup_commands_config(&config)?;
    let custom_popups = resolve_custom_popups_config(&config)?;
    validate_custom_popup_keybindings(&zellij_keybindings, &custom_popups)?;
    let terminal_label = request
        .session_terminal_label
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .map(Ok)
        .unwrap_or_else(|| active_terminal_from_runtime_dir(&request.runtime_dir))?;
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
        &render_plan.terminal_label,
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
            "Check config.toml values under workspace, editor, and zellij.",
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

fn child_integration_action_specs() -> Vec<ZellijKeybindActionSpec> {
    ZELLIJ_ACTIONS
        .iter()
        .map(|spec| ZellijKeybindActionSpec {
            local_id: spec.action.local_id.to_string(),
            mode: spec.mode.to_string(),
            plugin_alias: spec.plugin_alias.to_string(),
            message_name: spec.message_name.to_string(),
            payload: spec.payload.map(ToOwned::to_owned),
        })
        .collect()
}

fn child_native_keybinding_specs() -> Vec<ZellijNativeKeybindSpec> {
    ZELLIJ_NATIVE_KEYBINDINGS
        .iter()
        .map(|spec| ZellijNativeKeybindSpec {
            local_id: spec.action.local_id.to_string(),
            blocks: spec
                .blocks
                .iter()
                .map(|block| ZellijNativeKeybindBlockSpec {
                    mode: block.mode.to_string(),
                    action_lines: block
                        .action_lines
                        .iter()
                        .map(|line| (*line).to_string())
                        .collect(),
                })
                .collect(),
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
                    "Define [popups.zenith] with command = \"zenith\", keybinding = \"Alt Shift I\", and keep_alive = true."
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
        keybindings: DEFAULT_INFORMATION_POPUP_KEYS
            .iter()
            .map(|key| (*key).to_string())
            .collect(),
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

pub(crate) fn resolve_custom_popup_keybindings(
    config: &JsonMap<String, JsonValue>,
    popup_id: &str,
) -> Result<Option<Vec<String>>, CoreError> {
    Ok(resolve_custom_popups_config(config)?
        .into_iter()
        .find(|popup| popup.id == popup_id)
        .map(|popup| popup.keybindings))
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
    let config_path = user_config_paths::zellij_config(&config_dir);
    let plugins_path = user_config_paths::zellij_plugins(&config_dir);
    let config = read_text_if_exists(&config_path)?;
    let plugins = read_text_if_exists(&plugins_path)?;
    validate_zellij_config_sidecar(&config)
        .map_err(|error| zellij_sidecar_error(&config_path, error))?;
    validate_zellij_plugins_sidecar(&plugins)
        .map_err(|error| zellij_sidecar_error(&plugins_path, error))?;
    let content = combine_zellij_sidecars(&config, &plugins)
        .expect("individually validated Zellij sidecars combine");

    Ok(ZellijBaseConfigSource {
        source: "managed".to_string(),
        path: Some(config_path),
        content,
    })
}

fn zellij_sidecar_error(path: &Path, error: ZellijSidecarError) -> CoreError {
    CoreError::classified(
        ErrorClass::Config,
        error.code,
        format!("{}: {}:{}", error.message(), path.display(), error.line),
        error.remediation(),
        json!({
            "path": path.display().to_string(),
            "line": error.line,
            "node": error.node,
        }),
    )
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

pub(crate) fn resolve_zellij_keybindings(
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

pub(crate) fn resolve_zellij_native_keybindings(
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
    let request = ZellijKeybindRenderRequest {
        override_template_content: read_text(overrides_path, "read_zellij_overrides")?,
        runtime_dir: runtime_dir.to_string_lossy().into_owned(),
        home_dir: home_dir_from_env()?.to_string_lossy().into_owned(),
        zellij_keybindings: zellij_keybindings.clone(),
        zellij_native_keybindings: zellij_native_keybindings.clone(),
        custom_popups: child_custom_popups(custom_popups),
        integration_actions: child_integration_action_specs(),
        native_actions: child_native_keybinding_specs(),
        pane_orchestrator_plugin_alias: PANE_ORCHESTRATOR_PLUGIN_ALIAS.to_string(),
        popup_plugin_alias: YZPP_PLUGIN_ALIAS.to_string(),
    };
    Ok(zellij_config_pack::render_zellij_keybinds(&request).override_keybinds)
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
        widget_frame: render_plan.widget_frame.clone(),
        widget_separator: render_plan.widget_separator.clone(),
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
    session_terminal_label: &str,
) -> Result<(), CoreError> {
    let metadata_path = merged_config_dir.join(GENERATION_METADATA_NAME);
    let content = serde_json::to_string(&json!({
        "fingerprint": fingerprint,
        "session_terminal_label": session_terminal_label,
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
            zellij_widget_frame: "none".into(),
            zellij_widget_separator: "dot".into(),
            zellij_custom_text: None,
            zellij_theme: "default".into(),
            appearance_mode: "dark".into(),
            support_kitty_keyboard_protocol: "false".into(),
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

    // Defends: generated native mode leaders remain owned by the semantic keybinding contract.
    #[test]
    fn semantic_native_mode_leaders_render_from_the_contract() {
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

        assert!(!rendered.contains("TogglePaneInGroup"));
        assert!(rendered.contains(r#"bind "Ctrl p" { SwitchToMode "Pane"; }"#));
        assert!(rendered.contains(r#"bind "Ctrl p" { SwitchToMode "Normal"; }"#));
        assert!(rendered.contains(r#"bind "Ctrl n" { SwitchToMode "Resize"; }"#));
        assert!(rendered.contains(r#"bind "Ctrl n" { SwitchToMode "Normal"; }"#));
        assert!(rendered.contains(r#"bind "Ctrl t" { SwitchToMode "Tab"; }"#));
        assert!(rendered.contains(r#"bind "Ctrl t" { SwitchToMode "Normal"; }"#));
        assert!(rendered.contains(r#"bind "Ctrl Alt s" { SwitchToMode "Scroll"; }"#));
        assert!(rendered.contains(r#"bind "Ctrl Alt o" { SwitchToMode "Session"; }"#));
        assert!(rendered.contains(r#"bind "Ctrl Alt g" { SwitchToMode "Locked"; }"#));
        assert!(rendered.contains(r#"bind "Ctrl q" { Quit; }"#));
        assert!(rendered.contains(r#"unbind "Ctrl h""#));
        assert!(!rendered.contains(r#"unbind "Ctrl p""#));
        assert!(!rendered.contains(r#"unbind "Ctrl n""#));
        assert!(!rendered.contains(r#"unbind "Ctrl t""#));
        assert!(!rendered.contains(r#"unbind "Ctrl q""#));
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
  *'"widget_frame":"none"'*) ;;
  *) exit 18 ;;
esac
case "$3" in
  *'"widget_separator":"dot"'*) ;;
  *) exit 19 ;;
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

    // Regression: startup layouts name the initial tab without setting a cwd, so launch cwd stays
    // owned by Zellij --default-cwd while new tabs remain home-scoped.
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
                    r#"tab name=__YAZELIX_HOME_TAB_MARKER__"# => {
                        assert!(
                            initial_tab_line.replace(line_number).is_none(),
                            "{name} declares more than one named initial tab"
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
            initial_tab_line.unwrap_or_else(|| panic!("{name} missing named initial tab"));
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

    // Defends: the main materializer passes the configured home directory into the generated tab-mode new-tab action.
    #[test]
    fn semantic_keybinds_name_default_new_tabs_from_home() {
        let temp = tempfile::tempdir().unwrap();
        let overrides_path = temp.path().join("yazelix_overrides.kdl");
        std::fs::write(&overrides_path, "keybinds {}\n").unwrap();

        let override_lines = read_yazelix_override_keybinds(
            &overrides_path,
            std::path::Path::new("/opt/yazelix"),
            &sample_zellij_keybindings(),
            &sample_zellij_native_keybindings(),
            &[],
        )
        .unwrap();
        let merged = override_lines.join("\n");
        let home_dir = home_dir_from_env().unwrap();
        let quoted_home_dir = serde_json::to_string(home_dir.to_string_lossy().as_ref()).unwrap();

        assert!(merged.contains(&format!("NewTab {{ cwd {quoted_home_dir}; name ")));
        assert!(merged.contains(r#"; SwitchToMode "Normal"; }"#));
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

    // Defends: partial config.toml remaps inherit defaults for omitted Yazelix-owned Zellij actions.
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
