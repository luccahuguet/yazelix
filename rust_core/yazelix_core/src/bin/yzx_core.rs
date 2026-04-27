use lexopt::prelude::*;
use serde::de::DeserializeOwned;
use std::io::Write;
use std::path::PathBuf;
use yazelix_core::active_config_surface::resolve_active_config_paths;
use yazelix_core::control_plane::{
    config_dir_from_env, config_override_from_env, config_state_compute_request_from_env,
    config_state_record_request_from_env, ghostty_materialization_request_from_env,
    read_yazelix_version_from_runtime, runtime_dir_from_env, runtime_env_request_from_env,
    runtime_materialization_plan_request_from_env, state_dir_from_env,
    terminal_materialization_request_from_env,
};
use yazelix_core::{
    ComputeConfigStateRequest, CoreError, DoctorConfigEvaluateRequest,
    DoctorRuntimeEvaluateRequest, ErrorClass, GhosttyMaterializationRequest,
    HelixDoctorEvaluateRequest, HelixMaterializationRequest, InstallOwnershipEvaluateRequest,
    LaunchMaterializationRequest, NormalizeConfigRequest, RecordConfigStateRequest,
    RuntimeContractEvaluateRequest, RuntimeMaterializationPlanRequest,
    RuntimeMaterializationRepairEvaluateRequest, RuntimeMaterializationRepairRunData,
    RuntimeRepairDirective, StartupFactsData, StartupHandoffCaptureRequest,
    StartupLaunchPreflightRequest, TerminalMaterializationRequest, TransientPaneFactsData,
    YaziMaterializationRequest, YaziRenderPlanRequest, YzxExternBridgeSyncRequest,
    ZellijMaterializationRequest, ZellijRenderPlanRequest, capture_startup_handoff_context,
    compute_config_state, compute_integration_facts_from_env, compute_runtime_env,
    compute_startup_facts_from_env, compute_status_report, compute_transient_pane_facts_from_env,
    compute_yazi_render_plan, compute_zellij_render_plan, current_release_headline, error_envelope,
    evaluate_doctor_config_report, evaluate_doctor_runtime_report, evaluate_helix_doctor_report,
    evaluate_install_ownership_report, evaluate_runtime_contract,
    evaluate_startup_launch_preflight, generate_ghostty_materialization,
    generate_helix_materialization, generate_terminal_materialization,
    generate_yazi_materialization, generate_zellij_materialization,
    install_ownership_request_from_env, install_ownership_request_from_env_with_runtime_dir,
    launch_materialization_request_from_env, materialize_runtime_state,
    maybe_show_first_run_upgrade_summary, normalize_config, plan_runtime_materialization,
    prepare_launch_materialization, record_config_state, render_yzx_help,
    repair_runtime_materialization, success_envelope, sync_yzx_extern_bridge, yzx_command_metadata,
    yzx_command_metadata_data,
};

const CONFIG_NORMALIZE_COMMAND: &str = "config.normalize";
const CONFIG_SURFACE_RESOLVE_COMMAND: &str = "config-surface.resolve";
const CONFIG_STATE_COMPUTE_COMMAND: &str = "config-state.compute";
const CONFIG_STATE_RECORD_COMMAND: &str = "config-state.record";
const RUNTIME_CONTRACT_EVALUATE_COMMAND: &str = "runtime-contract.evaluate";
const STARTUP_LAUNCH_PREFLIGHT_EVALUATE_COMMAND: &str = "startup-launch-preflight.evaluate";
const RUNTIME_ENV_COMPUTE_COMMAND: &str = "runtime-env.compute";
const INTEGRATION_FACTS_COMPUTE_COMMAND: &str = "integration-facts.compute";
const TRANSIENT_PANE_FACTS_COMPUTE_COMMAND: &str = "transient-pane-facts.compute";
const STARTUP_FACTS_COMPUTE_COMMAND: &str = "startup-facts.compute";
const STARTUP_HANDOFF_CAPTURE_COMMAND: &str = "startup-handoff.capture";
const RUNTIME_MATERIALIZATION_PLAN_COMMAND: &str = "runtime-materialization.plan";
const RUNTIME_MATERIALIZATION_MATERIALIZE_COMMAND: &str = "runtime-materialization.materialize";
const RUNTIME_MATERIALIZATION_REPAIR_COMMAND: &str = "runtime-materialization.repair";
const STATUS_COMPUTE_COMMAND: &str = "status.compute";
const INSTALL_OWNERSHIP_EVALUATE_COMMAND: &str = "install-ownership.evaluate";
const DOCTOR_CONFIG_EVALUATE_COMMAND: &str = "doctor-config.evaluate";
const DOCTOR_HELIX_EVALUATE_COMMAND: &str = "doctor-helix.evaluate";
const DOCTOR_RUNTIME_EVALUATE_COMMAND: &str = "doctor-runtime.evaluate";
const ZELLIJ_RENDER_PLAN_COMPUTE_COMMAND: &str = "zellij-render-plan.compute";
const YAZI_RENDER_PLAN_COMPUTE_COMMAND: &str = "yazi-render-plan.compute";
const YAZI_MATERIALIZATION_GENERATE_COMMAND: &str = "yazi-materialization.generate";
const ZELLIJ_MATERIALIZATION_GENERATE_COMMAND: &str = "zellij-materialization.generate";
const HELIX_MATERIALIZATION_GENERATE_COMMAND: &str = "helix-materialization.generate";
const GHOSTTY_MATERIALIZATION_GENERATE_COMMAND: &str = "ghostty-materialization.generate";
const TERMINAL_MATERIALIZATION_GENERATE_COMMAND: &str = "terminal-materialization.generate";
const LAUNCH_MATERIALIZATION_PREPARE_COMMAND: &str = "launch-materialization.prepare";
const YZX_COMMAND_METADATA_LIST_COMMAND: &str = "yzx-command-metadata.list";
const YZX_COMMAND_METADATA_EXTERNS_COMMAND: &str = "yzx-command-metadata.externs";
const YZX_COMMAND_METADATA_SYNC_EXTERNS_COMMAND: &str = "yzx-command-metadata.sync-externs";
const YZX_COMMAND_METADATA_HELP_COMMAND: &str = "yzx-command-metadata.help";
const UPGRADE_SUMMARY_HEADLINE_COMMAND: &str = "upgrade-summary.headline";
const UPGRADE_SUMMARY_FIRST_RUN_COMMAND: &str = "upgrade-summary.first-run";
const UNKNOWN_COMMAND: &str = "unknown";

struct RuntimeMaterializationRepairCommand {
    request: RuntimeMaterializationRepairEvaluateRequest,
    summary: bool,
}

enum ErrorOutputMode {
    Json,
    RuntimeRepairSummary,
}

struct CommandError {
    command: String,
    error: CoreError,
    output_mode: ErrorOutputMode,
}

impl CommandError {
    fn new(command: impl Into<String>, error: CoreError) -> Box<Self> {
        Self::new_with_output_mode(command, error, ErrorOutputMode::Json)
    }

    fn runtime_repair_summary(command: impl Into<String>, error: CoreError) -> Box<Self> {
        Self::new_with_output_mode(command, error, ErrorOutputMode::RuntimeRepairSummary)
    }

    fn new_with_output_mode(
        command: impl Into<String>,
        error: CoreError,
        output_mode: ErrorOutputMode,
    ) -> Box<Self> {
        Box::new(Self {
            command: command.into(),
            error,
            output_mode,
        })
    }
}

