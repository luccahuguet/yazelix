//! Shared logic for the `yzx_control` CLI (`yzx env` / `yzx run`).

use crate::active_config_surface::{primary_config_paths, resolve_active_config_paths};
use crate::bridge::{CoreError, ErrorClass};
use crate::runtime_env::RuntimePathInput;
use crate::{
    ComputeConfigStateRequest, NormalizeConfigRequest, RecordConfigStateRequest,
    RuntimeEnvComputeRequest, RuntimeMaterializationPlanRequest, normalize_config,
};
use serde_json::{Map as JsonMap, Value as JsonValue};
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::Command;

const DEFAULT_SHELL: &str = "nu";
const VERSION_LINE_PREFIX: &str = "export const YAZELIX_VERSION = \"";

/// Expand `~` / `~/…` using `home` (POSIX-style).
pub fn expand_user_path(raw: &str, home: &Path) -> PathBuf {
    if raw == "~" {
        return home.to_path_buf();
    }
    if let Some(rest) = raw.strip_prefix("~/") {
        return home.join(rest);
    }
    PathBuf::from(raw)
}

/// Resolve the managed Yazelix config root (`YAZELIX_CONFIG_DIR` or XDG + `/yazelix`).
pub fn resolve_yazelix_config_dir(
    explicit: Option<&str>,
    xdg_config_home: Option<&str>,
    home: Option<&Path>,
) -> Result<PathBuf, CoreError> {
    if let Some(raw) = explicit.map(str::trim).filter(|raw| !raw.is_empty()) {
        return Ok(match home {
            Some(home_dir) => expand_user_path(raw, home_dir),
            None => PathBuf::from(raw),
        });
    }

    if let Some(raw) = xdg_config_home.map(str::trim).filter(|raw| !raw.is_empty()) {
        let root = match home {
            Some(home_dir) => expand_user_path(raw, home_dir),
            None => PathBuf::from(raw),
        };
        return Ok(root.join("yazelix"));
    }

    let home = home.ok_or_else(|| {
        CoreError::classified(
            ErrorClass::Runtime,
            "missing_home",
            "HOME is not set; cannot resolve YAZELIX_CONFIG_DIR.",
            "Export HOME, then retry.",
            serde_json::json!({}),
        )
    })?;
    Ok(home.join(".config").join("yazelix"))
}

pub fn config_dir_from_env() -> Result<PathBuf, CoreError> {
    let home = std::env::var_os("HOME").map(PathBuf::from);
    resolve_yazelix_config_dir(
        std::env::var("YAZELIX_CONFIG_DIR").ok().as_deref(),
        std::env::var("XDG_CONFIG_HOME").ok().as_deref(),
        home.as_deref(),
    )
}

pub fn runtime_dir_from_env() -> Result<PathBuf, CoreError> {
    let raw = std::env::var("YAZELIX_RUNTIME_DIR").map_err(|_| {
        CoreError::classified(
            ErrorClass::Runtime,
            "missing_runtime_dir",
            "YAZELIX_RUNTIME_DIR is not set.",
            "Run `yzx` through the packaged POSIX launcher so the runtime bootstraps correctly.",
            serde_json::json!({}),
        )
    })?;
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(CoreError::classified(
            ErrorClass::Runtime,
            "empty_runtime_dir",
            "YAZELIX_RUNTIME_DIR is empty.",
            "Run `yzx` through the packaged POSIX launcher so the runtime bootstraps correctly.",
            serde_json::json!({}),
        ));
    }
    Ok(PathBuf::from(trimmed))
}

pub fn config_override_from_env() -> Option<String> {
    std::env::var("YAZELIX_CONFIG_OVERRIDE")
        .ok()
        .filter(|s| !s.trim().is_empty())
}

pub fn read_yazelix_version_from_runtime(runtime_dir: &Path) -> Result<String, CoreError> {
    let constants_path = runtime_dir
        .join("nushell")
        .join("scripts")
        .join("utils")
        .join("constants.nu");
    let contents = std::fs::read_to_string(&constants_path).map_err(|source| {
        CoreError::io(
            "version",
            format!(
                "Failed to read Yazelix version from {}.",
                constants_path.display()
            ),
            "Restore nushell/scripts/utils/constants.nu or reinstall Yazelix.",
            ".",
            source,
        )
    })?;

    for line in contents.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix(VERSION_LINE_PREFIX) {
            if let Some(value) = rest.strip_suffix('"') {
                return Ok(value.to_string());
            }
        }
    }

    Err(CoreError::classified(
        ErrorClass::Runtime,
        "missing_version_constant",
        format!(
            "Could not find version constant in {}.",
            constants_path.display()
        ),
        "Restore nushell/scripts/utils/constants.nu or reinstall Yazelix.",
        serde_json::json!({"path": constants_path.display().to_string()}),
    ))
}

