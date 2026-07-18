use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    env,
    ffi::{OsStr, OsString},
    fs::{self, OpenOptions},
    io::{BufRead, BufReader, Write},
    os::unix::ffi::OsStrExt,
    os::unix::fs::{FileTypeExt, PermissionsExt},
    os::unix::net::UnixStream,
    path::{Path, PathBuf},
    process::{Command, ExitCode, Output},
    time::{SystemTime, UNIX_EPOCH},
};
use yzx_open::sidebar::{
    Config as OrchestratorConfig, SidebarYaziState, optional_sidebar_yazi_state, orchestrator_pipe,
    orchestrator_query,
};

#[derive(Debug)]
struct Config {
    editor: OsString,
    git: OsString,
    ya: OsString,
    zellij: OsString,
    state_dir: PathBuf,
    session_id: String,
    yazi_id: Option<String>,
    zellij_session_name: Option<String>,
    zellij_pane_id: Option<String>,
    log_level: LogLevel,
}

#[derive(Debug, Deserialize)]
struct Registry {
    schema_version: u64,
    session_id: Option<String>,
    transport: Transport,
    auth_token_path: PathBuf,
    zellij_session_name: Option<String>,
    zellij_pane_id: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Transport {
    path: PathBuf,
}

#[derive(Debug, Deserialize)]
struct PaneListEntry {
    id: u64,
    is_plugin: bool,
    tab_id: u64,
    #[serde(default)]
    exited: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct PaneId {
    id: u64,
    is_plugin: bool,
}

#[derive(Debug, Deserialize)]
struct BridgeResponse {
    status: String,
    error: Option<BridgeResponseError>,
}

#[derive(Debug, Deserialize)]
struct BridgeResponseError {
    message: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum OpenIntent {
    Ordinary,
    Retarget,
}

#[derive(Debug)]
struct OpenRequest {
    intent: OpenIntent,
    targets: Vec<OsString>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
struct CanonicalWorkspace {
    root: PathBuf,
    source: WorkspaceSource,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum WorkspaceSource {
    Bootstrap,
    Explicit,
}

#[derive(Debug, Deserialize)]
struct ActiveTabSessionState {
    schema_version: u64,
    active_tab_position: usize,
    workspace: Option<CanonicalWorkspace>,
}

#[derive(Debug, Deserialize)]
struct WorkspaceRetargetResponse {
    status: String,
}

#[derive(Debug, PartialEq, Eq)]
struct WorkspaceDecision {
    root: PathBuf,
    mutate: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum LogLevel {
    Off,
    Error,
    Info,
    Debug,
}

const LOG_MAX_BYTES: u64 = 64 * 1024;
const ZELLIJ_SESSION_NAME_ENV: &str = "ZELLIJ_SESSION_NAME";
const YAZELIX_ZELLIJ_SESSION_NAME_ENV: &str = "YAZELIX_ZELLIJ_SESSION_NAME";

fn main() -> ExitCode {
    let config = Config::from_env();
    match run(&config, env::args_os().skip(1)) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            log_error(&config, &format!("error: {error:#}"));
            eprintln!("yzx-open: {error:#}");
            ExitCode::FAILURE
        }
    }
}

fn run(config: &Config, raw_targets: impl IntoIterator<Item = OsString>) -> Result<()> {
    let request = parse_open_request(raw_targets)?;
    let targets = request
        .targets
        .into_iter()
        .map(abs_path)
        .collect::<Result<Vec<_>>>()?;
    for target in &targets {
        fs::metadata(target).with_context(|| {
            format!(
                "target does not exist or cannot be inspected: {}",
                target.display()
            )
        })?;
    }
    let current_state = active_tab_workspace(config)?;
    let candidate = if request.intent == OpenIntent::Ordinary
        && current_state.workspace.source == WorkspaceSource::Explicit
    {
        current_state.workspace.root.clone()
    } else {
        target_workspace_root(config, &targets)
    };
    let decision = decide_workspace(request.intent, &current_state.workspace, &candidate);
    log_debug(
        config,
        &format!(
            "operation={} tab={} source={:?} root={} candidate={} targets={}",
            request.intent.as_str(),
            current_state.active_tab_position,
            current_state.workspace.source,
            current_state.workspace.root.display(),
            candidate.display(),
            json!(targets),
        ),
    );

    if decision.mutate {
        set_workspace(config, &decision.root, WorkspaceSource::Explicit)
            .context("could not update the canonical tab workspace")?;
    }

    let open_result = if uses_helix_bridge(&config.editor) {
        try_bridge(config, &targets, &decision.root).and_then(|opened| {
            if opened {
                Ok(())
            } else {
                open_editor_pane(config, &targets, &decision.root)
            }
        })
    } else {
        log_info(
            config,
            &format!(
                "bridge skipped for non-Helix editor={}",
                config.editor.to_string_lossy()
            ),
        );
        open_editor_pane(config, &targets, &decision.root)
    };

    if let Err(error) = open_result {
        if decision.mutate
            && let Err(rollback_error) = set_workspace(
                config,
                &current_state.workspace.root,
                current_state.workspace.source,
            )
        {
            bail!(
                "editor operation failed: {error:#}; workspace rollback also failed: {rollback_error:#}"
            );
        }
        return Err(error);
    }
    if request.intent == OpenIntent::Ordinary {
        follow_originating_sidebar(config, &current_state, &targets)?;
    }
    Ok(())
}

impl OpenIntent {
    fn as_str(self) -> &'static str {
        match self {
            Self::Ordinary => "open",
            Self::Retarget => "retarget",
        }
    }
}

impl Config {
    fn from_env() -> Self {
        let state_dir = nonempty_env("YAZELIX_STATE_DIR")
            .map(PathBuf::from)
            .or_else(|| nonempty_env("XDG_DATA_HOME").map(|dir| PathBuf::from(dir).join("yazelix")))
            .or_else(|| {
                nonempty_env("HOME").map(|dir| PathBuf::from(dir).join(".local/share/yazelix"))
            })
            .unwrap_or_else(|| env::temp_dir().join("yazelix"));

        Self {
            editor: nonempty_env("YZX_EDITOR").unwrap_or_else(|| "yzx-hx".into()),
            git: "git".into(),
            ya: nonempty_env("YZX_YA").unwrap_or_else(|| "ya".into()),
            zellij: nonempty_env("YZX_ZELLIJ").unwrap_or_else(|| "zellij".into()),
            state_dir,
            session_id: bridge_session_id(env::var("YAZELIX_HELIX_BRIDGE_SESSION_ID").ok()),
            yazi_id: env::var("YAZI_ID").ok().filter(|id| !id.trim().is_empty()),
            zellij_session_name: zellij_session_name_from_env(),
            zellij_pane_id: env::var("ZELLIJ_PANE_ID").ok(),
            log_level: LogLevel::from_env(),
        }
    }
}

fn try_bridge(config: &Config, targets: &[PathBuf], cwd: &Path) -> Result<bool> {
    let bridge_dir = config
        .state_dir
        .join("helix_bridge")
        .join(&config.session_id);
    let Ok(entries) = fs::read_dir(&bridge_dir) else {
        return Ok(false);
    };
    let current_tab_panes = current_tab_panes(config);

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|value| value.to_str()) != Some("json") {
            continue;
        }
        let registry = match read_registry(&path) {
            Ok(registry) => registry,
            Err(error) => {
                log_debug(
                    config,
                    &format!(
                        "bridge skipped invalid registry={}: {error:#}",
                        path.display()
                    ),
                );
                continue;
            }
        };
        if registry.schema_version != 2 {
            log_debug(
                config,
                &format!(
                    "bridge skipped registry={} unsupported_schema={}",
                    path.display(),
                    registry.schema_version
                ),
            );
            continue;
        }
        if !registry.matches_session(config) {
            log_debug(
                config,
                &format!(
                    "bridge skipped registry={} registry_session={:?} registry_zellij_session={:?} current_session={} current_zellij_session={:?}",
                    path.display(),
                    registry.session_id,
                    registry.zellij_session_name,
                    config.session_id,
                    config.zellij_session_name,
                ),
            );
            continue;
        }
        if !registry.in_current_tab(config, current_tab_panes.as_ref()) {
            log_debug(
                config,
                &format!(
                    "bridge skipped registry={} pane={:?} current_pane={:?}",
                    path.display(),
                    registry.zellij_pane_id,
                    config.zellij_pane_id,
                ),
            );
            continue;
        }
        if !registry.is_live() {
            log_debug(
                config,
                &format!("bridge skipped stale registry={}", path.display()),
            );
            continue;
        }

