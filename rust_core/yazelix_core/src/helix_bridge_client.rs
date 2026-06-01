// Test lane: default
//! Internal Helix action bridge client for `yzx_control helix`.

use crate::bridge::{CoreError, ErrorClass};
use crate::control_plane::state_dir_from_env;
use crate::session_config_snapshot::{
    load_session_config_snapshot_from_path, session_config_snapshot_path_from_env,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const BRIDGE_SCHEMA_VERSION: u64 = 1;
const DEFAULT_TIMEOUT_MS: u64 = 1_500;
const MAX_TIMEOUT_MS: u64 = 10_000;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct HelixTargetArgs {
    session_id: Option<String>,
    instance_id: Option<String>,
    zellij_pane_id: Option<String>,
    json: bool,
}

#[derive(Debug, Clone, PartialEq)]
struct HelixActionArgs {
    target: HelixTargetArgs,
    action: String,
    payload: Value,
    timeout_ms: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
struct HelixBridgeRegistry {
    schema_version: u64,
    session_id: String,
    instance_id: String,
    socket_path: String,
    auth_token_path: String,
    pid: u32,
    zellij_session_name: Option<String>,
    zellij_tab_position: Option<String>,
    zellij_pane_id: Option<String>,
    started_at_unix_ms: u128,
    managed_config_path: Option<String>,
}

#[derive(Debug, Clone)]
struct BridgeTarget {
    registry: HelixBridgeRegistry,
    auth_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HelixBridgeResponse {
    pub schema_version: u64,
    pub request_id: String,
    pub status: String,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<HelixBridgeError>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HelixBridgeError {
    pub class: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
struct HelixBridgeStatusData {
    session_id: String,
    instance_id: String,
    socket_path: String,
    zellij_pane_id: Option<String>,
    managed_config_path: Option<String>,
}

#[derive(Debug, Serialize)]
struct HelixBridgeRequest<'a> {
    schema_version: u64,
    request_id: String,
    auth_token: &'a str,
    action: &'a str,
    timeout_ms: u64,
    payload: &'a Value,
}

pub fn run_yzx_helix(args: &[String]) -> Result<i32, CoreError> {
    if args.is_empty() || matches!(args[0].as_str(), "-h" | "--help" | "help") {
        print_helix_help();
        return Ok(0);
    }

    let subcommand = args[0].as_str();
    let tail = &args[1..];
    match subcommand {
        "action" => run_helix_action(tail),
        "status" => run_helix_status(tail),
        other => Err(CoreError::usage(format!(
            "Unknown helix subcommand: {other}"
        ))),
    }
}

pub fn internal_helix_control_subcommands_usage() -> &'static str {
    "action|status"
}

fn run_helix_action(args: &[String]) -> Result<i32, CoreError> {
    if args.len() == 1 && matches!(args[0].as_str(), "-h" | "--help" | "help") {
        print_helix_help();
        return Ok(0);
    }
    let parsed = parse_helix_action_args(args)?;
    validate_action_payload(&parsed.action, &parsed.payload)?;

    let state_dir = state_dir_from_env()?;
    let session_id = resolve_session_id(parsed.target.session_id.as_deref())?;
    let target = resolve_bridge_target(
        &state_dir,
        &BridgeTargetSelector {
            session_id,
            instance_id: parsed.target.instance_id.clone(),
            zellij_pane_id: parsed.target.zellij_pane_id.clone(),
        },
    )?;

    let response = send_bridge_action(&target, &parsed.action, &parsed.payload, parsed.timeout_ms)?;
    if response.status == "error" {
        return Err(bridge_response_error_to_core(&parsed.action, response));
    }

    if parsed.target.json {
        print_json(&response)?;
    } else if let Some(data) = response.data {
        print_json(&data)?;
    }
    Ok(0)
}

fn run_helix_status(args: &[String]) -> Result<i32, CoreError> {
    if args.len() == 1 && matches!(args[0].as_str(), "-h" | "--help" | "help") {
        print_helix_help();
        return Ok(0);
    }
    let target_args = parse_helix_status_args(args)?;
    let state_dir = state_dir_from_env()?;
    let session_id = resolve_session_id(target_args.session_id.as_deref())?;
    let target = resolve_bridge_target(
        &state_dir,
        &BridgeTargetSelector {
            session_id,
            instance_id: target_args.instance_id.clone(),
            zellij_pane_id: target_args.zellij_pane_id.clone(),
        },
    )?;
    let data = HelixBridgeStatusData {
        session_id: target.registry.session_id,
        instance_id: target.registry.instance_id,
        socket_path: target.registry.socket_path,
        zellij_pane_id: target.registry.zellij_pane_id,
        managed_config_path: target.registry.managed_config_path,
    };

    if target_args.json {
        print_json(&data)?;
    } else {
        println!("Helix bridge available: {}", data.instance_id);
    }
    Ok(0)
}

fn print_helix_help() {
    println!("Send structured actions to a Yazelix-managed Helix bridge");
    println!();
    println!("Usage:");
    println!(
        "  yzx_control helix action <name> [--payload <json>] [--session-id <id>] [--instance-id <id>] [--zellij-pane-id <id>] [--timeout-ms <ms>] [--json]"
    );
    println!(
        "  yzx_control helix status [--session-id <id>] [--instance-id <id>] [--zellij-pane-id <id>] [--json]"
    );
}

fn parse_helix_action_args(args: &[String]) -> Result<HelixActionArgs, CoreError> {
    let Some(action) = args.first() else {
        return Err(CoreError::usage(
            "helix action requires an action name. Try `yzx_control helix --help`.",
        ));
    };
    if matches!(action.as_str(), "-h" | "--help" | "help") {
        print_helix_help();
        return Err(CoreError::classified(
            ErrorClass::Usage,
            "help_requested",
            "Help was requested.",
            "",
            json!({}),
        ));
    }
    if action.starts_with('-') {
        return Err(CoreError::usage(
            "helix action requires the action name before flags.",
        ));
    }

    let mut parsed = HelixActionArgs {
        target: HelixTargetArgs::default(),
        action: action.clone(),
        payload: json!({}),
        timeout_ms: DEFAULT_TIMEOUT_MS,
    };
    parse_target_and_action_flags(&args[1..], &mut parsed)?;
    Ok(parsed)
}

fn parse_helix_status_args(args: &[String]) -> Result<HelixTargetArgs, CoreError> {
    let mut target = HelixTargetArgs::default();
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "-h" | "--help" | "help" => {
                print_helix_help();
                return Err(CoreError::classified(
                    ErrorClass::Usage,
                    "help_requested",
                    "Help was requested.",
                    "",
                    json!({}),
                ));
            }
            "--json" => target.json = true,
            "--session-id" => target.session_id = Some(next_flag_value(args, &mut index)?),
            "--instance-id" => target.instance_id = Some(next_flag_value(args, &mut index)?),
            "--zellij-pane-id" => target.zellij_pane_id = Some(next_flag_value(args, &mut index)?),
            other => {
                return Err(CoreError::usage(format!(
                    "Unknown argument for helix status: {other}"
                )));
            }
        }
        index += 1;
    }
    validate_target_args(&target)?;
    Ok(target)
}

fn parse_target_and_action_flags(
    args: &[String],
    parsed: &mut HelixActionArgs,
) -> Result<(), CoreError> {
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--payload" => {
                let raw = next_flag_value(args, &mut index)?;
                parsed.payload = serde_json::from_str::<Value>(&raw).map_err(|source| {
                    CoreError::classified(
                        ErrorClass::Usage,
                        "invalid_helix_action_payload",
                        format!("Could not parse helix action payload JSON: {source}"),
                        "Pass a valid JSON object to --payload.",
                        json!({ "payload": raw }),
                    )
                })?;
            }
            "--timeout-ms" => {
                let raw = next_flag_value(args, &mut index)?;
                parsed.timeout_ms = parse_timeout_ms(&raw)?;
            }
            "--json" => parsed.target.json = true,
            "--session-id" => parsed.target.session_id = Some(next_flag_value(args, &mut index)?),
            "--instance-id" => parsed.target.instance_id = Some(next_flag_value(args, &mut index)?),
            "--zellij-pane-id" => {
                parsed.target.zellij_pane_id = Some(next_flag_value(args, &mut index)?)
            }
            other => {
                return Err(CoreError::usage(format!(
                    "Unknown argument for helix action: {other}"
                )));
            }
        }
        index += 1;
    }
    validate_target_args(&parsed.target)?;
    Ok(())
}

