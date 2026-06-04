use super::config_override::{
    config_override_extra_env, prepare_session_config_override, resolve_cli_config_override,
};
use super::process::command_status_with_overrides;
use super::resolve_requested_working_dir;
use super::sidebar_bootstrap_extra_env;
use crate::bridge::{CoreError, ErrorClass};
use crate::command_metadata::{YzxExternBridgeSyncRequest, sync_yzx_extern_bridge};
use crate::control_plane::{
    config_dir_from_env, config_override_from_env, default_shell_from_config, expand_user_path,
    home_dir_from_env, load_normalized_config_for_control, read_yazelix_version_from_runtime,
    runtime_dir_from_env, runtime_env_request, runtime_materialization_plan_request_from_env,
    state_dir_from_env, zellij_default_shell_from_runtime,
};
use crate::initializer_commands::generate_shell_initializers_for_env;
use crate::runtime_contract::evaluate_startup_working_dir_preflight;
use crate::runtime_env::compute_runtime_env;
use crate::runtime_materialization::{RuntimeArtifact, materialize_runtime_state};
use crate::session_config_snapshot::{
    SessionConfigSnapshotCreateRequest, write_session_config_snapshot_for_launch,
};
use crate::startup_handoff::{
    StartupHandoffArtifact, StartupHandoffCaptureRequest, capture_startup_handoff_context,
};
use serde_json::{Map as JsonMap, Value as JsonValue};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

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

    let requested_working_dir = resolve_requested_working_dir(parsed.path.as_deref(), parsed.home)?;
    let working_dir = resolve_startup_working_dir(requested_working_dir)?;

    run_runtime_setup(&runtime_dir, &default_shell, true)?;
    extra_env.extend(sidebar_bootstrap_extra_env("enter", &working_dir)?);

    let startup = prepare_rust_startup(
        &runtime_dir,
        &working_dir,
        config_override.as_deref(),
        parsed.verbose,
    )?;
    extra_env.extend(startup.extra_env);

    if startup.profile_exit_before_zellij {
        return Ok(0);
    }

    let status = command_status_with_overrides(
        &startup.argv,
        Some(&startup.runtime_env),
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

struct RustStartupPlan {
    argv: Vec<String>,
    runtime_env: JsonMap<String, JsonValue>,
    extra_env: Vec<(String, Option<String>)>,
    profile_exit_before_zellij: bool,
}

fn resolve_startup_working_dir(requested_working_dir: PathBuf) -> Result<PathBuf, CoreError> {
    evaluate_startup_working_dir_preflight(requested_working_dir)
}

fn prepare_rust_startup(
    runtime_dir: &Path,
    working_dir: &Path,
    config_override: Option<&str>,
    verbose: bool,
) -> Result<RustStartupPlan, CoreError> {
    let state_dir = state_dir_from_env()?;
    let materialization = materialize_runtime_state(
        &runtime_materialization_plan_request_from_env(config_override)?,
    )?;
    if verbose && materialization.plan.status != "noop" {
        println!("✅ Generated runtime state materialized.");
    }

    let runtime_version = read_yazelix_version_from_runtime(runtime_dir)?;
    let snapshot = write_session_config_snapshot_for_launch(
        &SessionConfigSnapshotCreateRequest {
            state_dir: state_dir.clone(),
            snapshot_id: launch_snapshot_id()?,
            source_config_file: materialization.plan.config_state.config_file.clone(),
            source_config_hash: materialization.plan.config_state.config_hash.clone(),
            runtime_dir: runtime_dir.to_path_buf(),
            runtime_hash: materialization.plan.config_state.runtime_hash.clone(),
            normalized_config: materialization.plan.config_state.config.clone(),
        },
        &runtime_version,
    )?;
    let status_bar_cache_path = status_bar_cache_path_for_snapshot(&snapshot.snapshot_path)?;

    let runtime_env = compute_runtime_env(&runtime_env_request(
        runtime_dir.to_path_buf(),
        &materialization.plan.config_state.config,
    )?)?
    .runtime_env;
    let default_shell = default_shell_from_config(&materialization.plan.config_state.config);
    let zellij_default_shell = zellij_default_shell_from_runtime(runtime_dir, &default_shell);
    let zellij_config_dir =
        require_existing_directory(&materialization.plan.zellij_config_dir, "Zellij config dir")?;
    let layout_path =
        require_existing_file(&materialization.plan.zellij_layout_path, "Zellij layout")?;

    capture_startup_handoff(
        &state_dir,
        working_dir,
        &zellij_config_dir,
        &layout_path,
        &zellij_default_shell,
        &materialization.plan.status,
        &materialization.plan.reason,
        materialization.plan.should_regenerate,
        materialization.plan.should_sync_static_assets,
        &materialization.plan.missing_artifacts,
        verbose,
    );

    let mut argv = vec!["zellij".to_string()];
    if let Some(session_name) = zellij_session_name_from_env() {
        argv.extend(["--session".to_string(), session_name]);
    }
    argv.extend([
        "--config-dir".to_string(),
        zellij_config_dir.clone(),
        "options".to_string(),
        "--default-cwd".to_string(),
        working_dir.to_string_lossy().into_owned(),
        "--default-layout".to_string(),
        layout_path,
        "--default-shell".to_string(),
        zellij_default_shell,
    ]);

    Ok(RustStartupPlan {
        argv,
        runtime_env,
        extra_env: vec![
            (
                "YAZELIX_SESSION_CONFIG_PATH".to_string(),
                Some(snapshot.snapshot_path),
            ),
            (
                "YAZELIX_STATUS_BAR_CACHE_PATH".to_string(),
                Some(status_bar_cache_path),
            ),
        ],
        profile_exit_before_zellij: bool_env("YAZELIX_STARTUP_PROFILE_EXIT_BEFORE_ZELLIJ"),
    })
}

fn launch_snapshot_id() -> Result<String, CoreError> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos().to_string())
        .map_err(|source| {
            CoreError::classified(
                ErrorClass::Internal,
                "startup_snapshot_clock",
                format!("System clock error while preparing a Yazelix session snapshot: {source}"),
                "Fix the system clock, then retry.",
                serde_json::json!({}),
            )
        })
}

