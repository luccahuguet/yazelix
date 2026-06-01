// Test lane: default

use super::config_override::{
    config_override_extra_env, prepare_session_config_override, resolve_cli_config_override,
};
use super::process::{
    render_launch_failure, run_desktop_deferred_launch_probe, run_detached_launch_probe,
};
use super::resolve_requested_working_dir;
use super::terminal::{
    build_launch_command_argv, normalized_configured_terminals, print_empty_terminal_error,
    resolve_terminal_config_path, terminal_display_name,
};
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
    LaunchPreflightPayload, StartupLaunchPreflightRequest, evaluate_startup_launch_preflight,
};
use crate::runtime_env::compute_runtime_env;
use crate::runtime_materialization::{
    RuntimeMaterializationRepairEvaluateRequest, repair_runtime_materialization,
};
use std::path::PathBuf;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct LaunchArgs {
    path: Option<String>,
    config: Option<String>,
    with_overrides: Vec<String>,
    home: bool,
    terminal: Option<String>,
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
        parsed.terminal.as_deref(),
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

pub(super) fn run_launch_flow(
    requested_path: Option<&str>,
    config_override: Option<&str>,
    home: bool,
    requested_terminal: Option<&str>,
    verbose: bool,
    desktop_fast_path: bool,
    env_removals: &[&str],
) -> Result<i32, CoreError> {
    let runtime_dir = runtime_dir_from_env()?;
    let state_dir = state_dir_from_env()?;
    let config_state =
        compute_config_state(&config_state_compute_request_from_env(config_override)?)?;
    repair_desktop_runtime_state_if_required(
        desktop_fast_path,
        config_state.needs_refresh,
        config_override,
    )?;
    let configured_terminals = normalized_configured_terminals(&config_state.config);
    if configured_terminals.is_empty() {
        print_empty_terminal_error()?;
        return Ok(1);
    }

    let requested_working_dir = resolve_requested_working_dir(requested_path, home)?;
    let requested_terminal_text = requested_terminal.unwrap_or("").trim().to_string();
    let command_search_paths = std::env::var_os("PATH")
        .map(|raw| std::env::split_paths(&raw).collect::<Vec<_>>())
        .unwrap_or_default();
    let preflight = evaluate_startup_launch_preflight(&StartupLaunchPreflightRequest {
        startup: None,
        launch: Some(LaunchPreflightPayload {
            working_dir: requested_working_dir,
            requested_terminal: requested_terminal_text.clone(),
            terminals: configured_terminals.clone(),
            command_search_paths,
        }),
    })?;
    let working_dir = PathBuf::from(preflight.working_dir);
    let terminal_candidates = preflight.terminal_candidates.unwrap_or_default();

    let req = launch_materialization_request_from_env(
        terminal_candidates
            .iter()
            .map(|candidate| candidate.terminal.clone())
            .collect(),
        desktop_fast_path,
        desktop_fast_path && config_state.needs_refresh,
        config_override,
    )?;
    let materialization = prepare_launch_materialization(&req)?;
    if !desktop_fast_path && !materialization.generated_terminals.is_empty() {
        let generated = materialization
            .generated_terminals
            .iter()
            .map(|entry| terminal_display_name(&entry.terminal))
            .collect::<Vec<_>>()
            .join(", ");
        println!("Generating bundled terminal configurations...");
        println!("✓ Generated terminal configurations ({generated})");
        println!("📋 Static example configs for other terminals in configs/terminal_emulators/");
    }
    if materialization.rerolled_ghostty_cursor && verbose {
        println!("🎲 Rerolling Yazelix random cursor settings for this window...");
        println!("✓ Rerolled Yazelix cursor settings");
    }

    let runtime_data = compute_runtime_env(&runtime_env_request(
        runtime_dir.clone(),
        &config_state.config,
    )?)?;
    let runtime_env = runtime_data.runtime_env;

    let mut failures = Vec::new();
    for candidate in terminal_candidates {
        let config_path = match resolve_terminal_config_path(
            &home_dir_from_env()?,
            &state_dir,
            &materialization.terminal_config_mode,
            &candidate.terminal,
        ) {
            Ok(path) => path,
            Err(reason) => {
                failures.push((candidate.name.clone(), reason));
                continue;
            }
        };

        let argv = build_launch_command_argv(&runtime_dir, &candidate, &config_path, &working_dir)?;
        if verbose {
            println!("Using terminal: {}", candidate.name);
            println!("Running: {}", render_argv_for_display(&argv));
        }

        let mut extra_env = vec![
            (
                "YAZELIX_RUNTIME_DIR".to_string(),
                Some(runtime_dir.to_string_lossy().into_owned()),
            ),
            (
                "YAZELIX_TERMINAL".to_string(),
                Some(candidate.terminal.clone()),
            ),
            (
                "YAZELIX_CURSOR_NAME".to_string(),
                Some(launch_cursor_name_for_terminal(
                    &materialization,
                    &candidate.terminal,
                )),
            ),
            (
                "YAZELIX_CURSOR_COLOR".to_string(),
                launch_cursor_color_for_terminal(&materialization, &candidate.terminal),
            ),
            (
                "YAZELIX_CURSOR_FAMILY".to_string(),
                launch_cursor_fact_for_terminal(
                    &materialization.ghostty_cursor_family,
                    &candidate.terminal,
                ),
            ),
            (
                "YAZELIX_CURSOR_DIVIDER".to_string(),
                launch_cursor_fact_for_terminal(
                    &materialization.ghostty_cursor_divider,
                    &candidate.terminal,
                ),
            ),
            (
                "YAZELIX_CURSOR_PRIMARY_COLOR".to_string(),
                launch_cursor_fact_for_terminal(
                    &materialization.ghostty_cursor_primary_color_hex,
                    &candidate.terminal,
                ),
            ),
            (
                "YAZELIX_CURSOR_SECONDARY_COLOR".to_string(),
                launch_cursor_fact_for_terminal(
                    &materialization.ghostty_cursor_secondary_color_hex,
                    &candidate.terminal,
                ),
            ),
        ];
        if candidate.terminal == "yzxterm" {
            let config_dir = config_path.parent().ok_or_else(|| {
                CoreError::classified(
                    ErrorClass::Runtime,
                    "invalid_yzxterm_config_path",
                    format!(
                        "Generated Yazelix Terminal config path has no parent directory: {}.",
                        config_path.display()
                    ),
                    "Regenerate Yazelix runtime state with `yzx refresh`, then retry.",
                    serde_json::json!({}),
                )
            })?;
            extra_env.push((
                "YAZELIX_TERMINAL_CONFIG".to_string(),
                Some(config_dir.to_string_lossy().into_owned()),
            ));
        }
        if let Ok(value) = std::env::var("YAZELIX_SWEEP_TEST_ID") {
            if !value.trim().is_empty() {
                extra_env.push(("YAZELIX_SWEEP_TEST_ID".to_string(), Some(value)));
            }
        }
        if let Ok(value) = std::env::var("YAZELIX_LAYOUT_OVERRIDE") {
            if !value.trim().is_empty() {
                extra_env.push(("YAZELIX_LAYOUT_OVERRIDE".to_string(), Some(value)));
            }
        }
        extra_env.extend(config_override_extra_env(config_override));

        let output = if desktop_fast_path {
            run_desktop_deferred_launch_probe(
                &runtime_dir,
                &state_dir,
                &argv,
                &runtime_env,
                &working_dir,
                config_state.needs_refresh,
                env_removals,
                &extra_env,
            )?
        } else {
            run_detached_launch_probe(
                &runtime_dir,
                &state_dir,
                &argv,
                &runtime_env,
                &working_dir,
                config_state.needs_refresh,
                env_removals,
                &extra_env,
            )?
        };

        if output.status.success() {
            if verbose {
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
    let message = if requested_terminal_text.is_empty() {
        format!("Failed to launch any configured terminal.\n{summary}")
    } else {
        format!(
            "Failed to launch requested terminal '{}'.\n{}",
            requested_terminal_text, summary
        )
    };
    Err(CoreError::classified(
        ErrorClass::Runtime,
        "launch_failed",
        message,
        "Install a supported terminal or adjust [terminal].terminals to match what is available.",
        serde_json::json!({}),
    ))
}

fn launch_cursor_name_for_terminal(
    materialization: &LaunchMaterializationData,
    terminal: &str,
) -> String {
    if terminal_uses_yazelix_cursor(terminal) {
        materialization
            .ghostty_cursor_name
            .as_deref()
            .filter(|name| !name.trim().is_empty())
            .unwrap_or("n/a")
            .to_string()
    } else {
        "n/a".to_string()
    }
}

fn launch_cursor_color_for_terminal(
    materialization: &LaunchMaterializationData,
    terminal: &str,
) -> Option<String> {
    launch_cursor_fact_for_terminal(&materialization.ghostty_cursor_color_hex, terminal)
}

fn launch_cursor_fact_for_terminal(value: &Option<String>, terminal: &str) -> Option<String> {
    if terminal_uses_yazelix_cursor(terminal) {
        value.clone()
    } else {
        None
    }
}

fn terminal_uses_yazelix_cursor(terminal: &str) -> bool {
    matches!(terminal, "ghostty" | "yzxterm")
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
            "--terminal" | "-t" => {
                index += 1;
                let value = args.get(index).ok_or_else(|| {
                    CoreError::usage(
                        "Missing value for yzx launch --terminal. Try `yzx launch --help`.",
                    )
                })?;
                parsed.terminal = Some(value.clone());
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
        "  yzx launch [--path <dir> | --home] [--config <file>] [--with key=value] [--terminal <name>] [--verbose]"
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

    // Defends: Rust launch arg parsing keeps the public path and terminal flag aliases after the owner cut.
    #[test]
    fn parse_launch_args_accepts_aliases() {
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
            "kitty".into(),
            "--verbose".into(),
        ])
        .unwrap();

        assert_eq!(parsed.path.as_deref(), Some("/tmp/demo"));
        assert_eq!(parsed.config.as_deref(), Some(expected_config.as_str()));
        assert_eq!(parsed.with_overrides, vec!["editor.command=nvim"]);
        assert_eq!(parsed.terminal.as_deref(), Some("kitty"));
        assert!(parsed.verbose);
    }

    // Defends: launch publishes compact current-cursor facts for terminals that consume Yazelix cursor shaders and a clear n/a fallback elsewhere.
    #[test]
    fn launch_cursor_name_is_terminal_scoped() {
        let materialization = LaunchMaterializationData {
            terminal_config_mode: "yazelix".to_string(),
            selected_terminals: vec!["ghostty".to_string()],
            generated_terminals: Vec::new(),
            ghostty_cursor_name: Some("reef".to_string()),
            ghostty_cursor_color_hex: Some("#00ff66".to_string()),
            ghostty_cursor_family: Some("split".to_string()),
            ghostty_cursor_divider: Some("vertical".to_string()),
            ghostty_cursor_primary_color_hex: Some("#00e6ff".to_string()),
            ghostty_cursor_secondary_color_hex: Some("#00ff66".to_string()),
            rerolled_ghostty_cursor: false,
        };
        let missing = LaunchMaterializationData {
            ghostty_cursor_name: None,
            ghostty_cursor_color_hex: None,
            ghostty_cursor_family: None,
            ghostty_cursor_divider: None,
            ghostty_cursor_primary_color_hex: None,
            ghostty_cursor_secondary_color_hex: None,
            ..materialization.clone()
        };

        assert_eq!(
            launch_cursor_name_for_terminal(&materialization, "ghostty"),
            "reef"
        );
        assert_eq!(
            launch_cursor_name_for_terminal(&materialization, "yzxterm"),
            "reef"
        );
        assert_eq!(
            launch_cursor_name_for_terminal(&materialization, "wezterm"),
            "n/a"
        );
        assert_eq!(launch_cursor_name_for_terminal(&missing, "ghostty"), "n/a");
        assert_eq!(launch_cursor_name_for_terminal(&missing, "yzxterm"), "n/a");
        assert_eq!(
            launch_cursor_color_for_terminal(&materialization, "ghostty"),
            Some("#00ff66".to_string())
        );
        assert_eq!(
            launch_cursor_color_for_terminal(&materialization, "yzxterm"),
            Some("#00ff66".to_string())
        );
        assert_eq!(
            launch_cursor_color_for_terminal(&materialization, "wezterm"),
            None
        );
        assert_eq!(launch_cursor_color_for_terminal(&missing, "ghostty"), None);
        assert_eq!(
            launch_cursor_fact_for_terminal(&materialization.ghostty_cursor_family, "ghostty"),
            Some("split".to_string())
        );
        assert_eq!(
            launch_cursor_fact_for_terminal(&materialization.ghostty_cursor_family, "yzxterm"),
            Some("split".to_string())
        );
        assert_eq!(
            launch_cursor_fact_for_terminal(&materialization.ghostty_cursor_divider, "ghostty"),
            Some("vertical".to_string())
        );
        assert_eq!(
            launch_cursor_fact_for_terminal(
                &materialization.ghostty_cursor_primary_color_hex,
                "ghostty"
            ),
            Some("#00e6ff".to_string())
        );
        assert_eq!(
            launch_cursor_fact_for_terminal(
                &materialization.ghostty_cursor_secondary_color_hex,
                "ghostty"
            ),
            Some("#00ff66".to_string())
        );
        assert_eq!(
            launch_cursor_fact_for_terminal(&materialization.ghostty_cursor_family, "wezterm"),
            None
        );
    }
}
