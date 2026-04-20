//! Internal control-plane binary for `yzx env` and `yzx run` (invoked from `yzx_cli.sh`).

use std::path::{Path, PathBuf};
use yazelix_core::bridge::{CoreError, ErrorClass};
use yazelix_core::compute_runtime_env;
use yazelix_core::config_normalize::ConfigDiagnosticReport;
use yazelix_core::control_plane::{
    basename_shell, default_shell_from_config, load_normalized_config_for_control,
    parse_env_cli_args, run_child_in_runtime_env, runtime_env_request, setpriv_or_sh_exec,
    shell_command, split_run_argv,
};

fn usage() -> ! {
    eprintln!("Usage: yzx_control env [--no-shell|-n]");
    eprintln!("       yzx_control run <command> [args...]");
    std::process::exit(64);
}

const CONFIG_RECOVERY_HINT: &str = "Update the reported config fields manually, then retry. Use `yzx config reset` only as a blunt fallback.";

fn render_startup_config_error(report: &ConfigDiagnosticReport) -> String {
    let mut lines = vec![
        format!(
            "Yazelix found stale or unsupported config entries in {}.",
            report.config_path
        ),
        format!("Blocking issues: {}", report.blocking_count),
    ];

    for diagnostic in &report.blocking_diagnostics {
        lines.push(String::new());
        lines.push(diagnostic.headline.clone());
        for detail in &diagnostic.detail_lines {
            lines.push(format!("  {detail}"));
        }
    }

    lines.push(String::new());
    lines.push("Failure class: config problem.".to_string());
    lines.push(format!("Recovery: {CONFIG_RECOVERY_HINT}"));
    lines.join("\n")
}

fn print_control_error(err: &CoreError) {
    if matches!(err.class(), ErrorClass::Config) && err.code() == "unsupported_config" {
        if let Ok(report) = serde_json::from_value::<ConfigDiagnosticReport>(err.details()) {
            eprintln!("{}", render_startup_config_error(&report));
            return;
        }
    }
    eprintln!("{}", err.message());
    let remediation = err.remediation();
    if !remediation.is_empty() {
        eprintln!("{remediation}");
    }
}

fn runtime_dir_from_env() -> Result<PathBuf, CoreError> {
    let raw = std::env::var("YAZELIX_RUNTIME_DIR").map_err(|_| {
        CoreError::classified(
            yazelix_core::bridge::ErrorClass::Runtime,
            "missing_runtime_dir",
            "YAZELIX_RUNTIME_DIR is not set.",
            "Run `yzx` through the packaged POSIX launcher so the runtime bootstraps correctly.",
            serde_json::json!({}),
        )
    })?;
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(CoreError::classified(
            yazelix_core::bridge::ErrorClass::Runtime,
            "empty_runtime_dir",
            "YAZELIX_RUNTIME_DIR is empty.",
            "Run `yzx` through the packaged POSIX launcher so the runtime bootstraps correctly.",
            serde_json::json!({}),
        ));
    }
    Ok(PathBuf::from(trimmed))
}

fn expand_user_path(raw: &str, home: &Path) -> PathBuf {
    if raw == "~" {
        return home.to_path_buf();
    }
    if let Some(rest) = raw.strip_prefix("~/") {
        return home.join(rest);
    }
    PathBuf::from(raw)
}

