use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
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

fn main() -> ExitCode {
    let config = Config::from_env();
    match run(&config, env::args_os().skip(1)) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            log_event(&config, &format!("error: {error:#}"));
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
    log_event(config, &format!("targets={}", json!(targets)));
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
            session_id: env::var("YAZELIX_HELIX_BRIDGE_SESSION_ID")
                .unwrap_or_else(|_| "yzn".into()),
            zellij_session_name: env::var("ZELLIJ_SESSION_NAME").ok(),
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
            continue;
        }
        if !registry.matches_session(config) {
            log_event(
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
            continue;
        }

        if let Some(pane_id) = &registry.zellij_pane_id {
            log_event(
                config,
                &format!(
                    "bridge focus probe registry={} pane={pane_id}",
                    path.display()
                ),
            );
            if let Err(error) = focus_pane(config, pane_id) {
                log_event(
                    config,
                    &format!("focus failed pane={pane_id}; treating bridge as stale: {error:#}"),
                );
                continue;
            }
        }

        let (action, payload) = bridge_open_request(targets);
        log_event(
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
                log_event(config, &format!("bridge ok response={}", response.trim()));
                return Ok(true);
            }
            Err(BridgeSendError::Unavailable) => {
                log_event(
                    config,
                    &format!("bridge unavailable registry={}", path.display()),
                );
                continue;
            }
            Err(BridgeSendError::Rejected(message)) => {
                log_event(config, &format!("bridge rejected: {message}"));
                bail!(message);
            }
        }
    }

    log_event(config, "no live bridge found; opening a new editor pane");
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

    log_event(
        config,
        &format!(
            "opening editor pane program={} args={}",
            config.zellij.to_string_lossy(),
            json!(display_args(&args))
        ),
    );

    let output = Command::new(&config.zellij)
        .args(&args)
        .output()
        .context("could not run zellij")?;
    log_command_output(config, "open editor pane", &output);
    ensure_success(&output, "zellij failed to open editor pane")?;
    log_event(config, "editor pane opened");
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

fn command_status(output: &Output) -> String {
    output
        .status
        .code()
        .map(|code| code.to_string())
        .unwrap_or_else(|| "signal".into())
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

fn log_command_output(config: &Config, label: &str, output: &Output) {
    log_event(
        config,
        &format!(
            "{label} status={} stdout={} stderr={}",
            command_status(output),
            json!(String::from_utf8_lossy(&output.stdout).trim()),
            json!(String::from_utf8_lossy(&output.stderr).trim())
        ),
    );
}

fn is_socket(path: &Path) -> bool {
    fs::metadata(path)
        .map(|metadata| metadata.file_type().is_socket())
        .unwrap_or(false)
}

fn request_id() -> String {
    format!("yzn-open-{}-{}", unix_millis(), std::process::id())
}

fn unix_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or_default()
}

fn log_event(config: &Config, message: &str) {
    let log_dir = config.state_dir.join("logs");
    if fs::create_dir_all(&log_dir).is_err() {
        return;
    }
    let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_dir.join("yzn-open.log"))
    else {
        return;
    };
    let _ = writeln!(
        file,
        "[{}] {}",
        unix_millis(),
        message.replace('\n', "\n  ")
    );
}

enum BridgeSendError {
    Unavailable,
    Rejected(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::{os::unix::net::UnixListener, thread};

    static TEST_COUNTER: AtomicUsize = AtomicUsize::new(0);

    fn test_dir(name: &str) -> PathBuf {
        env::temp_dir().join(format!(
            "yzn-open-{name}-{}-{}-{}",
            std::process::id(),
            unix_millis(),
            TEST_COUNTER.fetch_add(1, Ordering::SeqCst)
        ))
    }

    #[test]
    fn normalizes_zellij_typed_pane_ids() {
        assert_eq!(zellij_pane_arg("terminal:7"), "terminal_7");
        assert_eq!(zellij_pane_arg("7"), "terminal_7");
    }

    #[test]
    fn builds_file_open_payload() {
        let targets = vec![PathBuf::from("/tmp/project/src/main.rs")];
        let (action, payload) = bridge_open_request(&targets);
        assert_eq!(action, "helix.open_files");
        assert_eq!(payload["working_dir"], "/tmp/project/src");
        assert_eq!(payload["file_paths"], json!(["/tmp/project/src/main.rs"]));
        assert_eq!(payload["focus"], true);
    }

    #[test]
    fn directory_selection_uses_directory_open() {
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
            &Config {
                editor: "unused".into(),
                zellij: "unused".into(),
                state_dir: root,
                session_id: session_id.into(),
                zellij_session_name: None,
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
    fn bridge_focus_failure_falls_back_to_new_editor_pane() {
        use std::os::unix::fs::PermissionsExt;

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
                "zellij_pane_id": "1",
            })
            .to_string(),
        )
        .unwrap();
        fs::write(
            &zellij,
            format!(
                r#"#!/bin/sh
printf '%s\n' "$*" >> '{}'
if [ "$1" = action ] && [ "$2" = focus-pane-id ]; then
  printf '%s\n' 'Pane with id Terminal(1) not found' >&2
  exit 1
fi
exit 0
"#,
                zellij_log.display()
            ),
        )
        .unwrap();
        let mut permissions = fs::metadata(&zellij).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&zellij, permissions).unwrap();

        let _listener = UnixListener::bind(&socket_path).unwrap();

        run(
            &Config {
                editor: "hx".into(),
                zellij: zellij.into(),
                state_dir: root.clone(),
                session_id: session_id.into(),
                zellij_session_name: Some("zellij-test".into()),
            },
            [OsString::from("/tmp/project/src/main.rs")],
        )
        .unwrap();

        let log = fs::read_to_string(zellij_log).unwrap();
        assert!(log.contains("action focus-pane-id terminal_1"));
        assert!(log.contains("run --name yzn-editor"));
    }
}
