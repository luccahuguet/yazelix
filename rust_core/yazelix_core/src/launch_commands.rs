// Test lane: default
//! Rust-owned `yzx enter`, `yzx launch`, `yzx desktop`, and `yzx restart` owners.

use crate::bridge::{CoreError, ErrorClass};
use crate::config_state::compute_config_state;
use crate::control_plane::{
    config_dir_from_env, config_override_from_env, config_state_compute_request_from_env,
    home_dir_from_env, load_normalized_config_for_control, run_child_in_runtime_env,
    runtime_dir_from_env, runtime_env_request, state_dir_from_env,
};
use crate::install_ownership_env::install_ownership_request_from_env_with_runtime_dir;
use crate::install_ownership_report::evaluate_install_ownership_report;
use crate::launch_materialization::{launch_materialization_request_from_env, prepare_launch_materialization};
use crate::runtime_contract::{
    LaunchPreflightPayload, StartupLaunchPreflightRequest, StartupPreflightPayload,
    TerminalCandidate, evaluate_startup_launch_preflight,
};
use crate::runtime_env::compute_runtime_env;
use serde_json::{Map as JsonMap, Value as JsonValue};
use std::collections::HashSet;
use std::ffi::OsString;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const DEFAULT_TERMINAL: &str = "ghostty";
const SUPPORTED_TERMINALS: &[&str] = &["ghostty", "wezterm", "kitty", "alacritty", "foot"];
const WINDOW_CLASS: &str = "com.yazelix.Yazelix";
const X11_INSTANCE: &str = "yazelix";
const DESKTOP_ICON_SIZES: &[&str] = &["48x48", "64x64", "128x128", "256x256"];
const DESKTOP_LAUNCH_CLEARED_ENV_KEYS: &[&str] = &[
    "IN_YAZELIX_SHELL",
    "YAZELIX_DIR",
    "YAZELIX_MENU_POPUP",
    "YAZELIX_POPUP_PANE",
    "YAZELIX_NU_BIN",
    "YAZELIX_TERMINAL",
    "YAZI_ID",
    "ZELLIJ",
    "ZELLIJ_DEFAULT_LAYOUT",
    "ZELLIJ_PANE_ID",
    "ZELLIJ_SESSION_NAME",
    "ZELLIJ_TAB_NAME",
    "ZELLIJ_TAB_POSITION",
];

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct EnterArgs {
    path: Option<String>,
    home: bool,
    verbose: bool,
    setup_only: bool,
    help: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct LaunchArgs {
    path: Option<String>,
    home: bool,
    terminal: Option<String>,
    verbose: bool,
    help: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct DesktopArgs {
    subcommand: Option<String>,
    print_path: bool,
    help: bool,
}

pub fn run_yzx_enter(args: &[String]) -> Result<i32, CoreError> {
    let parsed = parse_enter_args(args)?;
    if parsed.help {
        print_enter_help();
        return Ok(0);
    }

    let runtime_dir = runtime_dir_from_env()?;
    let config_dir = config_dir_from_env()?;
    let normalized = load_normalized_config_for_control(
        &runtime_dir,
        &config_dir,
        config_override_from_env().as_deref(),
    )?;
    let req = runtime_env_request(runtime_dir.clone(), &normalized)?;
    let runtime_data = compute_runtime_env(&req)?;
    let runtime_env = runtime_data.runtime_env;
    let nu_bin = resolve_nu_bin(&runtime_dir)?;

    if parsed.verbose {
        println!("🔍 start_yazelix: verbose mode enabled");
        println!("🔍 Startup runtime env computed");
    }

    if parsed.setup_only {
        println!("🔧 Setting up Yazelix generated environment files...");
        run_runtime_setup(&runtime_dir, &nu_bin, &runtime_env, false)?;
        println!("✅ Setup complete.");
        return Ok(0);
    }

    let requested_working_dir = resolve_requested_working_dir(parsed.path.as_deref(), parsed.home)?;
    let preflight = evaluate_startup_launch_preflight(&StartupLaunchPreflightRequest {
        startup: Some(StartupPreflightPayload {
            working_dir: requested_working_dir.clone(),
            runtime_script: crate::RuntimeScriptCheckRequest {
                id: "startup_runtime_script".to_string(),
                label: "startup script".to_string(),
                owner_surface: "startup".to_string(),
                path: runtime_dir
                    .join("nushell")
                    .join("scripts")
                    .join("core")
                    .join("start_yazelix_inner.nu"),
            },
        }),
        launch: None,
    })?;
    let working_dir = PathBuf::from(preflight.working_dir);
    let inner_script = preflight.script_path.ok_or_else(|| {
        CoreError::classified(
            ErrorClass::Internal,
            "missing_inner_script",
            "Startup preflight omitted the resolved inner startup script path.",
            "Report this as a Yazelix internal error.",
            serde_json::json!({}),
        )
    })?;

    run_runtime_setup(&runtime_dir, &nu_bin, &runtime_env, true)?;

    let mut argv = vec![
        nu_bin.to_string_lossy().into_owned(),
        "-i".to_string(),
        inner_script,
        working_dir.to_string_lossy().into_owned(),
    ];
    if parsed.verbose {
        argv.push("--verbose".to_string());
    }
    let status = run_child_in_runtime_env(&argv, &runtime_env, &working_dir)?;
    Ok(status.code().unwrap_or(1))
}

pub fn run_yzx_launch(args: &[String]) -> Result<i32, CoreError> {
    let parsed = parse_launch_args(args)?;
    if parsed.help {
        print_launch_help();
        return Ok(0);
    }

    run_launch_flow(
        parsed.path.as_deref(),
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
        Some(other) => Err(CoreError::usage(format!(
            "Unknown yzx desktop subcommand: {other}. Try `yzx desktop --help`."
        ))),
        None => unreachable!(),
    }
}

pub fn run_yzx_restart(args: &[String]) -> Result<i32, CoreError> {
    if !args.is_empty() {
        return Err(CoreError::usage(
            "yzx restart does not accept arguments. Try `yzx restart`.",
        ));
    }

    let session_to_kill = current_zellij_session();
    let restart_file = create_restart_sidebar_bootstrap_file(&std::env::current_dir().map_err(
        |source| {
            CoreError::io(
                "restart_cwd",
                "Could not read the current working directory.",
                "cd into a valid directory, then retry.",
                ".",
                source,
            )
        },
    )?)?;

    let is_yazelix_terminal = std::env::var_os("YAZELIX_TERMINAL").is_some();
    if is_yazelix_terminal {
        println!("🔄 Restarting Yazelix...");
    } else {
        println!("🔄 Restarting Yazelix (opening new window)...");
    }

    let runtime_dir = runtime_dir_from_env()?;
    let report = evaluate_install_ownership_report(
        &install_ownership_request_from_env_with_runtime_dir(runtime_dir.clone())?,
    );
    let launcher = report
        .stable_yzx_wrapper
        .map(PathBuf::from)
        .unwrap_or_else(|| runtime_dir.join("shells").join("posix").join("yzx_cli.sh"));

    let output = command_output_with_overrides(
        &[
            launcher.to_string_lossy().into_owned(),
            "launch".to_string(),
        ],
        None,
        &std::env::current_dir().map_err(|source| {
            CoreError::io(
                "restart_cwd",
                "Could not read the current working directory.",
                "cd into a valid directory, then retry.",
                ".",
                source,
            )
        })?,
        &[],
        &[(
            "YAZELIX_BOOTSTRAP_SIDEBAR_CWD_FILE".to_string(),
            Some(restart_file.to_string_lossy().into_owned()),
        )],
        "restart_launch",
        "Retry the restart from a working Yazelix install, or relaunch manually with `yzx launch`.",
    )?;
    if !output.status.success() {
        print_completed_output(&output);
        eprintln!("❌ Failed to relaunch Yazelix through the stable owner wrapper.");
        return Ok(output.status.code().unwrap_or(1));
    }

    thread::sleep(Duration::from_secs(1));
    kill_zellij_session(session_to_kill.as_deref());
    Ok(0)
}

fn run_desktop_install(print_path: bool) -> Result<i32, CoreError> {
    let runtime_dir = runtime_dir_from_env()?;
    let report = evaluate_install_ownership_report(
        &install_ownership_request_from_env_with_runtime_dir(runtime_dir.clone())?,
    );
    if report.install_owner == "home-manager" {
        return Err(CoreError::classified(
            ErrorClass::Runtime,
            "home_manager_desktop_owner",
            "Home Manager owns Yazelix desktop integration for this install.",
            "Reapply your Home Manager configuration for the profile desktop entry, or run `yzx desktop uninstall` only to remove a stale user-local entry.",
            serde_json::json!({}),
        ));
    }

    let launcher_path = PathBuf::from(report.desktop_launcher_path);
    if !runtime_dir.exists() {
        return Err(CoreError::classified(
            ErrorClass::Runtime,
            "missing_runtime_dir",
            format!("Missing Yazelix runtime at {}", runtime_dir.display()),
            "Reinstall Yazelix so the runtime tree is present, then retry.",
            serde_json::json!({}),
        ));
    }
    if !launcher_path.exists() {
        return Err(CoreError::classified(
            ErrorClass::Runtime,
            "missing_desktop_launcher",
            format!("Missing stable Yazelix CLI at {}", launcher_path.display()),
            "Restore the stable launcher path or reinstall Yazelix, then retry.",
            serde_json::json!({}),
        ));
    }

    let home_dir = home_dir_from_env()?;
    let xdg_data_home = xdg_data_home(&home_dir);
    let applications_dir = xdg_data_home.join("applications");
    let icons_root = xdg_data_home.join("icons").join("hicolor");
    let desktop_path = applications_dir.join("com.yazelix.Yazelix.desktop");
    let desktop_entry = render_desktop_entry(&launcher_path);

    fs::create_dir_all(&applications_dir).map_err(|source| {
        CoreError::io(
            "desktop_applications_dir",
            format!(
                "Could not create desktop applications directory {}.",
                applications_dir.display()
            ),
            "Create the directory or fix permissions, then retry.",
            applications_dir.display().to_string(),
            source,
        )
    })?;
    install_desktop_icons(&runtime_dir, &icons_root)?;
    fs::write(&desktop_path, desktop_entry).map_err(|source| {
        CoreError::io(
            "desktop_entry_write",
            format!("Could not write desktop entry {}.", desktop_path.display()),
            "Fix the directory permissions, then retry.",
            desktop_path.display().to_string(),
            source,
        )
    })?;

    maybe_validate_desktop_entry(&desktop_path)?;
    maybe_refresh_desktop_database(&applications_dir);
    maybe_refresh_icon_cache(&icons_root);

    if print_path {
        println!("{}", desktop_path.display());
    } else {
        println!("Installed Yazelix desktop entry: {}", desktop_path.display());
    }
    Ok(0)
}

fn run_desktop_uninstall(print_path: bool) -> Result<i32, CoreError> {
    let home_dir = home_dir_from_env()?;
    let xdg_data_home = xdg_data_home(&home_dir);
    let applications_dir = xdg_data_home.join("applications");
    let icons_root = xdg_data_home.join("icons").join("hicolor");
    let desktop_path = applications_dir.join("com.yazelix.Yazelix.desktop");

    if desktop_path.exists() {
        fs::remove_file(&desktop_path).map_err(|source| {
            CoreError::io(
                "desktop_entry_remove",
                format!("Could not remove desktop entry {}.", desktop_path.display()),
                "Fix the directory permissions, then retry.",
                desktop_path.display().to_string(),
                source,
            )
        })?;
    }
    for size in DESKTOP_ICON_SIZES {
        let path = icons_root.join(size).join("apps").join("yazelix.png");
        if path.exists() {
            let _ = fs::remove_file(path);
        }
    }
    maybe_refresh_desktop_database(&applications_dir);
    maybe_refresh_icon_cache(&icons_root);

    if print_path {
        println!("{}", desktop_path.display());
    } else {
        println!("Removed Yazelix desktop entry: {}", desktop_path.display());
    }
    Ok(0)
}

fn run_desktop_launch() -> Result<i32, CoreError> {
    print_desktop_progress("Preparing session...");
    let home_dir = home_dir_from_env()?;
    let home_dir_string = home_dir.to_string_lossy().to_string();
    match run_launch_flow(
        Some(&home_dir_string),
        false,
        None,
        false,
        true,
        DESKTOP_LAUNCH_CLEARED_ENV_KEYS,
    ) {
        Ok(code) => Ok(code),
        Err(err) => {
            acknowledge_desktop_failure(&err.message());
            Err(err)
        }
    }
}

fn run_launch_flow(
    requested_path: Option<&str>,
    home: bool,
    requested_terminal: Option<&str>,
    verbose: bool,
    desktop_fast_path: bool,
    env_removals: &[&str],
) -> Result<i32, CoreError> {
    let runtime_dir = runtime_dir_from_env()?;
    let state_dir = state_dir_from_env()?;
    let config_state = compute_config_state(&config_state_compute_request_from_env(
        config_override_from_env().as_deref(),
    )?)?;
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
            ("YAZELIX_RUNTIME_DIR".to_string(), Some(runtime_dir.to_string_lossy().into_owned())),
            ("YAZELIX_TERMINAL".to_string(), Some(candidate.terminal.clone())),
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

        let output = run_detached_launch_probe(
            &runtime_dir,
            &state_dir,
            &argv,
            &runtime_env,
            &working_dir,
            config_state.needs_refresh,
            env_removals,
            &extra_env,
        )?;

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

fn parse_enter_args(args: &[String]) -> Result<EnterArgs, CoreError> {
    let mut parsed = EnterArgs::default();
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--help" | "-h" | "help" => parsed.help = true,
            "--home" => parsed.home = true,
            "--verbose" => parsed.verbose = true,
            "--setup-only" => parsed.setup_only = true,
            "--path" | "-p" => {
                index += 1;
                let value = args.get(index).ok_or_else(|| {
                    CoreError::usage("Missing value for yzx enter --path. Try `yzx enter --help`.")
                })?;
                parsed.path = Some(value.clone());
            }
            other if other.starts_with('-') => {
                return Err(CoreError::usage(format!(
                    "Unknown argument for yzx enter: {other}. Try `yzx enter --help`."
                )));
            }
            other => {
                if parsed.path.is_some() {
                    return Err(CoreError::usage(
                        "yzx enter accepts at most one positional cwd override.",
                    ));
                }
                parsed.path = Some(other.to_string());
            }
        }
        index += 1;
    }
    Ok(parsed)
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
                    CoreError::usage("Missing value for yzx launch --path. Try `yzx launch --help`.")
                })?;
                parsed.path = Some(value.clone());
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
                if parsed.subcommand.is_some() {
                    return Err(CoreError::usage(
                        "yzx desktop requires exactly one subcommand: install, launch, or uninstall.",
                    ));
                }
                parsed.subcommand = Some(other.to_string());
            }
        }
    }
    Ok(parsed)
}