pub fn home_dir_from_env() -> Result<PathBuf, CoreError> {
    std::env::var_os("HOME").map(PathBuf::from).ok_or_else(|| {
        CoreError::classified(
            ErrorClass::Runtime,
            "missing_home",
            "HOME is not set.",
            "Export HOME, then retry.",
            serde_json::json!({}),
        )
    })
}

pub fn state_dir_from_env() -> Result<PathBuf, CoreError> {
    if let Ok(raw) = std::env::var("YAZELIX_STATE_DIR") {
        let trimmed = raw.trim();
        if !trimmed.is_empty() {
            return Ok(PathBuf::from(trimmed));
        }
    }
    if let Ok(raw) = std::env::var("XDG_DATA_HOME") {
        let trimmed = raw.trim();
        if !trimmed.is_empty() {
            return Ok(PathBuf::from(trimmed).join("yazelix"));
        }
    }
    Ok(home_dir_from_env()?
        .join(".local")
        .join("share")
        .join("yazelix"))
}

pub fn runtime_materialization_layout_override_from_env() -> Option<String> {
    if let Ok(raw) = std::env::var("YAZELIX_LAYOUT_OVERRIDE") {
        let trimmed = raw.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }
    }

    if std::env::var_os("YAZELIX_SWEEP_TEST_ID").is_some() {
        if let Ok(raw) = std::env::var("ZELLIJ_DEFAULT_LAYOUT") {
            let trimmed = raw.trim();
            if !trimmed.is_empty() {
                return Some(trimmed.to_string());
            }
        }
    }

    None
}

pub fn runtime_materialization_plan_request_from_env(
    config_override: Option<&str>,
) -> Result<RuntimeMaterializationPlanRequest, CoreError> {
    let runtime_dir = runtime_dir_from_env()?;
    let config_dir = config_dir_from_env()?;
    let paths = resolve_active_config_paths(&runtime_dir, &config_dir, config_override)?;
    let state_dir = state_dir_from_env()?;
    let zellij_config_dir = state_dir.join("configs").join("zellij");

    Ok(RuntimeMaterializationPlanRequest {
        config_path: paths.config_file,
        default_config_path: paths.default_config_path,
        contract_path: paths.contract_path,
        runtime_dir,
        state_path: state_dir.join("state").join("rebuild_hash"),
        yazi_config_dir: state_dir.join("configs").join("yazi"),
        zellij_layout_dir: zellij_config_dir.join("layouts"),
        zellij_config_dir,
        layout_override: runtime_materialization_layout_override_from_env(),
    })
}

pub fn config_state_compute_request_from_env(
    config_override: Option<&str>,
) -> Result<ComputeConfigStateRequest, CoreError> {
    let runtime_dir = runtime_dir_from_env()?;
    let config_dir = config_dir_from_env()?;
    let paths = resolve_active_config_paths(&runtime_dir, &config_dir, config_override)?;
    let state_dir = state_dir_from_env()?;

    Ok(ComputeConfigStateRequest {
        config_path: paths.config_file,
        default_config_path: paths.default_config_path,
        contract_path: paths.contract_path,
        runtime_dir,
        state_path: state_dir.join("state").join("rebuild_hash"),
    })
}

pub fn config_state_record_request_from_env(
    config_file: String,
    config_hash: String,
    runtime_hash: String,
) -> Result<RecordConfigStateRequest, CoreError> {
    let runtime_dir = runtime_dir_from_env()?;
    let config_dir = config_dir_from_env()?;
    let managed_config_path = primary_config_paths(&runtime_dir, &config_dir).user_config;
    let state_dir = state_dir_from_env()?;

    Ok(RecordConfigStateRequest {
        config_file,
        managed_config_path,
        state_path: state_dir.join("state").join("rebuild_hash"),
        config_hash,
        runtime_hash,
    })
}