fn status_bar_cache_path_for_snapshot(snapshot_path: &str) -> Result<String, CoreError> {
    let snapshot_path = Path::new(snapshot_path);
    let session_dir = snapshot_path.parent().ok_or_else(|| {
        CoreError::classified(
            ErrorClass::Internal,
            "startup_snapshot_path_parent",
            format!(
                "Yazelix session snapshot path has no parent directory: {}.",
                snapshot_path.display()
            ),
            "Report this as a Yazelix internal error.",
            serde_json::json!({ "snapshot_path": snapshot_path.to_string_lossy() }),
        )
    })?;
    Ok(session_dir
        .join("status_bar_cache.json")
        .to_string_lossy()
        .to_string())
}

fn require_existing_directory(path: &str, label: &str) -> Result<String, CoreError> {
    let path = PathBuf::from(path);
    if path.is_dir() {
        return Ok(path.to_string_lossy().to_string());
    }
    Err(CoreError::classified(
        ErrorClass::Runtime,
        "startup_missing_generated_directory",
        format!(
            "{label} is missing after Yazelix startup materialization: {}.",
            path.display()
        ),
        "Run `yzx doctor` to inspect the generated-state contract, then retry.",
        serde_json::json!({ "path": path.to_string_lossy() }),
    ))
}

fn require_existing_file(path: &str, label: &str) -> Result<String, CoreError> {
    let path = PathBuf::from(path);
    if path.is_file() {
        return Ok(path.to_string_lossy().to_string());
    }
    Err(CoreError::classified(
        ErrorClass::Runtime,
        "startup_missing_generated_file",
        format!(
            "{label} is missing after Yazelix startup materialization: {}.",
            path.display()
        ),
        "Run `yzx doctor` to inspect the generated-state contract, then retry.",
        serde_json::json!({ "path": path.to_string_lossy() }),
    ))
}

fn capture_startup_handoff(
    state_dir: &Path,
    working_dir: &Path,
    zellij_config_dir: &str,
    layout_path: &str,
    default_shell: &str,
    materialization_status: &str,
    materialization_reason: &str,
    materialization_should_regenerate: bool,
    materialization_should_sync_static_assets: bool,
    missing_artifacts: &[RuntimeArtifact],
    verbose: bool,
) {
    let result = capture_startup_handoff_context(&StartupHandoffCaptureRequest {
        state_dir: state_dir.to_path_buf(),
        working_dir: working_dir.to_string_lossy().to_string(),
        session_default_cwd: working_dir.to_string_lossy().to_string(),
        launch_process_cwd: working_dir.to_string_lossy().to_string(),
        zellij_config_dir: zellij_config_dir.to_string(),
        layout_path: layout_path.to_string(),
        default_shell: default_shell.to_string(),
        materialization_status: materialization_status.to_string(),
        materialization_reason: materialization_reason.to_string(),
        materialization_should_regenerate,
        materialization_should_sync_static_assets,
        missing_artifacts: missing_artifacts
            .iter()
            .map(|artifact| StartupHandoffArtifact {
                label: artifact.label.clone(),
                path: artifact.path.clone(),
            })
            .collect(),
    });

    match result {
        Ok(capture) if verbose && capture.recorded => {
            if let Some(path) = capture.context_path.or(capture.latest_path) {
                println!("📝 Startup handoff context: {path}");
            }
        }
        Err(error) => eprintln!(
            "⚠️ Failed to write startup handoff context: {}",
            error.message()
        ),
        _ => {}
    }
}

fn zellij_session_name_from_env() -> Option<String> {
    std::env::var("YAZELIX_ZELLIJ_SESSION_NAME")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn bool_env(key: &str) -> bool {
    std::env::var(key)
        .ok()
        .is_some_and(|value| value.trim() == "true")
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