fn print_enter_help() {
    println!("Start Yazelix in the current terminal");
    println!();
    println!("Usage:");
    println!("  yzx enter [--path <dir> | --home] [--verbose]");
}

fn print_launch_help() {
    println!("Launch Yazelix in a new terminal window");
    println!();
    println!("Usage:");
    println!("  yzx launch [--path <dir> | --home] [--terminal <name>] [--verbose]");
}

fn print_desktop_help() {
    println!("Desktop integration commands");
    println!();
    println!("Usage:");
    println!("  yzx desktop install [--print-path]");
    println!("  yzx desktop launch");
    println!("  yzx desktop uninstall [--print-path]");
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

fn resolve_nu_bin(runtime_dir: &Path) -> Result<PathBuf, CoreError> {
    if let Ok(raw) = std::env::var("YAZELIX_NU_BIN") {
        let trimmed = raw.trim();
        if !trimmed.is_empty() {
            let path = PathBuf::from(trimmed);
            if path.is_file() {
                return Ok(path);
            }
        }
    }

    let runtime_nu = runtime_dir.join("libexec").join("nu");
    if runtime_nu.is_file() {
        return Ok(runtime_nu);
    }

    if let Some(path) = find_command("nu") {
        return Ok(path);
    }

    Err(CoreError::classified(
        ErrorClass::Runtime,
        "missing_nu_bin",
        "Could not resolve a usable Nushell binary for Yazelix.",
        "Set YAZELIX_NU_BIN, restore runtime libexec/nu, or install `nu` on PATH.",
        serde_json::json!({}),
    ))
}

fn run_runtime_setup(
    runtime_dir: &Path,
    nu_bin: &Path,
    runtime_env: &JsonMap<String, JsonValue>,
    quiet: bool,
) -> Result<(), CoreError> {
    let mut argv = vec![
        nu_bin.to_string_lossy().into_owned(),
        runtime_dir
            .join("nushell")
            .join("scripts")
            .join("setup")
            .join("environment.nu")
            .to_string_lossy()
            .into_owned(),
    ];
    if quiet {
        argv.push("--skip-welcome".to_string());
    }
    let status = run_child_in_runtime_env(&argv, runtime_env, runtime_dir)?;
    if status.success() {
        Ok(())
    } else {
        Err(CoreError::classified(
            ErrorClass::Runtime,
            "environment_setup_failed",
            format!(
                "Yazelix environment setup failed with exit code {}.",
                status.code().unwrap_or(1)
            ),
            "Fix the reported setup failure, then retry.",
            serde_json::json!({}),
        ))
    }
}

fn normalized_configured_terminals(config: &JsonMap<String, JsonValue>) -> Vec<String> {
    let raw = match config.get("terminals") {
        Some(JsonValue::Array(items)) => items
            .iter()
            .filter_map(JsonValue::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(|value| value.to_ascii_lowercase())
            .collect::<Vec<_>>(),
        _ => vec![DEFAULT_TERMINAL.to_string()],
    };

    let mut out = Vec::new();
    for terminal in raw {
        if !SUPPORTED_TERMINALS.contains(&terminal.as_str()) {
            continue;
        }
        if !out.contains(&terminal) {
            out.push(terminal);
        }
    }
    out
}

fn print_empty_terminal_error() -> Result<(), CoreError> {
    let available = SUPPORTED_TERMINALS
        .iter()
        .filter(|terminal| find_command(terminal).is_some())
        .copied()
        .collect::<Vec<_>>();
    let available_text = if available.is_empty() {
        "none detected".to_string()
    } else {
        available.join(", ")
    };
    eprintln!("Error: terminal.terminals must include at least one terminal");
    eprintln!("Detected terminals: {available_text}");
    eprintln!("Set [terminal].terminals in ~/.config/yazelix/user_configs/yazelix.toml");
    Ok(())
}

fn generated_terminal_config_path(state_dir: &Path, terminal: &str) -> PathBuf {
    let root = state_dir.join("configs").join("terminal_emulators");
    match terminal {
        "ghostty" => root.join("ghostty").join("config"),
        "wezterm" => root.join("wezterm").join(".wezterm.lua"),
        "kitty" => root.join("kitty").join("kitty.conf"),
        "alacritty" => root.join("alacritty").join("alacritty.toml"),
        "foot" => root.join("foot").join("foot.ini"),
        other => root.join(other),
    }
}

fn user_terminal_config_path(home_dir: &Path, terminal: &str) -> Result<PathBuf, String> {
    match terminal {
        "ghostty" => Ok(home_dir.join(".config").join("ghostty").join("config")),
        "kitty" => Ok(home_dir.join(".config").join("kitty").join("kitty.conf")),
        "wezterm" => {
            let main = home_dir.join(".wezterm.lua");
            if main.exists() {
                Ok(main)
            } else {
                Ok(home_dir
                    .join(".config")
                    .join("wezterm")
                    .join("wezterm.lua"))
            }
        }
        "alacritty" => Ok(home_dir
            .join(".config")
            .join("alacritty")
            .join("alacritty.toml")),
        "foot" => Ok(home_dir.join(".config").join("foot").join("foot.ini")),
        other => Err(format!("Unsupported terminal config lookup: {other}")),
    }
}

fn resolve_terminal_config_path(
    home_dir: &Path,
    state_dir: &Path,
    mode: &str,
    terminal: &str,
) -> Result<PathBuf, String> {
    match mode {
        "yazelix" => Ok(generated_terminal_config_path(state_dir, terminal)),
        "user" => {
            let path = user_terminal_config_path(home_dir, terminal)?;
            if path.exists() {
                Ok(path)
            } else {
                Err(format!(
                    "terminal.config_mode = user requires a real {terminal} user config at {}",
                    path.display()
                ))
            }
        }
        other => Err(format!(
            "Unsupported terminal.config_mode '{other}'. Expected 'yazelix' or 'user'."
        )),
    }
}

fn terminal_display_name(terminal: &str) -> String {
    match terminal {
        "ghostty" => "Ghostty".to_string(),
        "wezterm" => "WezTerm".to_string(),
        "kitty" => "Kitty".to_string(),
        "alacritty" => "Alacritty".to_string(),
        "foot" => "Foot".to_string(),
        other => other.to_string(),
    }
}

fn get_working_dir_args(terminal: &str, working_dir: &Path) -> Vec<String> {
    let wd = working_dir.to_string_lossy().into_owned();
    match terminal {
        "ghostty" => vec![format!("--working-directory={wd}")],
        "wezterm" => vec!["--cwd".to_string(), wd],
        "kitty" => vec![format!("--directory={wd}")],
        "alacritty" => vec!["--working-directory".to_string(), wd],
        "foot" => vec![format!("--working-directory={wd}")],
        _ => vec![],
    }
}

fn current_platform_name() -> String {
    std::env::var("YAZELIX_TEST_OS")
        .ok()
        .map(|value| value.trim().to_ascii_lowercase())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| std::env::consts::OS.to_string())
}

fn resolve_nixgl_wrapper(runtime_dir: &Path) -> Option<String> {
    for relative in [
        ["libexec", "nixGL"].as_slice(),
        ["libexec", "nixGLDefault"].as_slice(),
        ["libexec", "nixGLMesa"].as_slice(),
        ["libexec", "nixGLIntel"].as_slice(),
        ["bin", "nixGLMesa"].as_slice(),
        ["bin", "nixGLIntel"].as_slice(),
    ] {
        let path = runtime_dir.join(relative.iter().collect::<PathBuf>());
        if path.is_file() {
            return Some(path.to_string_lossy().into_owned());
        }
    }

    for command in ["nixGL", "nixGLDefault", "nixGLMesa", "nixGLIntel"] {
        if find_command(command).is_some() {
            return Some(command.to_string());
        }
    }
    None
}

fn maybe_prepend(argv: Vec<String>, wrapper: Option<String>) -> Vec<String> {
    if let Some(wrapper) = wrapper.filter(|value| !value.trim().is_empty()) {
        let mut out = vec![wrapper];
        out.extend(argv);
        out
    } else {
        argv
    }
}

fn build_launch_command_argv(
    runtime_dir: &Path,
    terminal: &TerminalCandidate,
    config_path: &Path,
    working_dir: &Path,
) -> Result<Vec<String>, CoreError> {
    let working_dir_args = get_working_dir_args(&terminal.terminal, working_dir);
    let startup_script = runtime_dir
        .join("shells")
        .join("posix")
        .join("start_yazelix.sh");
    if !startup_script.is_file() {
        return Err(CoreError::classified(
            ErrorClass::Runtime,
            "missing_startup_script",
            format!("Missing Yazelix startup script at {}.", startup_script.display()),
            "Restore shells/posix/start_yazelix.sh or reinstall Yazelix.",
            serde_json::json!({}),
        ));
    }

    let title = format!("Yazelix - {}", terminal_display_name(&terminal.terminal));
    let config_string = config_path.to_string_lossy().into_owned();
    let nixgl = resolve_nixgl_wrapper(runtime_dir);

    let argv = match terminal.terminal.as_str() {
        "ghostty" => {
            let mut ghostty = if current_platform_name() == "macos" {
                vec![
                    terminal.command.clone(),
                    "--config-default-files=false".to_string(),
                    format!("--config-file={config_string}"),
                    format!("--title={title}"),
                ]
            } else {
                vec![
                    terminal.command.clone(),
                    "--config-default-files=false".to_string(),
                    format!("--config-file={config_string}"),
                    "--gtk-single-instance=false".to_string(),
                    format!("--class={WINDOW_CLASS}"),
                    format!("--x11-instance-name={X11_INSTANCE}"),
                    format!("--title={title}"),
                ]
            };
            ghostty.extend(working_dir_args);
            ghostty.push("-e".to_string());
            ghostty.push(startup_script.to_string_lossy().into_owned());
            let ghostty = maybe_prepend(ghostty, nixgl);
            let ghostty_wrapper = runtime_dir
                .join("shells")
                .join("posix")
                .join("yazelix_ghostty.sh");
            maybe_prepend(
                ghostty,
                ghostty_wrapper
                    .is_file()
                    .then(|| ghostty_wrapper.to_string_lossy().into_owned()),
            )
        }
        "wezterm" => {
            let mut wezterm = vec![
                terminal.command.clone(),
                "--config-file".to_string(),
                config_string,
                "start".to_string(),
                format!("--class={WINDOW_CLASS}"),
            ];
            wezterm.extend(working_dir_args);
            wezterm.push("--".to_string());
            wezterm.push(startup_script.to_string_lossy().into_owned());
            maybe_prepend(wezterm, nixgl)
        }
        "kitty" => {
            let mut kitty = vec![
                terminal.command.clone(),
                format!("--config={config_string}"),
                format!("--class={WINDOW_CLASS}"),
                format!("--title={title}"),
            ];
            kitty.extend(working_dir_args);
            kitty.push(startup_script.to_string_lossy().into_owned());
            maybe_prepend(kitty, nixgl)
        }
        "alacritty" => {
            let mut alacritty = vec![
                terminal.command.clone(),
                "--config-file".to_string(),
                config_string,
                "--class".to_string(),
                WINDOW_CLASS.to_string(),
                "--title".to_string(),
                title,
            ];
            alacritty.extend(working_dir_args);
            alacritty.push("-e".to_string());
            alacritty.push(startup_script.to_string_lossy().into_owned());
            maybe_prepend(alacritty, nixgl)
        }
        "foot" => {
            let mut foot = vec![
                terminal.command.clone(),
                "--config".to_string(),
                config_string,
                "--app-id".to_string(),
                WINDOW_CLASS.to_string(),
            ];
            foot.extend(working_dir_args);
            foot.push(startup_script.to_string_lossy().into_owned());
            maybe_prepend(foot, nixgl)
        }
        other => {
            return Err(CoreError::usage(format!("Unknown terminal: {other}")));
        }
    };

    Ok(argv)
}

fn render_argv_for_display(argv: &[String]) -> String {
    argv.iter()
        .map(|arg| {
            if arg.chars().all(|ch| ch.is_ascii_alphanumeric() || "/._:=,@+-".contains(ch)) {
                arg.clone()
            } else {
                format!("{arg:?}")
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn get_launch_probe_log_path(state_dir: &Path, terminal_name: &str) -> Result<PathBuf, CoreError> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| CoreError::classified(
            ErrorClass::Internal,
            "system_clock_error",
            format!("System clock error while preparing detached launch log path: {error}"),
            "Fix the system clock, then retry.",
            serde_json::json!({}),
        ))?
        .as_millis();
    let sanitized = terminal_name
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch.to_ascii_lowercase() } else { '_' })
        .collect::<String>()
        .trim_matches('_')
        .to_string();
    let log_dir = state_dir.join("logs").join("terminal_launch");
    fs::create_dir_all(&log_dir).map_err(|source| {
        CoreError::io(
            "launch_log_dir",
            format!("Could not create launch log directory {}.", log_dir.display()),
            "Fix the directory permissions, then retry.",
            log_dir.display().to_string(),
            source,
        )
    })?;
    Ok(log_dir.join(format!("{}_{}.log", sanitized, timestamp)))
}

fn run_detached_launch_probe(
    runtime_dir: &Path,
    state_dir: &Path,
    launch_argv: &[String],
    runtime_env: &JsonMap<String, JsonValue>,
    cwd: &Path,
    needs_reload: bool,
    env_removals: &[&str],
    extra_env: &[(String, Option<String>)],
) -> Result<Output, CoreError> {
    let probe_helper = runtime_dir
        .join("shells")
        .join("posix")
        .join("detached_launch_probe.sh");
    if !probe_helper.is_file() {
        return Err(CoreError::classified(
            ErrorClass::Runtime,
            "missing_detached_launch_probe",
            format!(
                "Cannot launch terminals: detached launch helper is missing at {}.",
                probe_helper.display()
            ),
            "Restore shells/posix/detached_launch_probe.sh or reinstall Yazelix.",
            serde_json::json!({}),
        ));
    }

    let log_path = get_launch_probe_log_path(state_dir, launch_argv.first().map(String::as_str).unwrap_or("terminal"))?;
    let mut argv = vec![
        probe_helper.to_string_lossy().into_owned(),
        log_path.to_string_lossy().into_owned(),
    ];
    if needs_reload {
        argv.push("--reload".to_string());
    }
    argv.push("--".to_string());
    argv.extend(launch_argv.iter().cloned());
    command_output_with_overrides(
        &argv,
        Some(runtime_env),
        cwd,
        env_removals,
        extra_env,
        "detached_launch_probe",
        "Retry with a valid configured terminal or reinstall Yazelix so the detached launch helper is present.",
    )
}

fn render_launch_failure(output: &Output) -> String {
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let logged_path = stdout
        .lines()
        .map(str::trim)
        .rev()
        .find(|line| !line.is_empty() && Path::new(line).exists())
        .map(PathBuf::from);
    if let Some(path) = logged_path {
        if let Ok(raw) = fs::read_to_string(&path) {
            let tail = raw
                .lines()
                .rev()
                .take(10)
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect::<Vec<_>>()
                .join(" ");
            if !tail.trim().is_empty() {
                return tail.trim().to_string();
            }
        }
    }

    let stderr = stderr.trim();
    if !stderr.is_empty() {
        stderr.to_string()
    } else {
        format!("exit code {}", output.status.code().unwrap_or(1))
    }
}

fn command_output_with_overrides(
    argv: &[String],
    runtime_env: Option<&JsonMap<String, JsonValue>>,
    cwd: &Path,
    env_removals: &[&str],
    extra_env: &[(String, Option<String>)],
    owner: &str,
    remediation: &str,
) -> Result<Output, CoreError> {
    let (command, args) = argv
        .split_first()
        .ok_or_else(|| CoreError::usage("Missing command argv"))?;
    let mut cmd = Command::new(command);
    cmd.args(args);
    configure_command_env(&mut cmd, runtime_env, cwd, env_removals, extra_env);
    cmd.output().map_err(|source| {
        CoreError::io(
            owner,
            format!("Failed to launch {owner}."),
            remediation,
            command.clone(),
            source,
        )
    })
}

fn configure_command_env(
    cmd: &mut Command,
    runtime_env: Option<&JsonMap<String, JsonValue>>,
    cwd: &Path,
    env_removals: &[&str],
    extra_env: &[(String, Option<String>)],
) {
    let removals: HashSet<&str> = env_removals.iter().copied().collect();
    cmd.current_dir(cwd);
    cmd.env_clear();
    for (key, value) in std::env::vars_os() {
        if removals.contains(key.to_string_lossy().as_ref()) {
            continue;
        }
        cmd.env(&key, &value);
    }
    if let Some(runtime_env) = runtime_env {
        for (key, value) in runtime_env {
            if let Some(text) = runtime_env_value(value) {
                cmd.env(key, text);
            } else {
                cmd.env_remove(key);
            }
        }
    }
    for (key, value) in extra_env {
        if let Some(value) = value {
            cmd.env(key, value);
        } else {
            cmd.env_remove(key);
        }
    }
}

fn runtime_env_value(value: &JsonValue) -> Option<OsString> {
    match value {
        JsonValue::Null => None,
        JsonValue::String(text) => Some(OsString::from(text)),
        JsonValue::Bool(flag) => Some(OsString::from(flag.to_string())),
        JsonValue::Number(number) => Some(OsString::from(number.to_string())),
        JsonValue::Array(items) => Some(OsString::from(
            items
                .iter()
                .filter_map(JsonValue::as_str)
                .collect::<Vec<_>>()
                .join(if cfg!(windows) { ";" } else { ":" }),
        )),
        JsonValue::Object(_) => Some(OsString::from(value.to_string())),
    }
}

fn print_completed_output(output: &Output) {
    if !output.stdout.is_empty() {
        print!("{}", String::from_utf8_lossy(&output.stdout));
    }
    if !output.stderr.is_empty() {
        eprint!("{}", String::from_utf8_lossy(&output.stderr));
    }
}

fn current_zellij_session() -> Option<String> {
    if let Ok(session) = std::env::var("ZELLIJ_SESSION_NAME") {
        let trimmed = session.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }
    }

    let output = Command::new("zellij")
        .arg("list-sessions")
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        if !line.contains("current") {
            continue;
        }
        let cleaned_line = strip_ansi(line);
        let clean = cleaned_line.trim_start_matches('>').trim();
        let token = clean
            .split_whitespace()
            .find(|token| !token.is_empty())
            .map(str::to_string);
        if token.is_some() {
            return token;
        }
    }
    None
}