fn main() {
    match run() {
        Ok(()) => {}
        Err(command_error) => {
            match command_error.output_mode {
                ErrorOutputMode::Json => {
                    let envelope = error_envelope(&command_error.command, &command_error.error);
                    let _ = serde_json::to_writer(std::io::stderr(), &envelope);
                    eprintln!();
                }
                ErrorOutputMode::RuntimeRepairSummary => {
                    let _ =
                        write_runtime_materialization_repair_error_summary(&command_error.error);
                }
            }
            std::process::exit(command_error.error.class().exit_code());
        }
    }
}

fn run() -> Result<(), Box<CommandError>> {
    let mut parser = lexopt::Parser::from_env();
    let Some(arg) = parser
        .next()
        .map_err(|error| CommandError::new(UNKNOWN_COMMAND, CoreError::usage(error.to_string())))?
    else {
        return Err(CommandError::new(
            UNKNOWN_COMMAND,
            CoreError::usage("Missing helper command"),
        ));
    };
    let command = match arg {
        Value(value) => value.into_string().map_err(|_| {
            CommandError::new(
                UNKNOWN_COMMAND,
                CoreError::usage("Helper command must be valid UTF-8"),
            )
        })?,
        _ => {
            return Err(CommandError::new(
                UNKNOWN_COMMAND,
                CoreError::usage("First argument must be a helper command"),
            ));
        }
    };

    match command.as_str() {
        CONFIG_NORMALIZE_COMMAND => {
            let command_for_error = command.clone();
            run_config_normalize(parser)
                .map_err(|error| CommandError::new(command_for_error, error))
        }
        CONFIG_SURFACE_RESOLVE_COMMAND => {
            let command_for_error = command.clone();
            run_config_surface_resolve(parser)
                .map_err(|error| CommandError::new(command_for_error, error))
        }
        CONFIG_STATE_COMPUTE_COMMAND => {
            let command_for_error = command.clone();
            run_config_state_compute(parser)
                .map_err(|error| CommandError::new(command_for_error, error))
        }
        CONFIG_STATE_RECORD_COMMAND => {
            let command_for_error = command.clone();
            run_config_state_record(parser)
                .map_err(|error| CommandError::new(command_for_error, error))
        }
        RUNTIME_CONTRACT_EVALUATE_COMMAND => {
            let command_for_error = command.clone();
            run_runtime_contract_evaluate(parser)
                .map_err(|error| CommandError::new(command_for_error, error))
        }
        STARTUP_LAUNCH_PREFLIGHT_EVALUATE_COMMAND => {
            let command_for_error = command.clone();
            run_startup_launch_preflight_evaluate(parser)
                .map_err(|error| CommandError::new(command_for_error, error))
        }
        RUNTIME_ENV_COMPUTE_COMMAND => {
            let command_for_error = command.clone();
            run_runtime_env_compute(parser)
                .map_err(|error| CommandError::new(command_for_error, error))
        }
        INTEGRATION_FACTS_COMPUTE_COMMAND => {
            let command_for_error = command.clone();
            run_integration_facts_compute(parser)
                .map_err(|error| CommandError::new(command_for_error, error))
        }
        TRANSIENT_PANE_FACTS_COMPUTE_COMMAND => {
            let command_for_error = command.clone();
            run_transient_pane_facts_compute(parser)
                .map_err(|error| CommandError::new(command_for_error, error))
        }
        STARTUP_FACTS_COMPUTE_COMMAND => {
            let command_for_error = command.clone();
            run_startup_facts_compute(parser)
                .map_err(|error| CommandError::new(command_for_error, error))
        }
        STARTUP_HANDOFF_CAPTURE_COMMAND => {
            let command_for_error = command.clone();
            run_startup_handoff_capture(parser)
                .map_err(|error| CommandError::new(command_for_error, error))
        }
        RUNTIME_MATERIALIZATION_PLAN_COMMAND => {
            let command_for_error = command.clone();
            run_runtime_materialization_plan(parser)
                .map_err(|error| CommandError::new(command_for_error, error))
        }
        RUNTIME_MATERIALIZATION_MATERIALIZE_COMMAND => {
            let command_for_error = command.clone();
            run_runtime_materialization_materialize(parser)
                .map_err(|error| CommandError::new(command_for_error, error))
        }
        RUNTIME_MATERIALIZATION_REPAIR_COMMAND => {
            run_runtime_materialization_repair(parser, command.clone())
        }
        STATUS_COMPUTE_COMMAND => {
            let command_for_error = command.clone();
            run_status_compute(parser).map_err(|error| CommandError::new(command_for_error, error))
        }
        INSTALL_OWNERSHIP_EVALUATE_COMMAND => {
            let command_for_error = command.clone();
            run_install_ownership_evaluate(parser)
                .map_err(|error| CommandError::new(command_for_error, error))
        }
        DOCTOR_CONFIG_EVALUATE_COMMAND => {
            let command_for_error = command.clone();
            run_doctor_config_evaluate(parser)
                .map_err(|error| CommandError::new(command_for_error, error))
        }
        DOCTOR_HELIX_EVALUATE_COMMAND => {
            let command_for_error = command.clone();
            run_doctor_helix_evaluate(parser)
                .map_err(|error| CommandError::new(command_for_error, error))
        }
        DOCTOR_RUNTIME_EVALUATE_COMMAND => {
            let command_for_error = command.clone();
            run_doctor_runtime_evaluate(parser)
                .map_err(|error| CommandError::new(command_for_error, error))
        }
        ZELLIJ_RENDER_PLAN_COMPUTE_COMMAND => {
            let command_for_error = command.clone();
            run_zellij_render_plan_compute(parser)
                .map_err(|error| CommandError::new(command_for_error, error))
        }
        YAZI_RENDER_PLAN_COMPUTE_COMMAND => {
            let command_for_error = command.clone();
            run_yazi_render_plan_compute(parser)
                .map_err(|error| CommandError::new(command_for_error, error))
        }
        YAZI_MATERIALIZATION_GENERATE_COMMAND => {
            let command_for_error = command.clone();
            run_yazi_materialization_generate(parser)
                .map_err(|error| CommandError::new(command_for_error, error))
        }
        ZELLIJ_MATERIALIZATION_GENERATE_COMMAND => {
            let command_for_error = command.clone();
            run_zellij_materialization_generate(parser)
                .map_err(|error| CommandError::new(command_for_error, error))
        }
        HELIX_MATERIALIZATION_GENERATE_COMMAND => {
            let command_for_error = command.clone();
            run_helix_materialization_generate(parser)
                .map_err(|error| CommandError::new(command_for_error, error))
        }
        GHOSTTY_MATERIALIZATION_GENERATE_COMMAND => {
            let command_for_error = command.clone();
            run_ghostty_materialization_generate(parser)
                .map_err(|error| CommandError::new(command_for_error, error))
        }
        TERMINAL_MATERIALIZATION_GENERATE_COMMAND => {
            let command_for_error = command.clone();
            run_terminal_materialization_generate(parser)
                .map_err(|error| CommandError::new(command_for_error, error))
        }
        LAUNCH_MATERIALIZATION_PREPARE_COMMAND => {
            let command_for_error = command.clone();
            run_launch_materialization_prepare(parser)
                .map_err(|error| CommandError::new(command_for_error, error))
        }
        YZX_COMMAND_METADATA_LIST_COMMAND => {
            let command_for_error = command.clone();
            run_yzx_command_metadata_list(parser)
                .map_err(|error| CommandError::new(command_for_error, error))
        }
        YZX_COMMAND_METADATA_EXTERNS_COMMAND => {
            let command_for_error = command.clone();
            run_yzx_command_metadata_externs(parser)
                .map_err(|error| CommandError::new(command_for_error, error))
        }
        YZX_COMMAND_METADATA_SYNC_EXTERNS_COMMAND => {
            let command_for_error = command.clone();
            run_yzx_command_metadata_sync_externs(parser)
                .map_err(|error| CommandError::new(command_for_error, error))
        }
        YZX_COMMAND_METADATA_HELP_COMMAND => {
            let command_for_error = command.clone();
            run_yzx_command_metadata_help(parser)
                .map_err(|error| CommandError::new(command_for_error, error))
        }
        UPGRADE_SUMMARY_HEADLINE_COMMAND => {
            let command_for_error = command.clone();
            run_upgrade_summary_headline(parser)
                .map_err(|error| CommandError::new(command_for_error, error))
        }
        UPGRADE_SUMMARY_FIRST_RUN_COMMAND => {
            let command_for_error = command.clone();
            run_upgrade_summary_first_run(parser)
                .map_err(|error| CommandError::new(command_for_error, error))
        }
        _ => Err(CommandError::new(
            command.clone(),
            CoreError::usage(format!("Unsupported helper command: {command}")),
        )),
    }
}

