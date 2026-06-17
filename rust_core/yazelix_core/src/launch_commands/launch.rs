// Test lane: default

use super::RUNTIME_RELAUNCH_CLEARED_ENV_KEYS;
use super::config_override::{
    config_override_extra_env, prepare_session_config_override, resolve_cli_config_override,
};
use super::process::{
    command_output_with_overrides, print_completed_output, render_launch_failure,
    run_desktop_deferred_launch_probe, run_detached_launch_probe,
};
use super::resolve_requested_working_dir;
use super::terminal::{build_launch_command_argv, resolve_terminal_config_path};
use crate::bridge::{CoreError, ErrorClass};
use crate::config_state::compute_config_state;
use crate::control_plane::{
    config_override_from_env, config_state_compute_request_from_env, home_dir_from_env,
    runtime_dir_from_env, runtime_env_request, runtime_materialization_plan_request_from_env,
    state_dir_from_env,
};
use crate::launch_materialization::{
    LaunchMaterializationData, launch_materialization_request_from_env,
    prepare_launch_materialization,
};
use crate::runtime_contract::{
    LaunchPreflightPayload, StartupLaunchPreflightRequest, TerminalCandidate,
    evaluate_startup_launch_preflight,
};
use crate::runtime_env::compute_runtime_env;
use crate::runtime_materialization::{
    RuntimeMaterializationRepairEvaluateRequest, repair_runtime_materialization,
};
use crate::terminal_materialization::{
    MARS_EMOJI_ENV_KEYS, MARS_EMOJI_FONT_ENV, MARS_EMOJI_FONT_SOURCE_ENV,
};
use crate::terminal_variant::{
    SUPPORTED_TERMINALS, active_terminal_from_runtime_dir, normalize_terminal_id,
    terminal_desktop_entry_file_name, terminal_display_name, terminal_startup_wm_class,
};
use std::fs;
use std::path::{Path, PathBuf};

const MARS_CHILD_ENV_SANITIZE: &str = "MARS_CHILD_ENV_SANITIZE";

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct LaunchArgs {
    path: Option<String>,
    terminal: Option<String>,
    config: Option<String>,
    with_overrides: Vec<String>,
    home: bool,
    verbose: bool,
    help: bool,
}

