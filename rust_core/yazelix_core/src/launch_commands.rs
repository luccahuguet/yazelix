// Test lane: default
//! Rust-owned `yzx enter`, `yzx launch`, `yzx desktop`, and `yzx restart` owners.

use crate::bridge::{CoreError, ErrorClass};
use crate::config_state::compute_config_state;
use crate::control_plane::{
    config_override_from_env, config_state_compute_request_from_env, home_dir_from_env,
    runtime_dir_from_env, runtime_env_request, state_dir_from_env,
};
use crate::launch_materialization::{
    LaunchMaterializationData, launch_materialization_request_from_env,
    prepare_launch_materialization,
};
use crate::runtime_contract::{
    LaunchPreflightPayload, StartupLaunchPreflightRequest, evaluate_startup_launch_preflight,
};
use crate::runtime_env::compute_runtime_env;
use std::path::PathBuf;

mod config_override;
mod desktop;
mod enter;
mod process;
mod restart;
mod terminal;

use config_override::*;
use desktop::*;
use process::*;
use terminal::*;

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

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct DesktopArgs {
    subcommand: Option<String>,
    action: Option<String>,
    print_path: bool,
    help: bool,
}

pub fn run_yzx_enter(args: &[String]) -> Result<i32, CoreError> {
    enter::run_enter(args)
}

pub fn run_yzx_restart(args: &[String]) -> Result<i32, CoreError> {
    restart::run_restart(args)
}

