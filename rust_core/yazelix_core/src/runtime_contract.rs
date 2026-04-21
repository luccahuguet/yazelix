use crate::bridge::{CoreError, ErrorClass};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

const SUPPORTED_TERMINALS: &[&str] = &["ghostty", "wezterm", "kitty", "alacritty", "foot"];
const NIXGL_WRAPPER_CANDIDATES: &[(&str, &[&str])] = &[
    ("nixGL", &["libexec", "nixGL"]),
    ("nixGLDefault", &["libexec", "nixGLDefault"]),
    ("nixGLMesa", &["libexec", "nixGLMesa"]),
    ("nixGLIntel", &["libexec", "nixGLIntel"]),
    ("nixGLMesa", &["bin", "nixGLMesa"]),
    ("nixGLIntel", &["bin", "nixGLIntel"]),
];
const HOST_NIXGL_COMMANDS: &[&str] = &["nixGL", "nixGLDefault", "nixGLMesa", "nixGLIntel"];

#[derive(Debug, Deserialize, Default)]
pub struct RuntimeContractEvaluateRequest {
    #[serde(default)]
    pub working_dir: Option<WorkingDirCheckRequest>,
    #[serde(default)]
    pub runtime_scripts: Vec<RuntimeScriptCheckRequest>,
    #[serde(default)]
    pub generated_layout: Option<GeneratedLayoutCheckRequest>,
    #[serde(default)]
    pub terminal_support: Option<TerminalSupportCheckRequest>,
    #[serde(default)]
    pub linux_ghostty_desktop_graphics_support: Option<LinuxGhosttyDesktopGraphicsRequest>,
}

#[derive(Debug, Deserialize)]
pub struct StartupLaunchPreflightRequest {
    pub startup: Option<StartupPreflightPayload>,
    pub launch: Option<LaunchPreflightPayload>,
}

#[derive(Debug, Deserialize)]
pub struct StartupPreflightPayload {
    pub working_dir: PathBuf,
    pub runtime_script: RuntimeScriptCheckRequest,
}