fn run_upgrade_summary_headline(parser: lexopt::Parser) -> Result<(), CoreError> {
    ensure_no_args(parser)?;
    let runtime_dir = runtime_dir_from_env()?;
    let version = read_yazelix_version_from_runtime(&runtime_dir)?;
    let headline = current_release_headline(&runtime_dir, &version)?;
    write_success_envelope(
        UPGRADE_SUMMARY_HEADLINE_COMMAND,
        serde_json::json!({
            "version": version,
            "headline": headline,
        }),
    )
}

fn run_upgrade_summary_first_run(parser: lexopt::Parser) -> Result<(), CoreError> {
    ensure_no_args(parser)?;
    let runtime_dir = runtime_dir_from_env()?;
    let state_dir = state_dir_from_env()?;
    let version = read_yazelix_version_from_runtime(&runtime_dir)?;
    let data = maybe_show_first_run_upgrade_summary(&runtime_dir, &state_dir, &version)?;
    write_success_envelope(UPGRADE_SUMMARY_FIRST_RUN_COMMAND, data)
}

fn run_yzx_command_metadata_list(parser: lexopt::Parser) -> Result<(), CoreError> {
    ensure_no_args(parser)?;
    write_success_envelope(
        YZX_COMMAND_METADATA_LIST_COMMAND,
        yzx_command_metadata_data(),
    )
}

fn run_yzx_command_metadata_externs(parser: lexopt::Parser) -> Result<(), CoreError> {
    ensure_no_args(parser)?;
    write_success_envelope(
        YZX_COMMAND_METADATA_EXTERNS_COMMAND,
        yzx_command_metadata_data(),
    )
}

fn run_yzx_command_metadata_sync_externs(mut parser: lexopt::Parser) -> Result<(), CoreError> {
    let mut runtime_dir: Option<PathBuf> = None;
    let mut state_dir: Option<PathBuf> = None;

    while let Some(arg) = parser
        .next()
        .map_err(|error| CoreError::usage(error.to_string()))?
    {
        match arg {
            Long("runtime-dir") => runtime_dir = Some(parser_path_value(&mut parser)?),
            Long("state-dir") => state_dir = Some(parser_path_value(&mut parser)?),
            _ => return Err(CoreError::usage(format!("Unexpected argument: {arg:?}"))),
        }
    }

    let request = YzxExternBridgeSyncRequest {
        runtime_dir: runtime_dir.ok_or_else(|| CoreError::usage("Missing --runtime-dir path"))?,
        state_dir: state_dir.ok_or_else(|| CoreError::usage("Missing --state-dir path"))?,
    };
    let data = sync_yzx_extern_bridge(&request)?;
    write_success_envelope(YZX_COMMAND_METADATA_SYNC_EXTERNS_COMMAND, data)
}

fn run_yzx_command_metadata_help(parser: lexopt::Parser) -> Result<(), CoreError> {
    ensure_no_args(parser)?;
    print!("{}", render_yzx_help(&yzx_command_metadata()));
    Ok(())
}

fn run_config_normalize(mut parser: lexopt::Parser) -> Result<(), CoreError> {
    let mut config_path: Option<PathBuf> = None;
    let mut default_config_path: Option<PathBuf> = None;
    let mut contract_path: Option<PathBuf> = None;
    let mut include_missing = false;

    while let Some(arg) = parser
        .next()
        .map_err(|error| CoreError::usage(error.to_string()))?
    {
        match arg {
            Long("config") => config_path = Some(parser_path_value(&mut parser)?),
            Long("default-config") => default_config_path = Some(parser_path_value(&mut parser)?),
            Long("contract") => contract_path = Some(parser_path_value(&mut parser)?),
            Long("include-missing") => include_missing = true,
            _ => return Err(CoreError::usage(format!("Unexpected argument: {arg:?}"))),
        }
    }

    let request = NormalizeConfigRequest {
        config_path: config_path.ok_or_else(|| CoreError::usage("Missing --config path"))?,
        default_config_path: default_config_path
            .ok_or_else(|| CoreError::usage("Missing --default-config path"))?,
        contract_path: contract_path.ok_or_else(|| CoreError::usage("Missing --contract path"))?,
        include_missing,
    };
    let data = normalize_config(&request)?;
    write_success_envelope(CONFIG_NORMALIZE_COMMAND, data)
}

fn run_config_surface_resolve(mut parser: lexopt::Parser) -> Result<(), CoreError> {
    let mut runtime_dir: Option<PathBuf> = None;
    let mut config_dir: Option<PathBuf> = None;
    let mut config_override: Option<String> = None;

    while let Some(arg) = parser
        .next()
        .map_err(|error| CoreError::usage(error.to_string()))?
    {
        match arg {
            Long("runtime-dir") => runtime_dir = Some(parser_path_value(&mut parser)?),
            Long("config-dir") => config_dir = Some(parser_path_value(&mut parser)?),
            Long("config-override") => config_override = Some(parser_string_value(&mut parser)?),
            _ => return Err(CoreError::usage(format!("Unexpected argument: {arg:?}"))),
        }
    }

    let runtime_dir = match runtime_dir {
        Some(path) => path,
        None => runtime_dir_from_env()?,
    };
    let config_dir = match config_dir {
        Some(path) => path,
        None => config_dir_from_env()?,
    };
    let config_override = config_override.or_else(config_override_from_env);
    let data = resolve_active_config_paths(&runtime_dir, &config_dir, config_override.as_deref())?;
    write_success_envelope(CONFIG_SURFACE_RESOLVE_COMMAND, data)
}