#[derive(Debug, Clone, Default)]
pub struct EnvCliArgs {
    pub no_shell: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StatusCliArgs {
    pub json: bool,
    pub versions: bool,
    pub help: bool,
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

pub fn parse_status_cli_args(args: &[String]) -> Result<StatusCliArgs, CoreError> {
    let mut out = StatusCliArgs::default();
    for token in args {
        match token.as_str() {
            "--json" => out.json = true,
            "--versions" | "-V" => out.versions = true,
            "--help" | "-h" | "help" => out.help = true,
            other => {
                return Err(CoreError::classified(
                    ErrorClass::Usage,
                    "unexpected_status_token",
                    format!("Unexpected argument for yzx status: {other}"),
                    "Run `yzx status`, `yzx status --json`, or `yzx status --versions`.",
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

/// Build argv for launching a shell inside `yzx env`.
///
/// Login Nushell must use `shells/posix/yazelix_nu.sh` so the managed
/// `config.nu` runs and generated initializers (including Starship) load.
pub fn shell_command(runtime_dir: &Path, login: bool, shell_name: &str) -> Vec<String> {
    let normalized = shell_name.to_lowercase();
    if normalized == "nu" && login {
        let wrapper = runtime_dir
            .join("shells")
            .join("posix")
            .join("yazelix_nu.sh");
        return vec![wrapper.to_string_lossy().into_owned()];
    }
    match (normalized.as_str(), login) {
        ("nu", false) => vec!["nu".into()],
        ("bash", true) => vec!["bash".into(), "--login".into()],
        ("fish", true) => vec!["fish".into(), "-l".into()],
        ("zsh", true) => vec!["zsh".into(), "-l".into()],
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
    let home_dir = home_dir_from_env().map_err(|_| {
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

    // Test lane: default
    // Defends: `yzx run` must preserve child flags after the public control-plane owner cut.
    // Strength: defect=2 behavior=2 resilience=2 cost=2 uniqueness=1 total=9/10
    #[test]
    fn run_argv_preserves_child_flags() {
        let argv = vec!["cargo".into(), "--verbose".into(), "check".into()];
        let (cmd, rest) = split_run_argv(&argv).unwrap();
        assert_eq!(cmd, "cargo");
        assert_eq!(rest, &["--verbose", "check"]);
    }

    // Defends: `yzx run` must reject an empty child argv instead of launching an unspecified command.
    // Strength: defect=2 behavior=2 resilience=2 cost=2 uniqueness=1 total=9/10
    #[test]
    fn run_argv_rejects_empty() {
        let argv: Vec<String> = vec![];
        assert!(split_run_argv(&argv).is_err());
    }

    // Defends: `yzx env` keeps the documented `--no-shell` alias family after the Rust control-plane owner cut.
    // Strength: defect=1 behavior=2 resilience=2 cost=2 uniqueness=1 total=8/10
    #[test]
    fn env_cli_accepts_no_shell_aliases() {
        let a = parse_env_cli_args(&["--no-shell".into()]).unwrap();
        assert!(a.no_shell);
        let b = parse_env_cli_args(&["-n".into()]).unwrap();
        assert!(b.no_shell);
        let c = parse_env_cli_args(&[]).unwrap();
        assert!(!c.no_shell);
    }

    // Test lane: default
    // Defends: `yzx env` login Nushell uses the managed wrapper (Starship + managed config.nu).
    // Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
    #[test]
    fn shell_command_login_nu_uses_managed_wrapper() {
        let rt = Path::new("/opt/yazelix");
        let argv = shell_command(rt, true, "nu");
        assert_eq!(argv.len(), 1);
        assert_eq!(
            argv[0],
            "/opt/yazelix/shells/posix/yazelix_nu.sh".to_string()
        );
    }

    // Test lane: default
    // Defends: `yzx env --no-shell` still launches plain Nushell when the invoking shell family is `nu`.
    // Strength: defect=1 behavior=2 resilience=1 cost=1 uniqueness=2 total=7/10
    #[test]
    fn shell_command_no_login_nu_stays_plain_nu() {
        let rt = Path::new("/opt/yazelix");
        let argv = shell_command(rt, false, "nu");
        assert_eq!(argv, vec!["nu".to_string()]);
    }

    // Defends: explicit `YAZELIX_CONFIG_DIR` still expands `~` before path use.
    // Strength: defect=1 behavior=2 resilience=2 cost=2 uniqueness=1 total=8/10
    #[test]
    fn resolve_yazelix_config_dir_prefers_explicit_and_expands_home() {
        let home = Path::new("/tmp/home");
        let path = resolve_yazelix_config_dir(Some("~/cfg/yazelix"), Some("/ignored"), Some(home))
            .unwrap();
        assert_eq!(path, home.join("cfg").join("yazelix"));
    }

    // Defends: config-dir resolution still prefers `XDG_CONFIG_HOME` before the home-default fallback.
    // Strength: defect=1 behavior=2 resilience=2 cost=2 uniqueness=1 total=8/10
    #[test]
    fn resolve_yazelix_config_dir_uses_xdg_before_home_default() {
        let home = Path::new("/tmp/home");
        let path = resolve_yazelix_config_dir(None, Some("~/xdg"), Some(home)).unwrap();
        assert_eq!(path, home.join("xdg").join("yazelix"));
    }
}
