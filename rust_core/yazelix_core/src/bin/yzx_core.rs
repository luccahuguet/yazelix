use lexopt::prelude::*;
use serde::de::DeserializeOwned;
use std::path::PathBuf;
use yazelix_core::{
    apply_runtime_materialization, compute_config_state, compute_runtime_env,
    compute_status_report, compute_yazi_render_plan, compute_zellij_render_plan, error_envelope,
    evaluate_doctor_config_report, evaluate_doctor_runtime_report, evaluate_helix_doctor_report,
    evaluate_install_ownership_report, evaluate_runtime_contract,
    evaluate_runtime_materialization_repair, evaluate_startup_launch_preflight,
    generate_helix_materialization, generate_yazi_materialization, generate_zellij_materialization,
    normalize_config, plan_runtime_materialization, record_config_state, render_yzx_help,
    success_envelope, yzx_command_metadata, yzx_command_metadata_data, ComputeConfigStateRequest,
    CoreError, DoctorConfigEvaluateRequest, DoctorRuntimeEvaluateRequest, ErrorClass,
    HelixDoctorEvaluateRequest, HelixMaterializationRequest, InstallOwnershipEvaluateRequest,
    NormalizeConfigRequest, RecordConfigStateRequest, RuntimeArtifact,
    RuntimeContractEvaluateRequest, RuntimeEnvComputeRequest, RuntimeMaterializationApplyRequest,
    RuntimeMaterializationPlanRequest, RuntimeMaterializationRepairEvaluateRequest,
    StartupLaunchPreflightRequest, YaziMaterializationRequest, YaziRenderPlanRequest,
    ZellijMaterializationRequest, ZellijRenderPlanRequest,
};

const CONFIG_NORMALIZE_COMMAND: &str = "config.normalize";
const CONFIG_STATE_COMPUTE_COMMAND: &str = "config-state.compute";
const CONFIG_STATE_RECORD_COMMAND: &str = "config-state.record";
const RUNTIME_CONTRACT_EVALUATE_COMMAND: &str = "runtime-contract.evaluate";
const STARTUP_LAUNCH_PREFLIGHT_EVALUATE_COMMAND: &str = "startup-launch-preflight.evaluate";
const RUNTIME_ENV_COMPUTE_COMMAND: &str = "runtime-env.compute";
const RUNTIME_MATERIALIZATION_PLAN_COMMAND: &str = "runtime-materialization.plan";
const RUNTIME_MATERIALIZATION_REPAIR_EVALUATE_COMMAND: &str =
    "runtime-materialization.repair-evaluate";
const RUNTIME_MATERIALIZATION_APPLY_COMMAND: &str = "runtime-materialization.apply";
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
const YZX_COMMAND_METADATA_LIST_COMMAND: &str = "yzx-command-metadata.list";
const YZX_COMMAND_METADATA_EXTERNS_COMMAND: &str = "yzx-command-metadata.externs";
const YZX_COMMAND_METADATA_HELP_COMMAND: &str = "yzx-command-metadata.help";
const UNKNOWN_COMMAND: &str = "unknown";

struct CommandError {
    command: String,
    error: CoreError,
}

impl CommandError {
    fn new(command: impl Into<String>, error: CoreError) -> Box<Self> {
        Box::new(Self {
            command: command.into(),
            error,
        })
    }
}

