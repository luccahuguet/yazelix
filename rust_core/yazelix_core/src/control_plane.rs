//! Shared logic for the `yzx_control` CLI (`yzx env` / `yzx run`).

use crate::bridge::{CoreError, ErrorClass};
use crate::runtime_env::RuntimePathInput;
use crate::{normalize_config, NormalizeConfigRequest, RuntimeEnvComputeRequest};
use serde_json::{Map as JsonMap, Value as JsonValue};
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::Command;

const DEFAULT_SHELL: &str = "nu";

#[derive(Debug, Clone, Default)]
pub struct EnvCliArgs {
    pub no_shell: bool,
}

/// Parse flags for `yzx_control env` (tokens after the `env` subcommand).
pub fn parse_env_cli_args(args: &[String]) -> Result<EnvCliArgs, CoreError> {
    let mut out = EnvCliArgs::default();
    for token in args {
        match token.as_str() {
            "--no-shell" | "-n" => out.no_shell = true,
            other => {
                return Err(CoreError::classified(
                    ErrorClass::Usage,
                    "unexpected_env_token",
                    format!("Unexpected argument for yzx env: {other}"),
                    "Run `yzx env` or `yzx env --no-shell`.",
                    serde_json::json!({}),
                ));
            }
        }
    }
    Ok(out)
}

/// `argv` is the full child invocation: `["cargo", "--verbose", "check"]`.
pub fn split_run_argv(argv: &[String]) -> Result<(&str, &[String]), CoreError> {
    let cmd = argv.first().ok_or_else(|| {
        CoreError::classified(
            ErrorClass::Usage,
            "missing_run_command",
            "Error: No command provided",
            "Usage: yzx run <command> [args...]",
            serde_json::json!({}),
        )
    })?;
    Ok((cmd.as_str(), &argv[1..]))
}

pub fn shell_command(login: bool, shell_name: &str) -> Vec<String> {
    let normalized = shell_name.to_lowercase();
    match (normalized.as_str(), login) {
        ("nu", true) => vec!["nu".into(), "--login".into()],
        ("bash", true) => vec!["bash".into(), "--login".into()],
        ("fish", true) => vec!["fish".into(), "-l".into()],
        ("zsh", true) => vec!["zsh".into(), "-l".into()],
        ("nu", false) => vec!["nu".into()],
        ("bash", false) => vec!["bash".into()],
        ("fish", false) => vec!["fish".into()],
        ("zsh", false) => vec!["zsh".into()],
        (_, true) => vec![normalized],
        (_, false) => vec![normalized],
    }
}

pub fn basename_shell(shell_env: Option<&str>) -> Option<String> {
    let raw = shell_env?.trim();
    if raw.is_empty() {
        return None;
    }
    Path::new(raw)
        .file_name()?
        .to_str()
        .map(|s| s.to_lowercase())
}

pub fn default_shell_from_config(normalized: &JsonMap<String, JsonValue>) -> String {
    normalized
        .get("default_shell")
        .and_then(|v| v.as_str())
        .unwrap_or(DEFAULT_SHELL)
        .to_lowercase()
}

pub fn runtime_env_request(
    runtime_dir: PathBuf,
    normalized: &JsonMap<String, JsonValue>,
) -> Result<RuntimeEnvComputeRequest, CoreError> {
    let home_dir = std::env::var_os("HOME").map(PathBuf::from).ok_or_else(|| {
        CoreError::classified(
            ErrorClass::Runtime,
            "missing_home",
            "HOME is not set; cannot build the Yazelix runtime environment.",
            "Export HOME, then retry.",
            serde_json::json!({}),
        )
    })?;

    let current_path = std::env::var("PATH").unwrap_or_default();
    let enable_sidebar = normalized
        .get("enable_sidebar")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);
    let editor_command = normalized
        .get("editor_command")
        .and_then(|v| v.as_str())
        .map(String::from);
    let helix_runtime_path = normalized
        .get("helix_runtime_path")
        .and_then(|v| v.as_str())
        .map(String::from);

    Ok(RuntimeEnvComputeRequest {
        runtime_dir,
        home_dir,
        current_path: RuntimePathInput::String(current_path),
        enable_sidebar,
        editor_command,
        helix_runtime_path,
    })
}