fn strip_ansi(text: &str) -> String {
    let mut out = String::new();
    let mut chars = text.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\u{1b}' {
            if matches!(chars.peek(), Some('[')) {
                chars.next();
                while let Some(next) = chars.next() {
                    if ('@'..='~').contains(&next) {
                        break;
                    }
                }
            }
            continue;
        }
        out.push(ch);
    }
    out
}

fn create_restart_sidebar_bootstrap_file(target_dir: &Path) -> Result<PathBuf, CoreError> {
    let state_dir = home_dir_from_env()?
        .join(".local")
        .join("share")
        .join("yazelix")
        .join("state")
        .join("restart");
    fs::create_dir_all(&state_dir).map_err(|source| {
        CoreError::io(
            "restart_state_dir",
            format!("Could not create restart state directory {}.", state_dir.display()),
            "Fix the directory permissions, then retry.",
            state_dir.display().to_string(),
            source,
        )
    })?;
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| CoreError::classified(
            ErrorClass::Internal,
            "system_clock_error",
            format!("System clock error while preparing restart bootstrap file: {error}"),
            "Fix the system clock, then retry.",
            serde_json::json!({}),
        ))?
        .as_millis();
    let path = state_dir.join(format!("sidebar_cwd_{timestamp}.tmp"));
    fs::write(&path, target_dir.to_string_lossy().into_owned()).map_err(|source| {
        CoreError::io(
            "restart_sidebar_bootstrap",
            format!("Could not write restart bootstrap file {}.", path.display()),
            "Fix the directory permissions, then retry.",
            path.display().to_string(),
            source,
        )
    })?;
    Ok(path)
}

