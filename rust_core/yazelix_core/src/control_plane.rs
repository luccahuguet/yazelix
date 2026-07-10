//! Shared logic for the `yzx_control` CLI (`yzx env` / `yzx run`).

use crate::active_config_surface::{primary_config_paths, resolve_active_config_paths};
use crate::bridge::{CoreError, ErrorClass};
use crate::helix_external::HelixExternalPair;
use crate::runtime_env::RuntimePathInput;
use crate::zellij_materialization::zellij_permissions_cache_path;
use crate::{
    ComputeConfigStateRequest, NormalizeConfigRequest, RecordConfigStateRequest,
    RuntimeEnvComputeRequest, RuntimeMaterializationPlanRequest, normalize_config,
};
use serde_json::{Map as JsonMap, Value as JsonValue};
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::Command;
use toml::Value as TomlValue;

const DEFAULT_SHELL: &str = "nu";
const RELEASE_METADATA_FILENAME: &str = "release_metadata.toml";

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
    let identity = read_runtime_identity_from_runtime(runtime_dir)?;
    runtime_identity_version(&identity, runtime_dir)
}

pub fn read_release_metadata_version(repo_root: &Path) -> Result<String, CoreError> {
    let metadata_path = repo_root.join(RELEASE_METADATA_FILENAME);
    let contents = std::fs::read_to_string(&metadata_path).map_err(|source| {
        CoreError::io(
            "release_metadata",
            format!(
                "Failed to read Yazelix release metadata from {}.",
                metadata_path.display()
            ),
            "Restore release_metadata.toml from the Yazelix repository.",
            metadata_path.display().to_string(),
            source,
        )
    })?;
    let metadata = toml::from_str::<TomlValue>(&contents).map_err(|source| {
        CoreError::classified(
            ErrorClass::Runtime,
            "invalid_release_metadata",
            format!(
                "Yazelix release metadata is invalid TOML at {}.",
                metadata_path.display()
            ),
            "Fix release_metadata.toml so it declares a string `version` field.",
            serde_json::json!({
                "path": metadata_path.display().to_string(),
                "error": source.to_string(),
            }),
        )
    })?;
    let version = metadata
        .get("version")
        .and_then(TomlValue::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            CoreError::classified(
                ErrorClass::Runtime,
                "missing_release_metadata_version",
                format!(
                    "Yazelix release metadata is missing a string `version` field at {}.",
                    metadata_path.display()
                ),
                "Fix release_metadata.toml so it declares the current Yazelix version.",
                serde_json::json!({ "path": metadata_path.display().to_string() }),
            )
        })?;

    Ok(version.to_string())
}

pub fn read_runtime_identity_from_runtime(runtime_dir: &Path) -> Result<JsonValue, CoreError> {
    let identity_path = runtime_dir.join("runtime_identity.json");
    let contents = std::fs::read_to_string(&identity_path).map_err(|source| {
        CoreError::io(
            "runtime_identity",
            format!(
                "Failed to read Yazelix runtime identity from {}.",
                identity_path.display()
            ),
            "Reinstall Yazelix from a current package so runtime_identity.json is present.",
            identity_path.display().to_string(),
            source,
        )
    })?;
    let identity = serde_json::from_str::<JsonValue>(&contents).map_err(|source| {
        CoreError::classified(
            ErrorClass::Runtime,
            "invalid_runtime_identity",
            format!(
                "Yazelix runtime identity is invalid JSON at {}.",
                identity_path.display()
            ),
            "Reinstall Yazelix from a current package so runtime_identity.json is valid.",
            serde_json::json!({
                "path": identity_path.display().to_string(),
                "error": source.to_string(),
            }),
        )
    })?;

    if !identity.is_object() {
        return Err(CoreError::classified(
            ErrorClass::Runtime,
            "invalid_runtime_identity_shape",
            format!(
                "Yazelix runtime identity must be a JSON object at {}.",
                identity_path.display()
            ),
            "Reinstall Yazelix from a current package so runtime_identity.json has the supported shape.",
            serde_json::json!({ "path": identity_path.display().to_string() }),
        ));
    }

    Ok(identity)
}

fn runtime_identity_version(identity: &JsonValue, runtime_dir: &Path) -> Result<String, CoreError> {
    let version = identity
        .get("version")
        .and_then(JsonValue::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            CoreError::classified(
                ErrorClass::Runtime,
                "missing_runtime_identity_version",
                format!(
                    "Yazelix runtime identity is missing a string `version` field at {}.",
                    runtime_dir.join("runtime_identity.json").display()
                ),
                "Reinstall Yazelix from a current package so runtime_identity.json includes the packaged release version.",
                serde_json::json!({
                    "path": runtime_dir.join("runtime_identity.json").display().to_string(),
                }),
            )
        })?;

    Ok(version.to_string())
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
        zellij_permissions_cache_path: Some(zellij_permissions_cache_path()?),
        layout_override: runtime_materialization_layout_override_from_env(),
        session_terminal_label: None,
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