#[derive(Debug, Deserialize)]
pub struct LaunchPreflightPayload {
    pub working_dir: PathBuf,
    #[serde(default)]
    pub requested_terminal: String,
    #[serde(default)]
    pub terminals: Vec<String>,
    #[serde(default)]
    pub command_search_paths: Vec<PathBuf>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PreflightKind {
    Startup,
    Launch,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct StartupLaunchPreflightData {
    pub kind: PreflightKind,
    pub working_dir: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub script_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub terminal_candidates: Option<Vec<TerminalCandidate>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WorkingDirCheckRequest {
    pub kind: WorkingDirKind,
    pub path: PathBuf,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkingDirKind {
    Startup,
    Launch,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RuntimeScriptCheckRequest {
    pub id: String,
    pub label: String,
    pub owner_surface: String,
    pub path: PathBuf,
}

#[derive(Debug, Deserialize)]
pub struct GeneratedLayoutCheckRequest {
    pub owner_surface: String,
    pub path: PathBuf,
}

#[derive(Debug, Deserialize)]
pub struct TerminalSupportCheckRequest {
    pub owner_surface: String,
    #[serde(default)]
    pub requested_terminal: String,
    #[serde(default)]
    pub terminals: Vec<String>,
    #[serde(default)]
    pub command_search_paths: Vec<PathBuf>,
}

#[derive(Debug, Deserialize)]
pub struct LinuxGhosttyDesktopGraphicsRequest {
    pub owner_surface: String,
    #[serde(default)]
    pub terminals: Vec<String>,
    pub runtime_dir: Option<PathBuf>,
    #[serde(default)]
    pub command_search_paths: Vec<PathBuf>,
    #[serde(default)]
    pub platform_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct RuntimeContractEvaluateData {
    pub checks: Vec<RuntimeCheckData>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RuntimeCheckData {
    pub id: String,
    pub status: String,
    pub severity: String,
    pub owner_surface: String,
    pub message: String,
    pub details: Option<String>,
    pub recovery: Option<String>,
    pub failure_class: Option<String>,
    pub blocking: bool,
    pub path: Option<String>,
    pub candidates: Option<Vec<TerminalCandidate>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TerminalCandidate {
    pub terminal: String,
    pub name: String,
    pub command: String,
}

pub fn evaluate_runtime_contract(
    request: &RuntimeContractEvaluateRequest,
) -> Result<RuntimeContractEvaluateData, CoreError> {
    let mut checks = Vec::new();

    if let Some(working_dir) = &request.working_dir {
        checks.push(check_working_directory(working_dir));
    }

    for script in &request.runtime_scripts {
        checks.push(check_runtime_script(script));
    }

    if let Some(layout) = &request.generated_layout {
        checks.push(check_generated_layout(layout));
    }

    if let Some(terminal_support) = &request.terminal_support {
        checks.push(check_terminal_support(terminal_support));
    }

    if let Some(graphics_support) = &request.linux_ghostty_desktop_graphics_support {
        if let Some(check) = check_linux_ghostty_desktop_graphics_support(graphics_support) {
            checks.push(check);
        }
    }

    Ok(RuntimeContractEvaluateData { checks })
}

pub fn evaluate_startup_launch_preflight(
    request: &StartupLaunchPreflightRequest,
) -> Result<StartupLaunchPreflightData, CoreError> {
    match (&request.startup, &request.launch) {
        (Some(startup), None) => evaluate_startup_preflight_inner(startup),
        (None, Some(launch)) => evaluate_launch_preflight_inner(launch),
        (Some(_), Some(_)) => Err(CoreError::usage(
            "Expected exactly one of `startup` or `launch` in the preflight request.",
        )),
        (None, None) => Err(CoreError::usage(
            "Missing `startup` or `launch` preflight payload.",
        )),
    }
}

fn evaluate_startup_preflight_inner(
    startup: &StartupPreflightPayload,
) -> Result<StartupLaunchPreflightData, CoreError> {
    let contract = evaluate_runtime_contract(&RuntimeContractEvaluateRequest {
        working_dir: Some(WorkingDirCheckRequest {
            kind: WorkingDirKind::Startup,
            path: startup.working_dir.clone(),
        }),
        runtime_scripts: vec![startup.runtime_script.clone()],
        ..Default::default()
    })?;
    let wd = find_runtime_check(&contract.checks, "startup_working_dir")?;
    let script = find_runtime_check(&contract.checks, &startup.runtime_script.id)?;
    require_runtime_contract_check_ok(wd)?;
    require_runtime_contract_check_ok(script)?;
    let working_dir = wd.path.clone().ok_or_else(|| {
        CoreError::classified(
            ErrorClass::Internal,
            "preflight_missing_path",
            "Working directory check succeeded but omitted a resolved path.",
            "Report this as a Yazelix internal error.",
            json!({}),
        )
    })?;
    let script_path = script.path.clone().ok_or_else(|| {
        CoreError::classified(
            ErrorClass::Internal,
            "preflight_missing_path",
            "Runtime script check succeeded but omitted a resolved path.",
            "Report this as a Yazelix internal error.",
            json!({}),
        )
    })?;
    Ok(StartupLaunchPreflightData {
        kind: PreflightKind::Startup,
        working_dir,
        script_path: Some(script_path),
        terminal_candidates: None,
    })
}

fn evaluate_launch_preflight_inner(
    launch: &LaunchPreflightPayload,
) -> Result<StartupLaunchPreflightData, CoreError> {
    let contract = evaluate_runtime_contract(&RuntimeContractEvaluateRequest {
        working_dir: Some(WorkingDirCheckRequest {
            kind: WorkingDirKind::Launch,
            path: launch.working_dir.clone(),
        }),
        terminal_support: Some(TerminalSupportCheckRequest {
            owner_surface: "launch".to_string(),
            requested_terminal: launch.requested_terminal.clone(),
            terminals: launch.terminals.clone(),
            command_search_paths: launch.command_search_paths.clone(),
        }),
        ..Default::default()
    })?;
    let wd = find_runtime_check(&contract.checks, "launch_working_dir")?;
    let term = find_runtime_check(&contract.checks, "launch_terminal_support")?;
    require_runtime_contract_check_ok(wd)?;
    require_runtime_contract_check_ok(term)?;
    let working_dir = wd.path.clone().ok_or_else(|| {
        CoreError::classified(
            ErrorClass::Internal,
            "preflight_missing_path",
            "Working directory check succeeded but omitted a resolved path.",
            "Report this as a Yazelix internal error.",
            json!({}),
        )
    })?;
    Ok(StartupLaunchPreflightData {
        kind: PreflightKind::Launch,
        working_dir,
        script_path: None,
        terminal_candidates: Some(term.candidates.clone().unwrap_or_default()),
    })
}

fn find_runtime_check<'a>(
    checks: &'a [RuntimeCheckData],
    id: &str,
) -> Result<&'a RuntimeCheckData, CoreError> {
    checks.iter().find(|check| check.id == id).ok_or_else(|| {
        CoreError::usage(format!(
            "Missing runtime-contract check result for `{id}` after preflight evaluation."
        ))
    })
}

fn format_runtime_check_user_message(check: &RuntimeCheckData) -> String {
    let mut lines = vec![check.message.clone()];
    if let Some(details) = &check.details {
        if !details.is_empty() {
            lines.push(details.clone());
        }
    }
    let recovery = check.recovery.as_deref().unwrap_or("").trim();
    let failure_class = check.failure_class.as_deref().unwrap_or("").trim();
    if !recovery.is_empty() && !failure_class.is_empty() {
        if let Some(label) = normalized_failure_class_label(failure_class) {
            lines.push(format!("Failure class: {label}.\nRecovery: {recovery}"));
        } else {
            lines.push(recovery.to_string());
        }
    } else if !recovery.is_empty() {
        lines.push(recovery.to_string());
    }
    lines.join("\n")
}

fn normalized_failure_class_label(class: &str) -> Option<&'static str> {
    let normalized = class.trim().to_ascii_lowercase();
    match normalized.as_str() {
        "config" => Some("config problem"),
        "generated-state" => Some("generated-state problem"),
        "host-dependency" => Some("host-dependency problem"),
        _ => None,
    }
}

fn require_runtime_contract_check_ok(check: &RuntimeCheckData) -> Result<(), CoreError> {
    if check.status == "ok" {
        return Ok(());
    }
    Err(CoreError::classified(
        ErrorClass::Runtime,
        "runtime_preflight_check_failed",
        format_runtime_check_user_message(check),
        "Fix the reported preflight issue and retry.",
        serde_json::to_value(check).unwrap_or_else(|_| json!({})),
    ))
}

fn check_working_directory(request: &WorkingDirCheckRequest) -> RuntimeCheckData {
    let resolved = path_to_string(&request.path);

    if !request.path.exists() {
        let (id, owner_surface, missing_label, guidance) = match request.kind {
            WorkingDirKind::Startup => (
                "startup_working_dir",
                "startup",
                "Startup directory does not exist",
                "Use an existing directory, or run yzx launch --home.",
            ),
            WorkingDirKind::Launch => (
                "launch_working_dir",
                "launch",
                "Launch directory does not exist",
                "Use an existing directory, or use --home to start from HOME.",
            ),
        };
        return build_runtime_check(
            id,
            "error",
            "error",
            owner_surface,
            format!("{missing_label}: {resolved}"),
            None,
            Some(guidance.to_string()),
            None,
            true,
            Some(resolved),
            None,
        );
    }

    if !request.path.is_dir() {
        let (id, owner_surface, invalid_label, guidance) = match request.kind {
            WorkingDirKind::Startup => (
                "startup_working_dir",
                "startup",
                "Startup path is not a directory",
                "Pass a directory to yzx launch --path.",
            ),
            WorkingDirKind::Launch => (
                "launch_working_dir",
                "launch",
                "Launch path is not a directory",
                "Pass a directory to yzx launch --path.",
            ),
        };
        return build_runtime_check(
            id,
            "error",
            "error",
            owner_surface,
            format!("{invalid_label}: {resolved}"),
            None,
            Some(guidance.to_string()),
            None,
            true,
            Some(resolved),
            None,
        );
    }

    build_runtime_check(
        match request.kind {
            WorkingDirKind::Startup => "startup_working_dir",
            WorkingDirKind::Launch => "launch_working_dir",
        },
        "ok",
        "info",
        match request.kind {
            WorkingDirKind::Startup => "startup",
            WorkingDirKind::Launch => "launch",
        },
        format!("Working directory is valid: {resolved}"),
        None,
        None,
        None,
        false,
        Some(resolved),
        None,
    )
}

fn check_runtime_script(request: &RuntimeScriptCheckRequest) -> RuntimeCheckData {
    check_runtime_file(
        &request.id,
        &request.owner_surface,
        &request.label,
        &request.label,
        &request.path,
        Some("Your runtime looks incomplete. Reinstall/regenerate Yazelix and try again."),
        Some("generated-state"),
    )
}

fn check_generated_layout(request: &GeneratedLayoutCheckRequest) -> RuntimeCheckData {
    check_runtime_file(
        "generated_layout",
        &request.owner_surface,
        "generated Zellij layout",
        "generated Zellij layout",
        &request.path,
        Some(
            "Run `yzx doctor` to inspect generated-state issues, or check the configured layout name.",
        ),
        Some("generated-state"),
    )
}

fn check_runtime_file(
    id: &str,
    owner_surface: &str,
    missing_label: &str,
    invalid_label: &str,
    path: &Path,
    recovery: Option<&str>,
    failure_class: Option<&str>,
) -> RuntimeCheckData {
    let resolved = path_to_string(path);

    if !path.exists() {
        return build_runtime_check(
            id,
            "error",
            "error",
            owner_surface,
            format!("Missing Yazelix {missing_label}: {resolved}"),
            None,
            recovery.map(str::to_string),
            failure_class.map(str::to_string),
            true,
            Some(resolved),
            None,
        );
    }

    if !path.is_file() {
        return build_runtime_check(
            id,
            "error",
            "error",
            owner_surface,
            format!("Yazelix {invalid_label} is not a file: {resolved}"),
            None,
            None,
            None,
            true,
            Some(resolved),
            None,
        );
    }

    build_runtime_check(
        id,
        "ok",
        "info",
        owner_surface,
        format!("Yazelix {missing_label} is present"),
        None,
        None,
        None,
        false,
        Some(resolved),
        None,
    )
}

fn check_terminal_support(request: &TerminalSupportCheckRequest) -> RuntimeCheckData {
    let requested_terminal = request.requested_terminal.trim();

    if !requested_terminal.is_empty() {
        if !SUPPORTED_TERMINALS.contains(&requested_terminal) {
            return build_runtime_check(
                "launch_terminal_support",
                "error",
                "error",
                &request.owner_surface,
                format!("Unsupported terminal '{requested_terminal}'"),
                Some(format!(
                    "Supported terminals: {}",
                    SUPPORTED_TERMINALS.join(", ")
                )),
                None,
                None,
                true,
                None,
                None,
            );
        }

        let candidates = detect_terminal_candidates(
            &[requested_terminal.to_string()],
            &request.command_search_paths,
        );
        if candidates.is_empty() {
            return build_runtime_check(
                "launch_terminal_support",
                "error",
                "error",
                &request.owner_surface,
                format!(
                    "Specified terminal '{requested_terminal}' is not available in the active Yazelix runtime or PATH."
                ),
                None,
                Some(
                    "Use a terminal shipped by the active Yazelix runtime, install it on PATH, or choose a different terminal for testing."
                        .to_string(),
                ),
                Some("host-dependency".to_string()),
                true,
                None,
                None,
            );
        }

        return build_runtime_check(
            "launch_terminal_support",
            "ok",
            "info",
            &request.owner_surface,
            format!("Terminal command discovery is available for {requested_terminal}"),
            None,
            None,
            None,
            false,
            None,
            Some(candidates),
        );
    }

    let candidates = detect_terminal_candidates(&request.terminals, &request.command_search_paths);
    if candidates.is_empty() {
        return build_runtime_check(
            "launch_terminal_support",
            "error",
            "error",
            &request.owner_surface,
            "None of the configured terminal binaries are available in the active Yazelix runtime or PATH.".to_string(),
            None,
            Some(
                "Use Ghostty from the active Yazelix runtime, install one of the other configured terminals on PATH, or adjust [terminal].terminals to match what is available."
                    .to_string(),
            ),
            Some("host-dependency".to_string()),
            true,
            None,
            None,
        );
    }

    build_runtime_check(
        "launch_terminal_support",
        "ok",
        "info",
        &request.owner_surface,
        "A configured terminal command is available".to_string(),
        None,
        None,
        None,
        false,
        None,
        Some(candidates),
    )
}

fn check_linux_ghostty_desktop_graphics_support(
    request: &LinuxGhosttyDesktopGraphicsRequest,
) -> Option<RuntimeCheckData> {
    if runtime_platform_name(request.platform_name.as_deref()) != "linux" {
        return None;
    }

    let candidates = detect_terminal_candidates(&request.terminals, &request.command_search_paths);
    let active_candidate = candidates.first()?;
    if active_candidate.terminal != "ghostty" {
        return None;
    }

    let nixgl_context = resolve_nixgl_launch_context(
        request.runtime_dir.as_deref(),
        &request.command_search_paths,
    );
    if nixgl_context.source == "runtime" {
        return None;
    }

    let details_lines = if nixgl_context.source == "host_path" {
        vec![
            "First launch candidate: Ghostty".to_string(),
            format!(
                "Detected host PATH graphics wrapper: {}",
                nixgl_context.command.unwrap_or_default()
            ),
            "Linux Ghostty launches can appear healthy from an interactive shell while still failing from desktop-entry launches that inherit a smaller GUI PATH".to_string(),
            "Update or reinstall Yazelix so the active runtime ships its own Linux graphics wrapper, or choose a different first terminal if you intentionally do not want Ghostty here".to_string(),
        ]
    } else {
        vec![
            "First launch candidate: Ghostty".to_string(),
            "No runtime-owned or PATH-provided nixGL wrapper was detected for Linux Ghostty launches".to_string(),
            "Ghostty can fail to acquire an OpenGL context from desktop-entry launches when this wrapper is missing".to_string(),
            "Update or reinstall Yazelix so the active runtime ships its Linux graphics wrapper, or choose a different first terminal".to_string(),
        ]
    };

    Some(build_runtime_check(
        "linux_ghostty_desktop_graphics_support",
        "warning",
        "warning",
        &request.owner_surface,
        "Linux Ghostty desktop-launch graphics support is not runtime-owned".to_string(),
        Some(details_lines.join("\n")),
        None,
        None,
        false,
        None,
        None,
    ))
}

fn detect_terminal_candidates(
    preferred: &[String],
    command_search_paths: &[PathBuf],
) -> Vec<TerminalCandidate> {
    let ordered_terminals: Vec<String> = preferred
        .iter()
        .filter(|terminal| SUPPORTED_TERMINALS.contains(&terminal.as_str()))
        .cloned()
        .collect();
    if ordered_terminals.is_empty() {
        return Vec::new();
    }

    ordered_terminals
        .into_iter()
        .filter(|terminal| command_exists(terminal, command_search_paths))
        .map(|terminal| TerminalCandidate {
            name: terminal_display_name(&terminal).to_string(),
            command: terminal.clone(),
            terminal,
        })
        .collect()
}

fn command_exists(command: &str, command_search_paths: &[PathBuf]) -> bool {
    resolve_command_path(command, command_search_paths).is_some()
}

fn resolve_command_path(command: &str, command_search_paths: &[PathBuf]) -> Option<PathBuf> {
    if command.contains(std::path::MAIN_SEPARATOR) {
        let candidate = PathBuf::from(command);
        if is_executable_file(&candidate) {
            return Some(candidate);
        }
        return None;
    }

    command_search_paths.iter().find_map(|base| {
        let candidate = base.join(command);
        if is_executable_file(&candidate) {
            Some(candidate)
        } else {
            None
        }
    })
}

fn is_executable_file(candidate: &Path) -> bool {
    let Ok(metadata) = fs::metadata(candidate) else {
        return false;
    };
    if !metadata.is_file() {
        return false;
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        metadata.permissions().mode() & 0o111 != 0
    }

    #[cfg(not(unix))]
    {
        true
    }
}

fn runtime_platform_name(explicit: Option<&str>) -> String {
    explicit
        .map(str::to_string)
        .or_else(|| env::var("YAZELIX_TEST_OS").ok())
        .unwrap_or_else(|| env::consts::OS.to_string())
        .trim()
        .to_lowercase()
}

struct NixglLaunchContext {
    source: &'static str,
    command: Option<String>,
}

fn resolve_nixgl_launch_context(
    runtime_dir: Option<&Path>,
    command_search_paths: &[PathBuf],
) -> NixglLaunchContext {
    if let Some(runtime_dir) = runtime_dir {
        for (command, segments) in NIXGL_WRAPPER_CANDIDATES {
            let candidate = segments
                .iter()
                .fold(runtime_dir.to_path_buf(), |path, segment| {
                    path.join(segment)
                });
            if is_executable_file(&candidate) {
                return NixglLaunchContext {
                    source: "runtime",
                    command: Some((*command).to_string()),
                };
            }
        }
    }

    for command in HOST_NIXGL_COMMANDS {
        if command_exists(command, command_search_paths) {
            return NixglLaunchContext {
                source: "host_path",
                command: Some((*command).to_string()),
            };
        }
    }

    NixglLaunchContext {
        source: "none",
        command: None,
    }
}

fn terminal_display_name(terminal: &str) -> String {
    match terminal {
        "ghostty" => "Ghostty".to_string(),
        "wezterm" => "WezTerm".to_string(),
        "kitty" => "Kitty".to_string(),
        "alacritty" => "Alacritty".to_string(),
        "foot" => "Foot".to_string(),
        _ => terminal.to_string(),
    }
}

fn path_to_string(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

fn build_runtime_check(
    id: &str,
    status: &str,
    severity: &str,
    owner_surface: &str,
    message: impl Into<String>,
    details: Option<String>,
    recovery: Option<String>,
    failure_class: Option<String>,
    blocking: bool,
    path: Option<String>,
    candidates: Option<Vec<TerminalCandidate>>,
) -> RuntimeCheckData {
    RuntimeCheckData {
        id: id.to_string(),
        status: status.to_string(),
        severity: severity.to_string(),
        owner_surface: owner_surface.to_string(),
        message: message.into(),
        details,
        recovery,
        failure_class,
        blocking,
        path,
        candidates,
    }
}

// Test lane: maintainer
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn write_executable(path: &Path) {
        fs::write(path, "#!/bin/sh\nexit 0\n").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut permissions = fs::metadata(path).unwrap().permissions();
            permissions.set_mode(0o755);
            fs::set_permissions(path, permissions).unwrap();
        }
    }

    // Defends: shared runtime-contract evaluation reports missing working-dir, script, and layout assets in one batch.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn evaluate_reports_missing_runtime_assets() {
        let temp = tempdir().unwrap();
        let data = evaluate_runtime_contract(&RuntimeContractEvaluateRequest {
            working_dir: Some(WorkingDirCheckRequest {
                kind: WorkingDirKind::Launch,
                path: temp.path().join("missing-dir"),
            }),
            runtime_scripts: vec![RuntimeScriptCheckRequest {
                id: "launch_runtime_script".to_string(),
                label: "launch script".to_string(),
                owner_surface: "doctor".to_string(),
                path: temp.path().join("missing-script.nu"),
            }],
            generated_layout: Some(GeneratedLayoutCheckRequest {
                owner_surface: "doctor".to_string(),
                path: temp.path().join("missing-layout.kdl"),
            }),
            terminal_support: None,
            linux_ghostty_desktop_graphics_support: None,
        })
        .unwrap();

        assert_eq!(data.checks.len(), 3);
        assert_eq!(data.checks[0].id, "launch_working_dir");
        assert_eq!(data.checks[1].id, "launch_runtime_script");
        assert_eq!(data.checks[2].id, "generated_layout");
        assert_eq!(
            data.checks[2].failure_class.as_deref(),
            Some("generated-state")
        );
    }

    // Defends: shared runtime-contract evaluation reports both terminal candidates and the Linux Ghostty graphics ownership warning.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn evaluate_reports_terminal_candidates_and_host_path_ghostty_warning() {
        let temp = tempdir().unwrap();
        let runtime_dir = temp.path().join("runtime");
        let host_bin = temp.path().join("host-bin");
        fs::create_dir_all(runtime_dir.join("libexec")).unwrap();
        fs::create_dir_all(&host_bin).unwrap();
        write_executable(&host_bin.join("ghostty"));
        write_executable(&host_bin.join("nixGLMesa"));

        let data = evaluate_runtime_contract(&RuntimeContractEvaluateRequest {
            working_dir: None,
            runtime_scripts: Vec::new(),
            generated_layout: None,
            terminal_support: Some(TerminalSupportCheckRequest {
                owner_surface: "launch".to_string(),
                requested_terminal: String::new(),
                terminals: vec!["ghostty".to_string()],
                command_search_paths: vec![host_bin.clone()],
            }),
            linux_ghostty_desktop_graphics_support: Some(LinuxGhosttyDesktopGraphicsRequest {
                owner_surface: "doctor".to_string(),
                terminals: vec!["ghostty".to_string()],
                runtime_dir: Some(runtime_dir),
                command_search_paths: vec![host_bin],
                platform_name: Some("linux".to_string()),
            }),
        })
        .unwrap();

        assert_eq!(data.checks.len(), 2);
        assert_eq!(
            data.checks[0].message,
            "A configured terminal command is available"
        );
        assert_eq!(
            data.checks[0]
                .candidates
                .as_ref()
                .and_then(|candidates| candidates.first())
                .map(|candidate| candidate.terminal.as_str()),
            Some("ghostty")
        );
        assert_eq!(
            data.checks[1].message,
            "Linux Ghostty desktop-launch graphics support is not runtime-owned"
        );
        assert!(
            data.checks[1]
                .details
                .as_deref()
                .unwrap()
                .contains("Detected host PATH graphics wrapper: nixGLMesa")
        );
    }

    // Defends: shared runtime-contract evaluation rejects unsupported requested terminals before launch fallback logic runs.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn evaluate_rejects_unsupported_requested_terminal() {
        let data = evaluate_runtime_contract(&RuntimeContractEvaluateRequest {
            working_dir: None,
            runtime_scripts: Vec::new(),
            generated_layout: None,
            terminal_support: Some(TerminalSupportCheckRequest {
                owner_surface: "launch".to_string(),
                requested_terminal: "warpterm".to_string(),
                terminals: vec!["ghostty".to_string()],
                command_search_paths: Vec::new(),
            }),
            linux_ghostty_desktop_graphics_support: None,
        })
        .unwrap();

        assert_eq!(data.checks.len(), 1);
        assert_eq!(data.checks[0].status, "error");
        assert_eq!(data.checks[0].message, "Unsupported terminal 'warpterm'");
        assert!(
            data.checks[0]
                .details
                .as_deref()
                .unwrap_or_default()
                .contains("Supported terminals: ghostty, wezterm, kitty, alacritty, foot")
        );
    }

    // Defends: startup-launch preflight bundles startup working-dir and runtime-script checks into one ok envelope.
    // Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
    #[test]
    fn startup_preflight_reports_resolved_paths_when_checks_pass() {
        let temp = tempdir().unwrap();
        let work = temp.path().join("work");
        fs::create_dir_all(&work).unwrap();
        let script = temp.path().join("start_yazelix_inner.nu");
        fs::write(&script, "").unwrap();

        let data = evaluate_startup_launch_preflight(&StartupLaunchPreflightRequest {
            startup: Some(StartupPreflightPayload {
                working_dir: work.clone(),
                runtime_script: RuntimeScriptCheckRequest {
                    id: "startup_runtime_script".to_string(),
                    label: "startup script".to_string(),
                    owner_surface: "startup".to_string(),
                    path: script.clone(),
                },
            }),
            launch: None,
        })
        .unwrap();

        assert_eq!(data.kind, PreflightKind::Startup);
        assert_eq!(data.working_dir, super::path_to_string(&work));
        assert_eq!(data.script_path, Some(super::path_to_string(&script)));
        assert!(data.terminal_candidates.is_none());
    }

    // Defends: startup-launch preflight rejects malformed requests that specify both startup and launch.
    // Strength: defect=2 behavior=1 resilience=1 cost=1 uniqueness=2 total=7/10
    #[test]
    fn startup_launch_preflight_rejects_ambiguous_request() {
        let temp = tempdir().unwrap();
        let err = evaluate_startup_launch_preflight(&StartupLaunchPreflightRequest {
            startup: Some(StartupPreflightPayload {
                working_dir: temp.path().to_path_buf(),
                runtime_script: RuntimeScriptCheckRequest {
                    id: "startup_runtime_script".to_string(),
                    label: "s".to_string(),
                    owner_surface: "startup".to_string(),
                    path: temp.path().join("x.nu"),
                },
            }),
            launch: Some(LaunchPreflightPayload {
                working_dir: temp.path().to_path_buf(),
                requested_terminal: String::new(),
                terminals: vec![],
                command_search_paths: vec![],
            }),
        })
        .unwrap_err();

        assert_eq!(err.class().as_str(), "usage");
    }

    // Defends: startup-launch preflight bundles launch working-dir and terminal support into one ok envelope.
    // Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
    #[test]
    fn launch_preflight_returns_terminal_candidates_when_checks_pass() {
        let temp = tempdir().unwrap();
        let work = temp.path().join("work");
        fs::create_dir_all(&work).unwrap();
        let host_bin = temp.path().join("host-bin");
        fs::create_dir_all(&host_bin).unwrap();
        write_executable(&host_bin.join("ghostty"));

        let data = evaluate_startup_launch_preflight(&StartupLaunchPreflightRequest {
            startup: None,
            launch: Some(LaunchPreflightPayload {
                working_dir: work.clone(),
                requested_terminal: String::new(),
                terminals: vec!["ghostty".to_string()],
                command_search_paths: vec![host_bin],
            }),
        })
        .unwrap();

        assert_eq!(data.kind, PreflightKind::Launch);
        assert_eq!(data.working_dir, super::path_to_string(&work));
        assert!(data.script_path.is_none());
        let candidates = data.terminal_candidates.as_ref().unwrap();
        assert_eq!(
            candidates.first().map(|c| c.terminal.as_str()),
            Some("ghostty")
        );
    }
}