fn run_config_state_compute(mut parser: lexopt::Parser) -> Result<(), CoreError> {
    let mut config_path: Option<PathBuf> = None;
    let mut default_config_path: Option<PathBuf> = None;
    let mut contract_path: Option<PathBuf> = None;
    let mut runtime_dir: Option<PathBuf> = None;
    let mut state_path: Option<PathBuf> = None;
    let mut from_env = false;

    while let Some(arg) = parser
        .next()
        .map_err(|error| CoreError::usage(error.to_string()))?
    {
        match arg {
            Long("config") => config_path = Some(parser_path_value(&mut parser)?),
            Long("default-config") => default_config_path = Some(parser_path_value(&mut parser)?),
            Long("contract") => contract_path = Some(parser_path_value(&mut parser)?),
            Long("runtime-dir") => runtime_dir = Some(parser_path_value(&mut parser)?),
            Long("state-path") => state_path = Some(parser_path_value(&mut parser)?),
            Long("from-env") => from_env = true,
            _ => return Err(CoreError::usage(format!("Unexpected argument: {arg:?}"))),
        }
    }

    let explicit_args_present = config_path.is_some()
        || default_config_path.is_some()
        || contract_path.is_some()
        || runtime_dir.is_some()
        || state_path.is_some();

    let request = if from_env {
        if explicit_args_present {
            return Err(CoreError::usage(
                "Use either --from-env or explicit config-state.compute paths, not both.",
            ));
        }
        config_state_compute_request_from_env(config_override_from_env().as_deref())?
    } else {
        ComputeConfigStateRequest {
            config_path: config_path.ok_or_else(|| CoreError::usage("Missing --config path"))?,
            default_config_path: default_config_path
                .ok_or_else(|| CoreError::usage("Missing --default-config path"))?,
            contract_path: contract_path
                .ok_or_else(|| CoreError::usage("Missing --contract path"))?,
            runtime_dir: runtime_dir
                .ok_or_else(|| CoreError::usage("Missing --runtime-dir path"))?,
            state_path: state_path.ok_or_else(|| CoreError::usage("Missing --state-path path"))?,
        }
    };
    let data = compute_config_state(&request)?;
    write_success_envelope(CONFIG_STATE_COMPUTE_COMMAND, data)
}

fn run_config_state_record(mut parser: lexopt::Parser) -> Result<(), CoreError> {
    let mut config_file: Option<String> = None;
    let mut managed_config_path: Option<PathBuf> = None;
    let mut state_path: Option<PathBuf> = None;
    let mut config_hash: Option<String> = None;
    let mut runtime_hash: Option<String> = None;
    let mut from_env = false;

    while let Some(arg) = parser
        .next()
        .map_err(|error| CoreError::usage(error.to_string()))?
    {
        match arg {
            Long("config-file") => config_file = Some(parser_string_value(&mut parser)?),
            Long("managed-config") => managed_config_path = Some(parser_path_value(&mut parser)?),
            Long("state-path") => state_path = Some(parser_path_value(&mut parser)?),
            Long("config-hash") => config_hash = Some(parser_string_value(&mut parser)?),
            Long("runtime-hash") => runtime_hash = Some(parser_string_value(&mut parser)?),
            Long("from-env") => from_env = true,
            _ => return Err(CoreError::usage(format!("Unexpected argument: {arg:?}"))),
        }
    }

    let explicit_args_present = managed_config_path.is_some() || state_path.is_some();

    let config_file = config_file.ok_or_else(|| CoreError::usage("Missing --config-file path"))?;
    let config_hash = config_hash.ok_or_else(|| CoreError::usage("Missing --config-hash value"))?;
    let runtime_hash =
        runtime_hash.ok_or_else(|| CoreError::usage("Missing --runtime-hash value"))?;

    let request = if from_env {
        if explicit_args_present {
            return Err(CoreError::usage(
                "Use either --from-env or explicit config-state.record paths, not both.",
            ));
        }
        config_state_record_request_from_env(config_file, config_hash, runtime_hash)?
    } else {
        RecordConfigStateRequest {
            config_file,
            managed_config_path: managed_config_path
                .ok_or_else(|| CoreError::usage("Missing --managed-config path"))?,
            state_path: state_path.ok_or_else(|| CoreError::usage("Missing --state-path path"))?,
            config_hash,
            runtime_hash,
        }
    };
    let data = record_config_state(&request)?;
    write_success_envelope(CONFIG_STATE_RECORD_COMMAND, data)
}

fn run_runtime_contract_evaluate(mut parser: lexopt::Parser) -> Result<(), CoreError> {
    let mut request_json: Option<String> = None;

    while let Some(arg) = parser
        .next()
        .map_err(|error| CoreError::usage(error.to_string()))?
    {
        match arg {
            Long("request-json") => request_json = Some(parser_string_value(&mut parser)?),
            _ => return Err(CoreError::usage(format!("Unexpected argument: {arg:?}"))),
        }
    }

    let request_json =
        request_json.ok_or_else(|| CoreError::usage("Missing --request-json payload"))?;
    let request: RuntimeContractEvaluateRequest =
        serde_json::from_str(&request_json).map_err(|error| {
            CoreError::classified(
                ErrorClass::Usage,
                "invalid_request_json",
                format!("Invalid runtime-contract request JSON: {error}"),
                "Pass one valid JSON payload via --request-json.",
                serde_json::json!({}),
            )
        })?;
    let data = evaluate_runtime_contract(&request)?;
    write_success_envelope(RUNTIME_CONTRACT_EVALUATE_COMMAND, data)
}

fn run_startup_launch_preflight_evaluate(mut parser: lexopt::Parser) -> Result<(), CoreError> {
    let request_json = take_request_json(&mut parser)?;
    let request: StartupLaunchPreflightRequest =
        deserialize_json_request(&request_json, "startup-launch-preflight")?;
    let data = evaluate_startup_launch_preflight(&request)?;
    write_success_envelope(STARTUP_LAUNCH_PREFLIGHT_EVALUATE_COMMAND, data)
}

fn run_doctor_helix_evaluate(mut parser: lexopt::Parser) -> Result<(), CoreError> {
    let request_json = take_request_json(&mut parser)?;
    let request: HelixDoctorEvaluateRequest =
        deserialize_json_request(&request_json, "doctor-helix")?;
    let data = evaluate_helix_doctor_report(&request);
    write_success_envelope(DOCTOR_HELIX_EVALUATE_COMMAND, data)
}

fn run_doctor_config_evaluate(mut parser: lexopt::Parser) -> Result<(), CoreError> {
    let request_json = take_request_json(&mut parser)?;
    let request: DoctorConfigEvaluateRequest =
        deserialize_json_request(&request_json, "doctor-config")?;
    let data = evaluate_doctor_config_report(&request);
    write_success_envelope(DOCTOR_CONFIG_EVALUATE_COMMAND, data)
}

fn run_doctor_runtime_evaluate(mut parser: lexopt::Parser) -> Result<(), CoreError> {
    let request_json = take_request_json(&mut parser)?;
    let request: DoctorRuntimeEvaluateRequest =
        deserialize_json_request(&request_json, "doctor-runtime")?;
    let data = evaluate_doctor_runtime_report(&request);
    write_success_envelope(DOCTOR_RUNTIME_EVALUATE_COMMAND, data)
}

fn run_install_ownership_evaluate(mut parser: lexopt::Parser) -> Result<(), CoreError> {
    let mut request_json: Option<String> = None;
    let mut from_env = false;
    let mut runtime_dir: Option<PathBuf> = None;

    while let Some(arg) = parser
        .next()
        .map_err(|error| CoreError::usage(error.to_string()))?
    {
        match arg {
            Long("request-json") => request_json = Some(parser_string_value(&mut parser)?),
            Long("from-env") => from_env = true,
            Long("runtime-dir") => runtime_dir = Some(parser_path_value(&mut parser)?),
            _ => return Err(CoreError::usage(format!("Unexpected argument: {arg:?}"))),
        }
    }

    let request = match (from_env, request_json) {
        (true, Some(_)) => {
            return Err(CoreError::usage(
                "Use either --from-env or --request-json for install-ownership.evaluate, not both.",
            ));
        }
        (true, None) => match runtime_dir {
            Some(runtime_dir) => install_ownership_request_from_env_with_runtime_dir(runtime_dir)?,
            None => install_ownership_request_from_env()?,
        },
        (false, Some(request_json)) => {
            if runtime_dir.is_some() {
                return Err(CoreError::usage(
                    "Use --runtime-dir only with --from-env for install-ownership.evaluate.",
                ));
            }
            serde_json::from_str::<InstallOwnershipEvaluateRequest>(&request_json).map_err(
                |error| {
                    CoreError::classified(
                        ErrorClass::Usage,
                        "invalid_request_json",
                        format!("Invalid install-ownership request JSON: {error}"),
                        "Pass one valid JSON payload via --request-json.",
                        serde_json::json!({}),
                    )
                },
            )?
        }
        (false, None) => {
            return Err(CoreError::usage(
                "Missing --request-json payload or --from-env for install-ownership.evaluate.",
            ));
        }
    };
    let data = evaluate_install_ownership_report(&request);
    write_success_envelope(INSTALL_OWNERSHIP_EVALUATE_COMMAND, data)
}

