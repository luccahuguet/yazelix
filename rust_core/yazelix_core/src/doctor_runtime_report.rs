//! Doctor findings for runtime distribution capability and shared runtime preflight.
//! Bead: yazelix-ulb2.4.3

use crate::bridge::CoreError;
use crate::runtime_components::read_runtime_component_manifest;
use crate::runtime_contract::{
    GeneratedLayoutCheckRequest, LinuxGhosttyDesktopGraphicsRequest, RuntimeCheckData,
    RuntimeContractEvaluateRequest, RuntimeScriptCheckRequest, TerminalSupportCheckRequest,
    evaluate_runtime_contract,
};
use crate::settings_surface::DEFAULT_SETTINGS_CONFIG_FILENAME;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::env;
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

const ZELLIJ_KITTY_PASSTHROUGH_FEATURE: &str = "zellij_kitty_passthrough";
const CHAFA_PROBE_SAFE_FEATURE: &str = "chafa_probe_safe";
const CHAFA_PROBE_UNSAFE_FEATURE: &str = "chafa_probe_unsafe";
const OPTIONAL_HOST_INTEGRATION_NOTE: &str = "optional_host_integration";

#[derive(Debug, Deserialize)]
pub struct SharedRuntimePreflightInput {
    pub zellij_layout_path: PathBuf,
    #[serde(default)]
    pub terminals: Vec<String>,
    pub startup_script_path: PathBuf,
    pub launch_script_path: PathBuf,
    #[serde(default)]
    pub command_search_paths: Vec<PathBuf>,
    pub platform_name: String,
}

#[derive(Debug, Deserialize)]
pub struct DoctorRuntimeEvaluateRequest {
    pub runtime_dir: PathBuf,
    pub yazelix_state_dir: PathBuf,
    pub has_home_manager_managed_install: bool,
    pub is_manual_runtime_reference_path: bool,
    #[serde(default)]
    pub shared_runtime: Option<SharedRuntimePreflightInput>,
}

#[derive(Debug, Serialize)]
pub struct DoctorRuntimeDoctorFinding {
    pub status: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
    pub fix_available: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fix_action: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub capability_tier: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub capability_mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runtime_contract_check: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner_surface: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct DoctorRuntimeEvaluateData {
    pub distribution: DoctorRuntimeDoctorFinding,
    pub shared_runtime_preflight: Vec<DoctorRuntimeDoctorFinding>,
}

#[derive(Debug, Deserialize)]
struct RuntimeToolManifestEntry {
    pub source: String,
    #[serde(default)]
    pub commands: Vec<String>,
    #[serde(default)]
    pub required_commands: Vec<String>,
    #[serde(default)]
    pub notes: Vec<String>,
}

pub fn evaluate_doctor_runtime_report(
    request: &DoctorRuntimeEvaluateRequest,
) -> DoctorRuntimeEvaluateData {
    let distribution = distribution_finding(
        &request.runtime_dir,
        request.has_home_manager_managed_install,
        request.is_manual_runtime_reference_path,
    );
    let managed_layouts_root = request
        .yazelix_state_dir
        .join("configs")
        .join("zellij")
        .join("layouts");

    let runtime_tool_command_search_paths = match &request.shared_runtime {
        Some(shared) => effective_command_search_paths(&shared.command_search_paths),
        None => effective_command_search_paths(&[]),
    };

    let mut shared_runtime_preflight = match &request.shared_runtime {
        None => Vec::new(),
        Some(shared) => match build_shared_preflight_findings(
            shared,
            &managed_layouts_root,
            &request.runtime_dir,
        ) {
            Ok(v) => v,
            Err(e) => vec![DoctorRuntimeDoctorFinding {
                status: "error".into(),
                message: "Shared runtime preflight evaluation failed".into(),
                details: Some(e.to_string()),
                fix_available: false,
                fix_action: None,
                capability_tier: None,
                capability_mode: None,
                runtime_contract_check: None,
                owner_surface: None,
            }],
        },
    };
    if let Some(shared) = &request.shared_runtime {
        shared_runtime_preflight.extend(build_runtime_graphics_findings(
            shared,
            &request.runtime_dir,
            &runtime_tool_command_search_paths,
        ));
    }
    shared_runtime_preflight.extend(build_runtime_tool_source_findings(
        &request.runtime_dir,
        &runtime_tool_command_search_paths,
    ));
    shared_runtime_preflight.extend(build_disabled_runtime_component_findings(
        &request.runtime_dir,
    ));
    shared_runtime_preflight.extend(build_shell_initializer_findings(&request.yazelix_state_dir));
    shared_runtime_preflight.extend(build_yzxterm_launch_log_findings(
        &request.yazelix_state_dir,
        &request.runtime_dir,
    ));
    if let Some(finding) = build_host_rio_env_isolation_finding() {
        shared_runtime_preflight.push(finding);
    }

    DoctorRuntimeEvaluateData {
        distribution,
        shared_runtime_preflight,
    }
}

fn runtime_tools_manifest_path(runtime_dir: &Path) -> PathBuf {
    runtime_dir.join("runtime_tools.json")
}

fn effective_command_search_paths(configured_paths: &[PathBuf]) -> Vec<PathBuf> {
    if !configured_paths.is_empty() {
        return configured_paths.to_vec();
    }

    env::var_os("PATH")
        .map(|path| env::split_paths(&path).collect())
        .unwrap_or_default()
}

fn command_exists_in_paths(command: &str, command_search_paths: &[PathBuf]) -> bool {
    command_search_paths
        .iter()
        .any(|dir| is_executable_command(&dir.join(command)))
}

#[cfg(unix)]
fn is_executable_command(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;

    let Ok(metadata) = fs::metadata(path) else {
        return false;
    };
    metadata.is_file() && metadata.permissions().mode() & 0o111 != 0
}

#[cfg(not(unix))]
fn is_executable_command(path: &Path) -> bool {
    path.is_file()
}

fn read_runtime_tool_manifest(
    runtime_dir: &Path,
) -> Result<Option<BTreeMap<String, RuntimeToolManifestEntry>>, String> {
    let manifest_path = runtime_tools_manifest_path(runtime_dir);
    let raw = match fs::read_to_string(&manifest_path) {
        Ok(raw) => raw,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => {
            return Err(format!(
                "Could not read runtime tool manifest at {}: {error}",
                manifest_path.display()
            ));
        }
    };

    serde_json::from_str(&raw).map(Some).map_err(|error| {
        format!(
            "Could not parse runtime tool manifest at {}: {error}",
            manifest_path.display()
        )
    })
}

fn runtime_tool_required_commands(tool: &RuntimeToolManifestEntry) -> &[String] {
    if tool.required_commands.is_empty() {
        &tool.commands
    } else {
        &tool.required_commands
    }
}

fn runtime_tool_is_optional_host_integration(tool: &RuntimeToolManifestEntry) -> bool {
    tool.notes
        .iter()
        .any(|note| note == OPTIONAL_HOST_INTEGRATION_NOTE)
}

fn format_path_list(command_search_paths: &[PathBuf]) -> String {
    if command_search_paths.is_empty() {
        return "No command search paths were available.".into();
    }

    command_search_paths
        .iter()
        .map(|path| format!("  - {}", path.display()))
        .collect::<Vec<_>>()
        .join("\n")
}

fn build_host_runtime_tool_findings(
    runtime_dir: &Path,
    command_search_paths: &[PathBuf],
) -> Vec<DoctorRuntimeDoctorFinding> {
    let manifest = match read_runtime_tool_manifest(runtime_dir) {
        Ok(Some(manifest)) => manifest,
        Ok(None) => return Vec::new(),
        Err(error) => {
            return vec![DoctorRuntimeDoctorFinding {
                status: "warning".into(),
                message: "Runtime tool manifest could not be read".into(),
                details: Some(error),
                fix_available: false,
                fix_action: None,
                capability_tier: None,
                capability_mode: None,
                runtime_contract_check: Some("runtime_tool_manifest".into()),
                owner_surface: Some("runtime_tool_sources".into()),
            }];
        }
    };

    manifest
        .into_iter()
        .filter(|(_, tool)| tool.source == "host")
        .map(|(name, tool)| {
            let required_commands = runtime_tool_required_commands(&tool);
            let missing_commands = required_commands
                .iter()
                .filter(|command| !command_exists_in_paths(command, command_search_paths))
                .cloned()
                .collect::<Vec<_>>();

            if missing_commands.is_empty() {
                DoctorRuntimeDoctorFinding {
                    status: "ok".into(),
                    message: format!("Host runtime tool available: {name}"),
                    details: Some(format!(
                        "Found required command(s): {}",
                        required_commands.join(", ")
                    )),
                    fix_available: false,
                    fix_action: None,
                    capability_tier: None,
                    capability_mode: None,
                    runtime_contract_check: Some(format!("host_runtime_tool:{name}")),
                    owner_surface: Some("runtime_tool_sources".into()),
                }
            } else if runtime_tool_is_optional_host_integration(&tool) {
                DoctorRuntimeDoctorFinding {
                    status: "info".into(),
                    message: format!("Optional host runtime tool unavailable: {name}"),
                    details: Some(format!(
                        "Missing optional command(s): {}\nInstall the command on the host if you use this integration.\nSearched PATH entries:\n{}",
                        missing_commands.join(", "),
                        format_path_list(command_search_paths)
                    )),
                    fix_available: false,
                    fix_action: None,
                    capability_tier: None,
                    capability_mode: Some("optional_host_runtime_tool_missing".into()),
                    runtime_contract_check: Some(format!("host_runtime_tool:{name}")),
                    owner_surface: Some("runtime_tool_sources".into()),
                }
            } else {
                DoctorRuntimeDoctorFinding {
                    status: "warning".into(),
                    message: format!("Host runtime tool missing: {name}"),
                    details: Some(format!(
                        "Missing required command(s): {}\nSearched PATH entries:\n{}",
                        missing_commands.join(", "),
                        format_path_list(command_search_paths)
                    )),
                    fix_available: false,
                    fix_action: None,
                    capability_tier: None,
                    capability_mode: None,
                    runtime_contract_check: Some(format!("host_runtime_tool:{name}")),
                    owner_surface: Some("runtime_tool_sources".into()),
                }
            }
        })
        .collect()
}

fn build_runtime_tool_source_findings(
    runtime_dir: &Path,
    command_search_paths: &[PathBuf],
) -> Vec<DoctorRuntimeDoctorFinding> {
    let mut findings = build_host_runtime_tool_findings(runtime_dir, command_search_paths);
    findings.extend(build_disabled_runtime_tool_findings(runtime_dir));
    findings
}

fn build_disabled_runtime_tool_findings(runtime_dir: &Path) -> Vec<DoctorRuntimeDoctorFinding> {
    let manifest = match read_runtime_tool_manifest(runtime_dir) {
        Ok(Some(manifest)) => manifest,
        Ok(None) => return Vec::new(),
        Err(_) => return Vec::new(),
    };

    manifest
        .into_iter()
        .filter(|(_, tool)| tool.source == "off")
        .map(|(name, tool)| DoctorRuntimeDoctorFinding {
            status: "info".into(),
            message: format!("Runtime tool disabled: {name}"),
            details: Some(format!(
                "Yazelix intentionally omitted command(s): {}",
                tool.commands.join(", ")
            )),
            fix_available: false,
            fix_action: None,
            capability_tier: None,
            capability_mode: Some("off".into()),
            runtime_contract_check: Some(format!("disabled_runtime_tool:{name}")),
            owner_surface: Some("runtime_tool_sources".into()),
        })
        .collect()
}

fn build_disabled_runtime_component_findings(
    runtime_dir: &Path,
) -> Vec<DoctorRuntimeDoctorFinding> {
    let manifest = match read_runtime_component_manifest(runtime_dir) {
        Ok(manifest) => manifest,
        Err(_) => return Vec::new(),
    };

    manifest
        .into_iter()
        .filter(|(_, component)| !component.enabled)
        .map(|(name, _)| DoctorRuntimeDoctorFinding {
            status: "info".into(),
            message: format!("Runtime component disabled: {name}"),
            details: Some(format!(
                "Yazelix intentionally omitted or bypassed the {name} runtime component."
            )),
            fix_available: false,
            fix_action: None,
            capability_tier: None,
            capability_mode: Some("off".into()),
            runtime_contract_check: Some(format!("disabled_runtime_component:{name}")),
            owner_surface: Some("components".into()),
        })
        .collect()
}

fn shell_initializers_root(state_dir: &Path) -> PathBuf {
    state_dir.join("initializers")
}

fn collect_initializer_files(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    collect_initializer_files_into(root, &mut files);
    files.sort();
    files
}

fn collect_initializer_files_into(path: &Path, files: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(path) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_initializer_files_into(&path, files);
        } else if path.is_file() {
            files.push(path);
        }
    }
}