fn next_flag_value(args: &[String], index: &mut usize) -> Result<String, CoreError> {
    let flag = &args[*index];
    *index += 1;
    let Some(value) = args.get(*index) else {
        return Err(CoreError::usage(format!("{flag} requires a value")));
    };
    if value.trim().is_empty() {
        return Err(CoreError::usage(format!(
            "{flag} requires a non-empty value"
        )));
    }
    Ok(value.clone())
}

fn parse_timeout_ms(raw: &str) -> Result<u64, CoreError> {
    let timeout = raw.parse::<u64>().map_err(|source| {
        CoreError::classified(
            ErrorClass::Usage,
            "invalid_helix_bridge_timeout",
            format!("Invalid Helix bridge timeout `{raw}`: {source}"),
            "Pass --timeout-ms as an integer from 1 to 10000.",
            json!({ "timeout_ms": raw }),
        )
    })?;
    if !(1..=MAX_TIMEOUT_MS).contains(&timeout) {
        return Err(CoreError::classified(
            ErrorClass::Usage,
            "invalid_helix_bridge_timeout",
            format!("Helix bridge timeout must be between 1 and {MAX_TIMEOUT_MS} ms."),
            "Pass --timeout-ms as an integer from 1 to 10000.",
            json!({ "timeout_ms": timeout }),
        ));
    }
    Ok(timeout)
}