fn run_zellij_render_plan_compute(mut parser: lexopt::Parser) -> Result<(), CoreError> {
    let request_json = take_request_json(&mut parser)?;
    let request: ZellijRenderPlanRequest =
        deserialize_json_request(&request_json, "zellij-render-plan")?;
    let data = compute_zellij_render_plan(&request)?;
    write_success_envelope(ZELLIJ_RENDER_PLAN_COMPUTE_COMMAND, data)
}

fn run_yazi_render_plan_compute(mut parser: lexopt::Parser) -> Result<(), CoreError> {
    let request_json = take_request_json(&mut parser)?;
    let request: YaziRenderPlanRequest =
        deserialize_json_request(&request_json, "yazi-render-plan")?;
    let data = compute_yazi_render_plan(&request)?;
    write_success_envelope(YAZI_RENDER_PLAN_COMPUTE_COMMAND, data)
}

fn run_yazi_materialization_generate(mut parser: lexopt::Parser) -> Result<(), CoreError> {
    let mut config_path: Option<PathBuf> = None;
    let mut default_config_path: Option<PathBuf> = None;
    let mut contract_path: Option<PathBuf> = None;
    let mut runtime_dir: Option<PathBuf> = None;
    let mut yazi_config_dir: Option<PathBuf> = None;
    let mut sync_static_assets = false;

    while let Some(arg) = parser
        .next()
        .map_err(|error| CoreError::usage(error.to_string()))?
    {
        match arg {
            Long("config") => config_path = Some(parser_path_value(&mut parser)?),
            Long("default-config") => default_config_path = Some(parser_path_value(&mut parser)?),
            Long("contract") => contract_path = Some(parser_path_value(&mut parser)?),
            Long("runtime-dir") => runtime_dir = Some(parser_path_value(&mut parser)?),
            Long("yazi-config-dir") => yazi_config_dir = Some(parser_path_value(&mut parser)?),
            Long("sync-static-assets") => sync_static_assets = true,
            _ => return Err(CoreError::usage(format!("Unexpected argument: {arg:?}"))),
        }
    }

    let request = YaziMaterializationRequest {
        config_path: config_path.ok_or_else(|| CoreError::usage("Missing --config path"))?,
        default_config_path: default_config_path
            .ok_or_else(|| CoreError::usage("Missing --default-config path"))?,
        contract_path: contract_path.ok_or_else(|| CoreError::usage("Missing --contract path"))?,
        runtime_dir: runtime_dir.ok_or_else(|| CoreError::usage("Missing --runtime-dir path"))?,
        yazi_config_dir: yazi_config_dir
            .ok_or_else(|| CoreError::usage("Missing --yazi-config-dir path"))?,
        sync_static_assets,
    };
    let data = generate_yazi_materialization(&request)?;
    write_success_envelope(YAZI_MATERIALIZATION_GENERATE_COMMAND, data)
}

fn run_zellij_materialization_generate(mut parser: lexopt::Parser) -> Result<(), CoreError> {
    let mut config_path: Option<PathBuf> = None;
    let mut default_config_path: Option<PathBuf> = None;
    let mut contract_path: Option<PathBuf> = None;
    let mut runtime_dir: Option<PathBuf> = None;
    let mut zellij_config_dir: Option<PathBuf> = None;
    let mut seed_plugin_permissions = false;

    while let Some(arg) = parser
        .next()
        .map_err(|error| CoreError::usage(error.to_string()))?
    {
        match arg {
            Long("config") => config_path = Some(parser_path_value(&mut parser)?),
            Long("default-config") => default_config_path = Some(parser_path_value(&mut parser)?),
            Long("contract") => contract_path = Some(parser_path_value(&mut parser)?),
            Long("runtime-dir") => runtime_dir = Some(parser_path_value(&mut parser)?),
            Long("zellij-config-dir") => zellij_config_dir = Some(parser_path_value(&mut parser)?),
            Long("seed-plugin-permissions") => seed_plugin_permissions = true,
            _ => return Err(CoreError::usage(format!("Unexpected argument: {arg:?}"))),
        }
    }

    let request = ZellijMaterializationRequest {
        config_path: config_path.ok_or_else(|| CoreError::usage("Missing --config path"))?,
        default_config_path: default_config_path
            .ok_or_else(|| CoreError::usage("Missing --default-config path"))?,
        contract_path: contract_path.ok_or_else(|| CoreError::usage("Missing --contract path"))?,
        runtime_dir: runtime_dir.ok_or_else(|| CoreError::usage("Missing --runtime-dir path"))?,
        zellij_config_dir: zellij_config_dir
            .ok_or_else(|| CoreError::usage("Missing --zellij-config-dir path"))?,
        seed_plugin_permissions,
    };
    let data = generate_zellij_materialization(&request)?;
    write_success_envelope(ZELLIJ_MATERIALIZATION_GENERATE_COMMAND, data)
}

fn run_helix_materialization_generate(mut parser: lexopt::Parser) -> Result<(), CoreError> {
    let mut runtime_dir: Option<PathBuf> = None;
    let mut config_dir: Option<PathBuf> = None;
    let mut state_dir: Option<PathBuf> = None;

    while let Some(arg) = parser
        .next()
        .map_err(|error| CoreError::usage(error.to_string()))?
    {
        match arg {
            Long("runtime-dir") => runtime_dir = Some(parser_path_value(&mut parser)?),
            Long("config-dir") => config_dir = Some(parser_path_value(&mut parser)?),
            Long("state-dir") => state_dir = Some(parser_path_value(&mut parser)?),
            _ => return Err(CoreError::usage(format!("Unexpected argument: {arg:?}"))),
        }
    }

    let request = HelixMaterializationRequest {
        runtime_dir: runtime_dir.ok_or_else(|| CoreError::usage("Missing --runtime-dir path"))?,
        config_dir: config_dir.ok_or_else(|| CoreError::usage("Missing --config-dir path"))?,
        state_dir: state_dir.ok_or_else(|| CoreError::usage("Missing --state-dir path"))?,
    };
    let data = generate_helix_materialization(&request)?;
    write_success_envelope(HELIX_MATERIALIZATION_GENERATE_COMMAND, data)
}

