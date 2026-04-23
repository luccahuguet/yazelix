//! Internal control-plane binary for Rust-owned `yzx` families (invoked from `yzx_cli.sh`).

use serde::Serialize;
use std::process::Command;
use yazelix_core::bridge::{CoreError, ErrorClass};
use yazelix_core::compute_runtime_env;
use yazelix_core::compute_status_report;
use yazelix_core::config_normalize::ConfigDiagnosticReport;
use yazelix_core::control_plane::{
    basename_shell, config_dir_from_env, config_override_from_env, default_shell_from_config,
    load_normalized_config_for_control, parse_env_cli_args, parse_status_cli_args,
    read_yazelix_version_from_runtime, run_child_in_runtime_env, runtime_dir_from_env,
    runtime_env_request, runtime_materialization_plan_request_from_env, setpriv_or_sh_exec,
    shell_command, split_run_argv,
};
use yazelix_core::run_yzx_config;
use yazelix_core::run_yzx_cwd;
use yazelix_core::run_yzx_desktop;
use yazelix_core::run_yzx_doctor;
use yazelix_core::run_yzx_edit;
use yazelix_core::run_yzx_edit_config;
use yazelix_core::run_yzx_enter;
use yazelix_core::run_generate_shell_initializers;
use yazelix_core::run_yzx_home_manager;
use yazelix_core::run_yzx_import;
use yazelix_core::run_yzx_keys;
use yazelix_core::run_yzx_launch;
use yazelix_core::run_yzx_popup;
use yazelix_core::run_yzx_reveal;
use yazelix_core::run_yzx_restart;
use yazelix_core::run_yzx_screen;
use yazelix_core::run_yzx_sponsor;
use yazelix_core::run_yzx_tutor;
use yazelix_core::run_yzx_why;
use yazelix_core::run_yzx_whats_new;
use yazelix_core::update_commands::run_yzx_update;

fn usage() -> ! {
    eprintln!("Usage: yzx_control env [--no-shell|-n]");
    eprintln!("       yzx_control run <command> [args...]");
    eprintln!("       yzx_control config [--path]");
    eprintln!("       yzx_control config reset [--yes] [--no-backup]");
    eprintln!("       yzx_control cwd [target]");
    eprintln!("       yzx_control desktop <install|launch|uninstall> [args...]");
    eprintln!("       yzx_control doctor [--verbose] [--fix] [--json]");
    eprintln!("       yzx_control edit [query...] [--print]");
    eprintln!("       yzx_control edit config [--print]");
    eprintln!("       yzx_control generate_shell_initializers [shells...]");
    eprintln!("       yzx_control import <zellij|yazi|helix> [--force]");
    eprintln!("       yzx_control enter [--path <dir> | --home] [--verbose]");
    eprintln!("       yzx_control status [--versions] [--json]");
    eprintln!("       yzx_control launch [--path <dir> | --home] [--terminal <name>] [--verbose]");
    eprintln!("       yzx_control home_manager [prepare] [args...]");
    eprintln!("       yzx_control keys [yzx|yazi|hx|helix|nu|nushell]");
    eprintln!("       yzx_control popup [program...]");
    eprintln!("       yzx_control reveal <path>");
    eprintln!("       yzx_control restart");
    eprintln!("       yzx_control screen [style]");
    eprintln!("       yzx_control why");
    eprintln!("       yzx_control sponsor");
    eprintln!("       yzx_control tutor [hx|helix|nu|nushell]");
    eprintln!("       yzx_control update [subcommand] [args...]");
    eprintln!("       yzx_control whats_new");
    std::process::exit(64);
}

const YAZELIX_DESCRIPTION: &str = "Yazi + Zellij + Helix integrated terminal environment";
const STATUS_VERSION_TOOLS: &[(&str, &str)] = &[
    ("yazi", "yazi"),
    ("zellij", "zellij"),
    ("helix", "hx"),
    ("nushell", "nu"),
    ("zoxide", "zoxide"),
    ("starship", "starship"),
    ("lazygit", "lazygit"),
    ("fzf", "fzf"),
    ("wezterm", "wezterm"),
    ("ghostty", "ghostty"),
    ("nix", "nix"),
    ("kitty", "kitty"),
    ("foot", "foot"),
    ("alacritty", "alacritty"),
    ("macchina", "macchina"),
];

#[derive(Debug, Clone, Serialize)]
struct ToolVersionEntry {
    tool: String,
    runtime: String,
}