fn validate_target_args(target: &HelixTargetArgs) -> Result<(), CoreError> {
    if let Some(session_id) = &target.session_id {
        validate_bridge_id("session id", session_id)?;
    }
    if let Some(instance_id) = &target.instance_id {
        validate_bridge_id("instance id", instance_id)?;
    }
    Ok(())
}

fn validate_bridge_id(label: &str, value: &str) -> Result<(), CoreError> {
    if value.is_empty() || value.trim() != value {
        return Err(CoreError::usage(format!(
            "Helix bridge {label} must be non-empty and untrimmed"
        )));
    }
    if !value
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'_'))
    {
        return Err(CoreError::usage(format!(
            "Helix bridge {label} may only contain ASCII letters, numbers, dots, hyphens, and underscores"
        )));
    }
    Ok(())
}

fn validate_action_payload(action: &str, payload: &Value) -> Result<(), CoreError> {
    match action {
        "helix.get_context" => {
            if !payload.is_object() {
                return Err(invalid_payload(
                    "helix.get_context payload must be an object",
                ));
            }
        }
        "helix.set_cwd" => {
            let Some(working_dir) = payload.get("working_dir").and_then(Value::as_str) else {
                return Err(invalid_payload(
                    "helix.set_cwd requires payload.working_dir",
                ));
            };
            require_absolute_payload_path("payload.working_dir", working_dir)?;
        }
        "helix.open_files" => {
            let Some(file_paths) = payload.get("file_paths").and_then(Value::as_array) else {
                return Err(invalid_payload(
                    "helix.open_files requires payload.file_paths",
                ));
            };
            if file_paths.is_empty() {
                return Err(invalid_payload(
                    "helix.open_files requires at least one file path",
                ));
            }
            for value in file_paths {
                let Some(path) = value.as_str() else {
                    return Err(invalid_payload(
                        "helix.open_files payload.file_paths entries must be strings",
                    ));
                };
                require_absolute_payload_path("payload.file_paths[]", path)?;
            }
            if let Some(working_dir) = payload.get("working_dir") {
                let Some(path) = working_dir.as_str() else {
                    return Err(invalid_payload(
                        "helix.open_files payload.working_dir must be a string",
                    ));
                };
                require_absolute_payload_path("payload.working_dir", path)?;
            }
            if let Some(focus) = payload.get("focus")
                && !focus.is_boolean()
            {
                return Err(invalid_payload(
                    "helix.open_files payload.focus must be a boolean",
                ));
            }
        }
        _ => {
            if !payload.is_object() {
                return Err(invalid_payload("Helix bridge payload must be an object"));
            }
        }
    }
    Ok(())
}

fn require_absolute_payload_path(label: &str, raw: &str) -> Result<(), CoreError> {
    if raw.trim().is_empty() {
        return Err(invalid_payload(format!("{label} must be non-empty")));
    }
    if !Path::new(raw).is_absolute() {
        return Err(invalid_payload(format!("{label} must be an absolute path")));
    }
    Ok(())
}

fn invalid_payload(message: impl Into<String>) -> CoreError {
    CoreError::classified(
        ErrorClass::Usage,
        "invalid_helix_action_payload",
        message,
        "Pass a payload matching the requested Helix bridge action.",
        json!({}),
    )
}

