use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    env,
    ffi::OsString,
    fs::{self, OpenOptions},
    io::{BufRead, BufReader, Write},
    path::{Path, PathBuf},
    process::{Command, ExitCode, Output},
    time::{SystemTime, UNIX_EPOCH},
};

use std::{os::unix::fs::FileTypeExt, os::unix::net::UnixStream};

#[derive(Debug)]
struct Config {
    editor: OsString,
    zellij: OsString,
    state_dir: PathBuf,
    session_id: String,
    zellij_session_name: Option<String>,
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
    if try_bridge(config, &targets)? {
        return Ok(());
    }
    open_editor_pane(config, &targets)
}

impl Config {
    fn from_env() -> Self {
        let state_dir = env::var_os("YAZELIX_STATE_DIR")
            .map(PathBuf::from)
            .or_else(|| {
                env::var_os("XDG_RUNTIME_DIR").map(|dir| PathBuf::from(dir).join("yazelix-next"))
            })
            .unwrap_or_else(|| env::temp_dir().join("yazelix-next"));

        Self {
            editor: env::var_os("YZN_EDITOR").unwrap_or_else(|| "yzn-hx".into()),
            zellij: env::var_os("YZN_ZELLIJ").unwrap_or_else(|| "zellij".into()),
            state_dir,
            session_id: bridge_session_id(env::var("YAZELIX_HELIX_BRIDGE_SESSION_ID").ok()),
            zellij_session_name: env::var("ZELLIJ_SESSION_NAME").ok(),
            log_level: LogLevel::from_env(),
        }
    }
}

