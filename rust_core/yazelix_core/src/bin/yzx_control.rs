//! Internal control-plane binary for Rust-owned `yzx` families (invoked from `yzx_cli.sh`).

use crossterm::style::Stylize;
use crossterm::terminal;
use serde::Serialize;
use std::io::IsTerminal;
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
use yazelix_core::run_generate_shell_initializers;
use yazelix_core::run_profile_create_run;
use yazelix_core::run_profile_load_report;
use yazelix_core::run_profile_print_report;
use yazelix_core::run_profile_record_step;
use yazelix_core::run_profile_wait_step;
use yazelix_core::run_yzx_config;
use yazelix_core::run_yzx_cwd;
use yazelix_core::run_yzx_desktop;
use yazelix_core::run_yzx_doctor;
use yazelix_core::run_yzx_edit;
use yazelix_core::run_yzx_edit_config;
use yazelix_core::run_yzx_enter;
use yazelix_core::run_yzx_home_manager;
use yazelix_core::run_yzx_import;
use yazelix_core::run_yzx_keys;
use yazelix_core::run_yzx_launch;
use yazelix_core::run_yzx_popup;
use yazelix_core::run_yzx_restart;
use yazelix_core::run_yzx_reveal;
use yazelix_core::run_yzx_screen;
use yazelix_core::run_yzx_sponsor;
use yazelix_core::run_yzx_tutor;
use yazelix_core::run_yzx_whats_new;
use yazelix_core::run_yzx_why;
use yazelix_core::run_zellij_get_workspace_root;
use yazelix_core::run_zellij_open_editor;
use yazelix_core::run_zellij_open_editor_cwd;
use yazelix_core::run_zellij_open_terminal;
use yazelix_core::run_zellij_pipe;
use yazelix_core::run_zellij_retarget;
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
    eprintln!("       yzx_control profile create-run <scenario> [--metadata <json>]");
    eprintln!(
        "       yzx_control profile record-step <component> <step> <started_ns> <ended_ns> [--metadata <json>]"
    );
    eprintln!("       yzx_control profile load-report <report_path>");
    eprintln!(
        "       yzx_control profile wait-step <report_path> <component> <step> [--timeout-ms <n>]"
    );
    eprintln!("       yzx_control profile print-report <report_path>");
    eprintln!("       yzx_control zellij pipe <command> [--payload <json>]");
    eprintln!("       yzx_control zellij get-workspace-root [--include-bootstrap]");
    eprintln!("       yzx_control zellij retarget <path> [--editor <kind>]");
    eprintln!("       yzx_control zellij open-editor <path>");
    eprintln!("       yzx_control zellij open-editor-cwd <path>");
    eprintln!("       yzx_control zellij open-terminal <path>");
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

#[derive(Clone, Copy)]
enum StatusTone {
    Default,
    Good,
    Warning,
    Muted,
}

struct StatusRow {
    label: &'static str,
    value: String,
    tone: StatusTone,
}

struct StatusSection {
    title: &'static str,
    rows: Vec<StatusRow>,
}

fn json_value_to_display(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::Null => "none".to_string(),
        serde_json::Value::Bool(true) => "yes".to_string(),
        serde_json::Value::Bool(false) => "no".to_string(),
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Array(arr) => arr
            .iter()
            .filter_map(|v| match v {
                serde_json::Value::String(s) => Some(s.clone()),
                other => Some(other.to_string()),
            })
            .collect::<Vec<_>>()
            .join(", "),
        serde_json::Value::Object(_) => serde_json::to_string(value).unwrap_or_default(),
    }
}

fn status_summary_value(data: &yazelix_core::StatusReportData, key: &str) -> String {
    data.summary
        .get(key)
        .map(json_value_to_display)
        .unwrap_or_else(|| "unknown".to_string())
}

fn humanize_token(value: &str) -> String {
    value.replace('_', " ")
}

fn status_badge(value: &str) -> (String, StatusTone) {
    match value {
        "noop" => ("up to date".to_string(), StatusTone::Good),
        "refresh_required" => ("refresh required".to_string(), StatusTone::Warning),
        "repair_missing_artifacts" => ("repair missing artifacts".to_string(), StatusTone::Warning),
        other => (humanize_token(other), StatusTone::Default),
    }
}

fn bool_summary_row(label: &'static str, value: &serde_json::Value) -> StatusRow {
    match value {
        serde_json::Value::Bool(true) => StatusRow {
            label,
            value: "yes".to_string(),
            tone: StatusTone::Warning,
        },
        serde_json::Value::Bool(false) => StatusRow {
            label,
            value: "no".to_string(),
            tone: StatusTone::Good,
        },
        _ => StatusRow {
            label,
            value: json_value_to_display(value),
            tone: StatusTone::Default,
        },
    }
}