pub(super) fn run_launch(args: &[String]) -> Result<i32, CoreError> {
    let parsed = parse_launch_args(args)?;
    if parsed.help {
        print_launch_help();
        return Ok(0);
    }

    let inherited_config_override = config_override_from_env();
    let config_override = prepare_session_config_override(
        parsed
            .config
            .as_deref()
            .or(inherited_config_override.as_deref()),
        &parsed.with_overrides,
    )?;
    if let Some(requested_terminal) = parsed.terminal.as_deref() {
        return run_explicit_terminal_launch(
            &parsed,
            requested_terminal,
            config_override.as_deref(),
        );
    }
    run_launch_flow(
        parsed.path.as_deref(),
        config_override.as_deref(),
        parsed.home,
        parsed.verbose,
        false,
        &[],
    )
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PackagedTerminalLauncher {
    launcher: PathBuf,
    env: Vec<(String, Option<String>)>,
    desktop_path: PathBuf,
}

fn run_explicit_terminal_launch(
    parsed: &LaunchArgs,
    requested_terminal: &str,
    config_override: Option<&str>,
) -> Result<i32, CoreError> {
    let runtime_dir = runtime_dir_from_env()?;
    let active_terminal = active_terminal_from_runtime_dir(&runtime_dir)?;
    if requested_terminal == active_terminal {
        return run_launch_flow(
            parsed.path.as_deref(),
            config_override,
            parsed.home,
            parsed.verbose,
            false,
            &[],
        );
    }

    let home_dir = home_dir_from_env()?;
    let launcher = resolve_profile_terminal_launcher(&home_dir, requested_terminal)?;
    let argv = explicit_terminal_launch_argv(&launcher.launcher, parsed, config_override);
    let mut extra_env = launcher.env;
    extra_env.push((
        "YAZELIX_SKIP_STABLE_WRAPPER_REDIRECT".to_string(),
        Some("1".to_string()),
    ));
    if requested_terminal != "mars" {
        extra_env.extend([
            ("MARS_APPEARANCE".to_string(), None),
            (MARS_EMOJI_FONT_ENV.to_string(), None),
            (MARS_EMOJI_FONT_SOURCE_ENV.to_string(), None),
            ("MARS_EFFECTS".to_string(), None),
            ("MARS_PROFILE".to_string(), None),
        ]);
    }

    let cwd = std::env::current_dir().map_err(|source| {
        CoreError::io(
            "launch_cwd",
            "Could not read the current working directory.",
            "cd into a valid directory, then retry.",
            ".",
            source,
        )
    })?;
    let output = command_output_with_overrides(
        &argv,
        None,
        &cwd,
        RUNTIME_RELAUNCH_CLEARED_ENV_KEYS,
        &extra_env,
        "terminal_variant_launch",
        "Install the requested Yazelix terminal variant through Home Manager and retry.",
    )?;
    if !output.status.success() {
        print_completed_output(&output);
        eprintln!(
            "❌ Failed to launch Yazelix terminal variant '{}' through {}.",
            terminal_display_name(requested_terminal),
            launcher.desktop_path.display()
        );
        return Ok(output.status.code().unwrap_or(1));
    }
    if parsed.verbose {
        println!(
            "✅ Launch request sent to {}",
            terminal_display_name(requested_terminal)
        );
    }
    Ok(0)
}

fn explicit_terminal_launch_argv(
    launcher_path: &Path,
    parsed: &LaunchArgs,
    config_override: Option<&str>,
) -> Vec<String> {
    let mut argv = vec![
        launcher_path.to_string_lossy().into_owned(),
        "launch".to_string(),
    ];
    if parsed.home {
        argv.push("--home".to_string());
    }
    if let Some(path) = parsed.path.as_deref() {
        argv.extend(["--path".to_string(), path.to_string()]);
    }
    if let Some(config) = config_override {
        argv.extend(["--config".to_string(), config.to_string()]);
    }
    if parsed.verbose {
        argv.push("--verbose".to_string());
    }
    argv
}

fn resolve_profile_terminal_launcher(
    home_dir: &Path,
    terminal: &str,
) -> Result<PackagedTerminalLauncher, CoreError> {
    let candidates = profile_terminal_desktop_entry_candidates(home_dir, terminal);
    for candidate in &candidates {
        if !candidate.exists() {
            continue;
        }
        let raw = fs::read_to_string(candidate).map_err(|source| {
            CoreError::io(
                "read_terminal_launcher_desktop_entry",
                format!(
                    "Could not read Yazelix terminal launcher desktop entry {}.",
                    candidate.display()
                ),
                "Regenerate the Home Manager profile and retry.",
                candidate.display().to_string(),
                source,
            )
        })?;
        let exec = desktop_entry_exec_value(&raw).ok_or_else(|| {
            CoreError::classified(
                ErrorClass::Runtime,
                "missing_terminal_launcher_exec",
                format!(
                    "Yazelix terminal launcher {} has no Exec= command.",
                    candidate.display()
                ),
                "Regenerate the Home Manager profile and retry.",
                serde_json::json!({ "desktop_entry": candidate }),
            )
        })?;
        let launcher = parse_packaged_terminal_launcher_exec(candidate, terminal, &exec)?;
        if fs::symlink_metadata(&launcher.launcher).is_err() {
            return Err(CoreError::classified(
                ErrorClass::Runtime,
                "missing_packaged_terminal_launcher",
                format!(
                    "Yazelix terminal variant '{}' points at missing launcher {}.",
                    terminal_display_name(terminal),
                    launcher.launcher.display()
                ),
                "Rebuild Home Manager so the extra terminal launcher points at a live Yazelix package.",
                serde_json::json!({
                    "terminal": terminal,
                    "desktop_entry": candidate,
                    "launcher": launcher.launcher,
                }),
            ));
        }
        return Ok(launcher);
    }

    Err(CoreError::classified(
        ErrorClass::Runtime,
        "missing_packaged_terminal_variant",
        format!(
            "Yazelix terminal variant '{}' is not installed as a packaged launcher.",
            terminal_display_name(terminal)
        ),
        format!(
            "Add '{}' to programs.yazelix.extra_terminal_launchers or make it programs.yazelix.terminal, rebuild Home Manager, then retry.",
            terminal
        ),
        serde_json::json!({
            "terminal": terminal,
            "checked_desktop_entries": candidates,
        }),
    ))
}

fn profile_terminal_desktop_entry_candidates(home_dir: &Path, terminal: &str) -> Vec<PathBuf> {
    let file_name = terminal_desktop_entry_file_name(terminal);
    let mut candidates = vec![
        home_dir
            .join(".nix-profile")
            .join("share")
            .join("applications")
            .join(&file_name),
    ];
    if let Ok(user) = std::env::var("USER") {
        let trimmed = user.trim();
        if !trimmed.is_empty() {
            candidates.push(
                PathBuf::from("/etc/profiles/per-user")
                    .join(trimmed)
                    .join("share")
                    .join("applications")
                    .join(file_name),
            );
        }
    }
    candidates
}

fn desktop_entry_exec_value(raw: &str) -> Option<String> {
    raw.lines()
        .find_map(|line| line.trim().strip_prefix("Exec="))
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

fn parse_packaged_terminal_launcher_exec(
    desktop_path: &Path,
    terminal: &str,
    exec: &str,
) -> Result<PackagedTerminalLauncher, CoreError> {
    let tokens = split_desktop_exec_tokens(exec)?;
    let mut index = 0;
    let mut env = Vec::new();
    if tokens.first().map(String::as_str) == Some("env") {
        index = 1;
        while let Some(token) = tokens.get(index) {
            let Some((key, value)) = parse_env_assignment(token) else {
                break;
            };
            env.push((key.to_string(), Some(value.to_string())));
            index += 1;
        }
    }
    let Some(launcher) = tokens.get(index) else {
        return Err(unsupported_desktop_exec(desktop_path, terminal, exec));
    };
    let trailing = &tokens[index + 1..];
    if trailing != ["desktop", "launch"] {
        return Err(unsupported_desktop_exec(desktop_path, terminal, exec));
    }

    let launcher = PathBuf::from(launcher);
    if !launcher.is_absolute() {
        return Err(CoreError::classified(
            ErrorClass::Runtime,
            "relative_terminal_launcher",
            format!(
                "Yazelix terminal launcher {} uses a relative Exec= launcher.",
                desktop_path.display()
            ),
            "Regenerate the Home Manager profile so the desktop entry points at a packaged Yazelix launcher.",
            serde_json::json!({
                "terminal": terminal,
                "desktop_entry": desktop_path,
                "exec": exec,
            }),
        ));
    }

    Ok(PackagedTerminalLauncher {
        launcher,
        env,
        desktop_path: desktop_path.to_path_buf(),
    })
}

fn unsupported_desktop_exec(desktop_path: &Path, terminal: &str, exec: &str) -> CoreError {
    CoreError::classified(
        ErrorClass::Runtime,
        "unsupported_terminal_launcher_exec",
        format!(
            "Yazelix terminal launcher {} has an unsupported Exec= command.",
            desktop_path.display()
        ),
        "Regenerate the Home Manager profile so the desktop entry uses a packaged Yazelix launcher.",
        serde_json::json!({
            "terminal": terminal,
            "desktop_entry": desktop_path,
            "exec": exec,
        }),
    )
}

fn parse_env_assignment(token: &str) -> Option<(&str, &str)> {
    let (key, value) = token.split_once('=')?;
    let mut chars = key.chars();
    let first = chars.next()?;
    if !(first == '_' || first.is_ascii_alphabetic()) {
        return None;
    }
    if !chars.all(|ch| ch == '_' || ch.is_ascii_alphanumeric()) {
        return None;
    }
    Some((key, value))
}

fn split_desktop_exec_tokens(exec: &str) -> Result<Vec<String>, CoreError> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut token_started = false;
    let mut in_quotes = false;
    let mut chars = exec.chars().peekable();

    while let Some(ch) = chars.next() {
        if in_quotes {
            match ch {
                '"' => in_quotes = false,
                '\\' => current.push(chars.next().unwrap_or('\\')),
                other => current.push(other),
            }
            token_started = true;
            continue;
        }

        match ch {
            '"' => {
                in_quotes = true;
                token_started = true;
            }
            '\\' => {
                current.push(chars.next().unwrap_or('\\'));
                token_started = true;
            }
            other if other.is_whitespace() => {
                if token_started {
                    tokens.push(std::mem::take(&mut current));
                    token_started = false;
                }
            }
            other => {
                current.push(other);
                token_started = true;
            }
        }
    }

    if in_quotes {
        return Err(CoreError::usage(format!(
            "Unterminated quoted string in desktop Exec= command: {exec}"
        )));
    }
    if token_started {
        tokens.push(current);
    }
    Ok(tokens)
}

fn repair_desktop_runtime_state_if_required(
    desktop_fast_path: bool,
    needs_refresh: bool,
    config_override: Option<&str>,
) -> Result<(), CoreError> {
    if !desktop_fast_path || !needs_refresh {
        return Ok(());
    }

    let request = RuntimeMaterializationRepairEvaluateRequest {
        plan: runtime_materialization_plan_request_from_env(config_override)?,
        force: false,
    };
    repair_runtime_materialization(&request)?;
    Ok(())
}

fn normalize_launch_session_name(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() || trimmed.eq_ignore_ascii_case("unknown") {
        return None;
    }
    Some(trimmed.to_string())
}

fn window_title_session_name_from_env(desktop_fast_path: bool) -> Option<String> {
    if desktop_fast_path {
        return None;
    }
    std::env::var("YAZELIX_ZELLIJ_SESSION_NAME")
        .ok()
        .and_then(|value| normalize_launch_session_name(&value))
}

fn terminal_window_title_prefix(terminal: &str) -> String {
    format!("Yazelix - {} - ", terminal_display_name(terminal))
}

struct LaunchFlowInput<'a> {
    requested_path: Option<&'a str>,
    config_override: Option<&'a str>,
    home: bool,
    verbose: bool,
    desktop_fast_path: bool,
    env_removals: &'a [&'a str],
}