fn kill_zellij_session(session_name: Option<&str>) {
    let Some(session_name) = session_name.map(str::trim).filter(|name| !name.is_empty()) else {
        println!("⚠️  No Zellij session detected to close");
        return;
    };
    println!("Killing Zellij session: {session_name}");
    let _ = Command::new("zellij")
        .args(["kill-session", session_name])
        .status();
}

fn xdg_data_home(home_dir: &Path) -> PathBuf {
    if let Ok(raw) = std::env::var("XDG_DATA_HOME") {
        let trimmed = raw.trim();
        if !trimmed.is_empty() {
            return expand_home_path(trimmed, home_dir);
        }
    }
    home_dir.join(".local").join("share")
}

fn expand_home_path(raw: &str, home_dir: &Path) -> PathBuf {
    if raw == "~" {
        return home_dir.to_path_buf();
    }
    if let Some(rest) = raw.strip_prefix("~/") {
        return home_dir.join(rest);
    }
    PathBuf::from(raw)
}

fn quote_desktop_exec_arg(value: &Path) -> String {
    let escaped = value
        .to_string_lossy()
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('$', "\\$")
        .replace('`', "\\`");
    format!("\"{escaped}\"")
}

fn render_desktop_entry(launcher_path: &Path) -> String {
    [
        "[Desktop Entry]".to_string(),
        "Version=1.4".to_string(),
        "Type=Application".to_string(),
        "Name=Yazelix".to_string(),
        "Comment=Yazi + Zellij + Helix integrated terminal environment".to_string(),
        "Icon=yazelix".to_string(),
        format!("StartupWMClass={WINDOW_CLASS}"),
        "Terminal=true".to_string(),
        "X-Yazelix-Managed=true".to_string(),
        format!("Exec={} desktop launch", quote_desktop_exec_arg(launcher_path)),
        "Categories=Development;".to_string(),
    ]
    .join("\n")
}

