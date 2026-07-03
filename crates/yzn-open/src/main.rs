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

#[derive(Debug)]
struct Config {
    editor: OsString,
    git: OsString,
    zellij: OsString,
    state_dir: PathBuf,
    session_id: String,
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

#[derive(Debug)]
struct CurrentTabPanes {
    caller_tab_id: u64,
    panes: Vec<PaneListEntry>,
}

#[derive(Debug, Serialize)]
struct BridgeRequest<'a> {
    schema_version: u64,
    request_id: String,
    auth_token: &'a str,
    action: &'a str,
    timeout_ms: u64,
    payload: Value,
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
            eprintln!("yzn-open: {error:#}");
            ExitCode::FAILURE
        }
    }
}

fn run(config: &Config, raw_targets: impl IntoIterator<Item = OsString>) -> Result<()> {
    let targets = raw_targets
        .into_iter()
        .map(abs_path)
        .collect::<Result<Vec<_>>>()?;
    if targets.is_empty() {
        bail!("no target paths passed");
    }
    log_debug(config, &format!("targets={}", json!(targets)));
    let cwd = editor_cwd(config, &targets);
    let opened = if uses_helix_bridge(&config.editor) {
        try_bridge(config, &targets, &cwd)?
    } else {
        log_info(
            config,
            &format!(
                "bridge skipped for non-Helix editor={}",
                config.editor.to_string_lossy()
            ),
        );
        false
    };
    rename_directory_tab(config, &targets, &cwd);
    if opened {
        return Ok(());
    }
    open_editor_pane(config, &targets, &cwd)
}