struct LaunchExecutionPlan {
    runtime_dir: PathBuf,
    state_dir: PathBuf,
    home_dir: PathBuf,
    working_dir: PathBuf,
    active_terminal: String,
    terminal_candidates: Vec<TerminalCandidate>,
    materialization: LaunchMaterializationData,
    runtime_env: serde_json::Map<String, serde_json::Value>,
    terminal_transparency: String,
    window_title_session_name: Option<String>,
    needs_refresh: bool,
}

type LaunchProbe = fn(
    &Path,
    &Path,
    &[String],
    &serde_json::Map<String, serde_json::Value>,
    &Path,
    bool,
    &[&str],
    &[(String, Option<String>)],
) -> Result<std::process::Output, CoreError>;

pub(super) fn run_launch_flow(
    requested_path: Option<&str>,
    config_override: Option<&str>,
    home: bool,
    verbose: bool,
    desktop_fast_path: bool,
    env_removals: &[&str],
) -> Result<i32, CoreError> {
    let input = LaunchFlowInput {
        requested_path,
        config_override,
        home,
        verbose,
        desktop_fast_path,
        env_removals,
    };
    let plan = build_launch_execution_plan(&input)?;
    print_launch_materialization_status(&plan, &input);
    execute_launch_plan(&plan, &input)
}

