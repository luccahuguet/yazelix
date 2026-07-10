// Test lane: default

use super::config_override::{
    config_override_extra_env, prepare_session_config_override, resolve_cli_config_override,
};
use super::process::{
    render_launch_failure, run_desktop_deferred_launch_probe, run_detached_launch_probe,
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
use crate::terminal_materialization::MARS_EMOJI_ENV_KEYS;
use crate::terminal_variant::{
    SESSION_TERMINAL_ENV, active_terminal_from_runtime_dir, terminal_display_name,
    terminal_startup_wm_class,
};
use std::path::{Path, PathBuf};

const MARS_CHILD_ENV_SANITIZE: &str = "MARS_CHILD_ENV_SANITIZE";

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct LaunchArgs {
    path: Option<String>,
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
    run_launch_flow(
        parsed.path.as_deref(),
        config_override.as_deref(),
        parsed.home,
        parsed.verbose,
        false,
        &[],
    )
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
    Ok(LaunchExecutionPlan {
        runtime_dir,
        state_dir,
        home_dir,
        working_dir,
        active_terminal,
        terminal_candidates,
        materialization,
        runtime_env: runtime_data.runtime_env,
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
        "Failed to launch Yazelix packaged terminal '{}'.\n{summary}",
        plan.active_terminal
    );
    Err(CoreError::classified(
        ErrorClass::Runtime,
        "launch_failed",
        message,
        "Reinstall Yazelix so the packaged Mars terminal is available, or configure a host terminal to run `yzx enter`.",
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
            SESSION_TERMINAL_ENV.to_string(),
            Some(candidate.terminal.clone()),
        ),
        (
            "MARS_WINDOW_TITLE_PREFIX".to_string(),
            Some(terminal_window_title_prefix(&candidate.terminal)),
        ),
    ];
    if candidate.terminal == "mars" {
        extra_env.extend(mars_process_boundary_env(config_path)?);
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
            "Run `yzx doctor --fix` to repair generated runtime state, then retry the launch.",
            serde_json::json!({}),
        )
    })?;

    let mut env = vec![
        ("RIO_CONFIG_HOME".to_string(), None),
        ("MARS_CONFIG".to_string(), None),
        (
            "MARS_CONFIG_HOME".to_string(),
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
        "  yzx launch [--path <dir> | --home] [--config <file>] [--with key=value] [--verbose]"
    );
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

    // Defends: Rust launch arg parsing keeps public path/config/session override flags without a packaged terminal selector.
    #[test]
    fn parse_launch_args_accepts_supported_flags() {
        let expected_config = resolve_config_override_path(
            "settings.jsonc",
            &std::env::current_dir().unwrap(),
            &crate::control_plane::home_dir_from_env().unwrap(),
        )
        .unwrap();
        let parsed = parse_launch_args(&[
            "-p".into(),
            "/tmp/demo".into(),
            "--config".into(),
            "settings.jsonc".into(),
            "--with".into(),
            "editor.command=nvim".into(),
            "--verbose".into(),
        ])
        .unwrap();

        assert_eq!(parsed.path.as_deref(), Some("/tmp/demo"));
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
                ("MARS_CONFIG".to_string(), None),
                (
                    "MARS_CONFIG_HOME".to_string(),
                    Some("/state/configs/terminal_emulators/mars".to_string())
                ),
                (MARS_CHILD_ENV_SANITIZE.to_string(), Some("1".to_string())),
                (
                    "MARS_APP_ID".to_string(),
                    Some("com.yazelix.Yazelix.Mars".to_string())
                ),
                (MARS_EMOJI_ENV_KEYS[0].to_string(), None),
                (MARS_EMOJI_ENV_KEYS[1].to_string(), None),
            ]
        );
    }

    // Regression: Mars is Rio-derived, but it does not accept a Yazelix CLI mode flag.
    #[test]
    fn mars_launch_argv_uses_supported_rio_compatible_flags() {
        let tmp = tempfile::TempDir::new().unwrap();
        let runtime_dir = tmp.path().join("runtime");
        let posix_dir = runtime_dir.join("shells").join("posix");
        std::fs::create_dir_all(&posix_dir).unwrap();
        let startup_script = posix_dir.join("start_yazelix.sh");
        std::fs::write(&startup_script, "#!/bin/sh\n").unwrap();
        let config_path = tmp
            .path()
            .join("state/configs/terminal_emulators/mars/config.toml");
        let working_dir = tmp.path().join("workspace");
        std::fs::create_dir_all(config_path.parent().unwrap()).unwrap();
        std::fs::create_dir_all(&working_dir).unwrap();

        let argv = build_launch_command_argv(
            &runtime_dir,
            &crate::runtime_contract::TerminalCandidate {
                terminal: "mars".to_string(),
                name: "Mars".to_string(),
                command: "mars".to_string(),
            },
            &config_path,
            &working_dir,
            Some("work"),
        )
        .unwrap();

        assert_eq!(
            argv,
            vec![
                "mars".to_string(),
                "--title-placeholder".to_string(),
                "Yazelix - Mars - work".to_string(),
                "--working-dir".to_string(),
                working_dir.to_string_lossy().into_owned(),
                "-e".to_string(),
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
                terminal: "mars".to_string(),
                path: "/state/terminal_launches/123/configs/terminal_emulators/mars/config.toml"
                    .to_string(),
            }],
            rerolled_ghostty_cursor: false,
        };

        assert_eq!(
            resolve_materialized_terminal_config_path(&materialization, "mars"),
            Some(PathBuf::from(
                "/state/terminal_launches/123/configs/terminal_emulators/mars/config.toml"
            ))
        );
        assert_eq!(
            resolve_materialized_terminal_config_path(&materialization, "ghostty"),
            None
        );
    }
}