fn run_ghostty_materialization_generate(mut parser: lexopt::Parser) -> Result<(), CoreError> {
    let mut runtime_dir: Option<PathBuf> = None;
    let mut config_dir: Option<PathBuf> = None;
    let mut state_dir: Option<PathBuf> = None;
    let mut transparency: Option<String> = None;
    let mut cursor_config_path: Option<PathBuf> = None;
    let mut from_env = false;

    while let Some(arg) = parser
        .next()
        .map_err(|error| CoreError::usage(error.to_string()))?
    {
        match arg {
            Long("from-env") => from_env = true,
            Long("runtime-dir") => runtime_dir = Some(parser_path_value(&mut parser)?),
            Long("config-dir") => config_dir = Some(parser_path_value(&mut parser)?),
            Long("state-dir") => state_dir = Some(parser_path_value(&mut parser)?),
            Long("transparency") => transparency = Some(parser_string_value(&mut parser)?),
            Long("cursor-config") => cursor_config_path = Some(parser_path_value(&mut parser)?),
            _ => return Err(CoreError::usage(format!("Unexpected argument: {arg:?}"))),
        }
    }

    let explicit_args_present = runtime_dir.is_some()
        || config_dir.is_some()
        || state_dir.is_some()
        || transparency.is_some()
        || cursor_config_path.is_some();

    let request = if from_env {
        if explicit_args_present {
            return Err(CoreError::usage(
                "Use either --from-env or explicit ghostty-materialization.generate flags, not both.",
            ));
        }
        ghostty_materialization_request_from_env(config_override_from_env().as_deref())?
    } else {
        GhosttyMaterializationRequest {
            runtime_dir: runtime_dir
                .ok_or_else(|| CoreError::usage("Missing --runtime-dir path"))?,
            config_dir: config_dir.ok_or_else(|| CoreError::usage("Missing --config-dir path"))?,
            state_dir: state_dir.ok_or_else(|| CoreError::usage("Missing --state-dir path"))?,
            transparency: transparency.ok_or_else(|| CoreError::usage("Missing --transparency"))?,
            cursor_config_path: cursor_config_path
                .ok_or_else(|| CoreError::usage("Missing --cursor-config path"))?,
        }
    };
    let data = generate_ghostty_materialization(&request)?;
    write_success_envelope(GHOSTTY_MATERIALIZATION_GENERATE_COMMAND, data)
}

fn run_terminal_materialization_generate(mut parser: lexopt::Parser) -> Result<(), CoreError> {
    let mut config_path: Option<PathBuf> = None;
    let mut default_config_path: Option<PathBuf> = None;
    let mut contract_path: Option<PathBuf> = None;
    let mut runtime_dir: Option<PathBuf> = None;
    let mut state_dir: Option<PathBuf> = None;
    let mut terminals_json: Option<String> = None;
    let mut from_env = false;

    while let Some(arg) = parser
        .next()
        .map_err(|error| CoreError::usage(error.to_string()))?
    {
        match arg {
            Long("from-env") => from_env = true,
            Long("config") => config_path = Some(parser_path_value(&mut parser)?),
            Long("default-config") => default_config_path = Some(parser_path_value(&mut parser)?),
            Long("contract") => contract_path = Some(parser_path_value(&mut parser)?),
            Long("runtime-dir") => runtime_dir = Some(parser_path_value(&mut parser)?),
            Long("state-dir") => state_dir = Some(parser_path_value(&mut parser)?),
            Long("terminals-json") => terminals_json = Some(parser_string_value(&mut parser)?),
            _ => return Err(CoreError::usage(format!("Unexpected argument: {arg:?}"))),
        }
    }

    let terminals: Vec<String> = serde_json::from_str(
        &terminals_json.ok_or_else(|| CoreError::usage("Missing --terminals-json"))?,
    )
    .map_err(|error| CoreError::usage(format!("Invalid --terminals-json: {error}")))?;

    let explicit_args_present = config_path.is_some()
        || default_config_path.is_some()
        || contract_path.is_some()
        || runtime_dir.is_some()
        || state_dir.is_some();

    let request = if from_env {
        if explicit_args_present {
            return Err(CoreError::usage(
                "Use either --from-env or explicit terminal-materialization.generate paths, not both.",
            ));
        }
        terminal_materialization_request_from_env(terminals, config_override_from_env().as_deref())?
    } else {
        TerminalMaterializationRequest {
            config_path: config_path.ok_or_else(|| CoreError::usage("Missing --config path"))?,
            default_config_path: default_config_path
                .ok_or_else(|| CoreError::usage("Missing --default-config path"))?,
            contract_path: contract_path
                .ok_or_else(|| CoreError::usage("Missing --contract path"))?,
            runtime_dir: runtime_dir
                .ok_or_else(|| CoreError::usage("Missing --runtime-dir path"))?,
            state_dir: state_dir.ok_or_else(|| CoreError::usage("Missing --state-dir path"))?,
            terminals,
        }
    };
    let data = generate_terminal_materialization(&request)?;
    write_success_envelope(TERMINAL_MATERIALIZATION_GENERATE_COMMAND, data)
}

fn run_launch_materialization_prepare(mut parser: lexopt::Parser) -> Result<(), CoreError> {
    let mut selected_terminals_json: Option<String> = None;
    let mut desktop_fast_path = false;
    let mut from_env = false;

    while let Some(arg) = parser
        .next()
        .map_err(|error| CoreError::usage(error.to_string()))?
    {
        match arg {
            Long("from-env") => from_env = true,
            Long("desktop-fast-path") => desktop_fast_path = true,
            Long("selected-terminals-json") => {
                selected_terminals_json = Some(parser_string_value(&mut parser)?)
            }
            _ => return Err(CoreError::usage(format!("Unexpected argument: {arg:?}"))),
        }
    }

    if !from_env {
        return Err(CoreError::usage(
            "launch-materialization.prepare currently requires --from-env.",
        ));
    }

    let selected_terminals = match selected_terminals_json {
        Some(raw) => serde_json::from_str::<Vec<String>>(&raw).map_err(|error| {
            CoreError::usage(format!("Invalid --selected-terminals-json: {error}"))
        })?,
        None => Vec::new(),
    };
    let request: LaunchMaterializationRequest =
        launch_materialization_request_from_env(selected_terminals, desktop_fast_path)?;
    let data = prepare_launch_materialization(&request)?;
    write_success_envelope(LAUNCH_MATERIALIZATION_PREPARE_COMMAND, data)
}

fn run_runtime_env_compute(mut parser: lexopt::Parser) -> Result<(), CoreError> {
    let mut request_json: Option<String> = None;
    let mut config_json: Option<String> = None;
    let mut from_env = false;

    while let Some(arg) = parser
        .next()
        .map_err(|error| CoreError::usage(error.to_string()))?
    {
        match arg {
            Long("request-json") => request_json = Some(parser_string_value(&mut parser)?),
            Long("config-json") => config_json = Some(parser_string_value(&mut parser)?),
            Long("from-env") => from_env = true,
            _ => return Err(CoreError::usage(format!("Unexpected argument: {arg:?}"))),
        }
    }

    let request = if from_env {
        if request_json.is_some() {
            return Err(CoreError::usage(
                "Use either --from-env or --request-json for runtime-env.compute, not both.",
            ));
        }
        runtime_env_request_from_env(
            config_json.as_deref(),
            config_override_from_env().as_deref(),
        )?
    } else {
        if config_json.is_some() {
            return Err(CoreError::usage(
                "runtime-env.compute only accepts --config-json together with --from-env.",
            ));
        }
        let request_json =
            request_json.ok_or_else(|| CoreError::usage("Missing --request-json payload"))?;
        serde_json::from_str(&request_json).map_err(|error| {
            CoreError::classified(
                ErrorClass::Usage,
                "invalid_request_json",
                format!("Invalid runtime-env request JSON: {error}"),
                "Pass one valid JSON payload via --request-json.",
                serde_json::json!({}),
            )
        })?
    };
    let data = compute_runtime_env(&request)?;
    write_success_envelope(RUNTIME_ENV_COMPUTE_COMMAND, data)
}