fn resolve_session_id(explicit_session_id: Option<&str>) -> Result<String, CoreError> {
    if let Some(session_id) = explicit_session_id {
        validate_bridge_id("session id", session_id)?;
        return Ok(session_id.to_string());
    }
    if let Ok(session_id) = std::env::var("YAZELIX_HELIX_BRIDGE_SESSION_ID")
        && !session_id.trim().is_empty()
    {
        validate_bridge_id("session id", &session_id)?;
        return Ok(session_id);
    }
    if let Some(path) = session_config_snapshot_path_from_env() {
        return load_session_config_snapshot_from_path(&path).map(|snapshot| snapshot.snapshot_id);
    }
    Err(CoreError::classified(
        ErrorClass::Runtime,
        "missing_helix_bridge_session",
        "Could not resolve the Yazelix Helix bridge session id.",
        "Run this command inside a Yazelix-managed session or pass --session-id.",
        json!({ "env": "YAZELIX_SESSION_CONFIG_PATH" }),
    ))
}

#[derive(Debug, Clone)]
struct BridgeTargetSelector {
    session_id: String,
    instance_id: Option<String>,
    zellij_pane_id: Option<String>,
}

fn resolve_bridge_target(
    state_dir: &Path,
    selector: &BridgeTargetSelector,
) -> Result<BridgeTarget, CoreError> {
    let bridge_dir = state_dir.join("helix_bridge").join(&selector.session_id);
    let registries = load_bridge_registries(&bridge_dir, &selector.session_id)?;
    let candidates = registries
        .into_iter()
        .filter(|registry| registry_matches_selector(registry, selector))
        .collect::<Vec<_>>();

    if candidates.is_empty() {
        return Err(CoreError::classified(
            ErrorClass::Runtime,
            "missing_helix_bridge_instance",
            "No Yazelix-managed Helix bridge matched the requested target.",
            "Open a managed Helix pane in this Yazelix session, then retry.",
            json!({
                "session_id": selector.session_id,
                "instance_id": selector.instance_id,
                "zellij_pane_id": selector.zellij_pane_id,
            }),
        ));
    }

    let mut live = Vec::new();
    let mut stale = Vec::new();
    for registry in candidates {
        if registry_socket_is_live(&registry) {
            live.push(registry);
        } else {
            stale.push(registry);
        }
    }

    if live.is_empty() {
        return Err(CoreError::classified(
            ErrorClass::Runtime,
            "stale_helix_bridge_instance",
            "Matched Helix bridge registry entries are stale.",
            "Restart the managed Helix pane or run a future doctor repair for stale bridge registries.",
            json!({
                "session_id": selector.session_id,
                "stale_instances": stale.iter().map(|entry| entry.instance_id.as_str()).collect::<Vec<_>>(),
            }),
        ));
    }
    if live.len() > 1 {
        return Err(CoreError::classified(
            ErrorClass::Runtime,
            "ambiguous_helix_bridge_instance",
            "More than one live Helix bridge matched the requested target.",
            "Pass --instance-id or --zellij-pane-id so Yazelix can target one Helix instance.",
            json!({
                "session_id": selector.session_id,
                "instances": live.iter().map(|entry| entry.instance_id.as_str()).collect::<Vec<_>>(),
            }),
        ));
    }

    let registry = live.remove(0);
    let auth_token = read_auth_token(&registry.auth_token_path)?;
    Ok(BridgeTarget {
        registry,
        auth_token,
    })
}

fn registry_matches_selector(
    registry: &HelixBridgeRegistry,
    selector: &BridgeTargetSelector,
) -> bool {
    if registry.session_id != selector.session_id {
        return false;
    }
    if let Some(instance_id) = &selector.instance_id
        && &registry.instance_id != instance_id
    {
        return false;
    }
    if let Some(zellij_pane_id) = &selector.zellij_pane_id
        && registry.zellij_pane_id.as_ref() != Some(zellij_pane_id)
    {
        return false;
    }
    true
}