fn build_launch_execution_plan(
    input: &LaunchFlowInput<'_>,
) -> Result<LaunchExecutionPlan, CoreError> {
    let runtime_dir = runtime_dir_from_env()?;
    let state_dir = state_dir_from_env()?;
    let home_dir = home_dir_from_env()?;
    let config_state = compute_config_state(&config_state_compute_request_from_env(
        input.config_override,
    )?)?;
    repair_desktop_runtime_state_if_required(
        input.desktop_fast_path,
        config_state.needs_refresh,
        input.config_override,
    )?;
    let active_terminal = active_terminal_from_runtime_dir(&runtime_dir)?;

    let requested_working_dir = resolve_requested_working_dir(input.requested_path, input.home)?;
    let command_search_paths = std::env::var_os("PATH")
        .map(|raw| std::env::split_paths(&raw).collect::<Vec<_>>())
        .unwrap_or_default();
    let preflight = evaluate_startup_launch_preflight(&StartupLaunchPreflightRequest {
        startup: None,
        launch: Some(LaunchPreflightPayload {
            working_dir: requested_working_dir,
            requested_terminal: String::new(),
            terminals: vec![active_terminal.clone()],
            command_search_paths,
        }),
    })?;
    let working_dir = PathBuf::from(preflight.working_dir);
    let terminal_candidates = preflight.terminal_candidates.unwrap_or_default();

    let req =
        launch_materialization_request_from_env(input.desktop_fast_path, input.config_override)?;
    let materialization = prepare_launch_materialization(&req, &config_state.config)?;
    let runtime_data = compute_runtime_env(&runtime_env_request(
        runtime_dir.clone(),
        &config_state.config,
    )?)?;
    let terminal_transparency = config_state
        .config
        .get("transparency")
        .and_then(|value| value.as_str())
        .unwrap_or("none")
        .to_string();

    Ok(LaunchExecutionPlan {
        runtime_dir,
        state_dir,
        home_dir,
        working_dir,
        active_terminal,
        terminal_candidates,
        materialization,
        runtime_env: runtime_data.runtime_env,
        terminal_transparency,
        window_title_session_name: window_title_session_name_from_env(input.desktop_fast_path),
        needs_refresh: config_state.needs_refresh,
    })
}

fn print_launch_materialization_status(plan: &LaunchExecutionPlan, input: &LaunchFlowInput<'_>) {
    if !input.desktop_fast_path && !plan.materialization.generated_terminals.is_empty() {
        let generated = plan
            .materialization
            .generated_terminals
            .iter()
            .map(|entry| terminal_display_name(&entry.terminal))
            .collect::<Vec<_>>()
            .join(", ");
        println!("Generating bundled terminal configurations...");
        println!("✓ Generated terminal configurations ({generated})");
        println!("📋 Static example configs for other terminals in configs/terminal_emulators/");
    }
    if plan.materialization.rerolled_ghostty_cursor && input.verbose {
        println!("🎲 Rerolling Yazelix random cursor settings for this window...");
        println!("✓ Rerolled Yazelix cursor settings");
    }
}

fn execute_launch_plan(
    plan: &LaunchExecutionPlan,
    input: &LaunchFlowInput<'_>,
) -> Result<i32, CoreError> {
    let mut failures = Vec::new();
    let launch_probe: LaunchProbe = if input.desktop_fast_path {
        run_desktop_deferred_launch_probe
    } else {
        run_detached_launch_probe
    };
    for candidate in &plan.terminal_candidates {
        let config_path = match launch_candidate_config_path(plan, candidate) {
            Ok(config_path) => config_path,
            Err(reason) => {
                failures.push((candidate.name.clone(), reason));
                continue;
            }
        };

        let argv = build_launch_command_argv(
            &plan.runtime_dir,
            candidate,
            &config_path,
            &plan.working_dir,
            plan.window_title_session_name.as_deref(),
        )?;
        if input.verbose {
            println!("Using terminal: {}", candidate.name);
            println!("Running: {}", render_argv_for_display(&argv));
        }

        let extra_env = launch_candidate_extra_env(plan, input, candidate, &config_path)?;

        let output = launch_probe(
            &plan.runtime_dir,
            &plan.state_dir,
            &argv,
            &plan.runtime_env,
            &plan.working_dir,
            plan.needs_refresh,
            input.env_removals,
            &extra_env,
        )?;

        if output.status.success() {
            if input.verbose {
                println!("✅ Launch request sent to {}", candidate.name);
            }
            return Ok(0);
        }

        let reason = render_launch_failure(&output);
        failures.push((candidate.name.clone(), reason));
    }

    let summary = failures
        .iter()
        .map(|(name, reason)| format!("  - {name}: {reason}"))
        .collect::<Vec<_>>()
        .join("\n");
    let message = format!(
        "Failed to launch Yazelix terminal variant '{}'.\n{summary}",
        plan.active_terminal
    );
    Err(CoreError::classified(
        ErrorClass::Runtime,
        "launch_failed",
        message,
        "Reinstall Yazelix so the selected terminal variant is packaged correctly, or install a different Yazelix terminal variant.",
        serde_json::json!({}),
    ))
}

fn launch_candidate_config_path(
    plan: &LaunchExecutionPlan,
    candidate: &TerminalCandidate,
) -> Result<PathBuf, String> {
    let fallback_config_path = resolve_terminal_config_path(
        &plan.home_dir,
        &plan.state_dir,
        &plan.materialization.terminal_config_mode,
        &candidate.terminal,
    )?;

    Ok(
        resolve_materialized_terminal_config_path(&plan.materialization, &candidate.terminal)
            .unwrap_or(fallback_config_path),
    )
}