fn install_desktop_icons(runtime_dir: &Path, icons_root: &Path) -> Result<(), CoreError> {
    for size in DESKTOP_ICON_SIZES {
        let source = runtime_dir.join("assets").join("icons").join(size).join("yazelix.png");
        if !source.is_file() {
            return Err(CoreError::classified(
                ErrorClass::Runtime,
                "missing_desktop_icon",
                format!("Missing Yazelix desktop icon asset: {}", source.display()),
                "Restore the runtime icon assets or reinstall Yazelix, then retry.",
                serde_json::json!({}),
            ));
        }
        let destination = icons_root.join(size).join("apps").join("yazelix.png");
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent).map_err(|source| {
                CoreError::io(
                    "desktop_icon_dir",
                    format!("Could not create icon directory {}.", parent.display()),
                    "Fix the directory permissions, then retry.",
                    parent.display().to_string(),
                    source,
                )
            })?;
        }
        fs::copy(&source, &destination).map_err(|error| {
            CoreError::io(
                "desktop_icon_copy",
                format!(
                    "Could not copy desktop icon {} to {}.",
                    source.display(),
                    destination.display()
                ),
                "Fix the directory permissions, then retry.",
                destination.display().to_string(),
                error,
            )
        })?;
    }
    Ok(())
}