fn load_bridge_registries(
    bridge_dir: &Path,
    session_id: &str,
) -> Result<Vec<HelixBridgeRegistry>, CoreError> {
    let entries = fs::read_dir(bridge_dir).map_err(|source| {
        CoreError::io(
            "helix_bridge_registry_dir",
            format!(
                "Could not read Helix bridge registry directory {}.",
                bridge_dir.display()
            ),
            "Open a managed Helix pane in this Yazelix session, then retry.",
            bridge_dir.to_string_lossy(),
            source,
        )
    })?;
    let mut registries = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|source| {
            CoreError::io(
                "helix_bridge_registry_entry",
                "Could not read a Helix bridge registry entry.",
                "Check permissions under the Yazelix state directory.",
                bridge_dir.to_string_lossy(),
                source,
            )
        })?;
        let path = entry.path();
        if path.extension().and_then(|value| value.to_str()) != Some("json") {
            continue;
        }
        let raw = fs::read_to_string(&path).map_err(|source| {
            CoreError::io(
                "helix_bridge_registry_read",
                format!("Could not read Helix bridge registry {}.", path.display()),
                "Restart the managed Helix pane so it recreates its registry.",
                path.to_string_lossy(),
                source,
            )
        })?;
        let registry = serde_json::from_str::<HelixBridgeRegistry>(&raw).map_err(|source| {
            CoreError::classified(
                ErrorClass::Runtime,
                "helix_bridge_registry_parse",
                format!(
                    "Could not parse Helix bridge registry {}: {source}",
                    path.display()
                ),
                "Restart the managed Helix pane so it recreates its registry.",
                json!({ "path": path.to_string_lossy() }),
            )
        })?;
        if registry.schema_version != BRIDGE_SCHEMA_VERSION {
            return Err(CoreError::classified(
                ErrorClass::Runtime,
                "helix_bridge_registry_schema",
                format!(
                    "Unsupported Helix bridge registry schema {} for {}.",
                    registry.schema_version,
                    path.display()
                ),
                "Update Yazelix and the bundled yazelix-helix fork together.",
                json!({
                    "path": path.to_string_lossy(),
                    "expected_schema_version": BRIDGE_SCHEMA_VERSION,
                    "actual_schema_version": registry.schema_version,
                }),
            ));
        }
        if registry.session_id != session_id {
            return Err(CoreError::classified(
                ErrorClass::Runtime,
                "helix_bridge_registry_session_mismatch",
                format!(
                    "Helix bridge registry {} belongs to session {} but was loaded from session {}.",
                    path.display(),
                    registry.session_id,
                    session_id
                ),
                "Restart the managed Helix pane so it recreates its registry in the correct session directory.",
                json!({
                    "path": path.to_string_lossy(),
                    "registry_session_id": registry.session_id,
                    "expected_session_id": session_id,
                }),
            ));
        }
        registries.push(registry);
    }
    Ok(registries)
}

#[cfg(unix)]
fn registry_socket_is_live(registry: &HelixBridgeRegistry) -> bool {
    use std::os::unix::fs::FileTypeExt;

    fs::metadata(&registry.socket_path)
        .map(|metadata| metadata.file_type().is_socket())
        .unwrap_or(false)
}

#[cfg(not(unix))]
fn registry_socket_is_live(_registry: &HelixBridgeRegistry) -> bool {
    false
}

fn read_auth_token(path: &str) -> Result<String, CoreError> {
    let token = fs::read_to_string(path).map_err(|source| {
        CoreError::io(
            "helix_bridge_auth_token_read",
            format!("Could not read Helix bridge auth token at {path}."),
            "Restart the managed Helix pane so it recreates its bridge token.",
            path,
            source,
        )
    })?;
    if token.is_empty() || token.trim() != token || token.contains(['\n', '\r']) {
        return Err(CoreError::classified(
            ErrorClass::Runtime,
            "helix_bridge_auth_token_invalid",
            "Helix bridge auth token is malformed.",
            "Restart the managed Helix pane so it recreates its bridge token.",
            json!({ "path": path }),
        ));
    }
    Ok(token)
}