fn run_integration_facts_compute(parser: lexopt::Parser) -> Result<(), CoreError> {
    ensure_no_args(parser)?;
    let data = compute_integration_facts_from_env()?;
    write_success_envelope(INTEGRATION_FACTS_COMPUTE_COMMAND, data)
}

fn run_transient_pane_facts_compute(parser: lexopt::Parser) -> Result<(), CoreError> {
    ensure_no_args(parser)?;
    let data: TransientPaneFactsData = compute_transient_pane_facts_from_env()?;
    write_success_envelope(TRANSIENT_PANE_FACTS_COMPUTE_COMMAND, data)
}

fn run_startup_facts_compute(parser: lexopt::Parser) -> Result<(), CoreError> {
    ensure_no_args(parser)?;
    let data: StartupFactsData = compute_startup_facts_from_env()?;
    write_success_envelope(STARTUP_FACTS_COMPUTE_COMMAND, data)
}

fn run_startup_handoff_capture(mut parser: lexopt::Parser) -> Result<(), CoreError> {
    let request_json = take_request_json(&mut parser)?;
    let request: StartupHandoffCaptureRequest =
        deserialize_json_request(&request_json, "startup-handoff.capture")?;
    let data = capture_startup_handoff_context(&request)?;
    write_success_envelope(STARTUP_HANDOFF_CAPTURE_COMMAND, data)
}

fn run_runtime_materialization_plan(mut parser: lexopt::Parser) -> Result<(), CoreError> {
    let request =
        take_runtime_materialization_plan_request(&mut parser, "runtime-materialization.plan")?;
    let data = plan_runtime_materialization(&request)?;
    write_success_envelope(RUNTIME_MATERIALIZATION_PLAN_COMMAND, data)
}

fn run_runtime_materialization_materialize(mut parser: lexopt::Parser) -> Result<(), CoreError> {
    let request = take_runtime_materialization_plan_request(
        &mut parser,
        "runtime-materialization.materialize",
    )?;
    let data = materialize_runtime_state(&request)?;
    write_success_envelope(RUNTIME_MATERIALIZATION_MATERIALIZE_COMMAND, data)
}

fn run_runtime_materialization_repair(
    mut parser: lexopt::Parser,
    command_name: String,
) -> Result<(), Box<CommandError>> {
    let command = take_runtime_materialization_repair_command(&mut parser)
        .map_err(|error| CommandError::new(command_name.clone(), error))?;
    let data = repair_runtime_materialization(&command.request).map_err(|error| {
        if command.summary {
            CommandError::runtime_repair_summary(command_name.clone(), error)
        } else {
            CommandError::new(command_name.clone(), error)
        }
    })?;
    if command.summary {
        write_runtime_materialization_repair_summary(&data)
            .map_err(|error| CommandError::new(command_name.clone(), error))
    } else {
        write_success_envelope(RUNTIME_MATERIALIZATION_REPAIR_COMMAND, data)
            .map_err(|error| CommandError::new(command_name.clone(), error))
    }
}

fn run_status_compute(mut parser: lexopt::Parser) -> Result<(), CoreError> {
    let mut config_path: Option<PathBuf> = None;
    let mut default_config_path: Option<PathBuf> = None;
    let mut contract_path: Option<PathBuf> = None;
    let mut runtime_dir: Option<PathBuf> = None;
    let mut state_path: Option<PathBuf> = None;
    let mut yazi_config_dir: Option<PathBuf> = None;
    let mut zellij_config_dir: Option<PathBuf> = None;
    let mut zellij_layout_dir: Option<PathBuf> = None;
    let mut layout_override: Option<String> = None;
    let mut yazelix_version: Option<String> = None;
    let mut yazelix_description: Option<String> = None;

    while let Some(arg) = parser
        .next()
        .map_err(|error| CoreError::usage(error.to_string()))?
    {
        match arg {
            Long("config") => config_path = Some(parser_path_value(&mut parser)?),
            Long("default-config") => default_config_path = Some(parser_path_value(&mut parser)?),
            Long("contract") => contract_path = Some(parser_path_value(&mut parser)?),
            Long("runtime-dir") => runtime_dir = Some(parser_path_value(&mut parser)?),
            Long("state-path") => state_path = Some(parser_path_value(&mut parser)?),
            Long("yazi-config-dir") => yazi_config_dir = Some(parser_path_value(&mut parser)?),
            Long("zellij-config-dir") => zellij_config_dir = Some(parser_path_value(&mut parser)?),
            Long("zellij-layout-dir") => zellij_layout_dir = Some(parser_path_value(&mut parser)?),
            Long("layout-override") => layout_override = Some(parser_string_value(&mut parser)?),
            Long("yazelix-version") => yazelix_version = Some(parser_string_value(&mut parser)?),
            Long("yazelix-description") => {
                yazelix_description = Some(parser_string_value(&mut parser)?)
            }
            _ => return Err(CoreError::usage(format!("Unexpected argument: {arg:?}"))),
        }
    }

    let request = RuntimeMaterializationPlanRequest {
        config_path: config_path.ok_or_else(|| CoreError::usage("Missing --config path"))?,
        default_config_path: default_config_path
            .ok_or_else(|| CoreError::usage("Missing --default-config path"))?,
        contract_path: contract_path.ok_or_else(|| CoreError::usage("Missing --contract path"))?,
        runtime_dir: runtime_dir.ok_or_else(|| CoreError::usage("Missing --runtime-dir path"))?,
        state_path: state_path.ok_or_else(|| CoreError::usage("Missing --state-path path"))?,
        yazi_config_dir: yazi_config_dir
            .ok_or_else(|| CoreError::usage("Missing --yazi-config-dir path"))?,
        zellij_config_dir: zellij_config_dir
            .ok_or_else(|| CoreError::usage("Missing --zellij-config-dir path"))?,
        zellij_layout_dir: zellij_layout_dir
            .ok_or_else(|| CoreError::usage("Missing --zellij-layout-dir path"))?,
        layout_override,
    };
    let version = yazelix_version
        .as_deref()
        .filter(|s| !s.trim().is_empty())
        .ok_or_else(|| CoreError::usage("Missing --yazelix-version"))?;
    let description = yazelix_description
        .as_deref()
        .filter(|s| !s.trim().is_empty())
        .ok_or_else(|| CoreError::usage("Missing --yazelix-description"))?;
    let data = compute_status_report(&request, version, description)?;
    write_success_envelope(STATUS_COMPUTE_COMMAND, data)
}

fn take_request_json(parser: &mut lexopt::Parser) -> Result<String, CoreError> {
    let mut request_json: Option<String> = None;

    while let Some(arg) = parser
        .next()
        .map_err(|error| CoreError::usage(error.to_string()))?
    {
        match arg {
            Long("request-json") => request_json = Some(parser_string_value(parser)?),
            _ => return Err(CoreError::usage(format!("Unexpected argument: {arg:?}"))),
        }
    }

    request_json.ok_or_else(|| CoreError::usage("Missing --request-json payload"))
}

fn take_runtime_materialization_plan_request(
    parser: &mut lexopt::Parser,
    kind: &str,
) -> Result<RuntimeMaterializationPlanRequest, CoreError> {
    let mut request_json: Option<String> = None;
    let mut from_env = false;

    while let Some(arg) = parser
        .next()
        .map_err(|error| CoreError::usage(error.to_string()))?
    {
        match arg {
            Long("request-json") => request_json = Some(parser_string_value(parser)?),
            Long("from-env") => from_env = true,
            _ => return Err(CoreError::usage(format!("Unexpected argument: {arg:?}"))),
        }
    }

    match (from_env, request_json) {
        (true, None) => {
            runtime_materialization_plan_request_from_env(config_override_from_env().as_deref())
        }
        (false, Some(raw)) => deserialize_json_request(&raw, kind),
        (true, Some(_)) => Err(CoreError::usage(
            "Use either --from-env or --request-json for runtime materialization, not both.",
        )),
        (false, None) => Err(CoreError::usage(
            "Missing --request-json payload or --from-env for runtime materialization.",
        )),
    }
}