fn trim_path_token(token: &str) -> &str {
    token.trim_matches(|ch: char| {
        matches!(
            ch,
            '"' | '\'' | '`' | ',' | ';' | ':' | ')' | '(' | ']' | '[' | '}' | '{' | '<' | '>'
        )
    })
}

fn extract_absolute_path_tokens(raw: &str) -> Vec<String> {
    raw.split(|ch: char| {
        ch.is_whitespace()
            || matches!(
                ch,
                '"' | '\'' | '`' | '$' | '(' | ')' | '[' | ']' | '{' | '}' | '<' | '>'
            )
    })
    .map(trim_path_token)
    .filter(|token| token.starts_with('/'))
    .map(ToString::to_string)
    .collect()
}

fn looks_like_transient_runtime_reference(path: &str) -> bool {
    path.split('/')
        .any(|component| component == "result" || component.starts_with("result_"))
}

fn deleted_transient_paths_in_initializer(path: &Path) -> Vec<String> {
    let Ok(raw) = fs::read_to_string(path) else {
        return Vec::new();
    };
    let mut out = extract_absolute_path_tokens(&raw)
        .into_iter()
        .filter(|candidate| looks_like_transient_runtime_reference(candidate))
        .filter(|candidate| !Path::new(candidate).exists())
        .collect::<Vec<_>>();
    out.sort();
    out.dedup();
    out
}