#[derive(Debug, Clone, Serialize)]
struct VersionReportData {
    title: String,
    tools: Vec<ToolVersionEntry>,
}

enum CommandProbe {
    Missing,
    Error,
    Output(String),
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

fn print_status_help() {
    println!("Show current Yazelix status");
    println!();
    println!("Usage:");
    println!("  yzx status [--versions] [--json]");
    println!();
    println!("Flags:");
    println!("  -V, --versions  Include the full tool version matrix");
    println!("      --json      Emit machine-readable status data");
}

fn first_nonempty_line(text: &str) -> Option<String> {
    text.lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .map(str::to_string)
}

fn nth_token(text: &str, index: usize) -> Option<String> {
    let line = first_nonempty_line(text)?;
    line.split_whitespace().nth(index).map(str::to_string)
}

fn semver_candidates(text: &str) -> Vec<String> {
    let chars: Vec<char> = text.chars().collect();
    let mut out = Vec::new();
    let mut index = 0;

    while index < chars.len() {
        if !chars[index].is_ascii_digit() {
            index += 1;
            continue;
        }

        let start = index;
        let mut end = index;
        let mut dots = 0;
        let mut previous_was_dot = false;

        while end < chars.len() && (chars[end].is_ascii_digit() || chars[end] == '.') {
            if chars[end] == '.' {
                if previous_was_dot {
                    break;
                }
                dots += 1;
                previous_was_dot = true;
            } else {
                previous_was_dot = false;
            }
            end += 1;
        }

        let candidate: String = chars[start..end].iter().collect();
        if dots >= 2 && !candidate.ends_with('.') {
            out.push(candidate);
        }
        index = end;
    }

    out
}

fn first_semver(text: &str) -> Option<String> {
    semver_candidates(text).into_iter().next()
}

fn last_semver(text: &str) -> Option<String> {
    semver_candidates(text).into_iter().last()
}

fn probe_command_output(command: &str, args: &[&str]) -> CommandProbe {
    match Command::new(command).args(args).output() {
        Ok(output) if output.status.success() => {
            let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
            let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
            if stdout.trim().is_empty() {
                CommandProbe::Output(stderr)
            } else {
                CommandProbe::Output(stdout)
            }
        }
        Ok(_) => CommandProbe::Error,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => CommandProbe::Missing,
        Err(_) => CommandProbe::Error,
    }
}

fn probe_version_with(command: &str, args: &[&str], parser: fn(&str) -> Option<String>) -> String {
    match probe_command_output(command, args) {
        CommandProbe::Missing => "not installed".to_string(),
        CommandProbe::Error => "error".to_string(),
        CommandProbe::Output(text) => parser(&text)
            .or_else(|| first_nonempty_line(&text))
            .unwrap_or_else(|| "unknown".to_string()),
    }
}

fn collect_version_info() -> VersionReportData {
    let tools = STATUS_VERSION_TOOLS
        .iter()
        .map(|(tool, command)| {
            let runtime = match *tool {
                "wezterm" => probe_version_with(command, &["--version"], |text| nth_token(text, 1)),
                "nix" => probe_version_with(command, &["--version"], last_semver),
                "macchina" => probe_version_with(command, &["-v"], first_semver),
                _ => probe_version_with(command, &["--version"], first_semver),
            };

            ToolVersionEntry {
                tool: (*tool).to_string(),
                runtime,
            }
        })
        .collect();

    VersionReportData {
        title: "Yazelix Tool Versions".to_string(),
        tools,
    }
}

fn print_aligned_rows(rows: &[(String, String)]) {
    let max_label_len = rows.iter().map(|(label, _)| label.len()).max().unwrap_or(0);
    for (label, value) in rows {
        println!("  {:<width$}  {}", label, value, width = max_label_len);
    }
}

fn render_status_report(data: &yazelix_core::StatusReportData) {
    println!("{}", data.title);
    println!();

    let rows: Vec<(String, String)> = data
        .summary
        .iter()
        .map(|(key, value)| {
            let display = match value {
                serde_json::Value::Null => "null".to_string(),
                serde_json::Value::Bool(b) => b.to_string(),
                serde_json::Value::Number(n) => n.to_string(),
                serde_json::Value::String(s) => s.clone(),
                serde_json::Value::Array(arr) => arr
                    .iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect::<Vec<_>>()
                    .join(", "),
                serde_json::Value::Object(_) => serde_json::to_string(value).unwrap_or_default(),
            };
            (key.clone(), display)
        })
        .collect();

    print_aligned_rows(&rows);
}

fn render_version_report(report: &VersionReportData) {
    println!("{}", report.title);
    let rows: Vec<(String, String)> = report
        .tools
        .iter()
        .map(|entry| (entry.tool.clone(), entry.runtime.clone()))
        .collect();
    print_aligned_rows(&rows);
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

fn run_status(args: &[String]) -> Result<i32, CoreError> {
    let parsed = parse_status_cli_args(args)?;
    if parsed.help {
        print_status_help();
        return Ok(0);
    }

    let request =
        runtime_materialization_plan_request_from_env(config_override_from_env().as_deref())?;
    let version = read_yazelix_version_from_runtime(&request.runtime_dir)?;
    let data = compute_status_report(&request, &version, YAZELIX_DESCRIPTION)?;
    let versions = parsed.versions.then(collect_version_info);

    if parsed.json {
        let mut envelope = serde_json::Map::new();
        envelope.insert(
            "title".to_string(),
            serde_json::Value::String(data.title.clone()),
        );
        envelope.insert(
            "summary".to_string(),
            serde_json::Value::Object(data.summary.clone()),
        );
        if let Some(report) = &versions {
            envelope.insert(
                "versions".to_string(),
                serde_json::to_value(report).unwrap_or(serde_json::Value::Null),
            );
        }
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::Value::Object(envelope)).unwrap_or_default()
        );
    } else {
        render_status_report(&data);
        if let Some(report) = &versions {
            println!();
            render_version_report(report);
        }
    }