fn maybe_validate_desktop_entry(desktop_path: &Path) -> Result<(), CoreError> {
    let Some(command) = find_command("desktop-file-validate") else {
        return Ok(());
    };
    let output = Command::new(command)
        .arg(desktop_path)
        .output()
        .map_err(|source| {
            CoreError::io(
                "desktop_file_validate",
                format!("Failed to run desktop-file-validate for {}.", desktop_path.display()),
                "Install desktop-file-validate or fix the host PATH, then retry.",
                desktop_path.display().to_string(),
                source,
            )
        })?;
    if output.status.success() {
        Ok(())
    } else {
        Err(CoreError::classified(
            ErrorClass::Runtime,
            "desktop_entry_invalid",
            format!(
                "Generated desktop entry failed validation: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            ),
            "Fix the generated desktop entry contract, then retry.",
            serde_json::json!({}),
        ))
    }
}

fn maybe_refresh_desktop_database(applications_dir: &Path) {
    if let Some(command) = find_command("update-desktop-database") {
        let _ = Command::new(command).arg(applications_dir).status();
    }
}

fn maybe_refresh_icon_cache(icons_root: &Path) {
    if let Some(command) = find_command("gtk-update-icon-cache") {
        let _ = Command::new(command)
            .args(["--force", "--ignore-theme-index"])
            .arg(icons_root)
            .status();
    }
}

fn print_desktop_progress(message: &str) {
    println!("Yazelix: {message}");
}

fn acknowledge_desktop_failure(error_text: &str) {
    println!();
    println!("Yazelix: Launch failed.");
    println!();
    println!("{error_text}");
    println!();
    print!("Press Enter to close this window.");
    let _ = io::stdout().flush();
    let mut line = String::new();
    let _ = io::stdin().read_line(&mut line);
}

fn find_command(name: &str) -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    std::env::split_paths(&path)
        .map(|entry| entry.join(name))
        .find(|candidate| candidate.is_file())
}