impl Config {
    fn from_env() -> Self {
        let state_dir = nonempty_env("YAZELIX_STATE_DIR")
            .map(PathBuf::from)
            .or_else(|| {
                nonempty_env("XDG_DATA_HOME").map(|dir| PathBuf::from(dir).join("yazelix-next"))
            })
            .or_else(|| {
                nonempty_env("HOME").map(|dir| PathBuf::from(dir).join(".local/share/yazelix-next"))
            })
            .unwrap_or_else(|| env::temp_dir().join("yazelix-next"));

        Self {
            editor: nonempty_env("YZN_EDITOR").unwrap_or_else(|| "yzn-hx".into()),
            git: "git".into(),
            zellij: nonempty_env("YZN_ZELLIJ").unwrap_or_else(|| "zellij".into()),
            state_dir,
            session_id: bridge_session_id(env::var("YAZELIX_HELIX_BRIDGE_SESSION_ID").ok()),
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
        let registry = read_registry(&path)?;
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

    fn in_current_tab(&self, config: &Config, current_tab_panes: Option<&CurrentTabPanes>) -> bool {
        let Some(current_tab_panes) = current_tab_panes else {
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
        current_tab_panes
            .panes
            .iter()
            .any(|pane| pane.matches(pane_id) && pane.tab_id == current_tab_panes.caller_tab_id)
    }
}

fn current_tab_panes(config: &Config) -> Option<CurrentTabPanes> {
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
    Some(CurrentTabPanes {
        caller_tab_id,
        panes,
    })
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
    let request = BridgeRequest {
        schema_version: 2,
        request_id: format!("yzn-open-{}-{}", unix_millis(), std::process::id()),
        auth_token: token.trim(),
        action,
        timeout_ms: 5000,
        payload,
    };
    writeln!(
        stream,
        "{}",
        serde_json::to_string(&request)
            .map_err(|error| BridgeSendError::Rejected(error.to_string()))?
    )
    .map_err(|_| BridgeSendError::Unavailable)?;

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
    if command_exists(&config.editor, env::var_os("PATH").as_deref()) {
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
    Path::new(command).file_name() == Some(OsStr::new("yzn-hx"))
}

fn focus_pane(config: &Config, pane_id: &str) -> Result<()> {
    let output = zellij_command(config)
        .args(["action", "focus-pane-id"])
        .arg(zellij_pane_arg(pane_id))
        .output()
        .context("could not focus editor pane")?;
    ensure_success(&output, "zellij failed to focus editor pane")
}

fn rename_directory_tab(config: &Config, targets: &[PathBuf], cwd: &Path) {
    if directory_target(targets).is_some() {
        let name = project_tab_name(cwd);
        if let Err(error) = zellij_command(config)
            .args(["action", "rename-tab"])
            .arg(&name)
            .output()
            .context("could not run zellij rename-tab")
            .and_then(|output| ensure_success(&output, "zellij failed to rename tab"))
        {
            log_info(
                config,
                &format!("tab rename skipped name={}: {error:#}", json!(&name)),
            );
        }
    }
}

fn project_tab_name(path: &Path) -> String {
    path.file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.trim().is_empty())
        .map_or_else(|| path.display().to_string(), str::to_owned)
}

fn editor_cwd(config: &Config, targets: &[PathBuf]) -> PathBuf {
    let target_dir = directory_target(targets).cloned().unwrap_or_else(|| {
        targets[0]
            .parent()
            .unwrap_or_else(|| Path::new("/"))
            .to_path_buf()
    });
    workspace_root(config, &target_dir)
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
        .unwrap_or_else(|| format!("yzn-open-{}-{}", unix_millis(), std::process::id()))
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
        Self::parse(env::var("YZN_OPEN_LOG").ok().as_deref())
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
    let path = log_dir.join("yzn-open.log");
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

    fn test_dir(name: &str) -> PathBuf {
        env::temp_dir().join(format!(
            "yzn-open-{name}-{}-{}-{}",
            std::process::id(),
            unix_millis(),
            TEST_COUNTER.fetch_add(1, Ordering::SeqCst)
        ))
    }

    fn write_executable(path: &Path, contents: String) {
        fs::write(path, contents).unwrap();
        let mut permissions = fs::metadata(path).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(path, permissions).unwrap();
    }

    fn write_zellij_log_script(
        path: &Path,
        log: &Path,
        fail_focus: bool,
        list_panes_json: Option<&str>,
    ) {
        let list_panes = list_panes_json.map_or_else(String::new, |panes| {
            format!(
                r#"if [ "$1" = action ] && [ "$2" = list-panes ]; then
  printf '%s\n' '{}'
  exit 0
fi
"#,
                panes
            )
        });
        write_executable(
            path,
            format!(
                r#"#!/bin/sh
printf 'args=%s\n' "$*" >> '{}'
printf 'session=%s\n' "${{YAZELIX_HELIX_BRIDGE_SESSION_ID:-}}" >> '{}'
printf 'zellij_session=%s\n' "${{ZELLIJ_SESSION_NAME:-}}" >> '{}'
{list_panes}
if [ "$1" = action ] && [ "$2" = focus-pane-id ] && {fail_focus}; then
  printf '%s\n' 'Pane with id Terminal(1) not found' >&2
  exit 1
fi
exit 0
"#,
                log.display(),
                log.display(),
                log.display(),
                list_panes = list_panes,
                fail_focus = if fail_focus { "true" } else { "false" },
            ),
        );
    }

    fn write_git_root_script(path: &Path, root: &Path) {
        write_executable(
            path,
            format!(
                r#"#!/bin/sh
case "$*" in
  *"rev-parse --show-toplevel"*)
    printf '%s\n' '{}'
    exit 0
    ;;
esac
exit 1
"#,
                root.display(),
            ),
        );
    }

    fn write_registry(
        bridge_dir: &Path,
        session_id: &str,
        zellij_session_name: Option<&str>,
        zellij_pane_id: Option<&str>,
    ) -> PathBuf {
        fs::create_dir_all(bridge_dir).unwrap();
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
        socket_path
    }

    fn test_config(root: PathBuf, session_id: &str, zellij: impl Into<OsString>) -> Config {
        fs::create_dir_all(&root).unwrap();
        let editor = root.join("yzn-hx");
        if !editor.exists() {
            write_executable(&editor, "#!/bin/sh\nexit 0\n".to_string());
        }
        Config {
            editor: editor.into_os_string(),
            git: "__missing_git__".into(),
            zellij: zellij.into(),
            state_dir: root,
            session_id: session_id.into(),
            zellij_session_name: None,
            zellij_pane_id: None,
            log_level: LogLevel::Debug,
        }
    }

    #[test]
    fn logging_can_be_disabled_or_limited_to_errors() {
        assert_eq!(LogLevel::parse(None), LogLevel::Info);
        assert_eq!(LogLevel::parse(Some("off")), LogLevel::Off);
        assert_eq!(LogLevel::parse(Some("0")), LogLevel::Off);
        assert_eq!(LogLevel::parse(Some("error")), LogLevel::Error);
        assert_eq!(LogLevel::parse(Some("debug")), LogLevel::Debug);
        assert_eq!(LogLevel::parse(Some("wat")), LogLevel::Info);

        let root = test_dir("log-levels");
        let mut config = test_config(root.clone(), "session", "zellij");
        config.log_level = LogLevel::Off;
        log_event(&config, LogLevel::Error, "hidden");
        assert!(!root.join("logs/yzn-open.log").exists());

        config.log_level = LogLevel::Error;
        log_event(&config, LogLevel::Info, "hidden");
        log_event(&config, LogLevel::Error, "visible");
        let log = fs::read_to_string(root.join("logs/yzn-open.log")).unwrap();
        assert!(log.contains("visible"));
        assert!(!log.contains("hidden"));
    }

    #[test]
    fn logging_rotates_large_log_file() {
        let root = test_dir("log-rotation");
        let config = test_config(root.clone(), "session", "zellij");
        let log_path = root.join("logs/yzn-open.log");
        fs::create_dir_all(log_path.parent().unwrap()).unwrap();
        fs::write(&log_path, "x".repeat(LOG_MAX_BYTES as usize)).unwrap();

        log_event(&config, LogLevel::Info, "fresh event");

        let current = fs::read_to_string(&log_path).unwrap();
        let rotated = fs::read_to_string(root.join("logs/yzn-open.log.1")).unwrap();
        assert!(current.contains("fresh event"));
        assert_eq!(rotated.len(), LOG_MAX_BYTES as usize);
    }

    #[test]
    fn generated_fallback_session_id_does_not_use_shared_yzn() {
        assert_eq!(bridge_session_id(Some("window-id".into())), "window-id");
        let fallback = bridge_session_id(None);
        assert!(fallback.starts_with("yzn-open-"));
        assert!(bridge_session_id(Some(" ".into())).starts_with("yzn-open-"));
    }

    #[test]
    fn saved_zellij_session_is_used_when_yazi_hides_current_session() {
        assert_eq!(
            zellij_session_name_from_values(Some("live".into()), Some("saved".into())),
            Some("live".into())
        );
        assert_eq!(
            zellij_session_name_from_values(Some("".into()), Some("saved".into())),
            Some("saved".into())
        );
        assert_eq!(
            zellij_session_name_from_values(Some(" ".into()), Some("saved".into())),
            Some("saved".into())
        );
        assert_eq!(
            zellij_session_name_from_values(None, Some("saved".into())),
            Some("saved".into())
        );
    }

    #[test]
    fn only_yzn_hx_uses_the_yazelix_helix_bridge() {
        for command in ["yzn-hx", "/nix/store/example/bin/yzn-hx"] {
            assert!(uses_helix_bridge(OsStr::new(command)), "{command}");
        }
        for command in ["hx", "helix", "nvim", "/usr/bin/nvim"] {
            assert!(!uses_helix_bridge(OsStr::new(command)), "{command}");
        }
    }

    #[test]
    fn builds_file_and_directory_open_payloads() {
        let config = test_config(test_dir("payloads"), "session", "zellij");
        let targets = vec![PathBuf::from("/tmp/project/src/main.rs")];
        let cwd = editor_cwd(&config, &targets);
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

        let cwd = editor_cwd(&config, &targets);
        let (action, payload) = bridge_open_request(&targets, &cwd);
        assert_eq!(action, "helix.open_directory");
        assert_eq!(payload["working_dir"], root.to_string_lossy().to_string());
        assert_eq!(payload["picker_dir"], root.to_string_lossy().to_string());
        assert!(payload.get("file_paths").is_none());
    }

    #[test]
    fn directory_open_uses_workspace_root_for_tab_and_editor_cwd() {
        let root = test_dir("workspace-root");
        let zellij_log = root.join("zellij.log");
        let zellij = root.join("zellij");
        let git = root.join("git");
        let repo = root.join("repo");
        let target = repo.join("docs/guides");
        fs::create_dir_all(&target).unwrap();
        write_zellij_log_script(&zellij, &zellij_log, false, None);
        write_git_root_script(&git, &repo);

        run(
            &Config {
                git: git.into_os_string(),
                ..test_config(root.clone(), "test-session", zellij)
            },
            [target.clone().into_os_string()],
        )
        .unwrap();

        let log = fs::read_to_string(zellij_log).unwrap();
        assert!(log.contains("args=action rename-tab repo"), "{log}");
        assert!(
            log.contains(&format!("args=run --name editor --cwd {}", repo.display())),
            "{log}"
        );
        assert!(log.contains(target.to_string_lossy().as_ref()), "{log}");
    }

    #[test]
    fn sends_file_open_to_live_bridge() {
        let root = test_dir("live-bridge");
        let session_id = "test-session";
        let bridge_dir = root.join("helix_bridge").join(session_id);
        let request_path = bridge_dir.join("request.json");
        let socket_path = write_registry(&bridge_dir, session_id, None, Some("terminal:7"));
        let zellij_log = root.join("zellij.log");
        let zellij = root.join("zellij");
        let panes = json!([
            {"id": 3, "is_plugin": false, "tab_id": 2, "exited": false},
            {"id": 7, "is_plugin": false, "tab_id": 2, "exited": false},
        ])
        .to_string();
        write_zellij_log_script(&zellij, &zellij_log, false, Some(&panes));

        let listener = UnixListener::bind(&socket_path).unwrap();
        let server = thread::spawn({
            let request_path = request_path.clone();
            move || {
                let (mut stream, _) = listener.accept().unwrap();
                let mut request = String::new();
                BufReader::new(&mut stream).read_line(&mut request).unwrap();
                fs::write(request_path, request).unwrap();
                writeln!(
                    stream,
                    r#"{{"schema_version":2,"request_id":"r","status":"ok"}}"#
                )
                .unwrap();
            }
        });

        run(
            &Config {
                zellij_pane_id: Some("terminal:3".into()),
                ..test_config(root, session_id, zellij)
            },
            [OsString::from("/tmp/project/src/main.rs")],
        )
        .unwrap();
        server.join().unwrap();

        let request: Value =
            serde_json::from_str(&fs::read_to_string(request_path).unwrap()).unwrap();
        assert_eq!(request["auth_token"], "secret");
        assert_eq!(request["action"], "helix.open_files");
        assert_eq!(
            request["payload"]["file_paths"],
            json!(["/tmp/project/src/main.rs"])
        );
    }

    #[test]
    fn host_owned_editor_bypasses_live_bridge() {
        for command in ["nvim", "hx"] {
            assert_host_editor_bypasses_live_bridge(command);
        }
    }

    fn assert_host_editor_bypasses_live_bridge(command: &str) {
        let root = test_dir(&format!("host-editor-bridge-bypass-{command}"));
        let session_id = "test-session";
        let bridge_dir = root.join("helix_bridge").join(session_id);
        let request_path = bridge_dir.join("request.json");
        let socket_path = write_registry(&bridge_dir, session_id, None, Some("terminal:7"));
        let zellij_log = root.join("zellij.log");
        let zellij = root.join("zellij");
        let editor = root.join(command);
        let panes = json!([
            {"id": 3, "is_plugin": false, "tab_id": 2, "exited": false},
            {"id": 7, "is_plugin": false, "tab_id": 2, "exited": false},
        ])
        .to_string();
        write_zellij_log_script(&zellij, &zellij_log, false, Some(&panes));
        write_executable(&editor, "#!/bin/sh\nexit 0\n".to_string());
        let _listener = UnixListener::bind(&socket_path).unwrap();

        run(
            &Config {
                editor: editor.into_os_string(),
                zellij_pane_id: Some("terminal:3".into()),
                ..test_config(root.clone(), session_id, zellij)
            },
            [OsString::from("/tmp/project/src/main.rs")],
        )
        .unwrap();

        let log = fs::read_to_string(zellij_log).unwrap();
        assert!(log.contains("args=run --name editor"), "{log}");
        assert!(!log.contains("focus-pane-id"), "{log}");
        assert!(
            !request_path.exists(),
            "{command} unexpectedly sent a Helix bridge request"
        );
    }

    #[test]
    fn missing_editor_command_errors_before_opening_pane() {
        let root = test_dir("missing-editor");
        let zellij_log = root.join("zellij.log");
        let zellij = root.join("zellij");
        let editor = root.join("missing-nvim");
        fs::create_dir_all(&root).unwrap();
        write_zellij_log_script(&zellij, &zellij_log, false, None);

        let error = run(
            &Config {
                editor: editor.into_os_string(),
                ..test_config(root.clone(), "test-session", zellij)
            },
            [OsString::from("/tmp/project/src/main.rs")],
        )
        .unwrap_err()
        .to_string();

        assert!(error.contains("editor command not found"), "{error}");
        let log = fs::read_to_string(zellij_log).unwrap_or_default();
        assert!(!log.contains("args=run --name editor"), "{log}");
    }

    #[test]
    fn bridge_focus_failure_falls_back_to_new_editor_pane() {
        let root = test_dir("focus-fallback");
        let session_id = "test-session";
        let bridge_dir = root.join("helix_bridge").join(session_id);
        let zellij_log = root.join("zellij.log");
        let zellij = root.join("zellij");
        let socket_path = write_registry(
            &bridge_dir,
            session_id,
            Some("zellij-test"),
            Some("terminal:1"),
        );
        let panes = json!([
            {"id": 3, "is_plugin": false, "tab_id": 1, "exited": false},
            {"id": 1, "is_plugin": false, "tab_id": 1, "exited": false},
        ])
        .to_string();
        write_zellij_log_script(&zellij, &zellij_log, true, Some(&panes));

        let _listener = UnixListener::bind(&socket_path).unwrap();

        run(
            &Config {
                zellij_session_name: Some("zellij-test".into()),
                zellij_pane_id: Some("terminal:3".into()),
                ..test_config(root.clone(), session_id, zellij)
            },
            [OsString::from("/tmp/project/src/main.rs")],
        )
        .unwrap();

        let log = fs::read_to_string(zellij_log).unwrap();
        assert!(log.contains("args=action focus-pane-id terminal_1"));
        assert!(log.contains("args=run --name editor"));
        assert!(log.contains("session=test-session"));
        assert!(log.contains("zellij_session=zellij-test"));
    }

    #[test]
    fn bridge_from_other_yzn_zellij_session_or_tab_is_not_used() {
        for (name, registry_session, registry_zellij, registry_pane, panes) in [
            ("yzn-session-isolation", "window-b", None, None, None),
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
                Some(
                    json!([
                        {"id": 3, "is_plugin": false, "tab_id": 1, "exited": false},
                        {"id": 1, "is_plugin": false, "tab_id": 0, "exited": false},
                    ])
                    .to_string(),
                ),
            ),
        ] {
            let root = test_dir(name);
            let bridge_dir = root.join("helix_bridge").join(registry_session);
            let zellij_log = root.join("zellij.log");
            let zellij = root.join("zellij");
            let socket_path = write_registry(
                &bridge_dir,
                registry_session,
                registry_zellij,
                registry_pane,
            );
            let _listener = UnixListener::bind(&socket_path).unwrap();
            write_zellij_log_script(&zellij, &zellij_log, false, panes.as_deref());

            run(
                &Config {
                    zellij_session_name: Some("zellij-a".into()),
                    zellij_pane_id: Some("terminal:3".into()),
                    ..test_config(root, "window-a", zellij)
                },
                [OsString::from("/tmp/project/src/main.rs")],
            )
            .unwrap();

            let log = fs::read_to_string(zellij_log).unwrap();
            assert!(log.contains("args=run --name editor"), "{name}:\n{log}");
            assert!(log.contains("session=window-a"), "{name}:\n{log}");
            assert!(!log.contains("focus-pane-id"), "{name}:\n{log}");
        }
    }
}