fn resolve_config_dir(
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

fn config_dir_from_env() -> Result<PathBuf, CoreError> {
    let home = std::env::var_os("HOME").map(PathBuf::from);
    resolve_config_dir(
        std::env::var("YAZELIX_CONFIG_DIR").ok().as_deref(),
        std::env::var("XDG_CONFIG_HOME").ok().as_deref(),
        home.as_deref(),
    )
}

fn config_override_from_env() -> Option<String> {
    std::env::var("YAZELIX_CONFIG_OVERRIDE")
        .ok()
        .filter(|s| !s.trim().is_empty())
}

fn run_env(args: &[String]) -> Result<i32, CoreError> {
    let parsed = parse_env_cli_args(args)?;
    let runtime_dir = runtime_dir_from_env()?;
    let config_dir = config_dir_from_env()?;
    let normalized = load_normalized_config_for_control(
        &runtime_dir,
        &config_dir,
        config_override_from_env().as_deref(),
    )?;
    let configured_shell = default_shell_from_config(&normalized);
    let invoking = basename_shell(
        std::env::var_os("SHELL")
            .and_then(|s| s.into_string().ok())
            .as_deref(),
    );
    let logical_shell = if parsed.no_shell {
        invoking.clone().unwrap_or_else(|| configured_shell.clone())
    } else {
        configured_shell.clone()
    };
    let shell_command = if parsed.no_shell {
        shell_command(&runtime_dir, false, &logical_shell)
    } else {
        shell_command(&runtime_dir, true, &configured_shell)
    };
    // Keep `SHELL=nu` when launching the managed wrapper (parity with historical `yzx env`).
    let shell_exec = if logical_shell == "nu" {
        "nu".to_string()
    } else {
        shell_command
            .first()
            .cloned()
            .unwrap_or_else(|| logical_shell.clone())
    };

    let req = runtime_env_request(runtime_dir.clone(), &normalized)?;
    let data = compute_runtime_env(&req)?;
    let mut runtime_map = data.runtime_env;
    runtime_map.insert("SHELL".to_string(), serde_json::Value::String(shell_exec));

    let cwd = std::env::current_dir().map_err(|source| {
        CoreError::io(
            "cwd",
            "Could not read the current working directory.",
            "cd into a valid directory, then retry.",
            ".",
            source,
        )
    })?;

    let status = match setpriv_or_sh_exec(&shell_command, &runtime_map, &cwd) {
        Ok(s) => s,
        Err(err) => {
            eprintln!(
                "❌ Failed to launch Yazelix runtime shell: {}",
                err.message()
            );
            eprintln!("   Tip: rerun with 'yzx env --no-shell' to stay in your current shell.");
            return Err(err);
        }
    };

    Ok(status.code().unwrap_or(1))
}

fn run_run(args: &[String]) -> Result<i32, CoreError> {
    let _ = split_run_argv(args)?;
    let runtime_dir = runtime_dir_from_env()?;
    let config_dir = config_dir_from_env()?;
    let normalized = load_normalized_config_for_control(
        &runtime_dir,
        &config_dir,
        config_override_from_env().as_deref(),
    )?;
    let req = runtime_env_request(runtime_dir, &normalized)?;
    let data = compute_runtime_env(&req)?;
    let cwd = std::env::current_dir().map_err(|source| {
        CoreError::io(
            "cwd",
            "Could not read the current working directory.",
            "cd into a valid directory, then retry.",
            ".",
            source,
        )
    })?;
    let status = run_child_in_runtime_env(args, &data.runtime_env, &cwd)?;
    Ok(status.code().unwrap_or(1))
}

fn main() {
    let mut argv: Vec<String> = std::env::args().skip(1).collect();
    if argv.is_empty() {
        usage();
    }
    let sub = argv.remove(0);
    let code = match sub.as_str() {
        "env" => run_env(&argv),
        "run" => run_run(&argv),
        _ => {
            eprintln!("Unknown yzx_control subcommand: {sub}");
            usage();
        }
    };

    match code {
        Ok(c) => std::process::exit(c),
        Err(e) => {
            print_control_error(&e);
            std::process::exit(e.class().exit_code());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use yazelix_core::config_normalize::ConfigDiagnostic;

    #[test]
    fn resolve_config_dir_prefers_explicit_and_expands_home() {
        let home = Path::new("/tmp/home");
        let path = resolve_config_dir(Some("~/cfg/yazelix"), Some("/ignored"), Some(home)).unwrap();
        assert_eq!(path, home.join("cfg").join("yazelix"));
    }

    #[test]
    fn resolve_config_dir_uses_xdg_before_home_default() {
        let home = Path::new("/tmp/home");
        let path = resolve_config_dir(None, Some("~/xdg"), Some(home)).unwrap();
        assert_eq!(path, home.join("xdg").join("yazelix"));
    }

    #[test]
    fn render_startup_config_error_includes_blocking_details() {
        let report = ConfigDiagnosticReport {
            config_path: "/tmp/yazelix.toml".to_string(),
            schema_diagnostics: vec![],
            doctor_diagnostics: vec![],
            blocking_diagnostics: vec![ConfigDiagnostic {
                category: "config".to_string(),
                path: "shell.default_shell".to_string(),
                status: "invalid".to_string(),
                blocking: true,
                fix_available: false,
                headline: "Invalid config value at shell.default_shell".to_string(),
                detail_lines: vec![
                    "Expected one of: nu, bash, fish, zsh".to_string(),
                    "Next: Update the field manually".to_string(),
                ],
            }],
            issue_count: 1,
            blocking_count: 1,
            fixable_count: 0,
            has_blocking: true,
            has_fixable_config_issues: false,
        };

        let rendered = render_startup_config_error(&report);
        assert!(rendered.contains("Blocking issues: 1"));
        assert!(rendered.contains("Invalid config value at shell.default_shell"));
        assert!(rendered.contains("Expected one of: nu, bash, fish, zsh"));
        assert!(rendered.contains("Failure class: config problem."));
    }
}