        if let Some(pane_id) = &registry.zellij_pane_id {
            log_debug(
                config,
                &format!(
                    "bridge focus probe registry={} pane={pane_id}",
                    path.display()
                ),
            );
            if let Err(error) = focus_pane(config, pane_id) {
                log_info(
                    config,
                    &format!("focus failed pane={pane_id}; treating bridge as stale: {error:#}"),
                );
                continue;
            }
        }

        let (action, payload) = bridge_open_request(targets, cwd);
        log_debug(
            config,
            &format!(
                "bridge registry={} pane={:?} action={} payload={}",
                path.display(),
                registry.zellij_pane_id,
                action,
                payload
            ),
        );

        match send_bridge_request(
            &registry.transport.path,
            &registry.auth_token_path,
            action,
            payload,
        ) {
            Ok(response) => {
                log_info(
                    config,
                    &format!(
                        "bridge reused action={} pane={:?}",
                        action, registry.zellij_pane_id
                    ),
                );
                log_debug(config, &format!("bridge ok response={}", response.trim()));
                return Ok(true);
            }
            Err(BridgeSendError::Unavailable) => {
                log_info(
                    config,
                    &format!("bridge unavailable registry={}", path.display()),
                );
                continue;
            }
            Err(BridgeSendError::Rejected(message)) => {
                log_error(config, &format!("bridge rejected: {message}"));
                bail!(message);
            }
        }
    }

    log_info(config, "no live bridge found; opening a new editor pane");
    Ok(false)
}

fn read_registry(path: &Path) -> Result<Registry> {
    let raw = fs::read_to_string(path)
        .with_context(|| format!("could not read Helix bridge registry {}", path.display()))?;
    serde_json::from_str::<Registry>(&raw)
        .with_context(|| format!("could not parse Helix bridge registry {}", path.display()))
}

impl Registry {
    fn matches_session(&self, config: &Config) -> bool {
        self.session_id.as_deref() == Some(config.session_id.as_str())
            && match (&self.zellij_session_name, &config.zellij_session_name) {
                (Some(registry_session), Some(current_session)) => {
                    registry_session == current_session
                }
                (Some(_), None) => false,
                _ => true,
            }
    }

    fn is_live(&self) -> bool {
        fs::metadata(&self.transport.path).is_ok_and(|metadata| metadata.file_type().is_socket())
            && self.auth_token_path.is_file()
    }