fn maybe_muted_row(label: &'static str, value: String) -> StatusRow {
    let tone = if value == "none" || value == "disabled" {
        StatusTone::Muted
    } else {
        StatusTone::Default
    };
    StatusRow { label, value, tone }
}

fn build_status_sections(data: &yazelix_core::StatusReportData) -> Vec<StatusSection> {
    let generated_status_raw = status_summary_value(data, "generated_state_materialization_status");
    let generated_reason = status_summary_value(data, "generated_state_materialization_reason");
    let (generated_status, generated_tone) = status_badge(&generated_status_raw);
    let repair_needed_value = data
        .summary
        .get("generated_state_repair_needed")
        .cloned()
        .unwrap_or(serde_json::Value::Null);
    let persistent_sessions = data
        .summary
        .get("persistent_sessions")
        .cloned()
        .unwrap_or(serde_json::Value::Null);
    let session_name = status_summary_value(data, "session_name");

    vec![
        StatusSection {
            title: "Runtime",
            rows: vec![
                StatusRow {
                    label: "Version",
                    value: status_summary_value(data, "version"),
                    tone: StatusTone::Default,
                },
                StatusRow {
                    label: "Description",
                    value: status_summary_value(data, "description"),
                    tone: StatusTone::Muted,
                },
                StatusRow {
                    label: "Config file",
                    value: status_summary_value(data, "config_file"),
                    tone: StatusTone::Default,
                },
                StatusRow {
                    label: "Runtime dir",
                    value: status_summary_value(data, "runtime_dir"),
                    tone: StatusTone::Default,
                },
                StatusRow {
                    label: "Logs dir",
                    value: status_summary_value(data, "logs_dir"),
                    tone: StatusTone::Default,
                },
            ],
        },
        StatusSection {
            title: "Generated State",
            rows: vec![
                StatusRow {
                    label: "Status",
                    value: generated_status,
                    tone: generated_tone,
                },
                bool_summary_row("Repair needed", &repair_needed_value),
                StatusRow {
                    label: "Reason",
                    value: generated_reason,
                    tone: StatusTone::Muted,
                },
            ],
        },
        StatusSection {
            title: "Workspace",
            rows: vec![
                StatusRow {
                    label: "Default shell",
                    value: status_summary_value(data, "default_shell"),
                    tone: StatusTone::Default,
                },
                StatusRow {
                    label: "Terminals",
                    value: status_summary_value(data, "terminals"),
                    tone: StatusTone::Default,
                },
                maybe_muted_row("Helix runtime", status_summary_value(data, "helix_runtime")),
                StatusRow {
                    label: "Persistent sessions",
                    value: match &persistent_sessions {
                        serde_json::Value::Bool(true) => "enabled".to_string(),
                        serde_json::Value::Bool(false) => "disabled".to_string(),
                        other => json_value_to_display(&other),
                    },
                    tone: match &persistent_sessions {
                        serde_json::Value::Bool(true) => StatusTone::Default,
                        serde_json::Value::Bool(false) => StatusTone::Muted,
                        _ => StatusTone::Default,
                    },
                },
                maybe_muted_row("Session name", session_name),
            ],
        },
    ]
}

fn colors_enabled() -> bool {
    if std::env::var_os("NO_COLOR").is_some() {
        return false;
    }
    if std::env::var_os("FORCE_COLOR").is_some() {
        return true;
    }
    std::io::stdout().is_terminal()
}

fn render_width() -> usize {
    terminal::size()
        .map(|(width, _)| width as usize)
        .ok()
        .filter(|width| *width >= 48)
        .unwrap_or(100)
}

fn tone_text(text: &str, tone: StatusTone, color: bool) -> String {
    if !color {
        return text.to_string();
    }

    match tone {
        StatusTone::Default => text.to_string(),
        StatusTone::Good => format!("{}", text.green().bold()),
        StatusTone::Warning => format!("{}", text.yellow().bold()),
        StatusTone::Muted => format!("{}", text.dark_grey()),
    }
}

fn style_title(text: &str, color: bool) -> String {
    if color {
        format!("{}", text.bold())
    } else {
        text.to_string()
    }
}

fn style_section_title(text: &str, color: bool) -> String {
    if color {
        format!("{}", text.cyan().bold())
    } else {
        text.to_string()
    }
}

fn style_label(text: &str, color: bool) -> String {
    if color {
        format!("{}", text.dark_grey())
    } else {
        text.to_string()
    }
}

