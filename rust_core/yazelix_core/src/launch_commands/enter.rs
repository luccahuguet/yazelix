use super::config_override::{
    config_override_extra_env, prepare_session_config_override, resolve_cli_config_override,
};
use super::process::{command_status_with_overrides, find_command};
use super::resolve_requested_working_dir;
use super::sidebar_bootstrap_extra_env;
use crate::bridge::{CoreError, ErrorClass};
use crate::command_metadata::{YzxExternBridgeSyncRequest, sync_yzx_extern_bridge};
use crate::control_plane::{
    config_dir_from_env, config_override_from_env, default_shell_from_config, expand_user_path,
    home_dir_from_env, load_normalized_config_for_control, runtime_dir_from_env,
    runtime_env_request, state_dir_from_env,
};
use crate::initializer_commands::generate_shell_initializers_for_env;
use crate::runtime_contract::{
    StartupLaunchPreflightRequest, StartupPreflightPayload, evaluate_startup_launch_preflight,
};
use crate::runtime_env::compute_runtime_env;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct EnterArgs {
    path: Option<String>,
    config: Option<String>,
    with_overrides: Vec<String>,
    home: bool,
    verbose: bool,
    setup_only: bool,
    help: bool,
}

pub(super) fn run_enter(args: &[String]) -> Result<i32, CoreError> {
    let parsed = parse_enter_args(args)?;
    if parsed.help {
        print_enter_help();
        return Ok(0);
    }

    let runtime_dir = runtime_dir_from_env()?;
    let config_dir = config_dir_from_env()?;
    let inherited_config_override = config_override_from_env();
    let config_override = prepare_session_config_override(
        parsed
            .config
            .as_deref()
            .or(inherited_config_override.as_deref()),
        &parsed.with_overrides,
    )?;
    let mut extra_env = config_override_extra_env(config_override.as_deref());
    let normalized =
        load_normalized_config_for_control(&runtime_dir, &config_dir, config_override.as_deref())?;
    let req = runtime_env_request(runtime_dir.clone(), &normalized)?;
    let runtime_data = compute_runtime_env(&req)?;
    let runtime_env = runtime_data.runtime_env;
    let default_shell = default_shell_from_config(&normalized);

    if parsed.verbose {
        println!("🔍 start_yazelix: verbose mode enabled");
        println!("🔍 Startup runtime env computed");
    }

    if parsed.setup_only {
        println!("🔧 Setting up Yazelix generated environment files...");
        run_runtime_setup(&runtime_dir, &default_shell, false)?;
        println!("✅ Setup complete.");
        return Ok(0);
    }

    let nu_bin = resolve_nu_bin(&runtime_dir)?;
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

    run_runtime_setup(&runtime_dir, &default_shell, true)?;
    extra_env.extend(sidebar_bootstrap_extra_env("enter", &working_dir)?);

    let mut argv = vec![
        nu_bin.to_string_lossy().into_owned(),
        "-i".to_string(),
        inner_script,
        working_dir.to_string_lossy().into_owned(),
    ];
    if parsed.verbose {
        argv.push("--verbose".to_string());
    }
    let status = command_status_with_overrides(
        &argv,
        Some(&runtime_env),
        &working_dir,
        &[],
        &extra_env,
        "enter_startup",
        "Retry from a valid Yazelix runtime or relaunch with `yzx launch`.",
    )?;
    Ok(status.code().unwrap_or(1))
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
            "--config" => {
                index += 1;
                let value = args.get(index).ok_or_else(|| {
                    CoreError::usage(
                        "Missing value for yzx enter --config. Try `yzx enter --help`.",
                    )
                })?;
                if parsed.config.is_some() {
                    return Err(CoreError::usage(
                        "yzx enter accepts at most one --config override.",
                    ));
                }
                parsed.config = Some(resolve_cli_config_override(value)?);
            }
            "--with" => {
                index += 1;
                let value = args.get(index).ok_or_else(|| {
                    CoreError::usage("Missing value for yzx enter --with. Try `yzx enter --help`.")
                })?;
                parsed.with_overrides.push(value.clone());
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

fn print_enter_help() {
    println!("Start Yazelix in the current terminal");
    println!();
    println!("Usage:");
    println!(
        "  yzx enter [--path <dir> | --home] [--config <file>] [--with key=value] [--verbose]"
    );
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
    default_shell: &str,
    quiet: bool,
) -> Result<(), CoreError> {
    let state_dir = state_dir_from_env()?;
    let log_dir = setup_log_dir(&state_dir)?;
    fs::create_dir_all(&state_dir).map_err(|source| {
        CoreError::io(
            "runtime_setup_state_dir",
            format!(
                "Cannot create Yazelix state directory {}.",
                state_dir.display()
            ),
            "Fix permissions or set YAZELIX_STATE_DIR to a writable path.",
            state_dir.display().to_string(),
            source,
        )
    })?;
    fs::create_dir_all(&log_dir).map_err(|source| {
        CoreError::io(
            "runtime_setup_log_dir",
            format!("Cannot create Yazelix log directory {}.", log_dir.display()),
            "Fix permissions or set YAZELIX_LOGS_DIR to a writable path.",
            log_dir.display().to_string(),
            source,
        )
    })?;

    let shells_to_configure = setup_shells(default_shell);
    let initializer_run = generate_shell_initializers_for_env(&shells_to_configure, quiet)?;
    for message in initializer_run.messages {
        println!("{message}");
    }

    if let Err(error) = sync_yzx_extern_bridge(&YzxExternBridgeSyncRequest {
        runtime_dir: runtime_dir.to_path_buf(),
        state_dir: state_dir.clone(),
    }) {
        eprintln!(
            "⚠️  Could not sync generated yzx extern bridge: {}",
            error.message()
        );
    }

    let zjstatus_target = runtime_dir
        .join("configs")
        .join("zellij")
        .join("plugins")
        .join("zjstatus.wasm");
    if !zjstatus_target.is_file() {
        return Err(CoreError::classified(
            ErrorClass::Runtime,
            "missing_zjstatus_plugin",
            format!(
                "Expected packaged zjstatus plugin at {}, but it was not found.",
                zjstatus_target.display()
            ),
            "Reinstall Yazelix or run from a packaged runtime that includes zjstatus.wasm.",
            serde_json::json!({"path": zjstatus_target.display().to_string()}),
        ));
    }

    Ok(())
}

fn setup_shells(default_shell: &str) -> Vec<String> {
    let mut out = Vec::new();
    for shell in ["nu", "bash", default_shell] {
        let trimmed = shell.trim().to_lowercase();
        if !trimmed.is_empty() && !out.contains(&trimmed) {
            out.push(trimmed);
        }
    }
    out
}

fn setup_log_dir(state_dir: &Path) -> Result<PathBuf, CoreError> {
    let Some(raw) = std::env::var("YAZELIX_LOGS_DIR")
        .ok()
        .map(|raw| raw.trim().to_string())
        .filter(|raw| !raw.is_empty())
    else {
        return Ok(state_dir.join("logs"));
    };
    Ok(expand_user_path(&raw, &home_dir_from_env()?))
}