fn build_shell_initializer_findings(state_dir: &Path) -> Vec<DoctorRuntimeDoctorFinding> {
    let root = shell_initializers_root(state_dir);
    if !root.exists() {
        return vec![DoctorRuntimeDoctorFinding {
            status: "info".into(),
            message: "Shell initializers have not been generated yet".into(),
            details: Some(format!(
                "Expected generated initializers under {}. They are created by `yzx enter` and by Home Manager activation.",
                root.display()
            )),
            fix_available: false,
            fix_action: None,
            capability_tier: None,
            capability_mode: Some("missing_generated_initializers".into()),
            runtime_contract_check: Some("shell_initializer_stale_paths".into()),
            owner_surface: Some("shell_initializers".into()),
        }];
    }

    let stale = collect_initializer_files(&root)
        .into_iter()
        .flat_map(|file| {
            deleted_transient_paths_in_initializer(&file)
                .into_iter()
                .map(move |path| format!("{} -> {path}", file.display()))
        })
        .collect::<Vec<_>>();

    if stale.is_empty() {
        return vec![DoctorRuntimeDoctorFinding {
            status: "ok".into(),
            message: "Shell initializers do not reference deleted transient runtime paths".into(),
            details: Some(format!(
                "Checked generated initializers under {}",
                root.display()
            )),
            fix_available: false,
            fix_action: None,
            capability_tier: None,
            capability_mode: Some("generated_initializers_current".into()),
            runtime_contract_check: Some("shell_initializer_stale_paths".into()),
            owner_surface: Some("shell_initializers".into()),
        }];
    }

    vec![DoctorRuntimeDoctorFinding {
        status: "warning".into(),
        message: "Shell initializers reference deleted transient runtime paths".into(),
        details: Some(format!(
            "{}\nRepair: run `yzx_control generate_shell_initializers` from the active profile runtime, or reapply Home Manager so activation regenerates them.",
            stale.join("\n")
        )),
        fix_available: false,
        fix_action: None,
        capability_tier: None,
        capability_mode: Some("stale_generated_initializers".into()),
        runtime_contract_check: Some("shell_initializer_stale_paths".into()),
        owner_surface: Some("shell_initializers".into()),
    }]
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct LaunchLogSummary {
    path: PathBuf,
    modified_seconds: u64,
    status: LaunchLogStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LaunchLogStatus {
    LifetimeCaptured,
    LifetimeWatching,
    MetadataOnly,
    LegacyOrMissingMetadata,
}

impl LaunchLogStatus {
    fn label(self) -> &'static str {
        match self {
            Self::LifetimeCaptured => "lifetime_captured",
            Self::LifetimeWatching => "lifetime_watching",
            Self::MetadataOnly => "metadata_only",
            Self::LegacyOrMissingMetadata => "legacy_or_missing_metadata",
        }
    }
}

fn yzxterm_launch_log_dir(state_dir: &Path) -> PathBuf {
    state_dir.join("logs").join("terminal_launch")
}

fn runtime_variant_path(runtime_dir: &Path) -> PathBuf {
    runtime_dir.join("runtime_variant")
}

fn runtime_variant_is_yzxterm(runtime_dir: &Path) -> bool {
    fs::read_to_string(runtime_variant_path(runtime_dir))
        .map(|raw| raw.trim() == "yzxterm")
        .unwrap_or(false)
}

fn is_yzxterm_launch_log(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.starts_with("yazelix_terminal_desktop_") && name.ends_with(".log"))
}

fn launch_log_modified_seconds(path: &Path) -> u64 {
    fs::metadata(path)
        .and_then(|metadata| metadata.modified())
        .ok()
        .and_then(|modified| modified.duration_since(UNIX_EPOCH).ok())
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

fn classify_launch_log(path: &Path) -> LaunchLogStatus {
    let Ok(raw) = fs::read_to_string(path) else {
        return LaunchLogStatus::LegacyOrMissingMetadata;
    };

    let has_metadata = raw.contains("desktop deferred launch")
        && raw.contains("argv:")
        && raw.contains("terminal_or_wrapper_pid");
    if !has_metadata {
        return LaunchLogStatus::LegacyOrMissingMetadata;
    }

    if raw.contains("final_exit_status=") && raw.contains("final_exit_kind=") {
        return LaunchLogStatus::LifetimeCaptured;
    }

    if raw.contains("lifetime_status=watching") {
        return LaunchLogStatus::LifetimeWatching;
    }

    LaunchLogStatus::MetadataOnly
}

fn collect_yzxterm_launch_logs(state_dir: &Path) -> Vec<LaunchLogSummary> {
    let log_dir = yzxterm_launch_log_dir(state_dir);
    let mut logs = fs::read_dir(log_dir)
        .ok()
        .into_iter()
        .flatten()
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.is_file() && is_yzxterm_launch_log(path))
        .map(|path| LaunchLogSummary {
            modified_seconds: launch_log_modified_seconds(&path),
            status: classify_launch_log(&path),
            path,
        })
        .collect::<Vec<_>>();
    logs.sort_by(|left, right| {
        right
            .modified_seconds
            .cmp(&left.modified_seconds)
            .then_with(|| left.path.cmp(&right.path))
    });
    logs
}