    fn in_current_tab(
        &self,
        config: &Config,
        current_tab_panes: Option<&(u64, Vec<PaneListEntry>)>,
    ) -> bool {
        let Some((caller_tab_id, panes)) = current_tab_panes else {
            return false;
        };
        let Some(pane_id) = self
            .zellij_pane_id
            .as_deref()
            .and_then(parse_zellij_pane_id)
        else {
            log_debug(config, "bridge skipped because registry has no pane id");
            return false;
        };
        panes
            .iter()
            .any(|pane| pane.matches(pane_id) && pane.tab_id == *caller_tab_id)
    }
}

fn current_tab_panes(config: &Config) -> Option<(u64, Vec<PaneListEntry>)> {
    let caller_pane_id = config
        .zellij_pane_id
        .as_deref()
        .and_then(parse_zellij_pane_id)?;
    let output = zellij_command(config)
        .args(["action", "list-panes", "--json", "--tab", "--state"])
        .output()
        .ok()?;
    if !output.status.success() {
        log_info(
            config,
            &format!(
                "bridge reuse skipped; list-panes failed: {}",
                json!(String::from_utf8_lossy(&output.stderr).trim())
            ),
        );
        return None;
    }
    let panes = serde_json::from_slice::<Vec<PaneListEntry>>(&output.stdout).ok()?;
    let caller_tab_id = panes
        .iter()
        .find(|pane| pane.matches(caller_pane_id))
        .map(|pane| pane.tab_id)?;
    Some((caller_tab_id, panes))
}

impl PaneListEntry {
    fn matches(&self, pane_id: PaneId) -> bool {
        !self.exited && self.id == pane_id.id && self.is_plugin == pane_id.is_plugin
    }
}

fn send_bridge_request(
    socket_path: &Path,
    token_path: &Path,
    action: &'static str,
    payload: Value,
) -> std::result::Result<String, BridgeSendError> {
    let token = fs::read_to_string(token_path).map_err(|_| BridgeSendError::Unavailable)?;
    let mut stream = UnixStream::connect(socket_path).map_err(|_| BridgeSendError::Unavailable)?;
    let request = json!({
        "schema_version": 2,
        "request_id": format!("yzx-open-{}-{}", unix_millis(), std::process::id()),
        "auth_token": token.trim(),
        "action": action,
        "timeout_ms": 5000,
        "payload": payload,
    });
    writeln!(stream, "{request}").map_err(|_| BridgeSendError::Unavailable)?;

    let mut response_raw = String::new();
    BufReader::new(stream)
        .read_line(&mut response_raw)
        .map_err(|_| BridgeSendError::Unavailable)?;
    let response = serde_json::from_str::<BridgeResponse>(&response_raw).map_err(|error| {
        BridgeSendError::Rejected(format!("Helix bridge returned invalid JSON: {error}"))
    })?;

    if response.status == "ok" {
        Ok(response_raw.trim().to_string())
    } else {
        Err(BridgeSendError::Rejected(
            response
                .error
                .map(|error| error.message)
                .unwrap_or_else(|| "Helix bridge rejected the open request".into()),
        ))
    }
}

fn bridge_open_request(targets: &[PathBuf], working_dir: &Path) -> (&'static str, Value) {
    if let Some(target) = directory_target(targets) {
        (
            "helix.open_directory",
            json!({
                "working_dir": working_dir,
                "picker_dir": target,
            }),
        )
    } else {
        (
            "helix.open_files",
            json!({
                "working_dir": working_dir,
                "file_paths": targets,
                "focus": true,
            }),
        )
    }
}

fn open_editor_pane(config: &Config, targets: &[PathBuf], cwd: &Path) -> Result<()> {
    ensure_editor_command(config)?;
    let mut args = vec![
        OsString::from("run"),
        OsString::from("--name"),
        OsString::from("editor"),
        OsString::from("--cwd"),
        cwd.as_os_str().to_os_string(),
        OsString::from("--"),
        config.editor.clone(),
    ];
    if let Some(target) = directory_target(targets) {
        args.push(target.as_os_str().to_os_string());
    } else {
        args.extend(
            targets
                .iter()
                .map(|target| target.as_os_str().to_os_string()),
        );
    }

    log_info(
        config,
        &format!(
            "opening editor pane program={} args={}",
            config.zellij.to_string_lossy(),
            json!(
                args.iter()
                    .map(|arg| arg.to_string_lossy().into_owned())
                    .collect::<Vec<_>>()
            )
        ),
    );

    let output = zellij_command(config)
        .args(&args)
        .env("YAZELIX_STATE_DIR", &config.state_dir)
        .env("YAZELIX_HELIX_BRIDGE_SESSION_ID", &config.session_id)
        .output()
        .context("could not run zellij")?;
    let output_log_level = if output.status.success() {
        LogLevel::Debug
    } else {
        LogLevel::Error
    };
    let output_status = output
        .status
        .code()
        .map(|code| code.to_string())
        .unwrap_or_else(|| "signal".into());
    log_event(
        config,
        output_log_level,
        &format!(
            "open editor pane status={} stdout={} stderr={}",
            output_status,
            json!(String::from_utf8_lossy(&output.stdout).trim()),
            json!(String::from_utf8_lossy(&output.stderr).trim())
        ),
    );
    ensure_success(&output, "zellij failed to open editor pane")?;
    log_info(config, "editor pane opened");
    Ok(())
}

fn ensure_editor_command(config: &Config) -> Result<()> {
    let path = env::var_os("PATH").filter(|path| !path.is_empty());
    if command_exists(&config.editor, path.as_deref()) {
        return Ok(());
    }
    bail!(
        "editor command not found: {}. Set editor.command to one executable name or path without arguments.",
        config.editor.to_string_lossy()
    )
}

fn command_exists(command: &OsStr, path: Option<&OsStr>) -> bool {
    if command.as_bytes().contains(&b'/') {
        return executable_file(Path::new(command));
    }
    path.into_iter()
        .flat_map(env::split_paths)
        .any(|dir| executable_file(&dir.join(command)))
}

fn executable_file(path: &Path) -> bool {
    fs::metadata(path)
        .is_ok_and(|metadata| metadata.is_file() && metadata.permissions().mode() & 0o111 != 0)
}

fn uses_helix_bridge(command: &OsStr) -> bool {
    Path::new(command).file_name() == Some(OsStr::new("yzx-hx"))
}

fn focus_pane(config: &Config, pane_id: &str) -> Result<()> {
    let output = zellij_command(config)
        .args(["action", "focus-pane-id"])
        .arg(zellij_pane_arg(pane_id))
        .output()
        .context("could not focus editor pane")?;
    ensure_success(&output, "zellij failed to focus editor pane")
}

fn follow_originating_sidebar(
    config: &Config,
    state: &ActiveWorkspaceState,
    targets: &[PathBuf],
) -> Result<()> {
    let Some(sidebar) = &state.sidebar_yazi else {
        return Ok(());
    };
    if config.yazi_id.as_deref() != Some(&sidebar.yazi_id) {
        return Ok(());
    }
    let primary = &targets[0];
    let target_dir = if primary.is_dir() {
        primary.as_path()
    } else {
        primary.parent().unwrap_or_else(|| Path::new("/"))
    };
    let output = Command::new(&config.ya)
        .args(["emit-to", &sidebar.yazi_id, "cd"])
        .arg(target_dir)
        .output()
        .context("could not run ya")?;
    ensure_success(&output, "ya failed to follow opened target")
}

fn target_workspace_root(config: &Config, targets: &[PathBuf]) -> PathBuf {
    let target_dir = directory_target(targets).cloned().unwrap_or_else(|| {
        targets[0]
            .parent()
            .unwrap_or_else(|| Path::new("/"))
            .to_path_buf()
    });
    workspace_root(config, &target_dir)
}

fn parse_open_request(raw_targets: impl IntoIterator<Item = OsString>) -> Result<OpenRequest> {
    let mut targets = raw_targets.into_iter().collect::<Vec<_>>();
    let intent =
        if targets.first().map(OsString::as_os_str) == Some(OsStr::new("--retarget-workspace")) {
            targets.remove(0);
            OpenIntent::Retarget
        } else {
            OpenIntent::Ordinary
        };
    if targets.is_empty() {
        bail!("no target paths passed");
    }
    if intent == OpenIntent::Retarget && targets.len() != 1 {
        bail!("--retarget-workspace requires exactly one target path");
    }
    Ok(OpenRequest { intent, targets })
}

fn decide_workspace(
    intent: OpenIntent,
    current: &CanonicalWorkspace,
    candidate: &Path,
) -> WorkspaceDecision {
    match (intent, current.source) {
        (OpenIntent::Ordinary, WorkspaceSource::Explicit) => WorkspaceDecision {
            root: current.root.clone(),
            mutate: false,
        },
        _ => WorkspaceDecision {
            root: candidate.to_path_buf(),
            mutate: current.source != WorkspaceSource::Explicit || current.root != candidate,
        },
    }
}

fn active_tab_workspace(config: &Config) -> Result<ActiveWorkspaceState> {
    let raw = orchestrator_query(&orchestrator_config(config), "get_active_tab_session_state")?;
    let sidebar_yazi = optional_sidebar_yazi_state(&raw)?;
    let state = serde_json::from_str::<ActiveTabSessionState>(&raw)
        .context("pane orchestrator returned invalid active-tab session state")?;
    if state.schema_version != 1 {
        bail!(
            "pane orchestrator returned unsupported active-tab schema {}",
            state.schema_version
        );
    }
    let workspace = state
        .workspace
        .context("pane orchestrator has no workspace for the active tab")?;
    if !workspace.root.is_absolute() {
        bail!(
            "pane orchestrator returned a non-absolute workspace root: {}",
            workspace.root.display()
        );
    }
    Ok(ActiveWorkspaceState {
        active_tab_position: state.active_tab_position,
        workspace,
        sidebar_yazi,
    })
}

#[derive(Debug)]
struct ActiveWorkspaceState {
    active_tab_position: usize,
    workspace: CanonicalWorkspace,
    sidebar_yazi: Option<SidebarYaziState>,
}

fn set_workspace(config: &Config, root: &Path, source: WorkspaceSource) -> Result<()> {
    let payload = json!({
        "workspace_root": root,
        "workspace_source": source,
        "cd_focused_pane": false,
        "editor": null,
        "sidebar_yazi": null,
    })
    .to_string();
    let raw = orchestrator_pipe(&orchestrator_config(config), "retarget_workspace", &payload)?;
    let response = serde_json::from_str::<WorkspaceRetargetResponse>(&raw)
        .with_context(|| format!("pane orchestrator rejected workspace state: {raw}"))?;
    if response.status != "ok" {
        bail!("pane orchestrator rejected workspace state: {raw}");
    }
    Ok(())
}

fn orchestrator_config(config: &Config) -> OrchestratorConfig {
    OrchestratorConfig {
        ya: config.ya.clone(),
        zellij: config.zellij.clone(),
        zellij_session_name: config.zellij_session_name.clone().map(OsString::from),
    }
}

fn workspace_root(config: &Config, target_dir: &Path) -> PathBuf {
    Command::new(&config.git)
        .arg("-C")
        .arg(target_dir)
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .ok()
        .filter(|output| output.status.success())
        .and_then(|output| {
            let root = String::from_utf8_lossy(&output.stdout).trim().to_string();
            (!root.is_empty()).then(|| PathBuf::from(root))
        })
        .unwrap_or_else(|| target_dir.to_path_buf())
}

fn directory_target(targets: &[PathBuf]) -> Option<&PathBuf> {
    targets.iter().find(|target| target.is_dir())
}

fn abs_path(raw: OsString) -> Result<PathBuf> {
    std::path::absolute(PathBuf::from(raw)).context("could not absolutize target path")
}

fn zellij_pane_arg(pane_id: &str) -> String {
    if pane_id.chars().all(|ch| ch.is_ascii_digit()) {
        format!("terminal_{pane_id}")
    } else {
        pane_id.replacen("terminal:", "terminal_", 1)
    }
}

fn parse_zellij_pane_id(raw: &str) -> Option<PaneId> {
    let raw = raw.trim();
    for (prefix, is_plugin) in [
        ("terminal:", false),
        ("terminal_", false),
        ("plugin:", true),
        ("plugin_", true),
    ] {
        if let Some(id) = raw.strip_prefix(prefix) {
            return id.parse().ok().map(|id| PaneId { id, is_plugin });
        }
    }
    raw.parse().ok().map(|id| PaneId {
        id,
        is_plugin: false,
    })
}

fn ensure_success(output: &Output, context: &str) -> Result<()> {
    if output.status.success() {
        return Ok(());
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stderr = stderr.trim();
    if stderr.is_empty() {
        bail!("{context}: exit code {}", output.status.code().unwrap_or(1));
    }
    bail!("{context}: {stderr}");
}

fn log_error(config: &Config, message: &str) {
    log_event(config, LogLevel::Error, message);
}

fn log_info(config: &Config, message: &str) {
    log_event(config, LogLevel::Info, message);
}

fn log_debug(config: &Config, message: &str) {
    log_event(config, LogLevel::Debug, message);
}

fn bridge_session_id(raw: Option<String>) -> String {
    raw.filter(|id| !id.trim().is_empty())
        .unwrap_or_else(|| format!("yzx-open-{}-{}", unix_millis(), std::process::id()))
}

fn zellij_session_name_from_env() -> Option<String> {
    zellij_session_name_from_values(
        env::var(ZELLIJ_SESSION_NAME_ENV).ok(),
        env::var(YAZELIX_ZELLIJ_SESSION_NAME_ENV).ok(),
    )
}

fn zellij_session_name_from_values(
    current: Option<String>,
    saved: Option<String>,
) -> Option<String> {
    current
        .filter(|session| !session.trim().is_empty())
        .or_else(|| saved.filter(|session| !session.trim().is_empty()))
}

fn zellij_command(config: &Config) -> Command {
    let mut command = Command::new(&config.zellij);
    if let Some(session_name) = &config.zellij_session_name {
        command.env(ZELLIJ_SESSION_NAME_ENV, session_name);
    }
    command
}

impl LogLevel {
    fn from_env() -> Self {
        Self::parse(env::var("YZX_OPEN_LOG").ok().as_deref())
    }

    fn parse(raw: Option<&str>) -> Self {
        match raw.unwrap_or_default().trim().to_ascii_lowercase().as_str() {
            "off" | "0" | "false" | "none" => Self::Off,
            "error" | "errors" => Self::Error,
            "debug" | "trace" | "verbose" => Self::Debug,
            _ => Self::Info,
        }
    }

    fn allows(self, event: Self) -> bool {
        event <= self
    }
}

fn unix_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or_default()
}

fn nonempty_env(name: &str) -> Option<OsString> {
    env::var_os(name).filter(|value| !value.is_empty())
}

fn log_event(config: &Config, level: LogLevel, message: &str) {
    if !config.log_level.allows(level) {
        return;
    }
    let log_dir = config.state_dir.join("logs");
    if fs::create_dir_all(&log_dir).is_err() {
        return;
    }
    let path = log_dir.join("yzx-open.log");
    rotate_log(&path);
    let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) else {
        return;
    };
    let _ = writeln!(
        file,
        "[{}] {}",
        unix_millis(),
        message.replace('\n', "\n  ")
    );
}