#[cfg(test)]
mod tests {
    use super::*;

    // Defends: Rust launch arg parsing keeps the public path and terminal flag aliases after the owner cut.
    // Strength: defect=2 behavior=2 resilience=1 cost=2 uniqueness=1 total=8/10
    #[test]
    fn parse_launch_args_accepts_aliases() {
        let parsed = parse_launch_args(&[
            "-p".into(),
            "/tmp/demo".into(),
            "-t".into(),
            "kitty".into(),
            "--verbose".into(),
        ])
        .unwrap();

        assert_eq!(parsed.path.as_deref(), Some("/tmp/demo"));
        assert_eq!(parsed.terminal.as_deref(), Some("kitty"));
        assert!(parsed.verbose);
    }

    // Defends: the Rust launch owner still filters duplicate or unsupported configured terminals before fallback logic runs.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=1 total=8/10
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

    // Defends: desktop entry rendering keeps a quoted launcher path so spaces do not corrupt the Exec owner surface.
    // Strength: defect=2 behavior=2 resilience=1 cost=2 uniqueness=1 total=8/10
    #[test]
    fn render_desktop_entry_quotes_exec_path() {
        let entry = render_desktop_entry(Path::new("/tmp/with space/yzx"));
        assert!(entry.contains("Exec=\"/tmp/with space/yzx\" desktop launch"));
        assert!(entry.contains("Terminal=true"));
    }
}