fn main() {
    match run() {
        Ok(()) => {}
        Err(command_error) => {
            let envelope = error_envelope(&command_error.command, &command_error.error);
            let _ = serde_json::to_writer(std::io::stderr(), &envelope);
            eprintln!();
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
        RUNTIME_MATERIALIZATION_PLAN_COMMAND => {
            let command_for_error = command.clone();
            run_runtime_materialization_plan(parser)
                .map_err(|error| CommandError::new(command_for_error, error))
        }
        RUNTIME_MATERIALIZATION_REPAIR_EVALUATE_COMMAND => {
            let command_for_error = command.clone();
            run_runtime_materialization_repair_evaluate(parser)
                .map_err(|error| CommandError::new(command_for_error, error))
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
        RUNTIME_MATERIALIZATION_APPLY_COMMAND => {
            let command_for_error = command.clone();
            run_runtime_materialization_apply(parser)
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
        YZX_COMMAND_METADATA_HELP_COMMAND => {
            let command_for_error = command.clone();
            run_yzx_command_metadata_help(parser)
                .map_err(|error| CommandError::new(command_for_error, error))
        }
        _ => Err(CommandError::new(
            command.clone(),
            CoreError::usage(format!("Unsupported helper command: {command}")),
        )),
    }
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

fn run_config_state_compute(mut parser: lexopt::Parser) -> Result<(), CoreError> {
    let mut config_path: Option<PathBuf> = None;
    let mut default_config_path: Option<PathBuf> = None;
    let mut contract_path: Option<PathBuf> = None;
    let mut runtime_dir: Option<PathBuf> = None;
    let mut state_path: Option<PathBuf> = None;

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
            _ => return Err(CoreError::usage(format!("Unexpected argument: {arg:?}"))),
        }
    }

    let request = ComputeConfigStateRequest {
        config_path: config_path.ok_or_else(|| CoreError::usage("Missing --config path"))?,
        default_config_path: default_config_path
            .ok_or_else(|| CoreError::usage("Missing --default-config path"))?,
        contract_path: contract_path.ok_or_else(|| CoreError::usage("Missing --contract path"))?,
        runtime_dir: runtime_dir.ok_or_else(|| CoreError::usage("Missing --runtime-dir path"))?,
        state_path: state_path.ok_or_else(|| CoreError::usage("Missing --state-path path"))?,
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
            _ => return Err(CoreError::usage(format!("Unexpected argument: {arg:?}"))),
        }
    }

    let request = RecordConfigStateRequest {
        config_file: config_file.ok_or_else(|| CoreError::usage("Missing --config-file path"))?,
        managed_config_path: managed_config_path
            .ok_or_else(|| CoreError::usage("Missing --managed-config path"))?,
        state_path: state_path.ok_or_else(|| CoreError::usage("Missing --state-path path"))?,
        config_hash: config_hash.ok_or_else(|| CoreError::usage("Missing --config-hash value"))?,
        runtime_hash: runtime_hash
            .ok_or_else(|| CoreError::usage("Missing --runtime-hash value"))?,
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
    let request: InstallOwnershipEvaluateRequest =
        serde_json::from_str(&request_json).map_err(|error| {
            CoreError::classified(
                ErrorClass::Usage,
                "invalid_request_json",
                format!("Invalid install-ownership request JSON: {error}"),
                "Pass one valid JSON payload via --request-json.",
                serde_json::json!({}),
            )
        })?;
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

fn run_runtime_env_compute(mut parser: lexopt::Parser) -> Result<(), CoreError> {
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
    let request: RuntimeEnvComputeRequest =
        serde_json::from_str(&request_json).map_err(|error| {
            CoreError::classified(
                ErrorClass::Usage,
                "invalid_request_json",
                format!("Invalid runtime-env request JSON: {error}"),
                "Pass one valid JSON payload via --request-json.",
                serde_json::json!({}),
            )
        })?;
    let data = compute_runtime_env(&request)?;
    write_success_envelope(RUNTIME_ENV_COMPUTE_COMMAND, data)
}

fn run_runtime_materialization_plan(mut parser: lexopt::Parser) -> Result<(), CoreError> {
    let mut config_path: Option<PathBuf> = None;
    let mut default_config_path: Option<PathBuf> = None;
    let mut contract_path: Option<PathBuf> = None;
    let mut runtime_dir: Option<PathBuf> = None;
    let mut state_path: Option<PathBuf> = None;
    let mut yazi_config_dir: Option<PathBuf> = None;
    let mut zellij_config_dir: Option<PathBuf> = None;
    let mut zellij_layout_dir: Option<PathBuf> = None;
    let mut layout_override: Option<String> = None;

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
    let data = plan_runtime_materialization(&request)?;
    write_success_envelope(RUNTIME_MATERIALIZATION_PLAN_COMMAND, data)
}

fn run_runtime_materialization_repair_evaluate(
    mut parser: lexopt::Parser,
) -> Result<(), CoreError> {
    let mut config_path: Option<PathBuf> = None;
    let mut default_config_path: Option<PathBuf> = None;
    let mut contract_path: Option<PathBuf> = None;
    let mut runtime_dir: Option<PathBuf> = None;
    let mut state_path: Option<PathBuf> = None;
    let mut yazi_config_dir: Option<PathBuf> = None;
    let mut zellij_config_dir: Option<PathBuf> = None;
    let mut zellij_layout_dir: Option<PathBuf> = None;
    let mut layout_override: Option<String> = None;
    let mut force = false;

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
            Long("force") => force = true,
            _ => return Err(CoreError::usage(format!("Unexpected argument: {arg:?}"))),
        }
    }

    let request = RuntimeMaterializationRepairEvaluateRequest {
        plan: RuntimeMaterializationPlanRequest {
            config_path: config_path.ok_or_else(|| CoreError::usage("Missing --config path"))?,
            default_config_path: default_config_path
                .ok_or_else(|| CoreError::usage("Missing --default-config path"))?,
            contract_path: contract_path
                .ok_or_else(|| CoreError::usage("Missing --contract path"))?,
            runtime_dir: runtime_dir
                .ok_or_else(|| CoreError::usage("Missing --runtime-dir path"))?,
            state_path: state_path.ok_or_else(|| CoreError::usage("Missing --state-path path"))?,
            yazi_config_dir: yazi_config_dir
                .ok_or_else(|| CoreError::usage("Missing --yazi-config-dir path"))?,
            zellij_config_dir: zellij_config_dir
                .ok_or_else(|| CoreError::usage("Missing --zellij-config-dir path"))?,
            zellij_layout_dir: zellij_layout_dir
                .ok_or_else(|| CoreError::usage("Missing --zellij-layout-dir path"))?,
            layout_override,
        },
        force,
    };
    let data = evaluate_runtime_materialization_repair(&request)?;
    write_success_envelope(RUNTIME_MATERIALIZATION_REPAIR_EVALUATE_COMMAND, data)
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

fn run_runtime_materialization_apply(mut parser: lexopt::Parser) -> Result<(), CoreError> {
    let mut config_file: Option<String> = None;
    let mut managed_config_path: Option<PathBuf> = None;
    let mut state_path: Option<PathBuf> = None;
    let mut config_hash: Option<String> = None;
    let mut runtime_hash: Option<String> = None;
    let mut expected_artifacts: Option<Vec<RuntimeArtifact>> = None;

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
            Long("expected-artifacts-json") => {
                let raw = parser_string_value(&mut parser)?;
                let parsed =
                    serde_json::from_str::<Vec<RuntimeArtifact>>(&raw).map_err(|error| {
                        CoreError::usage(format!(
                            "Invalid --expected-artifacts-json value: {error}"
                        ))
                    })?;
                expected_artifacts = Some(parsed);
            }
            _ => return Err(CoreError::usage(format!("Unexpected argument: {arg:?}"))),
        }
    }

    let request = RuntimeMaterializationApplyRequest {
        config_file: config_file.ok_or_else(|| CoreError::usage("Missing --config-file path"))?,
        managed_config_path: managed_config_path
            .ok_or_else(|| CoreError::usage("Missing --managed-config path"))?,
        state_path: state_path.ok_or_else(|| CoreError::usage("Missing --state-path path"))?,
        config_hash: config_hash.ok_or_else(|| CoreError::usage("Missing --config-hash value"))?,
        runtime_hash: runtime_hash
            .ok_or_else(|| CoreError::usage("Missing --runtime-hash value"))?,
        expected_artifacts: expected_artifacts
            .ok_or_else(|| CoreError::usage("Missing --expected-artifacts-json value"))?,
    };
    let data = apply_runtime_materialization(&request)?;
    write_success_envelope(RUNTIME_MATERIALIZATION_APPLY_COMMAND, data)
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