fn try_bridge(config: &Config, targets: &[PathBuf]) -> Result<bool> {
    let bridge_dir = config
        .state_dir
        .join("helix_bridge")
        .join(&config.session_id);
    let Ok(entries) = fs::read_dir(&bridge_dir) else {
        return Ok(false);
    };

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

        let (action, payload) = bridge_open_request(targets);
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

        match registry.send_request(action, payload) {
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
                _ => true,
            }
    }

    fn is_live(&self) -> bool {
        is_socket(&self.transport.path) && self.auth_token_path.is_file()
    }

    fn send_request(
        &self,
        action: &'static str,
        payload: Value,
    ) -> std::result::Result<String, BridgeSendError> {
        send_bridge_request(&self.transport.path, &self.auth_token_path, action, payload)
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
        request_id: request_id(),
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

fn bridge_open_request(targets: &[PathBuf]) -> (&'static str, Value) {
    let working_dir = editor_cwd(targets);
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

fn open_editor_pane(config: &Config, targets: &[PathBuf]) -> Result<()> {
    let mut args = vec![
        OsString::from("run"),
        OsString::from("--name"),
        OsString::from("yzn-editor"),
        OsString::from("--cwd"),
        editor_cwd(targets).into_os_string(),
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
            json!(display_args(&args))
        ),
    );

    let output = Command::new(&config.zellij)
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

fn focus_pane(config: &Config, pane_id: &str) -> Result<()> {
    let output = Command::new(&config.zellij)
        .args(["action", "focus-pane-id"])
        .arg(zellij_pane_arg(pane_id))
        .output()
        .context("could not focus editor pane")?;
    ensure_success(&output, "zellij failed to focus editor pane")
}

fn editor_cwd(targets: &[PathBuf]) -> PathBuf {
    if let Some(target) = directory_target(targets) {
        return target.clone();
    }
    let first = &targets[0];
    first
        .parent()
        .unwrap_or_else(|| Path::new("/"))
        .to_path_buf()
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

fn display_args(args: &[OsString]) -> Vec<String> {
    args.iter()
        .map(|arg| arg.to_string_lossy().into_owned())
        .collect()
}

fn command_error(output: &Output) -> String {
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    if stderr.is_empty() {
        format!("exit code {}", output.status.code().unwrap_or(1))
    } else {
        stderr
    }
}

fn ensure_success(output: &Output, context: &str) -> Result<()> {
    if output.status.success() {
        Ok(())
    } else {
        bail!("{context}: {}", command_error(output));
    }
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

fn is_socket(path: &Path) -> bool {
    fs::metadata(path)
        .map(|metadata| metadata.file_type().is_socket())
        .unwrap_or(false)
}

fn request_id() -> String {
    format!("yzn-open-{}-{}", unix_millis(), std::process::id())
}

fn bridge_session_id(raw: Option<String>) -> String {
    raw.filter(|id| !id.trim().is_empty())
        .unwrap_or_else(|| format!("yzn-open-{}-{}", unix_millis(), std::process::id()))
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

    fn write_zellij_log_script(path: &Path, log: &Path, fail_focus: bool) {
        fs::write(
            path,
            format!(
                r#"#!/bin/sh
printf 'args=%s\n' "$*" >> '{}'
printf 'session=%s\n' "${{YAZELIX_HELIX_BRIDGE_SESSION_ID:-}}" >> '{}'
if [ "$1" = action ] && [ "$2" = focus-pane-id ] && {fail_focus}; then
  printf '%s\n' 'Pane with id Terminal(1) not found' >&2
  exit 1
fi
exit 0
"#,
                log.display(),
                log.display(),
                fail_focus = if fail_focus { "true" } else { "false" },
            ),
        )
        .unwrap();
        let mut permissions = fs::metadata(path).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(path, permissions).unwrap();
    }

    fn test_config(root: PathBuf, session_id: &str, zellij: impl Into<OsString>) -> Config {
        Config {
            editor: "hx".into(),
            zellij: zellij.into(),
            state_dir: root,
            session_id: session_id.into(),
            zellij_session_name: None,
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
        assert_ne!(fallback, "yzn");
        assert!(bridge_session_id(Some(" ".into())).starts_with("yzn-open-"));
    }

    #[test]
    fn builds_file_and_directory_open_payloads() {
        let targets = vec![PathBuf::from("/tmp/project/src/main.rs")];
        let (action, payload) = bridge_open_request(&targets);
        assert_eq!(action, "helix.open_files");
        assert_eq!(payload["working_dir"], "/tmp/project/src");
        assert_eq!(payload["file_paths"], json!(["/tmp/project/src/main.rs"]));
        assert_eq!(payload["focus"], true);

        let root = test_dir("directory-selection");
        let file = root.join("README.md");
        fs::create_dir_all(&root).unwrap();
        fs::write(&file, "").unwrap();
        let targets = vec![file, root.clone()];

        let (action, payload) = bridge_open_request(&targets);
        assert_eq!(action, "helix.open_directory");
        assert_eq!(payload["working_dir"], root.to_string_lossy().to_string());
        assert_eq!(payload["picker_dir"], root.to_string_lossy().to_string());
        assert!(payload.get("file_paths").is_none());
    }

    #[test]
    fn sends_file_open_to_live_bridge() {
        let root = test_dir("live-bridge");
        let session_id = "test-session";
        let bridge_dir = root.join("helix_bridge").join(session_id);
        fs::create_dir_all(&bridge_dir).unwrap();
        let socket_path = bridge_dir.join("inst.sock");
        let token_path = bridge_dir.join("inst.token");
        let request_path = bridge_dir.join("request.json");
        fs::write(&token_path, "secret").unwrap();
        fs::write(
            bridge_dir.join("inst.json"),
            json!({
                "schema_version": 2,
                "session_id": session_id,
                "transport": { "kind": "unix_socket", "path": &socket_path },
                "auth_token_path": &token_path,
            })
            .to_string(),
        )
        .unwrap();

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
            &test_config(root, session_id, "unused"),
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
    fn bridge_focus_failure_falls_back_to_new_editor_pane() {
        let root = test_dir("focus-fallback");
        let session_id = "test-session";
        let bridge_dir = root.join("helix_bridge").join(session_id);
        fs::create_dir_all(&bridge_dir).unwrap();
        let socket_path = bridge_dir.join("inst.sock");
        let token_path = bridge_dir.join("inst.token");
        let zellij_log = root.join("zellij.log");
        let zellij = root.join("zellij");
        fs::write(&token_path, "secret").unwrap();
        fs::write(
            bridge_dir.join("inst.json"),
            json!({
                "schema_version": 2,
                "session_id": session_id,
                "instance_id": "inst",
                "transport": { "kind": "unix_socket", "path": &socket_path },
                "auth_token_path": &token_path,
                "zellij_session_name": "zellij-test",
                "zellij_pane_id": "terminal:1",
            })
            .to_string(),
        )
        .unwrap();
        write_zellij_log_script(&zellij, &zellij_log, true);

        let _listener = UnixListener::bind(&socket_path).unwrap();

        run(
            &Config {
                zellij_session_name: Some("zellij-test".into()),
                ..test_config(root.clone(), session_id, zellij)
            },
            [OsString::from("/tmp/project/src/main.rs")],
        )
        .unwrap();

        let log = fs::read_to_string(zellij_log).unwrap();
        assert!(log.contains("args=action focus-pane-id terminal_1"));
        assert!(log.contains("args=run --name yzn-editor"));
        assert!(log.contains("session=test-session"));
    }

    #[test]
    fn bridge_from_another_yzn_session_is_not_used() {
        let root = test_dir("session-isolation");
        let other_bridge_dir = root.join("helix_bridge").join("window-b");
        fs::create_dir_all(&other_bridge_dir).unwrap();
        let socket_path = other_bridge_dir.join("inst.sock");
        let token_path = other_bridge_dir.join("inst.token");
        let zellij_log = root.join("zellij.log");
        let zellij = root.join("zellij");
        fs::write(&token_path, "secret").unwrap();
        fs::write(
            other_bridge_dir.join("inst.json"),
            json!({
                "schema_version": 2,
                "session_id": "window-b",
                "transport": { "kind": "unix_socket", "path": &socket_path },
                "auth_token_path": &token_path,
            })
            .to_string(),
        )
        .unwrap();
        let _listener = UnixListener::bind(&socket_path).unwrap();
        write_zellij_log_script(&zellij, &zellij_log, false);

        run(
            &Config {
                zellij_session_name: Some("zellij-a".into()),
                ..test_config(root, "window-a", zellij)
            },
            [OsString::from("/tmp/project/src/main.rs")],
        )
        .unwrap();

        let log = fs::read_to_string(zellij_log).unwrap();
        assert!(log.contains("args=run --name yzn-editor"));
        assert!(log.contains("session=window-a"));
        assert!(!log.contains("focus-pane-id"));
    }

    #[test]
    fn bridge_from_another_zellij_session_is_not_used() {
        let root = test_dir("zellij-session-isolation");
        let session_id = "window-a";
        let bridge_dir = root.join("helix_bridge").join(session_id);
        fs::create_dir_all(&bridge_dir).unwrap();
        let socket_path = bridge_dir.join("inst.sock");
        let token_path = bridge_dir.join("inst.token");
        let zellij_log = root.join("zellij.log");
        let zellij = root.join("zellij");
        fs::write(&token_path, "secret").unwrap();
        fs::write(
            bridge_dir.join("inst.json"),
            json!({
                "schema_version": 2,
                "session_id": session_id,
                "transport": { "kind": "unix_socket", "path": &socket_path },
                "auth_token_path": &token_path,
                "zellij_session_name": "zellij-b",
                "zellij_pane_id": "1",
            })
            .to_string(),
        )
        .unwrap();
        let _listener = UnixListener::bind(&socket_path).unwrap();
        write_zellij_log_script(&zellij, &zellij_log, false);

        run(
            &Config {
                zellij_session_name: Some("zellij-a".into()),
                ..test_config(root, session_id, zellij)
            },
            [OsString::from("/tmp/project/src/main.rs")],
        )
        .unwrap();

        let log = fs::read_to_string(zellij_log).unwrap();
        assert!(log.contains("args=run --name yzn-editor"));
        assert!(log.contains("session=window-a"));
        assert!(!log.contains("focus-pane-id"));
    }
}