fn render_launch_log_summaries(logs: &[LaunchLogSummary]) -> String {
    logs.iter()
        .take(5)
        .map(|log| {
            format!(
                "{} (modified_unix_seconds={}, evidence={})",
                log.path.display(),
                log.modified_seconds,
                log.status.label()
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn build_yzxterm_launch_log_findings(
    state_dir: &Path,
    runtime_dir: &Path,
) -> Vec<DoctorRuntimeDoctorFinding> {
    let log_dir = yzxterm_launch_log_dir(state_dir);
    let logs = collect_yzxterm_launch_logs(state_dir);
    if logs.is_empty() {
        if !runtime_variant_is_yzxterm(runtime_dir) {
            return Vec::new();
        }

        return vec![DoctorRuntimeDoctorFinding {
            status: "info".into(),
            message: "Yazelix Terminal desktop launch logs have not been captured yet".into(),
            details: Some(format!(
                "Expected yzxterm desktop launch logs under {} after launching the yzxterm runtime from a desktop entry.",
                log_dir.display()
            )),
            fix_available: false,
            fix_action: None,
            capability_tier: None,
            capability_mode: Some("no_yzxterm_launch_logs".into()),
            runtime_contract_check: Some("yzxterm_launch_logs".into()),
            owner_surface: Some("terminal_launch_logs".into()),
        }];
    }

    let details = render_launch_log_summaries(&logs);
    if logs
        .iter()
        .any(|log| log.status == LaunchLogStatus::LifetimeCaptured)
    {
        return vec![DoctorRuntimeDoctorFinding {
            status: "ok".into(),
            message: "Yazelix Terminal desktop launch lifetime evidence is available".into(),
            details: Some(details),
            fix_available: false,
            fix_action: None,
            capability_tier: None,
            capability_mode: Some("yzxterm_launch_lifetime_captured".into()),
            runtime_contract_check: Some("yzxterm_launch_logs".into()),
            owner_surface: Some("terminal_launch_logs".into()),
        }];
    }

    if logs
        .iter()
        .any(|log| log.status == LaunchLogStatus::LifetimeWatching)
    {
        return vec![DoctorRuntimeDoctorFinding {
            status: "ok".into(),
            message: "Yazelix Terminal desktop launch lifetime watcher is active".into(),
            details: Some(details),
            fix_available: false,
            fix_action: None,
            capability_tier: None,
            capability_mode: Some("yzxterm_launch_lifetime_watching".into()),
            runtime_contract_check: Some("yzxterm_launch_logs".into()),
            owner_surface: Some("terminal_launch_logs".into()),
        }];
    }

    if logs
        .iter()
        .any(|log| log.status == LaunchLogStatus::MetadataOnly)
    {
        return vec![DoctorRuntimeDoctorFinding {
            status: "warning".into(),
            message: "Yazelix Terminal desktop launch logs lack lifetime evidence".into(),
            details: Some(format!(
                "{details}\nRelaunch yzxterm from the desktop entry to start the lifetime watcher; metadata-only logs cannot prove final exit status or signal."
            )),
            fix_available: false,
            fix_action: None,
            capability_tier: None,
            capability_mode: Some("yzxterm_launch_logs_metadata_only".into()),
            runtime_contract_check: Some("yzxterm_launch_logs".into()),
            owner_surface: Some("terminal_launch_logs".into()),
        }];
    }

    vec![DoctorRuntimeDoctorFinding {
        status: "warning".into(),
        message: "Yazelix Terminal desktop launch logs are stale or missing metadata".into(),
        details: Some(format!(
            "{details}\nRelaunch yzxterm from the desktop entry to capture argv, config environment, terminal PID, and early exit status."
        )),
        fix_available: false,
        fix_action: None,
        capability_tier: None,
        capability_mode: Some("yzxterm_launch_logs_legacy".into()),
        runtime_contract_check: Some("yzxterm_launch_logs".into()),
        owner_surface: Some("terminal_launch_logs".into()),
    }]
}

fn is_package_runtime_root(runtime_dir: &Path) -> bool {
    runtime_dir.join(DEFAULT_SETTINGS_CONFIG_FILENAME).exists()
        && runtime_dir.join("bin").join("yzx").exists()
        && runtime_dir.join("libexec").join("nu").exists()
}

fn distribution_finding(
    runtime_dir: &Path,
    has_home_manager_managed_install: bool,
    is_manual_runtime_reference_path: bool,
) -> DoctorRuntimeDoctorFinding {
    let (mode, tier, message, details) = if has_home_manager_managed_install {
        (
            "home_manager_managed",
            "full",
            "Runtime/distribution capability: Home Manager-managed full runtime",
            "Home Manager owns the packaged Yazelix runtime path and update transition in this mode.",
        )
    } else if is_manual_runtime_reference_path {
        (
            "installer_managed",
            "full",
            "Runtime/distribution capability: compatibility installer runtime",
            "This runtime still has legacy installer-owned artifacts from older releases. Current Yazelix no longer ships `#install`; reinstall into a Nix profile or move to Home Manager.",
        )
    } else if is_package_runtime_root(runtime_dir) {
        (
            "package_runtime",
            "narrowed",
            "Runtime/distribution capability: store/package runtime",
            "This Yazelix runtime runs directly from a packaged runtime root.",
        )
    } else {
        (
            "runtime_root_only",
            "narrowed",
            "Runtime/distribution capability: runtime-root-only mode",
            "This Yazelix session has a runtime root but no package-manager-owned distribution surface.",
        )
    };

    DoctorRuntimeDoctorFinding {
        status: "info".into(),
        message: message.into(),
        details: Some(details.into()),
        fix_available: false,
        fix_action: None,
        capability_tier: Some(tier.into()),
        capability_mode: Some(mode.into()),
        runtime_contract_check: None,
        owner_surface: None,
    }
}

fn normalize_failure_class(class: &str) -> String {
    match class.trim().to_lowercase().as_str() {
        "config" => "config problem".into(),
        "generated-state" => "generated-state problem".into(),
        "host-dependency" => "host-dependency problem".into(),
        _ => "problem".into(),
    }
}

fn format_failure_classification(failure_class: &str, recovery_hint: &str) -> String {
    let label = normalize_failure_class(failure_class);
    format!("Failure class: {label}.\nRecovery: {recovery_hint}")
}

fn build_runtime_check_detail_lines(check: &RuntimeCheckData) -> Vec<String> {
    let mut lines = Vec::new();
    if let Some(d) = &check.details {
        let t = d.trim();
        if !t.is_empty() {
            lines.push(d.clone());
        }
    }
    let recovery = check.recovery.as_deref().unwrap_or("").trim();
    let fc = check.failure_class.as_deref().unwrap_or("").trim();
    if !recovery.is_empty() && !fc.is_empty() {
        lines.push(format_failure_classification(fc, recovery));
    } else if !recovery.is_empty() {
        lines.push(recovery.to_string());
    }
    lines
}

fn runtime_check_to_doctor_finding(
    check: &RuntimeCheckData,
    managed_layouts_root: &Path,
) -> DoctorRuntimeDoctorFinding {
    let detail_lines = build_runtime_check_detail_lines(check);
    let details = if detail_lines.is_empty() {
        None
    } else {
        Some(detail_lines.join("\n"))
    };

    let status = if check.status == "ok" {
        "ok".to_string()
    } else {
        check.severity.clone()
    };

    let mut finding = DoctorRuntimeDoctorFinding {
        status,
        message: check.message.clone(),
        details,
        fix_available: false,
        fix_action: None,
        capability_tier: None,
        capability_mode: None,
        runtime_contract_check: Some(check.id.clone()),
        owner_surface: Some(check.owner_surface.clone()),
    };

    if check.id == "generated_layout"
        && check.status != "ok"
        && check.failure_class.as_deref() == Some("generated-state")
    {
        if let Some(ref p) = check.path {
            if is_managed_generated_layout_path(p, managed_layouts_root) {
                finding.fix_available = true;
                finding.fix_action = Some("repair_generated_runtime_state".into());
            }
        }
    }

    finding
}

fn is_managed_generated_layout_path(layout_path: &str, managed_dir: &Path) -> bool {
    let layout = Path::new(layout_path);
    if let (Ok(can_layout), Ok(can_root)) = (layout.canonicalize(), managed_dir.canonicalize()) {
        return can_layout.starts_with(&can_root);
    }
    let root_s = managed_dir.to_string_lossy();
    let root_norm = root_s.trim_end_matches('/');
    layout_path == root_norm || layout_path.starts_with(&format!("{root_norm}/"))
}

fn build_contract_request(
    shared: &SharedRuntimePreflightInput,
    runtime_dir: &Path,
) -> RuntimeContractEvaluateRequest {
    let runtime_script_requests: Vec<RuntimeScriptCheckRequest> = vec![
        RuntimeScriptCheckRequest {
            id: "startup_runtime_script".into(),
            label: "startup script".into(),
            owner_surface: "doctor".into(),
            path: shared.startup_script_path.clone(),
        },
        RuntimeScriptCheckRequest {
            id: "launch_runtime_script".into(),
            label: "launch script".into(),
            owner_surface: "doctor".into(),
            path: shared.launch_script_path.clone(),
        },
    ];

    RuntimeContractEvaluateRequest {
        working_dir: None,
        runtime_scripts: runtime_script_requests,
        generated_layout: Some(GeneratedLayoutCheckRequest {
            owner_surface: "doctor".into(),
            path: shared.zellij_layout_path.clone(),
        }),
        terminal_support: Some(TerminalSupportCheckRequest {
            owner_surface: "launch".into(),
            requested_terminal: String::new(),
            terminals: shared.terminals.clone(),
            command_search_paths: shared.command_search_paths.clone(),
        }),
        linux_ghostty_desktop_graphics_support: Some(LinuxGhosttyDesktopGraphicsRequest {
            owner_surface: "doctor".into(),
            terminals: shared.terminals.clone(),
            runtime_dir: Some(runtime_dir.to_path_buf()),
            command_search_paths: shared.command_search_paths.clone(),
            platform_name: Some(shared.platform_name.clone()),
        }),
    }
}

fn build_shared_preflight_findings(
    shared: &SharedRuntimePreflightInput,
    managed_layouts_root: &Path,
    runtime_dir: &Path,
) -> Result<Vec<DoctorRuntimeDoctorFinding>, CoreError> {
    let contract_req = build_contract_request(shared, runtime_dir);
    let data = evaluate_runtime_contract(&contract_req)?;
    Ok(data
        .checks
        .iter()
        .map(|c| runtime_check_to_doctor_finding(c, managed_layouts_root))
        .collect())
}

fn build_runtime_graphics_findings(
    shared: &SharedRuntimePreflightInput,
    runtime_dir: &Path,
    command_search_paths: &[PathBuf],
) -> Vec<DoctorRuntimeDoctorFinding> {
    vec![
        build_graphics_preview_strategy_finding(shared, runtime_dir, command_search_paths),
        build_chafa_probe_safety_finding(runtime_dir),
    ]
}

fn build_host_rio_env_isolation_finding() -> Option<DoctorRuntimeDoctorFinding> {
    build_host_rio_env_isolation_finding_from_values(
        env::var_os("RIO_CONFIG_HOME"),
        env::var_os("YAZELIX_TERMINAL_CONFIG"),
        env::var_os("YAZELIX_TERMINAL_LD_LIBRARY_PATH_PREFIX"),
        env::var_os("YAZELIX_TERMINAL_HOST_LD_LIBRARY_PATH"),
    )
}

fn build_host_rio_env_isolation_finding_from_values(
    rio_config_home: Option<OsString>,
    yazelix_terminal_config: Option<OsString>,
    loader_prefix: Option<OsString>,
    host_loader_snapshot: Option<OsString>,
) -> Option<DoctorRuntimeDoctorFinding> {
    let mut details = Vec::new();

    if let Some(value) = rio_config_home {
        let value = value.to_string_lossy();
        if value.contains("terminal_emulators/yzxterm") || yazelix_terminal_config.is_some() {
            details.push(format!("RIO_CONFIG_HOME={value}"));
        }
    }
    if let Some(value) = yazelix_terminal_config {
        details.push(format!(
            "YAZELIX_TERMINAL_CONFIG={}",
            value.to_string_lossy()
        ));
    }
    if loader_prefix.is_some() || host_loader_snapshot.is_some() {
        details.push("Yazelix Terminal package loader environment is present in the shell.".into());
    }

    if details.is_empty() {
        return None;
    }

    Some(DoctorRuntimeDoctorFinding {
        status: "warning".into(),
        message: "Host Rio environment may be contaminated by Yazelix Terminal launch state".into(),
        details: Some(details.join("\n")),
        fix_available: false,
        fix_action: None,
        capability_tier: None,
        capability_mode: Some("host_rio_env_contamination".into()),
        runtime_contract_check: Some("host_rio_env_isolation".into()),
        owner_surface: Some("terminal_runtime_env".into()),
    })
}

fn runtime_feature_path(runtime_dir: &Path, feature: &str) -> PathBuf {
    runtime_dir.join("runtime_features").join(feature)
}

fn runtime_feature_enabled(runtime_dir: &Path, feature: &str) -> bool {
    runtime_feature_path(runtime_dir, feature).exists()
}

fn runtime_command_present(runtime_dir: &Path, command: &str) -> bool {
    runtime_dir.join("libexec").join(command).is_file()
        || runtime_dir.join("toolbin").join(command).is_file()
}

fn first_configured_terminal(terminals: &[String]) -> String {
    terminals
        .iter()
        .map(|terminal| terminal.trim())
        .find(|terminal| !terminal.is_empty())
        .unwrap_or("unknown")
        .to_string()
}

fn terminal_uses_yazelix_kitty_bridge(terminal: &str) -> bool {
    matches!(terminal, "ghostty" | "kitty" | "yzxterm" | "ratty")
}

fn host_runtime_yazi_available(runtime_dir: &Path, command_search_paths: &[PathBuf]) -> bool {
    let manifest = match read_runtime_tool_manifest(runtime_dir) {
        Ok(Some(manifest)) => manifest,
        Ok(None) | Err(_) => return false,
    };
    let Some(tool) = manifest.get("yazi") else {
        return false;
    };
    if tool.source != "host" {
        return false;
    }

    let required_commands = runtime_tool_required_commands(tool);
    !required_commands.is_empty()
        && required_commands
            .iter()
            .all(|command| command_exists_in_paths(command, command_search_paths))
}

fn build_graphics_preview_strategy_finding(
    shared: &SharedRuntimePreflightInput,
    runtime_dir: &Path,
    command_search_paths: &[PathBuf],
) -> DoctorRuntimeDoctorFinding {
    let terminal = first_configured_terminal(&shared.terminals);
    let bridge_marker = runtime_feature_enabled(runtime_dir, ZELLIJ_KITTY_PASSTHROUGH_FEATURE);
    let zellij_present = runtime_command_present(runtime_dir, "zellij");
    let yazi_present = runtime_command_present(runtime_dir, "yazi");
    let host_yazi_present = host_runtime_yazi_available(runtime_dir, command_search_paths);
    let details = vec![
        format!("First configured terminal: {terminal}"),
        format!(
            "Runtime Kitty passthrough marker: {}",
            if bridge_marker { "present" } else { "missing" }
        ),
        format!(
            "Runtime Zellij command: {}",
            if zellij_present {
                "present"
            } else {
                "missing"
            }
        ),
        format!(
            "Runtime Yazi command: {}",
            if yazi_present {
                "present"
            } else if host_yazi_present {
                "host PATH"
            } else {
                "missing"
            }
        ),
        "Chafa probing is reported separately and is not treated as proof of high-quality image-preview support.".into(),
    ];

    if terminal_uses_yazelix_kitty_bridge(&terminal) {
        if bridge_marker && zellij_present && yazi_present {
            return DoctorRuntimeDoctorFinding {
                status: "ok".into(),
                message: "Graphics previews: Yazelix Kitty passthrough bridge is active".into(),
                details: Some(format!(
                    "{}\nPreview strategy: Yazi image previews use Kitty graphics through the packaged Zellij/Yazi bridge.",
                    details.join("\n")
                )),
                fix_available: false,
                fix_action: None,
                capability_tier: Some("full".into()),
                capability_mode: Some("kitty_passthrough_bridge".into()),
                runtime_contract_check: Some("runtime_graphics_preview_strategy".into()),
                owner_surface: Some("runtime_graphics".into()),
            };
        }

        if bridge_marker && zellij_present && host_yazi_present {
            return DoctorRuntimeDoctorFinding {
                status: "warning".into(),
                message: "Graphics previews: host-sourced Yazi is available, but bundled Yazelix Yazi is the fully supported Kitty bridge path".into(),
                details: Some(format!(
                    "{}\nPreview strategy: Ghostty/Ratty can use the packaged Zellij bridge, but host-sourced Yazi is not treated as equivalent to bundled Yazelix KGP Yazi without a capability check.",
                    details.join("\n")
                )),
                fix_available: false,
                fix_action: None,
                capability_tier: Some("degraded".into()),
                capability_mode: Some("kitty_passthrough_bridge_host_yazi".into()),
                runtime_contract_check: Some("runtime_graphics_preview_strategy".into()),
                owner_surface: Some("runtime_graphics".into()),
            };
        }

        return DoctorRuntimeDoctorFinding {
            status: "warning".into(),
            message:
                "Graphics previews: configured terminal expects the Yazelix Kitty bridge, but the runtime is incomplete"
                    .into(),
            details: Some(format!(
                "{}\nExpected combination: Ghostty or Ratty plus runtime-owned Zellij/Yazi Kitty passthrough marker.",
                details.join("\n")
            )),
            fix_available: false,
            fix_action: None,
            capability_tier: Some("degraded".into()),
            capability_mode: Some("kitty_passthrough_bridge_incomplete".into()),
            runtime_contract_check: Some("runtime_graphics_preview_strategy".into()),
            owner_surface: Some("runtime_graphics".into()),
        };
    }

    DoctorRuntimeDoctorFinding {
        status: "info".into(),
        message: "Graphics previews: no Yazelix Kitty passthrough bridge is expected".into(),
        details: Some(format!(
            "{}\nPreview strategy: this diagnostic only validates the Ghostty/Ratty Kitty bridge path. Other terminal preview behavior is terminal-native or outside the current Yazelix graphics contract.",
            details.join("\n")
        )),
        fix_available: false,
        fix_action: None,
        capability_tier: Some("unknown".into()),
        capability_mode: Some("terminal_native_or_unmanaged".into()),
        runtime_contract_check: Some("runtime_graphics_preview_strategy".into()),
        owner_surface: Some("runtime_graphics".into()),
    }
}

fn build_chafa_probe_safety_finding(runtime_dir: &Path) -> DoctorRuntimeDoctorFinding {
    if runtime_feature_enabled(runtime_dir, CHAFA_PROBE_UNSAFE_FEATURE) {
        return DoctorRuntimeDoctorFinding {
            status: "warning".into(),
            message: "Chafa probe safety: unsafe probe marker present".into(),
            details: Some(
                "This is separate from Kitty graphics capability. Do not use Chafa probing as proof that Zellij/Yazi image previews are supported in this runtime."
                    .into(),
            ),
            fix_available: false,
            fix_action: None,
            capability_tier: None,
            capability_mode: Some("chafa_probe_unsafe".into()),
            runtime_contract_check: Some("runtime_chafa_probe_safety".into()),
            owner_surface: Some("runtime_graphics".into()),
        };
    }

    if runtime_feature_enabled(runtime_dir, CHAFA_PROBE_SAFE_FEATURE) {
        return DoctorRuntimeDoctorFinding {
            status: "ok".into(),
            message: "Chafa probe safety: safe probe marker present".into(),
            details: Some(
                "This only means Chafa probing is safe to run. Kitty graphics bridge support is reported separately."
                    .into(),
            ),
            fix_available: false,
            fix_action: None,
            capability_tier: None,
            capability_mode: Some("chafa_probe_safe".into()),
            runtime_contract_check: Some("runtime_chafa_probe_safety".into()),
            owner_surface: Some("runtime_graphics".into()),
        };
    }

    DoctorRuntimeDoctorFinding {
        status: "info".into(),
        message: "Chafa probe safety: no Chafa probe is active".into(),
        details: Some(
            "Yazelix is not using a Chafa ghost-keypress probe for this runtime. Kitty graphics support is determined from the terminal and runtime bridge markers instead."
                .into(),
        ),
        fix_available: false,
        fix_action: None,
        capability_tier: None,
        capability_mode: Some("chafa_probe_not_used".into()),
        runtime_contract_check: Some("runtime_chafa_probe_safety".into()),
        owner_surface: Some("runtime_graphics".into()),
    }
}

// Test lane: default

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn write_runtime_command(runtime: &Path, command: &str) {
        let path = runtime.join("libexec").join(command);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, "").unwrap();
    }

    fn write_path_command(path_dir: &Path, command: &str) {
        let path = path_dir.join(command);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, "").unwrap();
        make_executable(&path);
    }

    #[cfg(unix)]
    fn make_executable(path: &Path) {
        use std::os::unix::fs::PermissionsExt;

        let mut permissions = std::fs::metadata(path).unwrap().permissions();
        permissions.set_mode(permissions.mode() | 0o755);
        std::fs::set_permissions(path, permissions).unwrap();
    }

    #[cfg(not(unix))]
    fn make_executable(_path: &Path) {}

    fn shared_with_terminal(terminal: &str) -> SharedRuntimePreflightInput {
        SharedRuntimePreflightInput {
            zellij_layout_path: PathBuf::from("layout.kdl"),
            terminals: vec![terminal.to_string()],
            startup_script_path: PathBuf::from("startup.nu"),
            launch_script_path: PathBuf::from("launch.sh"),
            command_search_paths: Vec::new(),
            platform_name: "linux".to_string(),
        }
    }

    // Regression: generated shell initializers must expose deleted local result-runtime references before user shells fail during startup.
    #[test]
    fn shell_initializer_finding_warns_on_deleted_transient_runtime_path() {
        let tmp = TempDir::new().unwrap();
        let state = tmp.path().join("state");
        let init = state
            .join("initializers")
            .join("nushell")
            .join("yazelix_init.nu");
        std::fs::create_dir_all(init.parent().unwrap()).unwrap();
        std::fs::write(
            &init,
            "source /home/demo/pjs/yazelix/result_yazelix_runtime/libexec/mise\n",
        )
        .unwrap();

        let findings = build_shell_initializer_findings(&state);

        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].status, "warning");
        assert_eq!(
            findings[0].runtime_contract_check.as_deref(),
            Some("shell_initializer_stale_paths")
        );
        let details = findings[0].details.as_deref().unwrap_or_default();
        assert!(details.contains("result_yazelix_runtime"));
        assert!(details.contains("yzx_control generate_shell_initializers"));
    }

    // Defends: doctor can point users at concrete yzxterm desktop lifetime evidence after a window disappears.
    #[test]
    fn yzxterm_launch_log_finding_reports_lifetime_logs() {
        let tmp = TempDir::new().unwrap();
        let state = tmp.path().join("state");
        let runtime = tmp.path().join("runtime");
        let log = state
            .join("logs")
            .join("terminal_launch")
            .join("yazelix_terminal_desktop_123.log");
        std::fs::create_dir_all(log.parent().unwrap()).unwrap();
        std::fs::write(
            &log,
            "[2026-06-02T00:00:00-0300] desktop deferred launch\nargv:\n  yazelix-terminal-desktop\n[2026-06-02T00:00:01-0300] spawned terminal_or_wrapper_pid=123\n[2026-06-02T00:00:02-0300] final_exit_status=0\nfinal_exit_kind=exit\nfinal_exit_code=0\n",
        )
        .unwrap();

        let findings = build_yzxterm_launch_log_findings(&state, &runtime);

        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].status, "ok");
        assert_eq!(
            findings[0].runtime_contract_check.as_deref(),
            Some("yzxterm_launch_logs")
        );
        assert_eq!(
            findings[0].capability_mode.as_deref(),
            Some("yzxterm_launch_lifetime_captured")
        );
        assert!(
            findings[0]
                .details
                .as_deref()
                .unwrap_or_default()
                .contains("evidence=lifetime_captured")
        );
    }

    // Regression: old short-probe metadata is not enough to diagnose a vanished yzxterm window.
    #[test]
    fn yzxterm_launch_log_finding_warns_on_metadata_only_logs() {
        let tmp = TempDir::new().unwrap();
        let state = tmp.path().join("state");
        let runtime = tmp.path().join("runtime");
        let log = state
            .join("logs")
            .join("terminal_launch")
            .join("yazelix_terminal_desktop_123.log");
        std::fs::create_dir_all(log.parent().unwrap()).unwrap();
        std::fs::write(
            &log,
            "[2026-06-02T00:00:00-0300] desktop deferred launch\nargv:\n  yazelix-terminal-desktop\n[2026-06-02T00:00:01-0300] spawned terminal_or_wrapper_pid=123\n[2026-06-02T00:00:02-0300] exit_status=not_observed_after_probe_window\n",
        )
        .unwrap();

        let findings = build_yzxterm_launch_log_findings(&state, &runtime);

        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].status, "warning");
        assert_eq!(
            findings[0].capability_mode.as_deref(),
            Some("yzxterm_launch_logs_metadata_only")
        );
        let details = findings[0].details.as_deref().unwrap_or_default();
        assert!(details.contains("evidence=metadata_only"));
        assert!(details.contains("cannot prove final exit status or signal"));
    }

    // Defends: non-yzxterm runtimes do not receive irrelevant missing-log doctor findings.
    #[test]
    fn yzxterm_launch_log_finding_is_scoped_to_yzxterm_runtime() {
        let tmp = TempDir::new().unwrap();
        let state = tmp.path().join("state");
        let runtime = tmp.path().join("runtime");
        std::fs::create_dir_all(&runtime).unwrap();
        std::fs::write(runtime.join("runtime_variant"), "ghostty\n").unwrap();

        let findings = build_yzxterm_launch_log_findings(&state, &runtime);

        assert!(findings.is_empty());
    }

    // Defends: doctor reports when a Yazelix shell would make plain host Rio read the generated yzxterm config.
    #[test]
    fn host_rio_env_isolation_reports_yzxterm_contamination() {
        let finding = build_host_rio_env_isolation_finding_from_values(
            Some("/home/user/.local/share/yazelix/configs/terminal_emulators/yzxterm".into()),
            Some("/home/user/.local/share/yazelix/configs/terminal_emulators/yzxterm".into()),
            None,
            None,
        )
        .unwrap();

        assert_eq!(finding.status, "warning");
        assert_eq!(
            finding.runtime_contract_check.as_deref(),
            Some("host_rio_env_isolation")
        );
        assert!(
            finding
                .details
                .as_deref()
                .unwrap()
                .contains("terminal_emulators/yzxterm")
        );
    }

    // Defends: doctor runtime distribution reporting still prefers Home Manager ownership over generic package shape.
    #[test]
    fn distribution_prefers_home_manager_over_package_shape() {
        let tmp = TempDir::new().unwrap();
        let rt = tmp.path().join("rt");
        std::fs::create_dir_all(rt.join("bin")).unwrap();
        std::fs::write(rt.join("settings_default.jsonc"), "").unwrap();
        std::fs::write(rt.join("bin").join("yzx"), "").unwrap();
        std::fs::create_dir_all(rt.join("libexec").join("nu")).unwrap();

        let f = distribution_finding(&rt, true, false);
        assert_eq!(f.capability_mode.as_deref(), Some("home_manager_managed"));
    }

    // Defends: missing managed layouts still surface the repair action in the doctor runtime report.
    #[test]
    fn managed_layout_sets_repair_fix_action() {
        let tmp = TempDir::new().unwrap();
        let layouts = tmp.path().join("layouts");
        std::fs::create_dir_all(&layouts).unwrap();
        let layout_file = layouts.join("x.kdl");
        std::fs::write(&layout_file, "").unwrap();

        let check = RuntimeCheckData {
            id: "generated_layout".into(),
            status: "missing".into(),
            severity: "error".into(),
            owner_surface: "doctor".into(),
            message: "test".into(),
            details: None,
            recovery: None,
            failure_class: Some("generated-state".into()),
            blocking: true,
            path: Some(layout_file.to_string_lossy().into_owned()),
            candidates: None,
        };

        let f = runtime_check_to_doctor_finding(&check, &layouts);
        assert!(f.fix_available);
        assert_eq!(
            f.fix_action.as_deref(),
            Some("repair_generated_runtime_state")
        );
    }

    // Regression: Home Manager host-sourced tools get an actionable doctor finding when PATH does not provide them.
    #[test]
    fn missing_host_runtime_tool_reports_warning() {
        let tmp = TempDir::new().unwrap();
        let runtime = tmp.path().join("runtime");
        let path_dir = tmp.path().join("empty_path");
        std::fs::create_dir_all(&runtime).unwrap();
        std::fs::create_dir_all(&path_dir).unwrap();
        std::fs::write(
            runtime.join("runtime_tools.json"),
            r#"{
              "lazygit": {
                "source": "host",
                "commands": ["lazygit", "lg"],
                "required_commands": ["lazygit"]
              }
            }"#,
        )
        .unwrap();

        let findings = build_host_runtime_tool_findings(&runtime, &[path_dir]);

        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].status, "warning");
        assert_eq!(findings[0].message, "Host runtime tool missing: lazygit");
        assert!(
            findings[0]
                .details
                .as_deref()
                .unwrap()
                .contains("Missing required command(s): lazygit")
        );
    }

    // Defends: default host-managed integrations do not turn doctor red when unused commands are absent.
    #[test]
    fn optional_host_runtime_tool_reports_info_when_missing() {
        let tmp = TempDir::new().unwrap();
        let runtime = tmp.path().join("runtime");
        let path_dir = tmp.path().join("empty_path");
        std::fs::create_dir_all(&runtime).unwrap();
        std::fs::create_dir_all(&path_dir).unwrap();
        std::fs::write(
            runtime.join("runtime_tools.json"),
            r#"{
              "mise": {
                "source": "host",
                "commands": ["mise"],
                "required_commands": ["mise"],
                "notes": ["optional_host_integration"]
              }
            }"#,
        )
        .unwrap();

        let findings = build_host_runtime_tool_findings(&runtime, &[path_dir]);

        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].status, "info");
        assert_eq!(
            findings[0].message,
            "Optional host runtime tool unavailable: mise"
        );
        assert!(
            findings[0]
                .details
                .as_deref()
                .unwrap()
                .contains("Install the command on the host if you use this integration")
        );
    }

    // Regression: host runtime commands must be executable, not merely present as files on PATH.
    #[test]
    fn host_runtime_tool_file_must_be_executable() {
        let tmp = TempDir::new().unwrap();
        let runtime = tmp.path().join("runtime");
        let path_dir = tmp.path().join("host_path");
        std::fs::create_dir_all(&runtime).unwrap();
        std::fs::create_dir_all(&path_dir).unwrap();
        std::fs::write(path_dir.join("lazygit"), "").unwrap();
        std::fs::write(
            runtime.join("runtime_tools.json"),
            r#"{
              "lazygit": {
                "source": "host",
                "commands": ["lazygit", "lg"],
                "required_commands": ["lazygit"]
              }
            }"#,
        )
        .unwrap();

        let findings = build_host_runtime_tool_findings(&runtime, &[path_dir]);

        assert_eq!(findings[0].status, "warning");
        assert!(
            findings[0]
                .details
                .as_deref()
                .unwrap()
                .contains("Missing required command(s): lazygit")
        );
    }

    // Defends: default bundled runtimes do not gain host-tool warnings from the runtime manifest.
    #[test]
    fn bundled_runtime_tool_manifest_produces_no_host_findings() {
        let tmp = TempDir::new().unwrap();
        let runtime = tmp.path().join("runtime");
        std::fs::create_dir_all(&runtime).unwrap();
        std::fs::write(
            runtime.join("runtime_tools.json"),
            r#"{
              "lazygit": {
                "source": "bundled",
                "commands": ["lazygit", "lg"],
                "required_commands": ["lazygit"]
              }
            }"#,
        )
        .unwrap();

        let findings = build_host_runtime_tool_findings(&runtime, &[]);

        assert!(findings.is_empty());
    }

    // Defends: runtime_tool_sources off mode is reported as intentional disablement instead of a missing host dependency.
    #[test]
    fn disabled_runtime_tool_reports_info_finding() {
        let tmp = TempDir::new().unwrap();
        let runtime = tmp.path().join("runtime");
        std::fs::create_dir_all(&runtime).unwrap();
        std::fs::write(
            runtime.join("runtime_tools.json"),
            r#"{
              "macchina": {
                "source": "off",
                "commands": ["macchina"],
                "required_commands": ["macchina"]
              }
            }"#,
        )
        .unwrap();

        let findings = build_runtime_tool_source_findings(&runtime, &[]);

        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].status, "info");
        assert_eq!(findings[0].message, "Runtime tool disabled: macchina");
        assert_eq!(
            findings[0].runtime_contract_check.as_deref(),
            Some("disabled_runtime_tool:macchina")
        );
    }

    // Defends: component toggles appear as intentional runtime capability changes in doctor output.
    #[test]
    fn disabled_runtime_component_reports_info_finding() {
        let tmp = TempDir::new().unwrap();
        let runtime = tmp.path().join("runtime");
        std::fs::create_dir_all(&runtime).unwrap();
        std::fs::write(
            runtime.join("runtime_components.json"),
            r#"{
              "cursors": { "enabled": false, "disableable": true, "notes": [] },
              "screen": { "enabled": true, "disableable": true, "notes": [] }
            }"#,
        )
        .unwrap();

        let findings = build_disabled_runtime_component_findings(&runtime);

        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].message, "Runtime component disabled: cursors");
        assert_eq!(
            findings[0].runtime_contract_check.as_deref(),
            Some("disabled_runtime_component:cursors")
        );
    }

    // Defends: runtime graphics diagnostics recognize the packaged Ghostty/Ratty Kitty bridge from runtime-owned markers instead of terminal name alone.
    #[test]
    fn graphics_strategy_reports_active_kitty_bridge_for_ghostty_runtime() {
        let tmp = TempDir::new().unwrap();
        let runtime = tmp.path().join("runtime");
        std::fs::create_dir_all(runtime.join("runtime_features")).unwrap();
        std::fs::write(
            runtime_feature_path(&runtime, ZELLIJ_KITTY_PASSTHROUGH_FEATURE),
            "",
        )
        .unwrap();
        write_runtime_command(&runtime, "zellij");
        write_runtime_command(&runtime, "yazi");

        let findings =
            build_runtime_graphics_findings(&shared_with_terminal("ghostty"), &runtime, &[]);

        assert_eq!(findings[0].status, "ok");
        assert_eq!(
            findings[0].message,
            "Graphics previews: Yazelix Kitty passthrough bridge is active"
        );
        assert_eq!(
            findings[0].capability_mode.as_deref(),
            Some("kitty_passthrough_bridge")
        );
        assert_eq!(
            findings[1].message,
            "Chafa probe safety: no Chafa probe is active"
        );
    }

    // Regression: host-sourced Yazi is allowed, but it is not equivalent to bundled Yazelix KGP Yazi for Ghostty/Ratty graphics support.
    #[test]
    fn graphics_strategy_warns_for_host_sourced_yazi() {
        let tmp = TempDir::new().unwrap();
        let runtime = tmp.path().join("runtime");
        let path_dir = tmp.path().join("host_path");
        std::fs::create_dir_all(runtime.join("runtime_features")).unwrap();
        std::fs::write(
            runtime_feature_path(&runtime, ZELLIJ_KITTY_PASSTHROUGH_FEATURE),
            "",
        )
        .unwrap();
        std::fs::write(
            runtime.join("runtime_tools.json"),
            r#"{
              "yazi": {
                "source": "host",
                "commands": ["yazi", "ya"],
                "required_commands": ["yazi"]
              }
            }"#,
        )
        .unwrap();
        write_runtime_command(&runtime, "zellij");
        write_path_command(&path_dir, "yazi");

        let findings = build_runtime_graphics_findings(
            &shared_with_terminal("ghostty"),
            &runtime,
            &[path_dir],
        );

        assert_eq!(findings[0].status, "warning");
        assert_eq!(
            findings[0].capability_mode.as_deref(),
            Some("kitty_passthrough_bridge_host_yazi")
        );
        assert!(
            findings[0]
                .details
                .as_deref()
                .unwrap_or_default()
                .contains("Runtime Yazi command: host PATH")
        );
    }

    // Regression: declaring host-sourced Yazi is not enough for graphics diagnostics when the host command is missing.
    #[test]
    fn graphics_strategy_warns_when_host_sourced_yazi_is_missing() {
        let tmp = TempDir::new().unwrap();
        let runtime = tmp.path().join("runtime");
        let path_dir = tmp.path().join("empty_path");
        std::fs::create_dir_all(runtime.join("runtime_features")).unwrap();
        std::fs::create_dir_all(&path_dir).unwrap();
        std::fs::write(
            runtime_feature_path(&runtime, ZELLIJ_KITTY_PASSTHROUGH_FEATURE),
            "",
        )
        .unwrap();
        std::fs::write(
            runtime.join("runtime_tools.json"),
            r#"{
              "yazi": {
                "source": "host",
                "commands": ["yazi", "ya"],
                "required_commands": ["yazi"]
              }
            }"#,
        )
        .unwrap();
        write_runtime_command(&runtime, "zellij");

        let findings = build_runtime_graphics_findings(
            &shared_with_terminal("ghostty"),
            &runtime,
            &[path_dir],
        );

        assert_eq!(findings[0].status, "warning");
        assert_eq!(
            findings[0].capability_mode.as_deref(),
            Some("kitty_passthrough_bridge_incomplete")
        );
        assert!(
            findings[0]
                .details
                .as_deref()
                .unwrap_or_default()
                .contains("Runtime Yazi command: missing")
        );
    }

    // Regression: Ghostty/Ratty graphics diagnostics must warn on an incomplete bridge instead of claiming support from terminal identity alone.
    #[test]
    fn graphics_strategy_warns_for_ratty_without_bridge_marker() {
        let tmp = TempDir::new().unwrap();
        let runtime = tmp.path().join("runtime");
        write_runtime_command(&runtime, "zellij");
        write_runtime_command(&runtime, "yazi");

        let findings =
            build_runtime_graphics_findings(&shared_with_terminal("ratty"), &runtime, &[]);

        assert_eq!(findings[0].status, "warning");
        assert_eq!(
            findings[0].capability_mode.as_deref(),
            Some("kitty_passthrough_bridge_incomplete")
        );
        assert!(
            findings[0]
                .details
                .as_deref()
                .unwrap_or_default()
                .contains("Runtime Kitty passthrough marker: missing")
        );
    }

    // Defends: Chafa probe safety is a separate doctor finding and cannot masquerade as Kitty graphics bridge capability.
    #[test]
    fn chafa_probe_safety_is_reported_separately_from_graphics_capability() {
        let tmp = TempDir::new().unwrap();
        let runtime = tmp.path().join("runtime");
        std::fs::create_dir_all(runtime.join("runtime_features")).unwrap();
        std::fs::write(
            runtime_feature_path(&runtime, ZELLIJ_KITTY_PASSTHROUGH_FEATURE),
            "",
        )
        .unwrap();
        std::fs::write(
            runtime_feature_path(&runtime, CHAFA_PROBE_UNSAFE_FEATURE),
            "",
        )
        .unwrap();
        write_runtime_command(&runtime, "zellij");
        write_runtime_command(&runtime, "yazi");

        let findings =
            build_runtime_graphics_findings(&shared_with_terminal("ghostty"), &runtime, &[]);

        assert_eq!(findings[0].status, "ok");
        assert_eq!(findings[1].status, "warning");
        assert_eq!(
            findings[1].capability_mode.as_deref(),
            Some("chafa_probe_unsafe")
        );
    }
}