fn launch_candidate_extra_env(
    plan: &LaunchExecutionPlan,
    input: &LaunchFlowInput<'_>,
    candidate: &TerminalCandidate,
    config_path: &Path,
) -> Result<Vec<(String, Option<String>)>, CoreError> {
    let mut extra_env = vec![
        (
            "YAZELIX_RUNTIME_DIR".to_string(),
            Some(plan.runtime_dir.to_string_lossy().into_owned()),
        ),
        ("MARS".to_string(), Some(candidate.terminal.clone())),
        (
            "MARS_WINDOW_TITLE_PREFIX".to_string(),
            Some(terminal_window_title_prefix(&candidate.terminal)),
        ),
    ];
    if candidate.terminal == "mars" {
        extra_env.extend(mars_process_boundary_env(config_path)?);
    }
    if candidate.terminal == "rio" {
        extra_env.extend(rio_process_boundary_env(
            config_path,
            &plan.terminal_transparency,
        )?);
    }
    for key in ["YAZELIX_SWEEP_TEST_ID", "YAZELIX_LAYOUT_OVERRIDE"] {
        if let Ok(value) = std::env::var(key) {
            if !value.trim().is_empty() {
                extra_env.push((key.to_string(), Some(value)));
            }
        }
    }
    extra_env.extend(config_override_extra_env(input.config_override));
    Ok(extra_env)
}

fn mars_process_boundary_env(
    config_path: &Path,
) -> Result<Vec<(String, Option<String>)>, CoreError> {
    let config_dir = config_path.parent().ok_or_else(|| {
        CoreError::classified(
            ErrorClass::Runtime,
            "invalid_mars_config_path",
            format!(
                "Generated Mars Terminal config path has no parent directory: {}.",
                config_path.display()
            ),
            "Regenerate Yazelix runtime state with `yzx refresh`, then retry.",
            serde_json::json!({}),
        )
    })?;

    let mut env = vec![
        ("RIO_CONFIG_HOME".to_string(), None),
        (
            "MARS_CONFIG".to_string(),
            Some(config_dir.to_string_lossy().into_owned()),
        ),
        (MARS_CHILD_ENV_SANITIZE.to_string(), Some("1".to_string())),
        (
            "MARS_APP_ID".to_string(),
            Some(terminal_startup_wm_class("mars")),
        ),
    ];
    env.extend(
        MARS_EMOJI_ENV_KEYS
            .iter()
            .map(|key| ((*key).to_string(), None)),
    );
    Ok(env)
}

fn rio_process_boundary_env(
    config_path: &Path,
    transparency: &str,
) -> Result<Vec<(String, Option<String>)>, CoreError> {
    rio_process_boundary_env_for_display(
        config_path,
        transparency,
        std::env::var_os("DISPLAY").is_some(),
    )
}

fn rio_process_boundary_env_for_display(
    config_path: &Path,
    transparency: &str,
    x11_display_available: bool,
) -> Result<Vec<(String, Option<String>)>, CoreError> {
    let config_dir = config_path.parent().ok_or_else(|| {
        CoreError::classified(
            ErrorClass::Runtime,
            "invalid_rio_config_path",
            format!(
                "Generated Rio config path has no parent directory: {}.",
                config_path.display()
            ),
            "Regenerate Yazelix runtime state with `yzx refresh`, then retry.",
            serde_json::json!({}),
        )
    })?;

    let mut env = vec![(
        "RIO_CONFIG_HOME".to_string(),
        Some(config_dir.to_string_lossy().into_owned()),
    )];
    if rio_should_force_x11_for_transparency(transparency, x11_display_available) {
        env.push(("WINIT_UNIX_BACKEND".to_string(), Some("x11".to_string())));
        env.push(("WAYLAND_DISPLAY".to_string(), None));
    }
    Ok(env)
}

fn rio_should_force_x11_for_transparency(transparency: &str, x11_display_available: bool) -> bool {
    #[cfg(target_os = "linux")]
    {
        transparency.trim() != "none" && x11_display_available
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = (transparency, x11_display_available);
        false
    }
}

fn resolve_materialized_terminal_config_path(
    materialization: &LaunchMaterializationData,
    terminal: &str,
) -> Option<PathBuf> {
    if materialization.terminal_config_mode != "yazelix" {
        return None;
    }

    materialization
        .generated_terminals
        .iter()
        .find(|entry| entry.terminal == terminal)
        .map(|entry| PathBuf::from(&entry.path))
}

fn parse_launch_args(args: &[String]) -> Result<LaunchArgs, CoreError> {
    let mut parsed = LaunchArgs::default();
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--help" | "-h" | "help" => parsed.help = true,
            "--home" => parsed.home = true,
            "--verbose" => parsed.verbose = true,
            "--term" | "--terminal" | "-t" => {
                index += 1;
                let value = args.get(index).ok_or_else(|| {
                    CoreError::usage(
                        "Missing value for yzx launch --term. Try `yzx launch --help`.",
                    )
                })?;
                if parsed.terminal.is_some() {
                    return Err(CoreError::usage(
                        "yzx launch accepts at most one terminal selector.",
                    ));
                }
                parsed.terminal = Some(normalize_terminal_id(value).ok_or_else(|| {
                    CoreError::usage(format!(
                        "Unsupported yzx launch terminal '{value}'. Supported terminals: {}.",
                        SUPPORTED_TERMINALS.join(", ")
                    ))
                })?);
            }
            "--path" | "-p" => {
                index += 1;
                let value = args.get(index).ok_or_else(|| {
                    CoreError::usage(
                        "Missing value for yzx launch --path. Try `yzx launch --help`.",
                    )
                })?;
                parsed.path = Some(value.clone());
            }
            "--config" => {
                index += 1;
                let value = args.get(index).ok_or_else(|| {
                    CoreError::usage(
                        "Missing value for yzx launch --config. Try `yzx launch --help`.",
                    )
                })?;
                if parsed.config.is_some() {
                    return Err(CoreError::usage(
                        "yzx launch accepts at most one --config override.",
                    ));
                }
                parsed.config = Some(resolve_cli_config_override(value)?);
            }
            "--with" => {
                index += 1;
                let value = args.get(index).ok_or_else(|| {
                    CoreError::usage(
                        "Missing value for yzx launch --with. Try `yzx launch --help`.",
                    )
                })?;
                parsed.with_overrides.push(value.clone());
            }
            other if other.starts_with('-') => {
                return Err(CoreError::usage(format!(
                    "Unknown argument for yzx launch: {other}. Try `yzx launch --help`."
                )));
            }
            other => {
                if parsed.path.is_some() {
                    return Err(CoreError::usage(
                        "yzx launch accepts at most one positional cwd override.",
                    ));
                }
                parsed.path = Some(other.to_string());
            }
        }
        index += 1;
    }
    Ok(parsed)
}

