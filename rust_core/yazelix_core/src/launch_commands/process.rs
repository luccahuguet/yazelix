use crate::bridge::{CoreError, ErrorClass};
use serde_json::{Map as JsonMap, Value as JsonValue};
use std::collections::HashSet;
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

fn get_launch_probe_log_path(state_dir: &Path, terminal_name: &str) -> Result<PathBuf, CoreError> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| {
            CoreError::classified(
                ErrorClass::Internal,
                "system_clock_error",
                format!("System clock error while preparing detached launch log path: {error}"),
                "Fix the system clock, then retry.",
                serde_json::json!({}),
            )
        })?
        .as_millis();
    let sanitized = terminal_name
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect::<String>()
        .trim_matches('_')
        .to_string();
    let log_dir = state_dir.join("logs").join("terminal_launch");
    fs::create_dir_all(&log_dir).map_err(|source| {
        CoreError::io(
            "launch_log_dir",
            format!(
                "Could not create launch log directory {}.",
                log_dir.display()
            ),
            "Fix the directory permissions, then retry.",
            log_dir.display().to_string(),
            source,
        )
    })?;
    Ok(log_dir.join(format!("{}_{}.log", sanitized, timestamp)))
}

pub(super) fn run_detached_launch_probe(
    runtime_dir: &Path,
    state_dir: &Path,
    launch_argv: &[String],
    runtime_env: &JsonMap<String, JsonValue>,
    cwd: &Path,
    needs_reload: bool,
    env_removals: &[&str],
    extra_env: &[(String, Option<String>)],
) -> Result<Output, CoreError> {
    let probe_helper = runtime_dir
        .join("shells")
        .join("posix")
        .join("detached_launch_probe.sh");
    if !probe_helper.is_file() {
        return Err(CoreError::classified(
            ErrorClass::Runtime,
            "missing_detached_launch_probe",
            format!(
                "Cannot launch terminals: detached launch helper is missing at {}.",
                probe_helper.display()
            ),
            "Restore shells/posix/detached_launch_probe.sh or reinstall Yazelix.",
            serde_json::json!({}),
        ));
    }

    let log_path = get_launch_probe_log_path(
        state_dir,
        launch_argv
            .first()
            .map(String::as_str)
            .unwrap_or("terminal"),
    )?;
    let mut argv = vec![
        probe_helper.to_string_lossy().into_owned(),
        log_path.to_string_lossy().into_owned(),
    ];
    if needs_reload {
        argv.push("--reload".to_string());
    }
    argv.push("--".to_string());
    argv.extend(launch_argv.iter().cloned());
    command_output_with_overrides(
        &argv,
        Some(runtime_env),
        cwd,
        env_removals,
        extra_env,
        "detached_launch_probe",
        "Retry with a valid configured terminal or reinstall Yazelix so the detached launch helper is present.",
    )
}

pub(super) fn run_desktop_deferred_launch_probe(
    runtime_dir: &Path,
    state_dir: &Path,
    launch_argv: &[String],
    runtime_env: &JsonMap<String, JsonValue>,
    cwd: &Path,
    needs_reload: bool,
    env_removals: &[&str],
    extra_env: &[(String, Option<String>)],
) -> Result<Output, CoreError> {
    let probe_helper = desktop_deferred_launch_probe_path(runtime_dir);
    if !probe_helper.is_file() {
        return Err(CoreError::classified(
            ErrorClass::Runtime,
            "missing_desktop_deferred_launch_probe",
            format!(
                "Cannot launch from desktop: deferred launch helper is missing at {}.",
                probe_helper.display()
            ),
            "Restore shells/posix/desktop_deferred_launch_probe.sh or reinstall Yazelix.",
            serde_json::json!({}),
        ));
    }

    let log_path = get_launch_probe_log_path(
        state_dir,
        launch_argv
            .first()
            .map(String::as_str)
            .unwrap_or("terminal"),
    )?;
    let mut argv = vec![
        probe_helper.to_string_lossy().into_owned(),
        log_path.to_string_lossy().into_owned(),
        std::process::id().to_string(),
    ];
    if needs_reload {
        argv.push("--reload".to_string());
    }
    argv.push("--".to_string());
    argv.extend(launch_argv.iter().cloned());
    command_output_with_overrides(
        &argv,
        Some(runtime_env),
        cwd,
        env_removals,
        extra_env,
        "desktop_deferred_launch_probe",
        "Retry with a valid configured terminal or reinstall Yazelix so the deferred desktop launch helper is present.",
    )
}