pub fn run_yzx_launch(args: &[String]) -> Result<i32, CoreError> {
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

pub fn run_yzx_desktop(args: &[String]) -> Result<i32, CoreError> {
    let parsed = parse_desktop_args(args)?;
    if parsed.help || parsed.subcommand.is_none() {
        print_desktop_help();
        return Ok(0);
    }

    match parsed.subcommand.as_deref() {
        Some("install") => run_desktop_install(parsed.print_path),
        Some("uninstall") => run_desktop_uninstall(parsed.print_path),
        Some("launch") => run_desktop_launch(),
        Some("macos_preview") => match parsed.action.as_deref() {
            Some("install") => run_macos_preview_install(parsed.print_path),
            Some("uninstall") => run_macos_preview_uninstall(parsed.print_path),
            Some(other) => Err(CoreError::usage(format!(
                "Unknown yzx desktop macos_preview action: {other}. Try `yzx desktop --help`."
            ))),
            None => Err(CoreError::usage(
                "yzx desktop macos_preview requires an action: install or uninstall.",
            )),
        },
        Some(other) => Err(CoreError::usage(format!(
            "Unknown yzx desktop subcommand: {other}. Try `yzx desktop --help`."
        ))),
        None => unreachable!(),
    }
}

fn run_launch_flow(
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
        println!("🎲 Rerolling Ghostty random cursor settings for this Yazelix window...");
        println!("✓ Rerolled Ghostty cursor settings");
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
    if terminal == "ghostty" {
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
    if terminal == "ghostty" {
        value.clone()
    } else {
        None
    }
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

fn parse_desktop_args(args: &[String]) -> Result<DesktopArgs, CoreError> {
    let mut parsed = DesktopArgs::default();
    for arg in args {
        match arg.as_str() {
            "--help" | "-h" | "help" => parsed.help = true,
            "--print-path" | "-p" => parsed.print_path = true,
            other if other.starts_with('-') => {
                return Err(CoreError::usage(format!(
                    "Unknown argument for yzx desktop: {other}. Try `yzx desktop --help`."
                )));
            }
            other => {
                if parsed.subcommand.as_deref() == Some("macos_preview") && parsed.action.is_none()
                {
                    parsed.action = Some(other.to_string());
                } else if parsed.subcommand.is_some() {
                    return Err(CoreError::usage(
                        "yzx desktop requires one subcommand: install, launch, uninstall, or macos_preview install|uninstall.",
                    ));
                } else if ["install", "launch", "uninstall"].contains(&other) {
                    parsed.subcommand = Some(other.to_string());
                } else if other == "macos_preview" {
                    parsed.subcommand = Some(other.to_string());
                } else {
                    parsed.subcommand = Some(other.to_string());
                }
            }
        }
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

fn print_desktop_help() {
    println!("Desktop integration commands");
    println!();
    println!("Usage:");
    println!("  yzx desktop install [--print-path]");
    println!("  yzx desktop launch");
    println!("  yzx desktop uninstall [--print-path]");
    println!("  yzx desktop macos_preview install [--print-path]");
    println!("  yzx desktop macos_preview uninstall [--print-path]");
    println!("  macos_preview is unsigned, unnotarized, and community-tested");
}

fn resolve_requested_working_dir(path: Option<&str>, home: bool) -> Result<PathBuf, CoreError> {
    if home {
        return home_dir_from_env();
    }
    if let Some(path) = path.map(str::trim).filter(|path| !path.is_empty()) {
        return Ok(PathBuf::from(path));
    }
    std::env::current_dir().map_err(|source| {
        CoreError::io(
            "cwd",
            "Could not read the current working directory.",
            "cd into a valid directory, then retry.",
            ".",
            source,
        )
    })
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
    use super::restart::*;
    use super::*;
    use crate::control_plane::load_normalized_config_for_control;
    use crate::settings_surface::read_settings_jsonc_value;
    use serde_json::Map as JsonMap;
    use std::collections::HashMap;
    use std::fs;
    use std::path::Path;
    use std::process::Command;
    use std::thread;
    use std::time::Duration;
    use tempfile::TempDir;

    fn write_runtime_layout(runtime: &Path) {
        fs::create_dir_all(runtime.join("config_metadata")).expect("metadata dir");
        fs::write(
            runtime
                .join("config_metadata")
                .join("main_config_contract.toml"),
            include_str!("../../../config_metadata/main_config_contract.toml"),
        )
        .expect("main config contract");
        fs::write(
            runtime.join("yazelix_default.toml"),
            include_str!("../../../yazelix_default.toml"),
        )
        .expect("main defaults");
        fs::write(
            runtime.join(crate::ghostty_cursor_registry::DEFAULT_CURSOR_CONFIG_FILENAME),
            include_str!("../../../yazelix_cursors_default.toml"),
        )
        .expect("cursor defaults");
        fs::write(
            runtime.join(crate::active_config_surface::TOML_TOOLING_CONFIG_FILENAME),
            include_str!("../../../tombi.toml"),
        )
        .expect("tombi config");
        fs::write(
            runtime.join("runtime_components.json"),
            r#"{
              "cursors": { "enabled": true, "disableable": true, "notes": [] },
              "screen": { "enabled": true, "disableable": true, "notes": [] }
            }"#,
        )
        .expect("runtime component manifest");
    }

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

    // Defends: public one-shot config file overrides resolve before launch so child terminals do not reinterpret relative paths from a different cwd.
    #[test]
    fn config_override_paths_are_invocation_scoped() {
        let cwd = Path::new("/tmp/project");
        let home = Path::new("/home/demo");

        assert_eq!(
            resolve_config_override_path("alt/settings.jsonc", cwd, home).unwrap(),
            "/tmp/project/alt/settings.jsonc"
        );
        assert_eq!(
            resolve_config_override_path("~/settings.jsonc", cwd, home).unwrap(),
            "/home/demo/settings.jsonc"
        );
        assert_eq!(
            config_override_extra_env(Some("/tmp/custom.jsonc")),
            vec![(
                "YAZELIX_CONFIG_OVERRIDE".to_string(),
                Some("/tmp/custom.jsonc".to_string())
            )]
        );
    }

    // Defends: restart exposes a one-shot welcome skip flag without making the config skip setting sticky.
    #[test]
    fn parse_restart_args_accepts_skip_aliases() {
        for arg in ["-s", "--skip"] {
            let parsed = parse_restart_args(&[arg.into()]).unwrap();

            assert!(parsed.skip_welcome);
            assert!(!parsed.help);
        }

        let help = parse_restart_args(&["--help".into()]).unwrap();
        assert!(help.help);
    }

    // Defends: restart can replace the inherited config override for one relaunched window without mutating settings.jsonc.
    #[test]
    fn parse_restart_args_accepts_config_override() {
        let expected_config = resolve_config_override_path(
            "minimal.jsonc",
            &std::env::current_dir().unwrap(),
            &home_dir_from_env().unwrap(),
        )
        .unwrap();
        let parsed = parse_restart_args(&[
            "--skip".into(),
            "--config".into(),
            "minimal.jsonc".into(),
            "--with".into(),
            "core.welcome_style=static".into(),
            "--with".into(),
            "zellij.pane_frames=false".into(),
        ])
        .unwrap();

        assert!(parsed.skip_welcome);
        assert_eq!(parsed.config.as_deref(), Some(expected_config.as_str()));
        assert_eq!(
            parsed.with_overrides,
            vec!["core.welcome_style=static", "zellij.pane_frames=false"]
        );
    }

    // Defends: repeatable --with patches stay contract-typed and reject unknown settings before launch materialization.
    #[test]
    fn session_config_patches_are_contract_typed() {
        let fields = HashMap::from([
            (
                "editor.command".to_string(),
                SessionConfigOverrideField {
                    kind: SessionConfigOverrideKind::String,
                },
            ),
            (
                "core.skip_welcome_screen".to_string(),
                SessionConfigOverrideField {
                    kind: SessionConfigOverrideKind::Bool,
                },
            ),
            (
                "core.welcome_duration_seconds".to_string(),
                SessionConfigOverrideField {
                    kind: SessionConfigOverrideKind::Float,
                },
            ),
            (
                "editor.sidebar_width_percent".to_string(),
                SessionConfigOverrideField {
                    kind: SessionConfigOverrideKind::Int,
                },
            ),
            (
                "terminal.terminals".to_string(),
                SessionConfigOverrideField {
                    kind: SessionConfigOverrideKind::StringList,
                },
            ),
            (
                "zellij.keybindings".to_string(),
                SessionConfigOverrideField {
                    kind: SessionConfigOverrideKind::StringListMap,
                },
            ),
        ]);
        let mut root = serde_json::json!({
            "core": { "skip_welcome_screen": false },
            "editor": {},
            "terminal": { "terminals": ["ghostty"] },
            "zellij": { "keybindings": { "toggle_popup": ["Alt p"] } }
        });

        for raw in [
            "editor.command=nvim",
            "core.skip_welcome_screen=true",
            "core.welcome_duration_seconds=3.5",
            "editor.sidebar_width_percent=24",
            "terminal.terminals=[\"wezterm\", \"kitty\"]",
            "zellij.keybindings={\"toggle_popup\":[\"Alt t\"],\"open_config\":[]}",
        ] {
            let patch = parse_session_config_patch(raw, &fields).unwrap();
            apply_session_config_patch(&mut root, &patch).unwrap();
        }

        assert_eq!(root["editor"]["command"], "nvim");
        assert_eq!(root["core"]["skip_welcome_screen"], true);
        assert_eq!(root["core"]["welcome_duration_seconds"], 3.5);
        assert_eq!(root["editor"]["sidebar_width_percent"], 24);
        assert_eq!(
            root["terminal"]["terminals"],
            serde_json::json!(["wezterm", "kitty"])
        );
        assert_eq!(
            root["zellij"]["keybindings"],
            serde_json::json!({
                "toggle_popup": ["Alt t"],
                "open_config": [],
            })
        );

        let unknown = parse_session_config_patch("editor.nope=true", &fields).unwrap_err();
        assert!(
            unknown
                .to_string()
                .contains("Unknown Yazelix config setting")
        );
        let invalid_bool =
            parse_session_config_patch("core.skip_welcome_screen=maybe", &fields).unwrap_err();
        assert!(invalid_bool.to_string().contains("Invalid boolean value"));
        let invalid_map =
            parse_session_config_patch("zellij.keybindings={\"toggle_popup\":\"Alt t\"}", &fields)
                .unwrap_err();
        assert!(
            invalid_map
                .to_string()
                .contains("Invalid string-list-map value")
        );
    }

    // Defends: --with writes an ephemeral settings.jsonc snapshot and validates it through the normal config contract without mutating the user's config.
    #[test]
    fn session_config_overrides_materialize_valid_ephemeral_settings() {
        let runtime = TempDir::new().unwrap();
        write_runtime_layout(runtime.path());
        let config = TempDir::new().unwrap();
        let state = TempDir::new().unwrap();

        let session_config = materialize_session_config_override(
            runtime.path(),
            config.path(),
            state.path(),
            None,
            &[
                "editor.command=nvim".to_string(),
                "core.welcome_style=static".to_string(),
                "zellij.pane_frames=false".to_string(),
                "terminal.terminals=[\"wezterm\"]".to_string(),
            ],
        )
        .unwrap();

        let session_path = Path::new(&session_config);
        assert_eq!(
            session_path.file_name().and_then(|name| name.to_str()),
            Some(crate::user_config_paths::SETTINGS_CONFIG)
        );
        assert!(session_path.starts_with(state.path()));

        let session_value = read_settings_jsonc_value(session_path).unwrap();
        assert_eq!(session_value["editor"]["command"], "nvim");
        assert_eq!(session_value["core"]["welcome_style"], "static");
        assert_eq!(session_value["zellij"]["pane_frames"], false);
        assert_eq!(
            session_value["terminal"]["terminals"],
            serde_json::json!(["wezterm"])
        );

        let user_value = read_settings_jsonc_value(&config.path().join("settings.jsonc")).unwrap();
        assert_ne!(user_value["editor"]["command"], "nvim");

        let normalized = load_normalized_config_for_control(
            runtime.path(),
            config.path(),
            Some(&session_config),
        )
        .unwrap();
        assert_eq!(normalized.get("editor_command").unwrap(), "nvim");
        assert_eq!(normalized.get("welcome_style").unwrap(), "static");
        assert_eq!(normalized.get("zellij_pane_frames").unwrap(), "false");
    }

    // Defends: the Rust launch owner still filters duplicate or unsupported configured terminals before fallback logic runs.
    #[test]
    fn normalized_configured_terminals_filters_and_dedupes() {
        let mut config = JsonMap::new();
        config.insert(
            "terminals".into(),
            serde_json::json!(["ghostty", "", "warp", "ghostty", "kitty"]),
        );

        assert_eq!(
            normalized_configured_terminals(&config),
            vec!["ghostty".to_string(), "kitty".to_string()]
        );
    }

    // Defends: missing terminal config keeps Ghostty as the default while preserving WezTerm as the first fallback.
    #[test]
    fn normalized_configured_terminals_defaults_to_ghostty_then_wezterm() {
        let config = JsonMap::new();

        assert_eq!(
            normalized_configured_terminals(&config),
            vec!["ghostty".to_string(), "wezterm".to_string()]
        );
    }

    // Defends: Ghostty user-mode config discovery follows upstream file-name and macOS path candidates instead of hard-coding the old config name.
    #[test]
    fn ghostty_user_config_candidates_follow_upstream_paths() {
        let home = Path::new("/Users/demo");
        let xdg = Path::new("/Users/demo/.config");

        assert_eq!(
            ghostty_user_config_candidates(home, xdg, "linux"),
            vec![
                PathBuf::from("/Users/demo/.config/ghostty/config.ghostty"),
                PathBuf::from("/Users/demo/.config/ghostty/config"),
            ]
        );
        assert_eq!(
            ghostty_user_config_candidates(home, xdg, "macos"),
            vec![
                PathBuf::from("/Users/demo/.config/ghostty/config.ghostty"),
                PathBuf::from("/Users/demo/.config/ghostty/config"),
                PathBuf::from(
                    "/Users/demo/Library/Application Support/com.mitchellh.ghostty/config.ghostty",
                ),
                PathBuf::from(
                    "/Users/demo/Library/Application Support/com.mitchellh.ghostty/config",
                ),
            ]
        );
    }

    // Regression: terminal.config_mode=user accepts Ghostty's current config.ghostty name and reports every checked candidate when none exists.
    #[test]
    fn ghostty_user_config_selection_accepts_config_ghostty_and_lists_misses() {
        let tmp = TempDir::new().unwrap();
        let home = tmp.path().join("home");
        let xdg = home.join(".config");
        let ghostty_dir = xdg.join("ghostty");
        fs::create_dir_all(&ghostty_dir).unwrap();
        let current = ghostty_dir.join("config.ghostty");
        fs::write(&current, "font-family = PragmataPro Mono Liga\n").unwrap();
        let candidates =
            user_terminal_config_candidates_for_platform(&home, "ghostty", &xdg, "macos").unwrap();

        assert_eq!(
            select_existing_user_terminal_config_path("ghostty", &candidates).unwrap(),
            current
        );

        fs::remove_file(&current).unwrap();
        let missing =
            select_existing_user_terminal_config_path("ghostty", &candidates).unwrap_err();
        assert!(missing.contains("config.ghostty"));
        assert!(missing.contains("Application Support/com.mitchellh.ghostty/config"));
    }

    // Defends: launch publishes a compact current-cursor fact only for Ghostty cursor-aware sessions and a clear n/a fallback elsewhere.
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
            launch_cursor_name_for_terminal(&materialization, "wezterm"),
            "n/a"
        );
        assert_eq!(launch_cursor_name_for_terminal(&missing, "ghostty"), "n/a");
        assert_eq!(
            launch_cursor_color_for_terminal(&materialization, "ghostty"),
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

    // Invariant: restart must relaunch through the stable owner without leaking old window runtime/session helper env.
    #[test]
    fn restart_launch_clears_stale_runtime_session_and_helper_env() {
        for key in [
            "YAZELIX_BOOTSTRAP_RUNTIME_DIR",
            "YAZELIX_CURSOR_COLOR",
            "YAZELIX_CURSOR_DIVIDER",
            "YAZELIX_CURSOR_FAMILY",
            "YAZELIX_CURSOR_NAME",
            "YAZELIX_CURSOR_PRIMARY_COLOR",
            "YAZELIX_CURSOR_SECONDARY_COLOR",
            "YAZELIX_RUNTIME_DIR",
            "YAZELIX_SESSION_CONFIG_PATH",
            "YAZELIX_SESSION_FACTS_PATH",
            "YAZELIX_STARTUP_PROFILE_SKIP_WELCOME",
            "YAZELIX_STATUS_BAR_CACHE_PATH",
            "YAZELIX_YZX_BIN",
            "YAZELIX_YZX_CONTROL_BIN",
            "YAZELIX_YZX_CORE_BIN",
        ] {
            assert!(
                RESTART_LAUNCH_CLEARED_ENV_KEYS.contains(&key),
                "restart launch must clear stale {key}"
            );
        }
    }

    // Defends: desktop entry rendering keeps a quoted launcher path so spaces do not corrupt the Exec owner surface.
    #[test]
    fn render_desktop_entry_quotes_exec_path() {
        let entry = render_desktop_entry(Path::new("/tmp/with space/yzx"));
        assert!(entry.contains("Exec=\"/tmp/with space/yzx\" desktop launch"));
        assert!(entry.contains("Terminal=true"));
    }

    // Regression: desktop launch must keep its starter terminal for visible preflight, but schedule the real terminal through a deferred helper so the starter can close first.
    #[test]
    fn desktop_deferred_launch_helper_schedules_after_starter_parent_exits() {
        let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .unwrap();
        let helper = desktop_deferred_launch_probe_path(repo_root);
        let tmp = TempDir::new().unwrap();
        let launch_log = tmp.path().join("deferred.log");
        let marker = tmp.path().join("marker");

        let output = Command::new(&helper)
            .arg(&launch_log)
            .arg("999999999")
            .arg("--")
            .arg("sh")
            .arg("-c")
            .arg(format!("printf done > {}", marker.display()))
            .output()
            .unwrap();

        assert!(output.status.success());
        assert_eq!(
            String::from_utf8_lossy(&output.stdout).trim(),
            launch_log.display().to_string()
        );

        for _ in 0..20 {
            if marker.is_file() {
                return;
            }
            thread::sleep(Duration::from_millis(50));
        }
        panic!("deferred desktop helper did not launch scheduled command");
    }

    // Defends: macOS preview desktop parsing keeps the opt-in nested action explicit.
    #[test]
    fn parse_desktop_args_accepts_macos_preview_action() {
        let parsed = parse_desktop_args(&[
            "macos_preview".into(),
            "install".into(),
            "--print-path".into(),
        ])
        .unwrap();

        assert_eq!(parsed.subcommand.as_deref(), Some("macos_preview"));
        assert_eq!(parsed.action.as_deref(), Some("install"));
        assert!(parsed.print_path);
    }

    // Defends: the macOS preview app bundle points at a stable package profile wrapper and reports actionable package-first repair steps.
    #[test]
    fn render_macos_preview_launcher_uses_profile_yzx_and_actionable_failures() {
        let script =
            render_macos_preview_launcher_script(Path::new("/Users/demo/.nix-profile/bin/yzx"));

        assert!(script.contains("YAZELIX_STABLE_YZX='/Users/demo/.nix-profile/bin/yzx'"));
        assert!(script.contains("\"$YAZELIX_STABLE_YZX\" desktop launch"));
        assert!(script.contains("yzx doctor --verbose"));
        assert!(script.contains("yzx desktop macos_preview install"));
        assert!(!script.contains("/pjs/yazelix"));
    }

    // Defends: the macOS preview bundle carries owned app metadata instead of looking like a throwaway script bundle.
    #[test]
    fn render_macos_preview_info_plist_carries_owned_app_metadata() {
        let info = render_macos_preview_info_plist();

        assert!(info.contains("<key>CFBundlePackageType</key>"));
        assert!(info.contains("<string>APPL</string>"));
        assert!(info.contains("<key>CFBundleShortVersionString</key>"));
        assert!(info.contains(&format!(
            "<string>{MACOS_PREVIEW_BUNDLE_SHORT_VERSION}</string>"
        )));
        assert!(info.contains("<key>CFBundleVersion</key>"));
        assert!(info.contains(&format!("<string>{MACOS_PREVIEW_BUNDLE_VERSION}</string>")));
        assert!(info.contains("<key>LSApplicationCategoryType</key>"));
        assert!(info.contains("<string>public.app-category.developer-tools</string>"));
        assert!(info.contains("<key>NSHighResolutionCapable</key>"));
    }

    // Defends: the macOS preview installer creates only a Yazelix-marked app bundle with a profile-owned launcher script.
    #[test]
    fn install_macos_preview_app_writes_managed_bundle() {
        let tmp = TempDir::new().unwrap();
        let app_path = tmp
            .path()
            .join("Applications")
            .join(MACOS_PREVIEW_APP_DIR_NAME);
        let launcher_path = tmp.path().join(".nix-profile").join("bin").join("yzx");

        install_macos_preview_app(&app_path, &launcher_path).unwrap();

        let info = fs::read_to_string(app_path.join("Contents").join("Info.plist")).unwrap();
        let marker = app_path
            .join("Contents")
            .join("Resources")
            .join(MACOS_PREVIEW_MARKER_FILE);
        let script = fs::read_to_string(
            app_path
                .join("Contents")
                .join("MacOS")
                .join(MACOS_PREVIEW_EXECUTABLE_NAME),
        )
        .unwrap();

        assert!(info.contains(MACOS_PREVIEW_BUNDLE_ID));
        assert!(marker.is_file());
        assert!(script.contains(&launcher_path.to_string_lossy().to_string()));
        assert!(macos_preview_bundle_is_managed(&app_path));
    }

    // Regression: uninstall and refresh paths must not take ownership of an unrelated app bundle at the preview path.
    #[test]
    fn macos_preview_bundle_guard_rejects_unmarked_app_path() {
        let tmp = TempDir::new().unwrap();
        let app_path = tmp
            .path()
            .join("Applications")
            .join(MACOS_PREVIEW_APP_DIR_NAME);
        fs::create_dir_all(app_path.join("Contents")).unwrap();
        fs::write(
            app_path.join("Contents").join("Info.plist"),
            render_macos_preview_info_plist(),
        )
        .unwrap();

        let err = ensure_macos_preview_bundle_is_managed(&app_path).unwrap_err();
        assert_eq!(err.code(), "macos_preview_app_conflict");
    }
}