fn take_runtime_materialization_repair_command(
    parser: &mut lexopt::Parser,
) -> Result<RuntimeMaterializationRepairCommand, CoreError> {
    let mut request_json: Option<String> = None;
    let mut from_env = false;
    let mut force = false;
    let mut summary = false;

    while let Some(arg) = parser
        .next()
        .map_err(|error| CoreError::usage(error.to_string()))?
    {
        match arg {
            Long("request-json") => request_json = Some(parser_string_value(parser)?),
            Long("from-env") => from_env = true,
            Long("force") => force = true,
            Long("summary") => summary = true,
            _ => return Err(CoreError::usage(format!("Unexpected argument: {arg:?}"))),
        }
    }

    let request = match (from_env, request_json) {
        (true, None) => Ok(RuntimeMaterializationRepairEvaluateRequest {
            plan: runtime_materialization_plan_request_from_env(
                config_override_from_env().as_deref(),
            )?,
            force,
        }),
        (false, Some(raw)) => {
            if force {
                return Err(CoreError::usage(
                    "Use --force only with --from-env for runtime materialization repair.",
                ));
            }
            deserialize_json_request(&raw, "runtime-materialization.repair")
        }
        (true, Some(_)) => Err(CoreError::usage(
            "Use either --from-env or --request-json for runtime materialization repair, not both.",
        )),
        (false, None) => Err(CoreError::usage(
            "Missing --request-json payload or --from-env for runtime materialization repair.",
        )),
    }?;

    Ok(RuntimeMaterializationRepairCommand { request, summary })
}

fn deserialize_json_request<T: DeserializeOwned>(raw: &str, kind: &str) -> Result<T, CoreError> {
    serde_json::from_str(raw).map_err(|error| {
        CoreError::classified(
            ErrorClass::Usage,
            "invalid_request_json",
            format!("Invalid {kind} request JSON: {error}"),
            "Pass one valid JSON payload via --request-json.",
            serde_json::json!({}),
        )
    })
}

fn ensure_no_args(mut parser: lexopt::Parser) -> Result<(), CoreError> {
    if let Some(arg) = parser
        .next()
        .map_err(|error| CoreError::usage(error.to_string()))?
    {
        return Err(CoreError::usage(format!("Unexpected argument: {arg:?}")));
    }
    Ok(())
}

fn parser_path_value(parser: &mut lexopt::Parser) -> Result<PathBuf, CoreError> {
    Ok(parser
        .value()
        .map_err(|error| CoreError::usage(error.to_string()))?
        .into())
}

fn parser_string_value(parser: &mut lexopt::Parser) -> Result<String, CoreError> {
    parser
        .value()
        .map_err(|error| CoreError::usage(error.to_string()))?
        .into_string()
        .map_err(|_| CoreError::usage("Argument value must be valid UTF-8"))
}

fn write_success_envelope<T: serde::Serialize>(command: &str, data: T) -> Result<(), CoreError> {
    let envelope = success_envelope(command, data);
    serde_json::to_writer(std::io::stdout(), &envelope).map_err(|source| {
        CoreError::io(
            "write_stdout",
            "Could not write helper JSON envelope",
            "Retry the command and report this as a Yazelix internal error if it persists.",
            "<stdout>",
            source.into(),
        )
    })?;
    println!();
    Ok(())
}

fn write_runtime_materialization_repair_summary(
    data: &RuntimeMaterializationRepairRunData,
) -> Result<(), CoreError> {
    let summary = match &data.repair {
        RuntimeRepairDirective::Noop { lines } => lines
            .first()
            .map(String::as_str)
            .unwrap_or("✅ Yazelix generated state is already up to date."),
        RuntimeRepairDirective::Regenerate { success_lines, .. } => success_lines
            .first()
            .map(String::as_str)
            .unwrap_or("✅ Generated runtime state repaired."),
    };

    writeln!(std::io::stdout(), "{summary}").map_err(|source| {
        CoreError::io(
            "write_stdout",
            "Could not write helper summary output",
            "Retry the command and report this as a Yazelix internal error if it persists.",
            "<stdout>",
            source.into(),
        )
    })?;
    Ok(())
}

fn write_runtime_materialization_repair_error_summary(error: &CoreError) -> Result<(), CoreError> {
    let summary = render_runtime_materialization_repair_error_summary(error);
    std::io::stderr()
        .write_all(summary.as_bytes())
        .map_err(|source| {
            CoreError::io(
                "write_stderr",
                "Could not write helper error summary output",
                "Retry the command and report this as a Yazelix internal error if it persists.",
                "<stderr>",
                source,
            )
        })
}

fn render_runtime_materialization_repair_error_summary(error: &CoreError) -> String {
    let mut lines = vec![
        "Yazelix generated runtime repair failed".to_string(),
        format!("Reason: {}", clean_human_error_line(&error.message())),
    ];

    if error.code() == "unsupported_config" {
        append_unsupported_config_summary(error, &mut lines);
    } else {
        lines.push(format!(
            "Recovery: {}",
            clean_human_error_line(&error.remediation())
        ));
    }

    lines.push(String::new());
    lines.join("\n")
}

fn append_unsupported_config_summary(error: &CoreError, lines: &mut Vec<String>) {
    let details = error.details();
    let diagnostics = details
        .get("blocking_diagnostics")
        .and_then(serde_json::Value::as_array)
        .or_else(|| {
            details
                .get("schema_diagnostics")
                .and_then(serde_json::Value::as_array)
        })
        .cloned()
        .unwrap_or_default();
    let blocking_count = details
        .get("blocking_count")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(diagnostics.len() as u64);

    lines.push(format!("Blocking config issues: {blocking_count}"));
    for diagnostic in diagnostics.iter().take(8) {
        let headline = diagnostic
            .get("headline")
            .and_then(serde_json::Value::as_str)
            .or_else(|| diagnostic.get("path").and_then(serde_json::Value::as_str))
            .unwrap_or("Unsupported config entry");
        lines.push(format!("- {}", clean_human_error_line(headline)));
    }
    if diagnostics.len() > 8 {
        lines.push(format!("- and {} more", diagnostics.len() - 8));
    }

    let mut next_steps = Vec::new();
    for diagnostic in &diagnostics {
        let Some(detail_lines) = diagnostic
            .get("detail_lines")
            .and_then(serde_json::Value::as_array)
        else {
            continue;
        };
        for detail in detail_lines
            .iter()
            .filter_map(serde_json::Value::as_str)
            .filter_map(|line| line.trim().strip_prefix("Next: "))
        {
            let cleaned = clean_human_error_line(detail);
            if !next_steps.contains(&cleaned) {
                next_steps.push(cleaned);
            }
        }
    }

    if next_steps.is_empty() {
        lines.push(format!(
            "Recovery: {}",
            clean_human_error_line(&error.remediation())
        ));
    } else {
        lines.push("Next:".to_string());
        for step in next_steps.iter().take(5) {
            lines.push(format!("- {step}"));
        }
    }
}

fn clean_human_error_line(value: &str) -> String {
    let mut cleaned = value.trim().to_string();
    while cleaned.ends_with('.') {
        cleaned.pop();
    }
    cleaned
}
