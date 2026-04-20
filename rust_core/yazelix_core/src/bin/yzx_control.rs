//! Internal control-plane binary for `yzx env`, `yzx run`, and `yzx update*` (invoked from `yzx_cli.sh`).

use yazelix_core::bridge::{CoreError, ErrorClass};
use yazelix_core::compute_runtime_env;
use yazelix_core::config_normalize::ConfigDiagnosticReport;
use yazelix_core::control_plane::{
    basename_shell, config_dir_from_env, default_shell_from_config,
    load_normalized_config_for_control, parse_env_cli_args, run_child_in_runtime_env,
    runtime_dir_from_env, runtime_env_request, setpriv_or_sh_exec, shell_command, split_run_argv,
};
use yazelix_core::update_commands::run_yzx_update;

fn usage() -> ! {
    eprintln!("Usage: yzx_control env [--no-shell|-n]");
    eprintln!("       yzx_control run <command> [args...]");
    eprintln!("       yzx_control update [subcommand] [args...]");
    std::process::exit(64);
}

fn print_env_help() {
    println!("Load the Yazelix environment without UI");
    println!();
    println!("Usage:");
    println!("  yzx env [--no-shell]");
    println!();
    println!("Flags:");
    println!("  -n, --no-shell  Load the Yazelix environment into the current shell family");
}

fn print_run_help() {
    println!("Run a command in the Yazelix environment and exit");
    println!();
    println!("Usage:");
    println!("  yzx run <command> [args...]");
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
        "env" => {
            if argv.len() == 1 && matches!(argv[0].as_str(), "--help" | "-h" | "help") {
                print_env_help();
                Ok(0)
            } else {
                run_env(&argv)
            }
        }
        "run" => {
            if argv.len() == 1 && matches!(argv[0].as_str(), "--help" | "-h" | "help") {
                print_run_help();
                Ok(0)
            } else {
                run_run(&argv)
            }
        }
        "update" => run_yzx_update(&argv),
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
    use std::path::Path;
    use yazelix_core::config_normalize::ConfigDiagnostic;
    use yazelix_core::control_plane::resolve_yazelix_config_dir;

    #[test]
    fn resolve_config_dir_prefers_explicit_and_expands_home() {
        let home = Path::new("/tmp/home");
        let path = resolve_yazelix_config_dir(Some("~/cfg/yazelix"), Some("/ignored"), Some(home))
            .unwrap();
        assert_eq!(path, home.join("cfg").join("yazelix"));
    }

    #[test]
    fn resolve_config_dir_uses_xdg_before_home_default() {
        let home = Path::new("/tmp/home");
        let path = resolve_yazelix_config_dir(None, Some("~/xdg"), Some(home)).unwrap();
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