    Ok(0)
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
        "config" => run_yzx_config(&argv),
        "cwd" => run_yzx_cwd(&argv),
        "desktop" => run_yzx_desktop(&argv),
        "doctor" => run_yzx_doctor(&argv),
        "edit" => {
            if argv.first().map(String::as_str) == Some("config") {
                run_yzx_edit_config(&argv[1..])
            } else {
                run_yzx_edit(&argv)
            }
        }
        "enter" => run_yzx_enter(&argv),
        "generate_shell_initializers" => run_generate_shell_initializers(&argv),
        "status" => run_status(&argv),
        "launch" => run_yzx_launch(&argv),
        "home_manager" => run_yzx_home_manager(&argv),
        "import" => run_yzx_import(&argv),
        "keys" => run_yzx_keys(&argv),
        "popup" => run_yzx_popup(&argv),
        "reveal" => run_yzx_reveal(&argv),
        "restart" => run_yzx_restart(&argv),
        "screen" => run_yzx_screen(&argv),
        "why" => run_yzx_why(&argv),
        "sponsor" => run_yzx_sponsor(&argv),
        "tutor" => run_yzx_tutor(&argv),
        "update" => run_yzx_update(&argv),
        "whats_new" => run_yzx_whats_new(&argv),
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

    // Test lane: default
    // Defends: public control-plane config-dir resolution still honors explicit `YAZELIX_CONFIG_DIR` with home expansion.
    // Strength: defect=1 behavior=2 resilience=2 cost=2 uniqueness=1 total=8/10
    #[test]
    fn resolve_config_dir_prefers_explicit_and_expands_home() {
        let home = Path::new("/tmp/home");
        let path = resolve_yazelix_config_dir(Some("~/cfg/yazelix"), Some("/ignored"), Some(home))
            .unwrap();
        assert_eq!(path, home.join("cfg").join("yazelix"));
    }

    // Defends: public control-plane config-dir resolution still prefers `XDG_CONFIG_HOME` before the home-default fallback.
    // Strength: defect=1 behavior=2 resilience=2 cost=2 uniqueness=1 total=8/10
    #[test]
    fn resolve_config_dir_uses_xdg_before_home_default() {
        let home = Path::new("/tmp/home");
        let path = resolve_yazelix_config_dir(None, Some("~/xdg"), Some(home)).unwrap();
        assert_eq!(path, home.join("xdg").join("yazelix"));
    }

    // Defends: startup config rendering still includes the blocking diagnostic details promised by the public control-plane surface.
    // Strength: defect=2 behavior=2 resilience=2 cost=2 uniqueness=1 total=9/10
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
        assert!(!rendered.contains("Known migration"));
        assert!(!rendered.contains("yzx doctor --fix"));
    }
}