pub fn runtime_env_request_from_env(
    config_json: Option<&str>,
    config_override: Option<&str>,
) -> Result<RuntimeEnvComputeRequest, CoreError> {
    let runtime_dir = runtime_dir_from_env()?;
    let normalized = match config_json {
        Some(raw) => serde_json::from_str::<JsonMap<String, JsonValue>>(raw).map_err(|error| {
            CoreError::classified(
                ErrorClass::Usage,
                "invalid_config_json",
                format!("Invalid runtime-env config JSON: {error}"),
                "Pass one valid JSON object via --config-json or omit it to load the canonical config.",
                serde_json::json!({}),
            )
        })?,
        None => {
            let config_dir = config_dir_from_env()?;
            load_normalized_config_for_control(&runtime_dir, &config_dir, config_override)?
        }
    };

    runtime_env_request(runtime_dir, &normalized)
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
        ("xonsh", true) => vec!["xonsh".into(), "--login".into()],
        ("zsh", true) => vec!["zsh".into(), "-l".into()],
        ("bash", false) => vec!["bash".into()],
        ("fish", false) => vec!["fish".into()],
        ("xonsh", false) => vec!["xonsh".into()],
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

pub fn zellij_default_shell_from_runtime(runtime_dir: &Path, default_shell: &str) -> String {
    if default_shell.eq_ignore_ascii_case("nu") {
        runtime_dir
            .join("shells")
            .join("posix")
            .join("yazelix_nu.sh")
            .to_string_lossy()
            .to_string()
    } else {
        default_shell.to_string()
    }
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
    let editor_command = normalized
        .get("editor_command")
        .and_then(|v| v.as_str())
        .map(String::from);
    let helix_external = normalized
        .get("helix_external")
        .and_then(HelixExternalPair::from_json);

    Ok(RuntimeEnvComputeRequest {
        runtime_dir,
        home_dir,
        xdg_config_home: std::env::var_os("XDG_CONFIG_HOME").map(PathBuf::from),
        host_path_prefix: std::env::var_os("YAZELIX_HOST_PATH_PREFIX").map(PathBuf::from),
        current_path: RuntimePathInput::String(current_path),
        current_lazygit_config_file: std::env::var("LG_CONFIG_FILE").ok(),
        editor_command,
        helix_external,
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
        include_missing: true,
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
    #[test]
    fn run_argv_preserves_child_flags() {
        let argv = vec!["cargo".into(), "--verbose".into(), "check".into()];
        let (cmd, rest) = split_run_argv(&argv).unwrap();
        assert_eq!(cmd, "cargo");
        assert_eq!(rest, &["--verbose", "check"]);
    }

    // Defends: `yzx run` must reject an empty child argv instead of launching an unspecified command.
    #[test]
    fn run_argv_rejects_empty() {
        let argv: Vec<String> = vec![];
        assert!(split_run_argv(&argv).is_err());
    }

    // Defends: `yzx env` keeps the documented `--no-shell` alias family after the Rust control-plane owner cut.
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
    #[test]
    fn shell_command_no_login_nu_stays_plain_nu() {
        let rt = Path::new("/opt/yazelix");
        let argv = shell_command(rt, false, "nu");
        assert_eq!(argv, vec!["nu".to_string()]);
    }

    // Test lane: default
    // Defends: host-owned xonsh default shell launches through xonsh's login entrypoint without a Yazelix wrapper.
    #[test]
    fn shell_command_login_xonsh_uses_host_xonsh() {
        let rt = Path::new("/opt/yazelix");
        let argv = shell_command(rt, true, "xonsh");
        assert_eq!(argv, vec!["xonsh".to_string(), "--login".to_string()]);
    }

    // Defends: explicit `YAZELIX_CONFIG_DIR` still expands `~` before path use.
    #[test]
    fn resolve_yazelix_config_dir_prefers_explicit_and_expands_home() {
        let home = Path::new("/tmp/home");
        let path = resolve_yazelix_config_dir(Some("~/cfg/yazelix"), Some("/ignored"), Some(home))
            .unwrap();
        assert_eq!(path, home.join("cfg").join("yazelix"));
    }

    // Defends: config-dir resolution still prefers `XDG_CONFIG_HOME` before the home-default fallback.
    #[test]
    fn resolve_yazelix_config_dir_uses_xdg_before_home_default() {
        let home = Path::new("/tmp/home");
        let path = resolve_yazelix_config_dir(None, Some("~/xdg"), Some(home)).unwrap();
        assert_eq!(path, home.join("xdg").join("yazelix"));
    }
}