fn print_launch_help() {
    println!("Launch Yazelix in a new terminal window");
    println!();
    println!("Usage:");
    println!(
        "  yzx launch [-t <terminal>] [--path <dir> | --home] [--config <file>] [--with key=value] [--verbose]"
    );
    println!();
    println!("Options:");
    println!("  -t, --term, --terminal    Launch an installed packaged terminal variant");
}

fn render_argv_for_display(argv: &[String]) -> String {
    argv.iter()
        .map(|arg| {
            if arg
                .chars()
                .all(|ch| ch.is_ascii_alphanumeric() || "/._:=,@+-".contains(ch))
            {
                arg.clone()
            } else {
                format!("{arg:?}")
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::super::config_override::resolve_config_override_path;
    use super::*;

    // Defends: Rust launch arg parsing keeps public path/config/session override flags and packaged terminal selection.
    #[test]
    fn parse_launch_args_accepts_supported_flags() {
        let expected_config = resolve_config_override_path(
            "settings.jsonc",
            &std::env::current_dir().unwrap(),
            &home_dir_from_env().unwrap(),
        )
        .unwrap();
        let parsed = parse_launch_args(&[
            "-p".into(),
            "/tmp/demo".into(),
            "--config".into(),
            "settings.jsonc".into(),
            "--with".into(),
            "editor.command=nvim".into(),
            "-t".into(),
            "Ghostty".into(),
            "--verbose".into(),
        ])
        .unwrap();

        assert_eq!(parsed.path.as_deref(), Some("/tmp/demo"));
        assert_eq!(parsed.terminal.as_deref(), Some("ghostty"));
        assert_eq!(parsed.config.as_deref(), Some(expected_config.as_str()));
        assert_eq!(parsed.with_overrides, vec!["editor.command=nvim"]);
        assert!(parsed.verbose);
    }

    // Defends: explicit launch session names are available for the initial terminal title.
    #[test]
    fn window_title_session_name_keeps_explicit_session() {
        assert_eq!(
            normalize_launch_session_name(" work ").as_deref(),
            Some("work")
        );
    }

    // Defends: desktop launch waits for Zellij's emitted session title instead of inheriting stale ambient state.
    #[test]
    fn window_title_session_name_omits_desktop_fast_path_env() {
        assert_eq!(window_title_session_name_from_env(true), None);
    }

    // Defends: Zellij receives the terminal-specific prefix needed to emit the final OS title.
    #[test]
    fn terminal_window_title_prefix_names_selected_terminal() {
        assert_eq!(
            terminal_window_title_prefix("ghostty"),
            "Yazelix - Ghostty - "
        );
        assert_eq!(terminal_window_title_prefix("mars"), "Yazelix - Mars - ");
    }

    // Defends: every documented terminal selector spelling maps to the same packaged-variant field.
    #[test]
    fn parse_launch_args_accepts_terminal_selector_aliases() {
        for flag in ["-t", "--term", "--terminal"] {
            let parsed = parse_launch_args(&[flag.into(), "mars".into()]).unwrap();
            assert_eq!(parsed.terminal.as_deref(), Some("mars"));
        }
    }

    // Defends: explicit terminal selection does not revive unsupported host-terminal fallback names.
    #[test]
    fn parse_launch_args_rejects_unsupported_terminal_selector() {
        let err = parse_launch_args(&["--term".into(), "alacritty".into()]).unwrap_err();

        assert_eq!(err.code(), "invalid_arguments");
        assert!(err.message().contains("Unsupported yzx launch terminal"));
        assert!(err.message().contains("ghostty"));
        assert!(err.message().contains("mars"));
    }

    // Defends: cross-variant launch forwards one materialized config override and leaves terminal selection behind to avoid recursion.
    #[test]
    fn explicit_terminal_launch_argv_replaces_term_and_with_with_config() {
        let parsed = LaunchArgs {
            path: Some("/tmp/work".to_string()),
            terminal: Some("ghostty".to_string()),
            config: None,
            with_overrides: vec!["editor.command=nvim".to_string()],
            home: false,
            verbose: true,
            help: false,
        };

        let argv = explicit_terminal_launch_argv(
            Path::new("/nix/store/yazelix-ghostty/bin/yzx"),
            &parsed,
            Some("/state/config_overrides/session/settings.jsonc"),
        );

        assert_eq!(
            argv,
            vec![
                "/nix/store/yazelix-ghostty/bin/yzx",
                "launch",
                "--path",
                "/tmp/work",
                "--config",
                "/state/config_overrides/session/settings.jsonc",
                "--verbose",
            ]
        );
        assert!(!argv.contains(&"--term".to_string()));
        assert!(!argv.contains(&"--with".to_string()));
    }

    // Defends: Home Manager extra launchers can carry env assignments before the packaged yzx launcher.
    #[test]
    fn parse_packaged_terminal_launcher_exec_accepts_env_and_quoted_path() {
        let desktop_path = Path::new(
            "/home/demo/.nix-profile/share/applications/com.yazelix.Yazelix.Mars.desktop",
        );
        let parsed = parse_packaged_terminal_launcher_exec(
            desktop_path,
            "mars",
            r#"env YAZELIX_SKIP_STABLE_WRAPPER_REDIRECT=1 MARS_APP_ID=com.yazelix.Yazelix.Mars MARS_APPEARANCE=light MARS_EMOJI_FONT=serenityos MARS_EMOJI_FONT_SOURCE=home-manager MARS_PROFILE=shaders "/nix/store/with space/bin/yzx" desktop launch"#,
        )
        .unwrap();

        assert_eq!(
            parsed.launcher,
            PathBuf::from("/nix/store/with space/bin/yzx")
        );
        assert_eq!(
            parsed.env,
            vec![
                (
                    "YAZELIX_SKIP_STABLE_WRAPPER_REDIRECT".to_string(),
                    Some("1".to_string())
                ),
                (
                    "MARS_APP_ID".to_string(),
                    Some("com.yazelix.Yazelix.Mars".to_string())
                ),
                ("MARS_APPEARANCE".to_string(), Some("light".to_string())),
                (
                    MARS_EMOJI_FONT_ENV.to_string(),
                    Some("serenityos".to_string())
                ),
                (
                    MARS_EMOJI_FONT_SOURCE_ENV.to_string(),
                    Some("home-manager".to_string())
                ),
                ("MARS_PROFILE".to_string(), Some("shaders".to_string())),
            ]
        );
        assert_eq!(parsed.desktop_path, desktop_path);
    }

    // Regression: --term must resolve installed Yazelix package launchers from profile desktop entries, not host PATH terminal commands.
    #[test]
    fn resolve_profile_terminal_launcher_reads_home_manager_profile_entry() {
        let tmp = tempfile::TempDir::new().unwrap();
        let home = tmp.path().join("home");
        let launcher = tmp.path().join("store/yazelix-ghostty/bin/yzx");
        let desktop_entry = home
            .join(".nix-profile/share/applications")
            .join(terminal_desktop_entry_file_name("ghostty"));
        std::fs::create_dir_all(launcher.parent().unwrap()).unwrap();
        std::fs::create_dir_all(desktop_entry.parent().unwrap()).unwrap();
        std::fs::write(&launcher, "#!/bin/sh\n").unwrap();
        std::fs::write(
            &desktop_entry,
            format!(
                "[Desktop Entry]\nName=New Yazelix - Ghostty\nExec=env YAZELIX_SKIP_STABLE_WRAPPER_REDIRECT=1 {} desktop launch\n",
                launcher.display()
            ),
        )
        .unwrap();

        let resolved = resolve_profile_terminal_launcher(&home, "ghostty").unwrap();

        assert_eq!(resolved.launcher, launcher);
        assert_eq!(
            resolved.env,
            vec![(
                "YAZELIX_SKIP_STABLE_WRAPPER_REDIRECT".to_string(),
                Some("1".to_string())
            )]
        );
        assert_eq!(resolved.desktop_path, desktop_entry);
    }

    // Defends: mars gets Yazelix config only at the terminal process boundary, while ambient host Rio config is cleared.
    #[test]
    fn mars_process_boundary_env_clears_host_rio_config_and_sets_app_id() {
        let env = mars_process_boundary_env(Path::new(
            "/state/configs/terminal_emulators/mars/config.toml",
        ))
        .unwrap();

        assert_eq!(
            env,
            vec![
                ("RIO_CONFIG_HOME".to_string(), None),
                (
                    "MARS_CONFIG".to_string(),
                    Some("/state/configs/terminal_emulators/mars".to_string())
                ),
                (MARS_CHILD_ENV_SANITIZE.to_string(), Some("1".to_string())),
                (
                    "MARS_APP_ID".to_string(),
                    Some("com.yazelix.Yazelix.Mars".to_string())
                ),
                (MARS_EMOJI_FONT_ENV.to_string(), None),
                (MARS_EMOJI_FONT_SOURCE_ENV.to_string(), None),
            ]
        );
    }

    // Defends: vanilla Rio uses Rio's supported RIO_CONFIG_HOME lookup instead of ambient host config or mars-only env.
    #[test]
    fn rio_process_boundary_env_points_at_selected_config_dir() {
        let env = rio_process_boundary_env(
            Path::new("/state/configs/terminal_emulators/rio/config.toml"),
            "none",
        )
        .unwrap();

        assert_eq!(
            env,
            vec![(
                "RIO_CONFIG_HOME".to_string(),
                Some("/state/configs/terminal_emulators/rio".to_string())
            )]
        );
    }

    // Regression: upstream Rio 0.4.5+ ignores opacity on COSMIC Wayland; transparent Linux launches use XWayland when available.
    #[cfg(target_os = "linux")]
    #[test]
    fn rio_process_boundary_env_forces_x11_for_transparent_linux_launches() {
        let env = rio_process_boundary_env_for_display(
            Path::new("/state/configs/terminal_emulators/rio/config.toml"),
            "low",
            true,
        )
        .unwrap();

        assert_eq!(
            env,
            vec![
                (
                    "RIO_CONFIG_HOME".to_string(),
                    Some("/state/configs/terminal_emulators/rio".to_string())
                ),
                ("WINIT_UNIX_BACKEND".to_string(), Some("x11".to_string())),
                ("WAYLAND_DISPLAY".to_string(), None),
            ]
        );
    }

    // Defends: pure Wayland sessions without DISPLAY still launch vanilla Rio instead of forcing an unavailable backend.
    #[test]
    fn rio_process_boundary_env_keeps_default_backend_without_x11_display() {
        let env = rio_process_boundary_env_for_display(
            Path::new("/state/configs/terminal_emulators/rio/config.toml"),
            "high",
            false,
        )
        .unwrap();

        assert_eq!(
            env,
            vec![(
                "RIO_CONFIG_HOME".to_string(),
                Some("/state/configs/terminal_emulators/rio".to_string())
            )]
        );
    }

    // Defends: vanilla Rio launches through Rio's own CLI shape instead of mars-only flags.
    #[test]
    fn rio_launch_argv_uses_selected_config_and_working_dir() {
        let tmp = tempfile::TempDir::new().unwrap();
        let runtime_dir = tmp.path().join("runtime");
        let posix_dir = runtime_dir.join("shells").join("posix");
        std::fs::create_dir_all(&posix_dir).unwrap();
        let startup_script = posix_dir.join("start_yazelix.sh");
        std::fs::write(&startup_script, "#!/bin/sh\n").unwrap();
        let config_path = tmp
            .path()
            .join("state/configs/terminal_emulators/rio/config.toml");
        let working_dir = tmp.path().join("workspace");
        std::fs::create_dir_all(config_path.parent().unwrap()).unwrap();
        std::fs::create_dir_all(&working_dir).unwrap();

        let argv = build_launch_command_argv(
            &runtime_dir,
            &crate::runtime_contract::TerminalCandidate {
                terminal: "rio".to_string(),
                name: "Rio".to_string(),
                command: "rio".to_string(),
            },
            &config_path,
            &working_dir,
            Some("work"),
        )
        .unwrap();

        assert_eq!(
            argv,
            vec![
                "rio".to_string(),
                "--title-placeholder".to_string(),
                "Yazelix - Rio - work".to_string(),
                "--working-dir".to_string(),
                working_dir.to_string_lossy().into_owned(),
                "-e".to_string(),
                startup_script.to_string_lossy().into_owned(),
            ]
        );
    }

    // Defends: Foot launches through its native CLI flags and the packaged Linux graphics wrapper boundary.
    #[test]
    fn foot_launch_argv_uses_selected_config_and_working_dir() {
        let tmp = tempfile::TempDir::new().unwrap();
        let runtime_dir = tmp.path().join("runtime");
        let posix_dir = runtime_dir.join("shells").join("posix");
        let libexec_dir = runtime_dir.join("libexec");
        std::fs::create_dir_all(&posix_dir).unwrap();
        std::fs::create_dir_all(&libexec_dir).unwrap();
        let startup_script = posix_dir.join("start_yazelix.sh");
        let nixgl = libexec_dir.join("nixGL");
        std::fs::write(&startup_script, "#!/bin/sh\n").unwrap();
        std::fs::write(&nixgl, "#!/bin/sh\n").unwrap();
        let config_path = tmp
            .path()
            .join("state/configs/terminal_emulators/foot/foot.ini");
        let working_dir = tmp.path().join("workspace");
        std::fs::create_dir_all(config_path.parent().unwrap()).unwrap();
        std::fs::create_dir_all(&working_dir).unwrap();

        let argv = build_launch_command_argv(
            &runtime_dir,
            &crate::runtime_contract::TerminalCandidate {
                terminal: "foot".to_string(),
                name: "Foot".to_string(),
                command: "foot".to_string(),
            },
            &config_path,
            &working_dir,
            None,
        )
        .unwrap();

        assert_eq!(
            argv,
            vec![
                nixgl.to_string_lossy().into_owned(),
                "foot".to_string(),
                format!("--config={}", config_path.to_string_lossy()),
                "--app-id=com.yazelix.Yazelix".to_string(),
                "--title=Yazelix - Foot".to_string(),
                format!("--working-directory={}", working_dir.to_string_lossy()),
                "--".to_string(),
                startup_script.to_string_lossy().into_owned(),
            ]
        );
    }

    // Regression: launch must pass a launch-scoped materialized config path when random cursor materialization produced one, otherwise it falls back to the shared generated path and leaks cursor preset changes across windows.
    #[test]
    fn materialized_yazelix_config_path_wins_for_launch() {
        let materialization = LaunchMaterializationData {
            terminal_config_mode: "yazelix".to_string(),
            generated_terminals: vec![crate::terminal_materialization::TerminalGeneratedConfig {
                terminal: "ghostty".to_string(),
                path: "/state/terminal_launches/123/configs/terminal_emulators/ghostty/config"
                    .to_string(),
            }],
            rerolled_ghostty_cursor: false,
        };

        assert_eq!(
            resolve_materialized_terminal_config_path(&materialization, "ghostty"),
            Some(PathBuf::from(
                "/state/terminal_launches/123/configs/terminal_emulators/ghostty/config"
            ))
        );
        assert_eq!(
            resolve_materialized_terminal_config_path(&materialization, "wezterm"),
            None
        );
    }
}