fn rotate_log(path: &Path) {
    if !fs::metadata(path).is_ok_and(|metadata| metadata.len() >= LOG_MAX_BYTES) {
        return;
    }
    let rotated = path.with_extension("log.1");
    let _ = fs::remove_file(&rotated);
    let _ = fs::rename(path, rotated);
}

enum BridgeSendError {
    Unavailable,
    Rejected(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::{
        os::unix::{fs::PermissionsExt, net::UnixListener},
        thread,
    };

    static TEST_COUNTER: AtomicUsize = AtomicUsize::new(0);

    fn test_dir(_name: &str) -> PathBuf {
        let root = env::temp_dir().join(format!(
            "yo{}-{}",
            std::process::id(),
            TEST_COUNTER.fetch_add(1, Ordering::SeqCst)
        ));
        let _ = fs::remove_dir_all(&root);
        root
    }

    fn write_executable(path: &Path, contents: impl AsRef<[u8]>) {
        fs::write(path, contents).unwrap();
        fs::set_permissions(path, fs::Permissions::from_mode(0o755)).unwrap();
    }

    struct TestRuntime {
        root: PathBuf,
        zellij: PathBuf,
        zellij_log: PathBuf,
    }

    impl TestRuntime {
        fn new(name: &str) -> Self {
            let root = test_dir(name);
            fs::create_dir_all(&root).unwrap();
            write_executable(
                &root.join("ya"),
                format!(
                    "#!/bin/sh\nprintf '%s\\n' \"$*\" >> '{}'\n",
                    root.join("ya.log").display()
                ),
            );
            Self {
                zellij: root.join("zellij"),
                zellij_log: root.join("zellij.log"),
                root,
            }
        }

        fn config(&self, session_id: &str) -> Config {
            let editor = self.root.join("yzx-hx");
            if !editor.exists() {
                write_executable(&editor, "#!/bin/sh\nexit 0\n");
            }
            Config {
                editor: editor.into_os_string(),
                git: "__missing_git__".into(),
                ya: self.root.join("ya").into_os_string(),
                zellij: self.zellij.clone().into_os_string(),
                state_dir: self.root.clone(),
                session_id: session_id.into(),
                yazi_id: None,
                zellij_session_name: None,
                zellij_pane_id: None,
                log_level: LogLevel::Debug,
            }
        }

        fn write_zellij(&self, fail_focus: bool, list_panes_json: Option<&str>) {
            self.write_zellij_with_workspace(
                fail_focus,
                list_panes_json,
                &self.root,
                WorkspaceSource::Bootstrap,
                false,
            );
        }

        fn write_zellij_with_workspace(
            &self,
            fail_focus: bool,
            list_panes_json: Option<&str>,
            workspace_root: &Path,
            workspace_source: WorkspaceSource,
            fail_editor_open: bool,
        ) {
            let list_panes = list_panes_json.map_or_else(String::new, |panes| {
                format!(
                    "if [ \"$1\" = action ] && [ \"$2\" = list-panes ]; then printf '%s\\n' '{}'; exit 0; fi\n",
                    panes
                )
            });
            let session_state = json!({
                "schema_version": 1,
                "active_tab_position": 0,
                "sidebar_yazi": {
                    "yazi_id": "managed-yazi",
                    "cwd": workspace_root,
                },
                "workspace": {
                    "root": workspace_root,
                    "source": workspace_source,
                },
            });
            write_executable(
                &self.zellij,
                format!(
                    r#"#!/bin/sh
	printf 'args=%s\nsession=%s\nzellij_session=%s\n' "$*" "${{YAZELIX_HELIX_BRIDGE_SESSION_ID:-}}" "${{ZELLIJ_SESSION_NAME:-}}" >> '{}'
	{list_panes}
	if [ "$1" = action ] && [ "$2" = pipe ]; then
	  case "$*" in
	    *"--name get_active_tab_session_state"*) printf '%s\n' '{session_state}'; exit 0 ;;
	    *"--name retarget_workspace"*) printf '%s\n' '{{"status":"ok"}}'; exit 0 ;;
	  esac
	fi
	if [ "$1" = action ] && [ "$2" = focus-pane-id ] && {fail_focus}; then
  printf '%s\n' 'Pane with id Terminal(1) not found' >&2; exit 1
fi
	if [ "$1" = run ] && {fail_editor_open}; then
  printf '%s\n' 'editor pane failed' >&2; exit 1
fi
"#,
                    self.zellij_log.display(),
                    list_panes = list_panes,
                    session_state = session_state,
                    fail_focus = if fail_focus { "true" } else { "false" },
                    fail_editor_open = if fail_editor_open { "true" } else { "false" },
                ),
            );
        }