fn find_wrap_boundary(text: &str, max_chars: usize) -> usize {
    let mut last_space = None;
    let mut count = 0usize;
    let mut hard_break = text.len();

    for (idx, ch) in text.char_indices() {
        if count == max_chars {
            hard_break = idx;
            break;
        }
        count += 1;
        if ch.is_whitespace() {
            last_space = Some(idx);
        }
    }

    if count <= max_chars {
        return text.len();
    }

    last_space.filter(|idx| *idx > 0).unwrap_or(hard_break)
}

fn wrap_text(text: &str, width: usize) -> Vec<String> {
    if width == 0 {
        return vec![text.to_string()];
    }

    let mut lines = Vec::new();
    for paragraph in text.lines() {
        if paragraph.is_empty() {
            lines.push(String::new());
            continue;
        }

        let mut remaining = paragraph.trim_end();
        loop {
            if remaining.chars().count() <= width {
                lines.push(remaining.to_string());
                break;
            }

            let split_at = find_wrap_boundary(remaining, width);
            let (head, tail) = remaining.split_at(split_at);
            lines.push(head.trim_end().to_string());
            remaining = tail.trim_start();

            if remaining.is_empty() {
                break;
            }
        }
    }

    if lines.is_empty() {
        vec![String::new()]
    } else {
        lines
    }
}

fn render_status_section(section: &StatusSection, width: usize, color: bool) {
    println!("{}", style_section_title(section.title, color));

    let max_label_len = section
        .rows
        .iter()
        .map(|row| row.label.len())
        .max()
        .unwrap_or(0);
    let value_width = width.saturating_sub(2 + max_label_len + 2).max(24);

    for row in &section.rows {
        let wrapped = wrap_text(&row.value, value_width);
        let label = style_label(
            &format!("{:<width$}", row.label, width = max_label_len),
            color,
        );

        if let Some(first) = wrapped.first() {
            println!("  {}  {}", label, tone_text(first, row.tone, color));
        }

        for continuation in wrapped.iter().skip(1) {
            println!(
                "  {:<width$}  {}",
                "",
                tone_text(continuation, row.tone, color),
                width = max_label_len
            );
        }
    }
}

fn render_status_report(data: &yazelix_core::StatusReportData) {
    let color = colors_enabled();
    let width = render_width();

    println!("{}", style_title(&data.title, color));
    println!();

    for (index, section) in build_status_sections(data).iter().enumerate() {
        if index > 0 {
            println!();
        }
        render_status_section(section, width, color);
    }
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

fn run_profile(args: &[String]) -> Result<i32, CoreError> {
    if args.is_empty() {
        eprintln!(
            "Usage: yzx_control profile <create-run|record-step|load-report|wait-step|print-report> [args...]"
        );
        return Ok(64);
    }
    let mut argv = args.to_vec();
    let sub = argv.remove(0);
    match sub.as_str() {
        "create-run" => run_profile_create_run(&argv),
        "record-step" => run_profile_record_step(&argv),
        "load-report" => run_profile_load_report(&argv),
        "wait-step" => run_profile_wait_step(&argv),
        "print-report" => run_profile_print_report(&argv),
        _ => {
            eprintln!("Unknown profile subcommand: {sub}");
            eprintln!(
                "Usage: yzx_control profile <create-run|record-step|load-report|wait-step|print-report> [args...]"
            );
            Ok(64)
        }
    }
}

fn run_zellij(args: &[String]) -> Result<i32, CoreError> {
    if args.is_empty() {
        eprintln!(
            "Usage: yzx_control zellij <pipe|get-workspace-root|retarget|open-editor|open-editor-cwd|open-terminal> [args...]"
        );
        return Ok(64);
    }
    let mut argv = args.to_vec();
    let sub = argv.remove(0);
    match sub.as_str() {
        "pipe" => run_zellij_pipe(&argv),
        "get-workspace-root" => run_zellij_get_workspace_root(&argv),
        "retarget" => run_zellij_retarget(&argv),
        "open-editor" => run_zellij_open_editor(&argv),
        "open-editor-cwd" => run_zellij_open_editor_cwd(&argv),
        "open-terminal" => run_zellij_open_terminal(&argv),
        _ => {
            eprintln!("Unknown zellij subcommand: {sub}");
            eprintln!(
                "Usage: yzx_control zellij <pipe|get-workspace-root|retarget|open-editor|open-editor-cwd|open-terminal> [args...]"
            );
            Ok(64)
        }
    }
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
        "profile" => run_profile(&argv),
        "zellij" => run_zellij(&argv),
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