#[cfg(unix)]
fn send_bridge_action(
    target: &BridgeTarget,
    action: &str,
    payload: &Value,
    timeout_ms: u64,
) -> Result<HelixBridgeResponse, CoreError> {
    use std::os::unix::net::UnixStream;

    let socket_path = Path::new(&target.registry.socket_path);
    let mut stream = UnixStream::connect(socket_path).map_err(|source| {
        CoreError::io(
            "helix_bridge_socket_connect",
            format!(
                "Could not connect to Helix bridge socket {}.",
                socket_path.display()
            ),
            "Restart the managed Helix pane, then retry.",
            socket_path.to_string_lossy(),
            source,
        )
    })?;
    let timeout = Duration::from_millis(timeout_ms.clamp(1, MAX_TIMEOUT_MS));
    stream.set_read_timeout(Some(timeout)).map_err(|source| {
        CoreError::io(
            "helix_bridge_socket_timeout",
            "Could not set Helix bridge socket read timeout.",
            "Retry from a fresh Yazelix session.",
            socket_path.to_string_lossy(),
            source,
        )
    })?;
    stream.set_write_timeout(Some(timeout)).map_err(|source| {
        CoreError::io(
            "helix_bridge_socket_timeout",
            "Could not set Helix bridge socket write timeout.",
            "Retry from a fresh Yazelix session.",
            socket_path.to_string_lossy(),
            source,
        )
    })?;

    let request = HelixBridgeRequest {
        schema_version: BRIDGE_SCHEMA_VERSION,
        request_id: request_id(),
        auth_token: &target.auth_token,
        action,
        timeout_ms,
        payload,
    };
    let encoded = serde_json::to_string(&request).map_err(|source| {
        CoreError::classified(
            ErrorClass::Internal,
            "helix_bridge_request_encode",
            format!("Could not encode Helix bridge request: {source}"),
            "Report this Yazelix bug.",
            json!({ "action": action }),
        )
    })?;
    writeln!(stream, "{encoded}").map_err(|source| {
        CoreError::io(
            "helix_bridge_socket_write",
            "Could not write Helix bridge request.",
            "Retry from a fresh Yazelix session.",
            socket_path.to_string_lossy(),
            source,
        )
    })?;

    let mut reader = BufReader::new(stream);
    let mut raw_response = String::new();
    reader.read_line(&mut raw_response).map_err(|source| {
        CoreError::io(
            "helix_bridge_socket_read",
            "Could not read Helix bridge response.",
            "Retry from a fresh Yazelix session.",
            socket_path.to_string_lossy(),
            source,
        )
    })?;
    if raw_response.trim().is_empty() {
        return Err(CoreError::classified(
            ErrorClass::Runtime,
            "helix_bridge_empty_response",
            "Helix bridge returned an empty response.",
            "Restart the managed Helix pane, then retry.",
            json!({ "socket_path": socket_path.to_string_lossy() }),
        ));
    }
    let response =
        serde_json::from_str::<HelixBridgeResponse>(&raw_response).map_err(|source| {
            CoreError::classified(
                ErrorClass::Runtime,
                "helix_bridge_response_parse",
                format!("Could not parse Helix bridge response: {source}"),
                "Update Yazelix and the bundled yazelix-helix fork together.",
                json!({ "socket_path": socket_path.to_string_lossy() }),
            )
        })?;
    if response.schema_version != BRIDGE_SCHEMA_VERSION {
        return Err(CoreError::classified(
            ErrorClass::Runtime,
            "helix_bridge_response_schema",
            format!(
                "Unsupported Helix bridge response schema {}.",
                response.schema_version
            ),
            "Update Yazelix and the bundled yazelix-helix fork together.",
            json!({
                "expected_schema_version": BRIDGE_SCHEMA_VERSION,
                "actual_schema_version": response.schema_version,
            }),
        ));
    }
    Ok(response)
}

#[cfg(not(unix))]
fn send_bridge_action(
    _target: &BridgeTarget,
    _action: &str,
    _payload: &Value,
    _timeout_ms: u64,
) -> Result<HelixBridgeResponse, CoreError> {
    Err(CoreError::classified(
        ErrorClass::Runtime,
        "helix_bridge_unsupported_platform",
        "The Helix bridge currently requires Unix sockets.",
        "Use Yazelix on a Unix-like platform for Helix bridge actions.",
        json!({}),
    ))
}

fn bridge_response_error_to_core(action: &str, response: HelixBridgeResponse) -> CoreError {
    let error = response.error.unwrap_or(HelixBridgeError {
        class: "internal_error".to_string(),
        message: "Helix bridge returned status=error without an error body".to_string(),
    });
    let class = match error.class.as_str() {
        "invalid_payload" => ErrorClass::Usage,
        "unsupported_action" | "permission_denied" | "stale_instance" | "editor_busy"
        | "timeout" => ErrorClass::Runtime,
        _ => ErrorClass::Runtime,
    };
    CoreError::classified(
        class,
        format!("helix_bridge_{}", error.class),
        error.message,
        "Retry after fixing the managed Helix bridge target, or run `yzx_control helix status --json` to inspect availability.",
        json!({
            "action": action,
            "request_id": response.request_id,
            "bridge_error_class": error.class,
        }),
    )
}