pub(super) fn desktop_deferred_launch_probe_path(runtime_dir: &Path) -> PathBuf {
    runtime_dir
        .join("shells")
        .join("posix")
        .join("desktop_deferred_launch_probe.sh")
}

pub(super) fn render_launch_failure(output: &Output) -> String {
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let logged_path = stdout
        .lines()
        .map(str::trim)
        .rev()
        .find(|line| !line.is_empty() && Path::new(line).exists())
        .map(PathBuf::from);
    if let Some(path) = logged_path {
        if let Ok(raw) = fs::read_to_string(&path) {
            let tail = raw
                .lines()
                .rev()
                .take(10)
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect::<Vec<_>>()
                .join(" ");
            if !tail.trim().is_empty() {
                return tail.trim().to_string();
            }
        }
    }

    let stderr = stderr.trim();
    if !stderr.is_empty() {
        stderr.to_string()
    } else {
        format!("exit code {}", output.status.code().unwrap_or(1))
    }
}

pub(super) fn command_output_with_overrides(
    argv: &[String],
    runtime_env: Option<&JsonMap<String, JsonValue>>,
    cwd: &Path,
    env_removals: &[&str],
    extra_env: &[(String, Option<String>)],
    owner: &str,
    remediation: &str,
) -> Result<Output, CoreError> {
    let (command, args) = argv
        .split_first()
        .ok_or_else(|| CoreError::usage("Missing command argv"))?;
    let mut cmd = Command::new(command);
    cmd.args(args);
    configure_command_env(&mut cmd, runtime_env, cwd, env_removals, extra_env);
    cmd.output().map_err(|source| {
        CoreError::io(
            owner,
            format!("Failed to launch {owner}."),
            remediation,
            command.clone(),
            source,
        )
    })
}

pub(super) fn command_status_with_overrides(
    argv: &[String],
    runtime_env: Option<&JsonMap<String, JsonValue>>,
    cwd: &Path,
    env_removals: &[&str],
    extra_env: &[(String, Option<String>)],
    owner: &str,
    remediation: &str,
) -> Result<std::process::ExitStatus, CoreError> {
    let (command, args) = argv
        .split_first()
        .ok_or_else(|| CoreError::usage("Missing command argv"))?;
    let mut cmd = Command::new(command);
    cmd.args(args);
    configure_command_env(&mut cmd, runtime_env, cwd, env_removals, extra_env);
    cmd.status().map_err(|source| {
        CoreError::io(
            owner,
            format!("Failed to launch {owner}."),
            remediation,
            command.clone(),
            source,
        )
    })
}

fn configure_command_env(
    cmd: &mut Command,
    runtime_env: Option<&JsonMap<String, JsonValue>>,
    cwd: &Path,
    env_removals: &[&str],
    extra_env: &[(String, Option<String>)],
) {
    let removals: HashSet<&str> = env_removals.iter().copied().collect();
    cmd.current_dir(cwd);
    cmd.env_clear();
    for (key, value) in std::env::vars_os() {
        if removals.contains(key.to_string_lossy().as_ref()) {
            continue;
        }
        cmd.env(&key, &value);
    }
    if let Some(runtime_env) = runtime_env {
        for (key, value) in runtime_env {
            if let Some(text) = runtime_env_value(value) {
                cmd.env(key, text);
            } else {
                cmd.env_remove(key);
            }
        }
    }
    for (key, value) in extra_env {
        if let Some(value) = value {
            cmd.env(key, value);
        } else {
            cmd.env_remove(key);
        }
    }
}

fn runtime_env_value(value: &JsonValue) -> Option<OsString> {
    match value {
        JsonValue::Null => None,
        JsonValue::String(text) => Some(OsString::from(text)),
        JsonValue::Bool(flag) => Some(OsString::from(flag.to_string())),
        JsonValue::Number(number) => Some(OsString::from(number.to_string())),
        JsonValue::Array(items) => Some(OsString::from(
            items
                .iter()
                .filter_map(JsonValue::as_str)
                .collect::<Vec<_>>()
                .join(if cfg!(windows) { ";" } else { ":" }),
        )),
        JsonValue::Object(_) => Some(OsString::from(value.to_string())),
    }
}

pub(super) fn print_completed_output(output: &Output) {
    if !output.stdout.is_empty() {
        print!("{}", String::from_utf8_lossy(&output.stdout));
    }
    if !output.stderr.is_empty() {
        eprint!("{}", String::from_utf8_lossy(&output.stderr));
    }
}

pub(super) fn find_command(name: &str) -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    std::env::split_paths(&path)
        .map(|entry| entry.join(name))
        .find(|candidate| candidate.is_file())
}