pub fn json_map_to_child_env(map: &JsonMap<String, JsonValue>) -> Vec<(OsString, OsString)> {
    let mut out = Vec::new();
    for (k, v) in map {
        let value = match v {
            JsonValue::String(s) => s.clone(),
            JsonValue::Array(items) => items
                .iter()
                .filter_map(|item| item.as_str())
                .collect::<Vec<_>>()
                .join(if cfg!(windows) { ";" } else { ":" }),
            _ => continue,
        };
        out.push((OsString::from(k), OsString::from(value)));
    }
    out
}

pub fn load_normalized_config_for_control(
    runtime_dir: &Path,
    config_dir: &Path,
    config_override: Option<&str>,
) -> Result<JsonMap<String, JsonValue>, CoreError> {
    let paths = crate::active_config_surface::resolve_active_config_paths(
        runtime_dir,
        config_dir,
        config_override,
    )?;
    let data = normalize_config(&NormalizeConfigRequest {
        config_path: paths.config_file.clone(),
        default_config_path: paths.default_config_path.clone(),
        contract_path: paths.contract_path.clone(),
        include_missing: false,
    })?;
    Ok(data.normalized_config)
}

pub fn run_child_in_runtime_env(
    argv: &[String],
    runtime_env: &JsonMap<String, JsonValue>,
    cwd: &Path,
) -> Result<std::process::ExitStatus, CoreError> {
    let (cmd, args) = split_run_argv(argv)?;
    let mut c = Command::new(cmd);
    c.args(args);
    c.current_dir(cwd);
    c.env_clear();
    for (k, v) in std::env::vars_os() {
        c.env(k, v);
    }
    for (k, v) in json_map_to_child_env(runtime_env) {
        c.env(k, v);
    }
    c.status().map_err(|source| {
        CoreError::io(
            "run_spawn",
            "Could not run the requested command in the Yazelix runtime environment.",
            "Retry with a valid executable on PATH inside the Yazelix tool surface.",
            cmd.to_string(),
            source,
        )
    })
}

pub fn setpriv_or_sh_exec(
    shell_argv: &[String],
    runtime_env: &JsonMap<String, JsonValue>,
    cwd: &Path,
) -> Result<std::process::ExitStatus, CoreError> {
    let has_setpriv = Command::new("sh")
        .args(["-c", "command -v setpriv >/dev/null 2>&1"])
        .status()
        .map(|s| s.success())
        .unwrap_or(false);

    let mut cmd = if has_setpriv {
        let mut c = Command::new("setpriv");
        c.args(["--pdeathsig", "TERM", "--"]);
        c.args(shell_argv);
        c
    } else {
        let mut c = Command::new("sh");
        c.args(["-c", "trap 'kill 0' HUP TERM; exec \"$@\"", "_"]);
        c.args(shell_argv);
        c
    };

    cmd.current_dir(cwd);
    cmd.env_clear();
    for (k, v) in std::env::vars_os() {
        cmd.env(k, v);
    }
    for (k, v) in json_map_to_child_env(runtime_env) {
        cmd.env(k, v);
    }
    cmd.status().map_err(|source| {
        CoreError::io(
            "env_shell_spawn",
            "Could not launch the Yazelix runtime shell.",
            "Rerun with `yzx env --no-shell` to stay in your current shell.",
            shell_argv
                .first()
                .map(|s| s.as_str())
                .unwrap_or("shell")
                .to_string(),
            source,
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_argv_preserves_child_flags() {
        let argv = vec!["cargo".into(), "--verbose".into(), "check".into()];
        let (cmd, rest) = split_run_argv(&argv).unwrap();
        assert_eq!(cmd, "cargo");
        assert_eq!(rest, &["--verbose", "check"]);
    }

    #[test]
    fn run_argv_rejects_empty() {
        let argv: Vec<String> = vec![];
        assert!(split_run_argv(&argv).is_err());
    }

    #[test]
    fn env_cli_accepts_no_shell_aliases() {
        let a = parse_env_cli_args(&["--no-shell".into()]).unwrap();
        assert!(a.no_shell);
        let b = parse_env_cli_args(&["-n".into()]).unwrap();
        assert!(b.no_shell);
        let c = parse_env_cli_args(&[]).unwrap();
        assert!(!c.no_shell);
    }
}