fn request_id() -> String {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    format!("yzx-{}-{millis}", std::process::id())
}

fn print_json(value: &impl Serialize) -> Result<(), CoreError> {
    let encoded = serde_json::to_string_pretty(value).map_err(|source| {
        CoreError::classified(
            ErrorClass::Internal,
            "json_encode",
            format!("Could not encode Helix bridge output: {source}"),
            "Report this Yazelix bug.",
            json!({}),
        )
    })?;
    println!("{encoded}");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::net::UnixListener;

    fn registry(
        state_dir: &Path,
        session_id: &str,
        instance_id: &str,
        socket_path: &Path,
    ) -> HelixBridgeRegistry {
        let token_path = state_dir
            .join("helix_bridge")
            .join(session_id)
            .join(format!("{instance_id}.token"));
        fs::write(&token_path, "secret").unwrap();
        HelixBridgeRegistry {
            schema_version: BRIDGE_SCHEMA_VERSION,
            session_id: session_id.to_string(),
            instance_id: instance_id.to_string(),
            socket_path: socket_path.to_string_lossy().to_string(),
            auth_token_path: token_path.to_string_lossy().to_string(),
            pid: std::process::id(),
            zellij_session_name: None,
            zellij_tab_position: None,
            zellij_pane_id: None,
            started_at_unix_ms: 1,
            managed_config_path: None,
        }
    }

    fn write_registry(state_dir: &Path, registry: &HelixBridgeRegistry) {
        let dir = state_dir.join("helix_bridge").join(&registry.session_id);
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            dir.join(format!("{}.json", registry.instance_id)),
            serde_json::to_string(registry).unwrap(),
        )
        .unwrap();
    }

    // Defends: the client rejects unsafe instance/session path components before filesystem lookup.
    #[test]
    fn parse_action_rejects_path_traversal_instance_id() {
        let args = vec![
            "helix.get_context".to_string(),
            "--instance-id".to_string(),
            "../escape".to_string(),
        ];
        let error = parse_helix_action_args(&args).unwrap_err();
        assert_eq!(error.code(), "invalid_arguments");
        assert!(error.message().contains("instance id"));
    }

    // Defends: known bridge actions fail before IPC when the typed payload is invalid.
    #[test]
    fn validate_open_files_requires_absolute_paths() {
        let error = validate_action_payload(
            "helix.open_files",
            &json!({ "file_paths": ["relative.rs"] }),
        )
        .unwrap_err();
        assert_eq!(error.code(), "invalid_helix_action_payload");
        assert!(error.message().contains("absolute"));
    }

    // Defends: target discovery does not guess when more than one live Helix bridge matches.
    #[test]
    fn resolve_bridge_target_rejects_ambiguous_live_instances() {
        let tmp = tempfile::tempdir().unwrap();
        let state_dir = tmp.path();
        let session_id = "session-1";
        let dir = state_dir.join("helix_bridge").join(session_id);
        fs::create_dir_all(&dir).unwrap();
        let socket_a = dir.join("a.sock");
        let socket_b = dir.join("b.sock");
        let _listener_a = UnixListener::bind(&socket_a).unwrap();
        let _listener_b = UnixListener::bind(&socket_b).unwrap();
        write_registry(state_dir, &registry(state_dir, session_id, "a", &socket_a));
        write_registry(state_dir, &registry(state_dir, session_id, "b", &socket_b));

        let error = resolve_bridge_target(
            state_dir,
            &BridgeTargetSelector {
                session_id: session_id.to_string(),
                instance_id: None,
                zellij_pane_id: None,
            },
        )
        .unwrap_err();
        assert_eq!(error.code(), "ambiguous_helix_bridge_instance");
    }

    // Defends: bridge-side typed errors are preserved as control-plane error codes.
    #[test]
    fn bridge_response_error_maps_to_core_error() {
        let error = bridge_response_error_to_core(
            "helix.open_files",
            HelixBridgeResponse {
                schema_version: BRIDGE_SCHEMA_VERSION,
                request_id: "r1".to_string(),
                status: "error".to_string(),
                data: None,
                error: Some(HelixBridgeError {
                    class: "unsupported_action".to_string(),
                    message: "nope".to_string(),
                }),
            },
        );
        assert_eq!(error.code(), "helix_bridge_unsupported_action");
        assert_eq!(error.class().as_str(), "runtime");
    }
}