        fn write_registry(
            &self,
            session_id: &str,
            zellij_session_name: Option<&str>,
            zellij_pane_id: Option<&str>,
        ) -> (PathBuf, PathBuf) {
            let bridge_dir = self.root.join("helix_bridge").join(session_id);
            fs::create_dir_all(&bridge_dir).unwrap();
            let socket_path = bridge_dir.join("inst.sock");
            let token_path = bridge_dir.join("inst.token");
            fs::write(&token_path, "secret").unwrap();
            fs::write(
                bridge_dir.join("inst.json"),
                json!({
                    "schema_version": 2,
                    "session_id": session_id,
                    "transport": { "kind": "unix_socket", "path": &socket_path },
                    "auth_token_path": &token_path,
                    "zellij_session_name": zellij_session_name,
                    "zellij_pane_id": zellij_pane_id,
                })
                .to_string(),
            )
            .unwrap();
            (socket_path, bridge_dir.join("request.json"))
        }

        fn zellij_log(&self) -> String {
            fs::read_to_string(&self.zellij_log).unwrap()
        }
    }

    fn pane_list(entries: &[(u64, u64)]) -> String {
        json!(
            entries
                .iter()
                .map(|(id, tab_id)| json!({
                    "id": id,
                    "is_plugin": false,
                    "tab_id": tab_id,
                    "exited": false,
                }))
                .collect::<Vec<_>>()
        )
        .to_string()
    }

    fn open_main_rs(config: &Config) -> Result<()> {
        let target = config.state_dir.join("project/src/main.rs");
        fs::create_dir_all(target.parent().unwrap()).unwrap();
        fs::write(&target, "").unwrap();
        run(config, [target.into_os_string()])
    }

    fn spawn_ok_bridge(socket_path: &Path, request_path: PathBuf) -> thread::JoinHandle<()> {
        let listener = UnixListener::bind(socket_path).unwrap();
        thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut request = String::new();
            BufReader::new(&mut stream).read_line(&mut request).unwrap();
            fs::write(request_path, request).unwrap();
            writeln!(
                stream,
                r#"{{"schema_version":2,"request_id":"r","status":"ok"}}"#
            )
            .unwrap();
        })
    }

    #[test]
    fn logging_can_be_disabled_or_limited_to_errors() {
        assert_eq!(LogLevel::parse(None), LogLevel::Info);
        assert_eq!(LogLevel::parse(Some("off")), LogLevel::Off);
        assert_eq!(LogLevel::parse(Some("0")), LogLevel::Off);
        assert_eq!(LogLevel::parse(Some("error")), LogLevel::Error);
        assert_eq!(LogLevel::parse(Some("debug")), LogLevel::Debug);
        assert_eq!(LogLevel::parse(Some("wat")), LogLevel::Info);

        let runtime = TestRuntime::new("log-levels");
        let mut config = runtime.config("session");
        config.log_level = LogLevel::Off;
        log_event(&config, LogLevel::Error, "hidden");
        assert!(!runtime.root.join("logs/yzx-open.log").exists());

        config.log_level = LogLevel::Error;
        log_event(&config, LogLevel::Info, "hidden");
        log_event(&config, LogLevel::Error, "visible");
        let log = fs::read_to_string(runtime.root.join("logs/yzx-open.log")).unwrap();
        assert!(log.contains("visible"));
        assert!(!log.contains("hidden"));
    }

    #[test]
    fn logging_rotates_large_log_file() {
        let runtime = TestRuntime::new("log-rotation");
        let config = runtime.config("session");
        let log_path = runtime.root.join("logs/yzx-open.log");
        fs::create_dir_all(log_path.parent().unwrap()).unwrap();
        fs::write(&log_path, "x".repeat(LOG_MAX_BYTES as usize)).unwrap();

        log_event(&config, LogLevel::Info, "fresh event");

        let current = fs::read_to_string(&log_path).unwrap();
        let rotated = fs::read_to_string(runtime.root.join("logs/yzx-open.log.1")).unwrap();
        assert!(current.contains("fresh event"));
        assert_eq!(rotated.len(), LOG_MAX_BYTES as usize);
    }

    #[test]
    fn generated_fallback_session_id_does_not_use_shared_yzx() {
        assert_eq!(bridge_session_id(Some("window-id".into())), "window-id");
        let fallback = bridge_session_id(None);
        assert!(fallback.starts_with("yzx-open-"));
        assert!(bridge_session_id(Some(" ".into())).starts_with("yzx-open-"));
    }

    #[test]
    fn saved_zellij_session_is_used_when_yazi_hides_current_session() {
        for (current, expected) in [
            (Some("live"), Some("live")),
            (Some(""), Some("saved")),
            (Some(" "), Some("saved")),
            (None, Some("saved")),
        ] {
            assert_eq!(
                zellij_session_name_from_values(current.map(str::to_owned), Some("saved".into())),
                expected.map(str::to_owned)
            );
        }
    }

    #[test]
    fn only_yzx_hx_uses_the_yazelix_helix_bridge() {
        for command in ["yzx-hx", "/nix/store/example/bin/yzx-hx"] {
            assert!(uses_helix_bridge(OsStr::new(command)), "{command}");
        }
        for command in ["hx", "helix", "nvim", "/usr/bin/nvim"] {
            assert!(!uses_helix_bridge(OsStr::new(command)), "{command}");
        }
    }

    #[test]
    fn open_intent_and_workspace_source_define_the_only_retarget_paths() {
        let explicit = CanonicalWorkspace {
            root: "/repo".into(),
            source: WorkspaceSource::Explicit,
        };
        for candidate in [
            "/repo/docs",
            "/repo/docs/text/personal_files",
            "/repo/vendor/nested-repo",
            "/repo/non-git-tree",
        ] {
            assert_eq!(
                decide_workspace(OpenIntent::Ordinary, &explicit, Path::new(candidate)),
                WorkspaceDecision {
                    root: "/repo".into(),
                    mutate: false,
                },
                "ordinary open must preserve the canonical root for {candidate}"
            );
        }

        let bootstrap = CanonicalWorkspace {
            root: "/bootstrap".into(),
            source: WorkspaceSource::Bootstrap,
        };
        assert_eq!(
            decide_workspace(OpenIntent::Ordinary, &bootstrap, Path::new("/repo")),
            WorkspaceDecision {
                root: "/repo".into(),
                mutate: true,
            }
        );
        assert_eq!(
            decide_workspace(OpenIntent::Retarget, &explicit, Path::new("/other")),
            WorkspaceDecision {
                root: "/other".into(),
                mutate: true,
            }
        );
    }

    #[test]
    fn retarget_flag_is_explicit_and_single_target() {
        let request = parse_open_request([
            OsString::from("--retarget-workspace"),
            OsString::from("/repo"),
        ])
        .unwrap();
        assert_eq!(request.intent, OpenIntent::Retarget);
        assert_eq!(request.targets, [OsString::from("/repo")]);
        assert!(
            parse_open_request([
                OsString::from("--retarget-workspace"),
                OsString::from("/one"),
                OsString::from("/two"),
            ])
            .is_err()
        );
    }

    #[test]
    fn builds_file_and_directory_open_payloads() {
        let config = TestRuntime::new("payloads").config("session");
        let targets = vec![PathBuf::from("/tmp/project/src/main.rs")];
        let cwd = target_workspace_root(&config, &targets);
        let (action, payload) = bridge_open_request(&targets, &cwd);
        assert_eq!(action, "helix.open_files");
        assert_eq!(payload["working_dir"], "/tmp/project/src");
        assert_eq!(payload["file_paths"], json!(["/tmp/project/src/main.rs"]));
        assert_eq!(payload["focus"], true);

        let root = test_dir("directory-selection");
        let file = root.join("README.md");
        fs::create_dir_all(&root).unwrap();
        fs::write(&file, "").unwrap();
        let targets = vec![file, root.clone()];

        let cwd = target_workspace_root(&config, &targets);
        let (action, payload) = bridge_open_request(&targets, &cwd);
        assert_eq!(action, "helix.open_directory");
        assert_eq!(payload["working_dir"], root.to_string_lossy().to_string());
        assert_eq!(payload["picker_dir"], root.to_string_lossy().to_string());
        assert!(payload.get("file_paths").is_none());
    }

    #[test]
    fn bootstrap_directory_open_publishes_workspace_before_opening_editor() {
        let runtime = TestRuntime::new("workspace-root");
        let git = runtime.root.join("git");
        let repo = runtime.root.join("repo");
        let target = repo.join("docs/guides");
        fs::create_dir_all(&target).unwrap();
        runtime.write_zellij(false, None);
        write_executable(
            &git,
            format!(
                "#!/bin/sh\ncase \"$*\" in *\"rev-parse --show-toplevel\"*) printf '%s\\n' '{}'; exit 0;; esac\nexit 1\n",
                repo.display()
            ),
        );

        run(
            &Config {
                git: git.into_os_string(),
                yazi_id: Some("managed-yazi".into()),
                ..runtime.config("test-session")
            },
            [target.clone().into_os_string()],
        )
        .unwrap();

        let log = runtime.zellij_log();
        assert!(log.contains("--name retarget_workspace"), "{log}");
        assert!(
            log.contains(&format!(r#""workspace_root":"{}""#, repo.display())),
            "{log}"
        );
        assert!(!log.contains("rename-tab"), "{log}");
        assert!(
            log.contains(&format!("args=run --name editor --cwd {}", repo.display())),
            "{log}"
        );
        assert!(log.contains(target.to_string_lossy().as_ref()), "{log}");
        assert_eq!(
            fs::read_to_string(runtime.root.join("ya.log")).unwrap(),
            format!("emit-to managed-yazi cd {}\n", target.display())
        );
    }

    #[test]
    fn sends_file_open_to_live_bridge() {
        let runtime = TestRuntime::new("live-bridge");
        let session_id = "test-session";
        let (socket_path, request_path) =
            runtime.write_registry(session_id, None, Some("terminal:7"));
        let panes = pane_list(&[(3, 2), (7, 2)]);
        let workspace = runtime.root.join("project");
        runtime.write_zellij_with_workspace(
            false,
            Some(&panes),
            &workspace,
            WorkspaceSource::Explicit,
            false,
        );
        let server = spawn_ok_bridge(&socket_path, request_path.clone());
        let git_probe = runtime.root.join("git-probe");
        let git = runtime.root.join("git");
        write_executable(
            &git,
            format!("#!/bin/sh\ntouch '{}'\n", git_probe.display()),
        );

        let config = Config {
            git: git.into_os_string(),
            yazi_id: Some("managed-yazi".into()),
            zellij_pane_id: Some("terminal:3".into()),
            ..runtime.config(session_id)
        };
        let primary_dir = workspace.join("docs/personal files");
        let primary = primary_dir.join("transcription.md");
        let secondary = workspace.join("src/other.rs");
        for target in [&primary, &secondary] {
            fs::create_dir_all(target.parent().unwrap()).unwrap();
            fs::write(target, "").unwrap();
        }
        run(
            &config,
            [
                primary.clone().into_os_string(),
                secondary.clone().into_os_string(),
            ],
        )
        .unwrap();
        server.join().unwrap();

        let request: Value =
            serde_json::from_str(&fs::read_to_string(request_path).unwrap()).unwrap();
        assert_eq!(request["auth_token"], "secret");
        assert_eq!(request["action"], "helix.open_files");
        assert_eq!(
            request["payload"]["file_paths"],
            json!([primary, secondary])
        );
        assert_eq!(request["payload"]["working_dir"], json!(workspace));
        assert!(!runtime.zellij_log().contains("--name retarget_workspace"));
        assert_eq!(
            fs::read_to_string(runtime.root.join("ya.log")).unwrap(),
            format!("emit-to managed-yazi cd {}\n", primary_dir.display())
        );
        assert!(
            !git_probe.exists(),
            "ordinary explicit opens must not probe Git"
        );
    }

    #[test]
    fn host_owned_editor_bypasses_live_bridge() {
        for command in ["nvim", "hx"] {
            assert_host_editor_bypasses_live_bridge(command);
        }
    }

    fn assert_host_editor_bypasses_live_bridge(command: &str) {
        let runtime = TestRuntime::new(&format!("host-editor-bridge-bypass-{command}"));
        let session_id = "test-session";
        let (socket_path, request_path) =
            runtime.write_registry(session_id, None, Some("terminal:7"));
        let editor = runtime.root.join(command);
        let panes = pane_list(&[(3, 2), (7, 2)]);
        runtime.write_zellij(false, Some(&panes));
        write_executable(&editor, "#!/bin/sh\nexit 0\n");
        let _listener = UnixListener::bind(&socket_path).unwrap();

        open_main_rs(&Config {
            editor: editor.into_os_string(),
            zellij_pane_id: Some("terminal:3".into()),
            ..runtime.config(session_id)
        })
        .unwrap();

        let log = runtime.zellij_log();
        assert!(log.contains("args=run --name editor"), "{log}");
        assert!(!log.contains("focus-pane-id"), "{log}");
        assert!(
            !request_path.exists(),
            "{command} unexpectedly sent a Helix bridge request"
        );
        assert!(!runtime.root.join("ya.log").exists());
    }

    #[test]
    fn missing_editor_command_errors_before_opening_pane() {
        let runtime = TestRuntime::new("missing-editor");
        let editor = runtime.root.join("missing-nvim");
        runtime.write_zellij(false, None);

        let error = open_main_rs(&Config {
            editor: editor.into_os_string(),
            ..runtime.config("test-session")
        })
        .unwrap_err()
        .to_string();

        assert!(error.contains("editor command not found"), "{error}");
        let log = fs::read_to_string(&runtime.zellij_log).unwrap_or_default();
        assert!(!log.contains("args=run --name editor"), "{log}");
    }

    #[test]
    fn failed_editor_open_restores_the_previous_root_and_source() {
        let runtime = TestRuntime::new("workspace-rollback");
        let editor = runtime.root.join("nvim");
        write_executable(&editor, "#!/bin/sh\nexit 0\n");
        runtime.write_zellij_with_workspace(
            false,
            None,
            &runtime.root,
            WorkspaceSource::Bootstrap,
            true,
        );

        let error = open_main_rs(&Config {
            editor: editor.into_os_string(),
            yazi_id: Some("managed-yazi".into()),
            ..runtime.config("test-session")
        })
        .unwrap_err()
        .to_string();

        assert!(
            error.contains("zellij failed to open editor pane"),
            "{error}"
        );
        let log = runtime.zellij_log();
        assert_eq!(log.matches("--name retarget_workspace").count(), 2, "{log}");
        assert!(log.contains(r#""workspace_source":"explicit""#), "{log}");
        assert!(log.contains(r#""workspace_source":"bootstrap""#), "{log}");
        assert!(!runtime.root.join("ya.log").exists());
    }

    #[test]
    fn bridge_focus_failure_falls_back_to_new_editor_pane() {
        let runtime = TestRuntime::new("focus-fallback");
        let session_id = "test-session";
        let (socket_path, _) =
            runtime.write_registry(session_id, Some("zellij-test"), Some("terminal:1"));
        let panes = pane_list(&[(3, 1), (1, 1)]);
        runtime.write_zellij(true, Some(&panes));
        let _listener = UnixListener::bind(&socket_path).unwrap();

        open_main_rs(&Config {
            zellij_session_name: Some("zellij-test".into()),
            zellij_pane_id: Some("terminal:3".into()),
            ..runtime.config(session_id)
        })
        .unwrap();

        let log = runtime.zellij_log();
        assert!(log.contains("args=action focus-pane-id terminal_1"));
        assert!(log.contains("args=run --name editor"));
        assert!(log.contains("session=test-session"));
        assert!(log.contains("zellij_session=zellij-test"));
    }

    #[test]
    fn invalid_bridge_registry_falls_back_to_new_editor_pane() {
        let runtime = TestRuntime::new("invalid-registry");
        let config = runtime.config("test-session");
        let bridge_dir = runtime.root.join("helix_bridge/test-session");
        fs::create_dir_all(&bridge_dir).unwrap();
        fs::write(bridge_dir.join("stale.json"), "not json").unwrap();
        runtime.write_zellij(false, None);

        open_main_rs(&config).unwrap();

        assert!(runtime.zellij_log().contains("args=run --name editor"));
    }

    #[test]
    fn bridge_from_other_yzx_zellij_session_or_tab_is_not_used() {
        for (name, registry_session, registry_zellij, registry_pane, panes) in [
            ("yzx-session-isolation", "window-b", None, None, None),
            (
                "zellij-session-isolation",
                "window-a",
                Some("zellij-b"),
                None,
                None,
            ),
            (
                "zellij-tab-isolation",
                "window-a",
                Some("zellij-a"),
                Some("1"),
                Some(pane_list(&[(3, 1), (1, 0)])),
            ),
        ] {
            let runtime = TestRuntime::new(name);
            let (socket_path, _) =
                runtime.write_registry(registry_session, registry_zellij, registry_pane);
            let _listener = UnixListener::bind(&socket_path).unwrap();
            runtime.write_zellij(false, panes.as_deref());

            open_main_rs(&Config {
                zellij_session_name: Some("zellij-a".into()),
                zellij_pane_id: Some("terminal:3".into()),
                ..runtime.config("window-a")
            })
            .unwrap();

            let log = runtime.zellij_log();
            assert!(log.contains("args=run --name editor"), "{name}:\n{log}");
            assert!(log.contains("session=window-a"), "{name}:\n{log}");
            assert!(!log.contains("focus-pane-id"), "{name}:\n{log}");
        }
    }
}
